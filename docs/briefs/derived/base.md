<!--meta
id: D-base-layout
layer: derived
premises: [D-metal-is-the-game, D-energy-second, D-tier2-sprint, F-map-build, F-fight-death, F-eco-build, F-eco-reclaim, F-eco-consequences, F-unit-armnanotc, F-unit-armsolar, F-unit-armfus, F-unit-armestor, F-unit-armmakr, F-units-economy-armada, F-units-structures-armada, F-units-air-armada, M-pool-opening, M-pool-openers, M-gecko-vs-us-comet]
lessons: [L-energy-nanos, L-sheet-labs, L-gecko-one-more-con]
tests: [E-center-command-2v4, the replay of thebluegecko against InactiveAI (docs/experiments.md replay-gecko-inactiveai-comet-2026-10-21), the pool's Comet duels and our advance-3 and advance-4 under run/base_layout.py (docs/studies/2026-10-29-base-layout.md)]
sources: {people: [thebluegecko (his records), the pool's players at 35 OS and above, the user (the problem statement of 2026-10-29)], dates: [2026-10-29], code: [upstream/RecoilEngine/rts/Sim/Units/UnitTypes/Builder.cpp:374 (buildDistance + buildeeRadius), upstream/RecoilEngine/rts/Game/GameHelper.cpp:132-149 (blast falloff), upstream/RecoilEngine/rts/Sim/Weapons/WeaponDef.cpp:71 (areaOfEffect is a diameter), upstream/RecoilEngine/rts/Sim/Objects/SolidObject.h:38-49 (yardmap states), upstream/Beyond-All-Reason/weapons/Unit_Explosions.lua, the unit files' footprints and yardmaps], web: [beyondallreason.info/unit/armnanotc, beyondallreason.info/guide/in-depth-look-at-economy, beyondallreason.info/guide/important-knowledge-on-advanced-mechanics, beyondallreason.info/commands/build-grid-line-spacing]}
status: foundation, checked (run/base_layout.py, 2026-10-29: 118 seats of the pool's Comet duels at 35 OS and above, thebluegecko's two records, our advance-3 and advance-4, the fencing-in instrument over 49 pool seats at 40 OS and our fourteen latest games). Not in any manifest: the user (2026-10-29) does not expect the player or the commander to be given it; it is the foundation for what the base-design arc builds, read by people and by the agents who build it
-->
### The base as three machines

**The heuristic.** The people lay a base out three times, and each time the shape follows one number. In the first
minute the commander stands still and places everything its arm reaches, two extractors, two solars and the plant,
walking only when the reach is spent; the unit page's advice, "place extractors closer to your constructor to save
time", and a streamer's "fit all your first buildings inside that green circle" (as heard, undated). In the middle
game the plant gets two or three construction turrets beside it, never in front of it, and the solars pack edge to
edge behind. From the advanced plant on, the turrets become the base: thebluegecko stands them touching, in rows,
twenty to forty within reach of the plant, eats each plant where it stands and puts the next one on the same ground,
and later lays a block of three hundred south of the base and places every advanced fusion and converter inside it,
reclaiming the solars that stood where the block was to go. The unit page on the turret: "Use multiple on a single
factory to increase build power ... They uniquely damage only other nearby nanos when killed. Packing them too close
can lead to chain explosions. Initiating a self-destruct (or transporting) one or two can prevent a domino effect."
Safe to follow as it stands, with the chain bill read before the block is placed.

**The quantity.** Each machine has its own: in the opening, the commander's seconds spent walking or idle between
orders, each worth a third of a solar; in the middle, the turrets in reach of the plant's pad and the clear ground in
front of it; in the late game, the turrets in reach of the frame being stood, the flat ground the block has left to
grow into, and the metal a single death takes with it. Not the building count and not the base's footprint: a base is
good when the next frame starts inside the reach of the build power that will finish it.

**The mechanism.**

- *The arm.* A builder starts a frame when the frame's edge is within its build distance of it (the engine adds the
  buildee's radius to the builder's build distance: `buildDistance + buildeeRadius`). The commander's distance is 145,
  so a solar's centre may be about 190 away, an extractor's about 180, a plant's about 200: everything inside a disc
  of that radius is built without a step. It walks 37.5 elmos a second and turns half a circle in about a second
  (1,148 of 65,536 angular units a frame, 189 degrees a second), so turning is never the cost; a step of 176 elmos,
  the pool's median between consecutive early sites, is 4.7 seconds, and the dead second between one structure
  finishing and the next order is the other cost. At the commander's 300 of build an extractor is 6 seconds, a solar
  8.7, a turbine 5.3, a light laser 8, a bot lab 16.7, a vehicle plant 19, a construction turret 17.7: a second lost
  is a ninth of a solar, and eleven seconds are a solar and a quarter. What the people do with this, 118 seats of the
  pool's Comet duels at 35 OS and above (`run/base_layout.py`): the first plant finished at 1:11 median; the
  commander stood still for 87 of its first 101 seconds and walked 399 elmos in them; 53 of the 118 built every
  structure up to and including the plant from one standing place and 49 more from two; and the gap from a structure
  finishing to the next order was 0.6 seconds median (0.3 to 0.8 between quartiles), because the orders are queued:
  twelve build orders in the first hundred seconds in four batches, batches of three to eight common, 39 percent of
  the sites out of reach when ordered, the commander walking to them with the queue in hand. One seat as a picture
  (Hellontoast, 41 OS): extractor 0:03, extractor 0:10, solar 0:16, solar 0:25, vehicle plant 0:34, solar 0:54, all
  six from (1416, 3556) without moving, the farthest 178 away, each ordered within half a second of the last
  finishing; then two walks of 450 and 590 for the next extractors. Comet's start boxes have two spots within the
  arm's reach of the start, which is why the pool's opening is "mex mex solar solar plant" (37 of 118) or "mex mex
  solar LLT mex solar" (33). Ours on the same ground (advance-3 and advance-4, one opening measured twice on seed 1):
  the same five structures in the same order, from three standing places, 480 elmos walked, and gaps of 3.5, 1.0,
  0.8 and 5.8 seconds, eleven seconds against the pool's two and a half, every order issued alone (queue depth one in
  six of six; the hands send the list's next step when the builder is free or its frame is 60 percent done, which is
  how the 0.8 arises and the 5.8 too, when the site is out of reach). The plant stood at 1:03 against the pool's
  1:11, a median that holds the 33 seats building a laser tower and a third extractor first; against the seats on our
  own sequence it is late by the eleven seconds (Hellontoast's stood at 0:54). thebluegecko against InactiveAI opened
  with three extractors and three solars before the plant from two stands, 11 of 14 sites in reach when ordered, and
  gaps of 0.8, 8.3, 0.3, 0.7, 0.7 and 0.7: one walk, and nothing else.
- *Footprints and packing.* A footprint is in 16-elmo squares snapped to that grid (F-map-build): a turret, a converter, an
  energy storage or a turbine 48 elmos a side; an extractor or a moho 64; an advanced converter 64; a solar 80; a
  fusion 96 by 80; an advanced fusion 96; a lab or plant 96; an advanced plant 144. Two buildings touch when their
  centres are a footprint apart, and that is where the people put them: the pool's solars stand 80 apart nearest
  neighbour (median of 118 seats' medians), their converters 48, thebluegecko's turrets 48 (43 of 45 in the Center
  Command game, the other two 64 and 100), his converters 48 and advanced converters 64. Ours stand apart: solars 97
  and 128 nearest neighbour in advance-4 and advance-3, turrets 181 and 234 (the hands place a building beside its
  builder turned in 45-degree steps until the site is clear, so each one lands where the builder happened to stand).
  The extractor's own footprint is mostly build-only squares (its yardmap's `b`: blocked for walking, open for
  building), so other buildings may overlap its edges; a spot is not a hole in the base. Slopes bound the packing more
  than the grid does: a solar, a turret or a fusion stands only on ground within 10.7 elmos of its centre's height
  across the whole footprint, a plant 17, so a block wants the flattest ground there is, and Comet's plateau is why
  the pool's bases are dense there.
- *The exit.* A plant's units leave through its front, and the yardmap's exit squares (`e`, "closed for walk into,
  closed for build") are the front half of the lab's own footprint and the inner columns of the vehicle plant's:
  the engine protects nothing outside the footprint. The pool keeps the apron clear by habit and not by much: of 225
  land factories read, 205 faced the enemy's side of the map, 95 had a building of their own inside a lane 320 deep
  and the plant's width, and those buildings' near edges stood 128, 176 and 256 from the front at the quartiles, 18
  percent within 100: the people clear about a hundred elmos and build in the rest, solars and extractors mostly,
  turrets too. Of the pool's turrets within reach of a plant, 56 percent stood beside it, 26 behind and 17 in front
  (past the clearance). Ours keep the whole 320 clear (H-ECO-YARD-LANE) and stand the turrets 56 to 69 from the
  plant's edge where the pool's stand 129 and his 121: we hug the plant and keep its front emptier than anyone.
- *The courtyard.* The apron is not the exit; the exit is a path from the apron to open ground, and the base grows
  across it. A unit walks only where every square under its movement footprint is free, and the footprints come in
  sizes: 2 squares (32 elmos) for the bots and the Flash, 3 (48) for the vehicles and the heavier bots, 4 (64) for
  the Bull, the Fatboy, the Razorback and the Catapult, 5 (80) for the Bantha, the Marauder and the Shiva, 6 (96) for
  the Vanguard, 7 (112) for the Thor and the Juggernaut. So a gap of one or two squares between solars passes a Stumpy
  and stops a Bull, and the courtyard the user has seen is that: a plant facing into its own solar field, the solars
  at 97 to 128 apart (one footprint and a square or two), the small units out and the large ones penned. Measured
  (`run/base_layout.py pen`: at each minute, whether a unit of each size can walk from the plant's door to ground 1,200
  away through our own standing buildings and the terrain's slopes): in contact-1 the second plant's path closed at
  size 7 by 13:00, 6 by 14:00, 4 and 5 by 17:00, and never for 2 and 3; in airstun-1 sizes 5 to 7 at 14:00; in
  apex-v0405-comet the first plant's at 5 to 7 by 10:00 and an advanced plant's at 6 and 7 by 22:00; the other 39
  plants of our fourteen latest games never closed at any size, and none closed for the sizes we make. The pool at
  40 OS (49 seats, 97 plants, buildings only since those records carry no terrain): 24 plants closed, 22 of them only
  at sizes 6 and 7, two at 4, one at everything (a plant walled in at 8:00 and presumably eaten). Neither side keeps a
  street for what it does not build; the pool's duels end before anyone makes a size-5 unit, so nobody pays. The bill
  comes when the first Bull or Bantha stands in a base laid for Stumpies, and it is paid by reclaiming a solar or two;
  the alternative is a street of the widest size the plant will ever make, 7 squares for a gantry's, decided when the
  plant is placed.
- *The reach of the turret.* A construction turret helps anything whose edge is within 400 of it (the same rule as the
  arm, so a 9 by 9 plant's centre may be 472 away), at 200 of build for 230 metal and 3,200 energy, and it does nothing
  on its own (K-mech-nano-turrets-idle-without-an-order). Reach and footprint set the ceiling: a lattice of touching
  turrets has at most 274 within reach of a 6 by 6 frame's centre and 304 of a 9 by 9's, before ground, streets and the
  frame's own footprint take their share. Build time over the turrets in reach is the frame's clock: a fusion
  (54,000) under 10 turrets 27 seconds, under 20 thirteen; an advanced fusion (312,500) under 30 turrets 52 seconds,
  under 60 twenty-six, drawing 1,300 energy a second meanwhile; an advanced plant (25,000) under 20 turrets 6 seconds.
  The turrets are the cheaper half of the pair: forty of them cost 9,200 metal, one advanced fusion's price, and
  thebluegecko's block of 276 stood thirty advanced fusions and 162 advanced converters (39 and 28 turrets within 300 of
  each when placed, median). The pool's duels, thirteen minutes long, never get there: four turrets a seat median, two
  on a bot lab, two and a half on a vehicle plant, three and a half on an air plant, eleven on an advanced vehicle plant
  (20 at most) — the T1 plant takes two or three and no more, as the people say (RogueL1ke, "T1 assist caps at about 2
  extra cons"), and the pile begins with the advanced plant. His Center Command game shows the pile's shape: 45 turrets
  in two touching blocks of 22 (384 by 320 elmos) and 20 (240 by 240), 36 within reach of the advanced vehicle plant,
  the air plant 30, the advanced air plant 42; the advanced plant eaten at 18:45 and the advanced air plant placed
  sixteen elmos from where it stood, so the same block served the next plant in six seconds; his thirty turbines and
  tide generators with no turret in reach of any of them: the T1 economy and the block never meet. The block stood 4,500
  elmos from his first lab, forward, where the advanced plant went; the block's place is the advanced plant's place,
  chosen once. His block against InactiveAI, 304 turrets by 21:00 (`run/base_layout.py block`), is the same shape grown
  up: 303 of 304 at 48 from their nearest neighbour, in bars four turrets wide (192 elmos) and 576 to 960 long, the
  two largest running north-south and two others east-west, with rows of advanced converters (64 pitch, 120 from the
  nearest turret) and the advanced fusions (76 from the nearest turret) between them; the converters had 46 turrets in
  reach when placed (121 at most) and their frames took 5.5 seconds median, the advanced fusions 53 and 35 seconds,
  the first of each at 13:48 and 13:24;
  his 25 solars had no turret in reach of any of them and his first two plants were eaten (4:41, 10:22) with none and
  29; and in the last ninety seconds he placed eight air plants inside the block, each with 35 to 87 turrets in reach,
  which is the end-game production the block was for: an air plant has no exit lane to keep, so it goes where the
  build power is. Only 17 percent of his turrets were ever within 400 of a factory; the block is laid for the frames,
  and the plants come to it.
- *The gantry.* The experimental gantry is the block's natural tenant and its hardest. It is 12 by 12 (192 elmos),
  7,900 metal and 62,000 of build (Cortex's 8,400 and 68,000), with 1,800 of build power of its own, which makes a
  Bantha (360,000) in 200 seconds, a Thor (320,000) in 178 and a Juggernaut (780,000) in 433; forty turrets round it
  bring those to 37, 33 and 80 seconds, and the Bantha's 286,000 energy over 37 seconds is a draw of 7,700 a second,
  which only a block's fusions and converters supply. Its death blast is 1,330 to 210 and kills a turret within 121 of
  its wall, so the ring from 122 to 472 (400 plus its half footprint) both builds and survives it; nothing of the
  advanced fusion's contradiction here. Its exit is its own front half (Armada; Cortex's the inner eight columns, 128
  wide), and what leaves it is size 5 to 7: the street from a gantry's door to the map must be seven squares (112
  elmos) clear for the whole way, the widest requirement in the game, so a base that will hold a gantry lays that
  street first and packs round it. No person in our records built one (the pool's duels are too short; thebluegecko's
  two games ended before); the eight-seat AI games did, with 13 to 24 turrets in reach when placed, the frames taking
  17 to 42 seconds, and one gantry in a 1v1 of ours placed at 20:30 with no turret in reach and never finished.
- *Moving the block.* A turret weighs 700 and is not barred from transport (`cantbetransported = false`); the Atlas
  and the Valkyrie carry one unit of mass up to 750 and size 3, which the turret is exactly, at 207 elmos a second; the
  Dragonfly size 4 with no mass bound. One turret a trip, so a block of forty moves 2,000 elmos in about two minutes
  with ten transports (ten seconds each way and the load and set-down); the saving is the 9,200 metal and 128,000
  energy of forty new turrets, not time, since under its own block a new turret stands in three seconds. The unit page
  gives the people's use: "placing buildpower on the front line to reclaim and repair units", and the domino cure,
  lifting one or two out of a chain. We have no hand for it yet.
- *The chain bill.* Every death blast falls linearly from its full damage at the centre to nothing at its radius,
  measured to the victim's surface, the radius half the file's `areaOfEffect` (F-fight-death). So the distance within
  which a death at full health kills a full-health neighbour is `radius × (1 − victim health / blast damage)`: an
  advanced fusion (10,600 to 640) kills a turret (560) within 606 of its surface, an advanced converter (445) within
  613, a fusion (3,800) within 410, a plant (2,900 to 4,750) within 350 to 470, another advanced fusion within 139
  (Armada's 8,300) or 48 (Cortex's 9,800) of its surface: touching advanced fusions chain, and at one footprint apart
  (96 of surface) Armada's still do. A fusion (2,650 to 240) kills a turret within 189, a converter within 225, a
  solar within 209, and no fusion by itself. That last qualifier is the general rule (the user, 2026-10-29): in a
  lattice a cell takes the sum of every dead neighbour's blast, so the chain criterion is that sum against its health,
  not one blast; a dead row of three touching fusions puts about 9,800 on the next fusion, a dead row of turbines 245
  on the next turbine (196), a dead row of touching turrets 920 on the next turret (560). The sums by kind and pitch
  are the study's §9 table (`run/base_layout.py chain`): solars and advanced solars never chain at any pitch, turbines
  are safe at one footprint between, basic converters chain at every pitch to 96 and advanced converters to 128,
  turrets at 48 are a train, at 64 a hole spreads but a front does not, at 96 nothing. An energy
  storage (980 to 210) kills a turret within 90, a converter within 174, a turbine within 168. An advanced converter
  (1,220 to 160) kills a turret within 87 and another advanced converter within 102, so touching ones (64) chain; a
  converter (660 to 105) kills another converter within 78, touching ones (48) chain, and no turret. The turret's own
  blast is 11 to anything else and 530 to another turret at a radius of 64 with the edge at 0.75 of the centre, so a
  touching neighbour (its surface about 32 from the blast) takes about 420 of its 560: one dying turret does not take
  the next, two dying together do, and a block at 48 spacing is a train once two adjacent ones go; at 96 (one
  footprint between) it is not. The bill in his Center Command block: the two fusions stood 81 from the nearest
  turrets, and the death of either at full health would kill 24 finished buildings; the advanced converters touching
  at 64 would chain three; the energy storages were spread (232 apart). The bill in his InactiveAI block, read the same
  way: the thirty advanced fusions stand touching (96 apart, nearest neighbour, all thirty) among the turrets (76 from
  the nearest), so a dying one takes the three it touches (Cortex's 9,800 falls within its 48) and the death of one at
  full health kills 174 finished buildings median, 220 at most; the 155 advanced converters touch at 64 (a death takes
  eight buildings, five of them converters); the eight fusions stand 80 to 96 apart and 69 from the turrets (a death
  takes 19). He takes the whole bill for the density, against an opponent that cannot present it. The pool pays little of this in duels: by the same arithmetic 198 of 5,343 building deaths, 3.7
  percent, fell to a neighbour's blast rather than the enemy's gun (56 of 118 seats had at least one: solars and
  extractors beside a dying laser tower, converters beside converters, turrets beside a storage or a plant). The late
  game is where the bill comes due, and the contradiction in it is exact: the turrets that build an advanced fusion
  must stand within 400 of it, and its death kills every turret within 606, so a block that builds one cannot stand
  outside its blast. A fusion is different, 400 of reach against 189 of kill, so a ring from 190 to 448 both builds
  and survives it. For the advanced fusion the choices are the people's (take it: 9,700 metal of fusion lost with
  9,200 of turrets, and the energy with it), or a block that builds the fusion and then moves, or constructors and
  aircraft placing it beside a block that reaches it with fewer, cheaper turrets than it would kill. No one in the
  pool spaces for it.
- *Eating the old base.* Reclaiming a finished structure returns its metal whole in the second the reclaim ends and
  none of its energy (F-eco-reclaim); a solar under one turret takes 13 seconds, under ten 1.3, and gives back 155,
  under a second of a 14-minute income of 210 a second. The pool does it: 62 of 118 seats reclaimed something of their
  own, 330 solars across the pool, 31 vehicle plants and 9 bot labs, 39 turrets and 26 laser towers, at a median
  clock of 15:00; he did it in the InactiveAI game at 4:41 (the plant, its nine constructors out), 10:22 (the advanced
  plant, under its own turrets), 13:44 to 14:41 (seventeen solars, the first six where thirteen to twenty turret frames
  stood within 150 elmos a minute later) and 15:01 to 15:26 (the four advanced solars). The metal is five to twelve
  seconds of income a lump; the purchase is the ground. A solar is 6,400 square elmos, two and three quarter turret
  footprints, seventeen solars forty-seven turrets' worth; the plant's 96 by 96 is four turrets or the next plant.
  The eaten plant is the cleaner case: its ground is the block's centre already, and the next plant goes on it.

**Consider, and why it may work.** The problem has fewer free decisions than it looks:

1. *Three machines, three constants.* The commander's stand (190), the plant's apron (about 100 clear in front, the
   turrets beside), the block (400 of reach, 48 of pitch). Each has one site decision: where the commander stands,
   where the plant faces, where the block's centre is. Everything else in the pool's bases follows by packing.
2. *The block's centre is the advanced plant's.* The people place the block where the advanced plant will stand and
   feed every later plant and every advanced fusion and converter into it. So the late-game layout is decided when the
   advanced plant is placed, at about 5:30, and the decision is which flat ground within the base's protection takes a
   block that will grow to 600 elmos across. His 304 turrets at 48 pitch are 700,000 square elmos of footprint alone;
   with the converter and fusion rows between the bars the block covered about 1,500,000, a square 1,200 a side, and
   his solars were in the way. The bars are the people's packing solved: a bar four turrets wide puts every turret
   within 72 of the bar's edge, so a frame in the row beside it has the whole bar in reach, and a row of frames between
   two bars has both.
3. *Pitch is the one lever the people leave.* 48 (theirs: 274 turrets in reach of a frame, a train), 64 (the territory
   bot's T-FARM row pitch: 154 in reach, still a train by the turret's own blast), 96 (68 in reach, no turret-to-turret
   chain, and a street between every two rows that a vehicle fits through). The ceiling at 96 is above the 46 and 53
   thebluegecko's converter and advanced fusion frames had in reach when placed (median; 121 at most), so his frames
   would have stood at the same clocks under a quarter of his density, had the block been laid at 96 from the start.
   Whether the lower density costs anything is a question of ground, the block at 96 needing four times the flat area
   for the same count, and of the people's hands, which place at the grid's pitch because it is the default. (The
   user's ruling, 2026-10-29: the 96 sub-lattice first and the 48 lattice back-filled once anti-air is robust; chain
   death a second or third criterion, the fusions beside the active farm; `docs/design/2026-10-29-base-layout.md`.)
4. *The big blasts have a geometry of their own.* Fusions at one footprint from each other, inside a ring of turrets
   190 to 448 out; energy storages at least 90 from a turret and spread; converters one footprint apart; advanced
   fusions where their 606 covers the least metal, which inside a block is nowhere, so the question for them is who
   builds them and whether the block that did moves on.
5. *Transport turns the block's site into a soft decision.* With a hand for it, a block built where the advanced
   plant stood can follow the fusions, the gantry or the front, and a turret or two lifted out of a chain is the
   people's own cure for a burning block.
6. *Streets are sized by what will walk them, and the size is known at the plant.* A plant's products are its
   roster: a bot lab's largest is 3, a vehicle plant's 4 (the Bull), an advanced lab's 6 (the Vanguard), an advanced
   vehicle plant's 7 (the Thor), a gantry's 7. Every other building packs at its footprint; the one gap a base keeps
   is the street from each plant's door to the open, as wide as the plant's largest unit, kept open as the base grows
   round it. Letting the small out while the large are penned is what happens when no such street was decided, and
   the courtyard is the picture of it. Reclaim is the way out after the fact, a solar or two along the shortest line
   to the open; the street is the way before.

**Where it bends.** (1) Wind maps: a turbine is a 48 footprint at 196 health with an 89 blast; a turbine field packed
at 48 is the base's largest flammable, and the pool's advice ("Wind generators chain explode terribly. Use build
spacing") is for it, not for turrets. (2) Maps whose flat ground is scarce or sloped (Glitters' hills, Great Divide's
ledges): the slope tolerance of 10.7 elmos across an 80 or 96 footprint decides where a block can be at all before any
other consideration. (3) Team games: an ally's base beside ours shares the chain bill and the apron, and allied
turrets do not help our frames unless they are in reach of them. (4) Sea starts: the first machine is a tide
generator and the arm reaches onto the water (his Center Command opening: tide, extractor, extractor, lab, from three
stands). (5) The hands' lists: the pool's 0.6 seconds come from a queue the commander walks with; a list that issues
one step at a time pays a second per step however good the site.

**When to depart from the line.** When bombers or a nuke are shown: a nuke's 640 radius takes a block whole, and a
bomber's fusion kill inside a block costs the block (D-surprise-weapons, D-energy-second's bomber target). When the
front is close enough that the block's ground is the fight's: a block forward of the base is build power on the
front line by the people's own use, and a target by the same token.

**The tests.** `run/base_layout.py` (2026-10-29; `docs/studies/2026-10-29-base-layout.md`) over the pool's 70 Comet
Catcher duels with records (118 seats at 35 OS and above), thebluegecko's Center Command 2v4 and InactiveAI records,
and our advance-3 and advance-4: the opening's stands, path, gaps and queue batches; the turrets' pitch, their count
per plant and their side of it; the lane's contents and depths; nearest-neighbour spacing by kind; own reclaims by
kind and clock; chain victims by the falloff arithmetic; and the fencing-in instrument (`pen`), the first minute a unit
of each footprint can no longer walk from a plant's door to ground 1,200 away through our own buildings and the
terrain's slopes (49 pool seats at 40 OS, our fourteen latest games, his two). Wrong if a pool at 40 OS and above showed gaps over two
seconds or three stands before the plant as the norm; if the people's blocks turned out to be spaced at 96 rather than
touching; if turrets in reach of a frame did not shorten it in proportion (K-t2-moho-frames-finished-by-tier-1-build-power
says they do); or if a block's death did not take the turrets the arithmetic says it must.
