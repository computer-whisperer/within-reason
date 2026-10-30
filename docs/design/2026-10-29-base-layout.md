# Base layout: districts, streets and the block (2026-10-29)

The hands' placement is rebuilt round the three site decisions a base has, so that what the people's bases show
(`docs/briefs/derived/base.md`, D-base-layout; the study `docs/studies/2026-10-29-base-layout.md`) is what ours do by
construction: the commander's first structures from one stand with the orders queued, a plant with a clear apron and a
street as wide as its largest unit, and a turret block laid for the frames when the advanced plant is placed. The
user's framing (2026-10-29): "little order or reason to the layout of our bases"; the early game gated by the
commander's walking and turning; exits kept open then "fenced in to a courtyard made of our developing economy"; the
late game's turret pile, fusions, gantries and end-game production; chain deaths as the cost of packing; reclaim for
ground; turret transport coming. Written before building (the user: "write up the layout plan as a design doc before
building").

## 0. What the code does today (read 2026-10-29, HEAD a86be8b)

- **Every build is a player list step; Jev places nothing.** In twelve recent games (advance-1 to -4, files-1,
  contact-1, airstun-1, apex-v0405, companies-1 and -2, migration-3-1 and -4-1) all 1,948 build plays came from the
  player's `queue` steps (`source: list`). A step is `<kind> [<place>]`; a place is a mark or a spot and is required for
  defences and mohos (`lists.rs` `at_place`: "it needs a place"). The player named a place for about a quarter of the
  base structures (solars 151 of 679, turrets 24 of 96, advanced plants 44 of 112, fusions 39 of 102, converters 10 of
  44, advanced solars 9 of 97, plants and storages never), about nine marks a game.
- **The site is the algorithm's in every case.** `economy.rs` `place_planned`: a factory one reach from the builder
  toward a yard 350 ahead of home (`LAB_YARD`), facing toward the enemy, its lane 320 deep kept clear (`yards.rs`,
  H-ECO-YARD-LANE); an armed building on a line 650 ahead (`TURRET_LINE`); a stockpiler and, when the builder is over
  1,000 from home, a generator at a back-field point behind home (`BACK_FIELD`); a construction turret `Beside` the
  nearest factory; everything else `beside_builder`: one build reach from where the builder stands, toward the enemy
  base, turned in 45-degree steps until nothing of ours stands or is started within 90 (a factory 230) and no spot
  within 150 (260). A named place is an anchor: `Plan::Near(def, at)` with `search_radius: 1000` and `min_dist:
  gap_around(def)` (`BUILDING_GAP` or `LAB_GAP` squares), a defence 120 toward the enemy from the place
  (`DEFENCE_FORWARD`). The shim then asks the engine for the closest free site (`BuildSite`: `near`, `search_radius`,
  `min_dist`, `placements` with facings and keep-out strips, `exact` for the territory bot's slots).
- **One order at a time.** A list's next step goes when the builder is idle or its frame is 60 percent done
  (`QUEUE_AT`), as a single `Command::Build` (`queue: false`); the protocol's `queue: true` exists and is unused by the
  lists. Measured: our commander's gaps between a structure finishing and the next order 3.5 / 1.0 / 0.8 / 5.8 s and
  three standing places for five structures, against the pool's 0.6 s and one or two stands (the study, §3 and §5).
- **The result is the courtyard.** Solars at 97 to 128 apart (one footprint plus a square or two), turrets 56 to 69 from
  a plant's edge, nothing in rows; contact-1's second plant penned Bulls at 17:00 through gaps that pass a Stumpy
  (`run/base_layout.py pen`). The 320 lane keeps the apron emptier than anyone's (the pool clears about 100) and
  protects nothing beyond it.
- **The territory bot has a lattice of its own** (T-FARM, `bots/territory/src/hands.rs` `farm_slots`): rows along x
  from an origin 800 to 1,200 from home, turret slots 64 apart and building slots 112 apart in alternate rows 176
  apart, one building column in three a street, exact sites. It is the one block design in the repository, written
  for the bot's planner; the flagship has none.
- **The micro engine already carries the ground model a street check needs**: D18 (2026-10-25), the lane's ground
  carrying every unit without speed by its footprint, and `crates/terrain` (`passable`, `clearance`, `Field`), which
  the fencing-in instrument re-implemented in Python for the study.

## 1. The design

**One placement layer, a layout plan.** Placement stays a code decision (`docs/code-decisions.md` §6; the no-live-
heuristics ruling governs what and when, not where). The layer answers one question, "where does this building go",
and answers it from a plan of districts rather than from where the builder stands. The player keeps the three site
decisions through marks it already makes; Jev stays out, as it is today.

### 1.0 The ground budget, allies and water

The user (2026-10-29, after the first draft): "base layout is often a function of the available surface area. In the
8v8 games in particular, our existing sprawl runs into contention very quickly and most 8v8 attempts involved many
blocked-in units. Responsibility for avoiding block-in extends to avoiding blocking in other seats' facilities as
well. In other cases, the maps are populated with sparse islands and force most construction to be at sea ... We will
not always have the luxury of a relatively wide open base area like comet catcher." So the plan begins with what it
has to work with, not with the districts.

- **The ground budget is the first input.** Before any district is laid the layer computes the buildable ground of
  the seat: the cells that take a footprint (the slope tolerance of the kinds it will build, F-map-build; the water's
  depth for what floats or stands under water), less spots and their extractor footprints, less allies' structures and
  the claims round their starts (the game's start-box claim per seat, `CLAIM_RADIUS`, the territory bot's T-ALLIES),
  less every plant's street (ours and, as far as their facing shows it, allies'). The districts are fitted to that
  budget in order of what they are worth: the stand (fixed by the start), the plant district with its street, then the
  block as large as the budget allows at the chosen pitch, then the generators' rows in what is left. A short budget
  changes the block's size and shape (a bar is a bar in any direction, bent round what is there), never the streets:
  a street is the last thing to give, since a penned plant's output is worth nothing. When the budget cannot hold a
  district at all the layer says so in the report ("no ground for a block within 1,500 of the advanced plant; the
  nearest ground that holds one is at D6") and the player decides whether to move the plant or the base.
- **Allies' buildings are obstacles and allies' plants are streets we must not close.** Measured (`run/base_layout.py
  pen --allies`, our four Glitters 8v8s, 32 seats, 61 plants): with our own buildings alone 10 plants closed to some
  size, with allies' structures counted 17, and six closed to Bulls (size 4), one at 2:00, four by 7:00, seat 7's plant
  in three of four games at the same place (1184, 2030). The sizes we make (2 and 3) still pass on the grid, so the
  "many blocked-in units" the user saw come from what the grid does not carry: allied and own units standing in the
  gaps, wrecks, the engine's pathfinder refusing at its coarse resolution what the fine grid allows, and enemy
  structures. The street check therefore counts allied structures as obstacles (the `al` samples the host already
  sends), treats every allied plant's lane as a street of its largest product's width that no placement of ours may
  close, and leaves a margin of one square round allied footprints. An ally's building that closes one of our
  streets is reported as such ("street from plant_12 closed by an allied armsolar at C4"), for the player to say in
  the team channel; we do not reclaim allies.
- **Water is ground with its own footprints.** On a map whose buildable land is islands, the stand and the plant
  district may be on water from the start (the hands already place the on-water kinds at the nearest water,
  H-ECO-TIDAL): the tidal generator, the shipyard and the amphibious plant, the floating converter, radar and
  turrets, the naval construction turret (`armnanotcplat`, the block's turret at sea), the underwater fusion, storages
  and advanced converter, the floating hover platform, the underwater gantry. A sea district is a lattice on water
  deep enough for its kinds (each has a minimum and maximum depth; the ship classes their own), with a shipyard's
  street a ship's footprint wide through water that ships can take, and the block's bars of naval turrets round the
  underwater fusions. The same code, a different passable set and a different kind table; the ground budget is
  computed over land and water together, each kind over the cells it can stand on. Judge: the sea maps in the
  catalogue (Cape Violet, SailAway, Isidis) under the same instruments, which need the water depth added to the pen
  check's passable set.

### 1.1 Districts

A district is a region with a lattice and a purpose. Three kinds:

- **The stand** (the commander's, from the start). The slots within the arm's reach of the start position (145 plus
  the buildee's radius, about 190 to a solar's centre), packed at footprint on the 16-elmo grid, spots left for their
  extractors, the first plant's slot and facing among them. Computed at the start from the start position, the spots
  and the terrain, before the first order. The commander's opening steps are matched to the stand's slots in list
  order; a step whose slot is in reach now is ordered now with `queue: true`, so the whole stand is one queue and the
  commander never idles between structures. The pool's number to beat: 0.6 s from a structure finishing to the next
  order, one or two stands up to the plant.
- **The plant district** (one per land plant). The plant with its facing (toward the enemy as now, or the player's
  mark), two or three turret slots beside it (the pool's 56 percent beside, never in front), an apron of 100 in
  front (the pool's clearance; the 320 lane goes), and a **street**: a corridor from the apron to the open, as wide as
  the plant's largest product's movement footprint (a bot lab 3 squares, a vehicle plant 4, an advanced lab 6, an
  advanced vehicle plant 7, a gantry 7), kept free of everything but extractors on spots. The street's line is the
  shortest walkable line from the apron to ground outside the base's districts, chosen when the plant is placed and
  bent round spots.
- **The block** (one, from the advanced plant). Laid when the advanced plant is placed, on the flattest ground within
  the base that holds it, centred so that the plant's slot is in it: bars of turret slots alternating with frame rows
  for the things the block builds (advanced converters, fusions, the later plants, the gantry), each frame slot having
  the most turret slots within 400 of it. **Sparse first, back-filled later** (the user's ruling, 2026-10-29): the
  turret slots are a 48-pitch lattice of which only the 96-pitch sub-lattice (every other slot in both directions, a
  quarter of the cells) is offered at first, so no turret stands within another's blast (at 96 a neighbour's surface
  is 72 from the exploder, past the 64: the sum over every dead neighbour is zero; at the territory bot's 64 a dead
  row behind puts 297 on the next turret, under its 560, but all its neighbours dead put 1,189, so a hole spreads
  though a front does not; at 48 a dead row puts 920 and the block is a train) and up to 68 reach a frame, above
  thebluegecko's 46 to 53 in use; the other
  three sub-lattices are the back-fill, offered when the player says the block may densify (a word on the block's
  mark, `block dense`; the territory bot densifies when its program wants more turrets in reach than the sparse slots
  give). The frame rows, streets and blast slots are reserved in the sparse phase, so densifying changes nothing but
  the turret count. The people's 48 is the end state; the checkerboard is the road to it while anti-air is thin. The block has blast slots (§1.3) and its own streets: every plant placed in it gets the plant district's
  street, and a gantry's slot is reserved with a 7-square street from the first bar. Solars and other T1 structures
  standing on block slots are listed as **reclaim-for-ground** in the order the block grows into them: the plan says
  which solar goes when the slot it stands on is next.

- **Banks** (the generators and converters: solar, advanced solar, wind, tidal, basic converter; the user, 2026-10-29:
  "usually placed in banks by constructors (or sometimes the commander) rather than in range of the construction
  turrets, although key ones can be ... Humans typically build them in clusters of various sizes according to what is
  easy to arrange with the games' placement tools. This is in contrast to our current loose basic solar sprawl which
  currently is what takes up the majority of our base surface area"). A bank is a lattice of one kind at its footprint
  (solars 80, advanced solars 64, winds and converters 48, tidal on water), of a bounded size, with a vehicle street (3
  squares) between banks. A generator step goes to the nearest bank of its kind with a free slot that the builder
  reaches with a short walk, else founds a bank at the builder's lattice point nearest its work: banks grow, nothing
  scatters. Measured against the people (`docs/studies/2026-10-29-base-layout.md` §9): the pool's solars stand in
  clusters of up to 7 (median largest), thebluegecko's of 10, 26 solars a seat; ours 42 solars a seat, every one a
  singleton at linkage 88, never touching. The bank's size bound and pitch are the chain precaution, and the chain
  criterion is the sum of the blasts a cell takes from every dead neighbour within reach, not one blast against its
  health (the user, 2026-10-29: "two adjacent structures may blow, chain to a third, and continue through the block").
  By that sum, at full health, over a square lattice (`run/base_layout.py chain`, the study's §9 table): solars and
  advanced solars never chain at any pitch (146 and 283 from all eight neighbours dead, against 340 and 1,130); wind
  turbines chain through a touching bank (a dead row behind puts 245 on the next turbine, against 196) and are safe
  only at one footprint between them (pitch 96: 127 from every neighbour dead); basic converters chain at every pitch
  up to 96 (a dead row puts 182 on the next, against 167), advanced converters at every pitch up to 128 (610 against
  445); energy storages chain each other under about 260 between centres. So solar banks may be any size; wind banks
  are laid at pitch 96 or kept small; converter banks are small (the pool's touching clusters are 3 to 4; six at most)
  with the street between them as the firebreak; storages stand apart from one another and from every bank (a
  storage's 980 to 210 kills solars within 137 of its wall, turbines within 168, converters within 174); and a fusion
  (kills solars within 209) is not a bank's neighbour. Advanced converters are the block's: they are "brought up at speed late game with the advanced
  fusion reactors that power them" and take the frame rows beside the farm, in rows whose length is their bill (a row
  at 64 chains end to end). An advanced solar that is wanted at speed early goes to a block or plant-district slot as a
  frame; otherwise to its bank. Banks are the first ground the block grows into: a bank's slots are listed for
  reclaim-for-ground as the block reaches them, the way his seventeen solars went.

Extractors stay on spots, exactly as now. Defences stay at the player's places (the line 650 ahead and the defence
120 toward the enemy go; a defence without a place is refused as now).

### 1.2 The street rule

No placement may close a street. Before a site is asked of the engine, the layer runs the ground check: with the
candidate footprint added to the ground model, every plant's door still reaches the open with clearance for that
plant's largest product (the micro engine's ground carrying the structures by footprint, `terrain::clearance` over it,
a `Field` from the door with the clearance bound). A site that fails is moved to the next slot. This is the one rule
that holds when the plan is disturbed (spots, slopes, losses, the player's own marks), and it is what the fencing-in
instrument measures after the fact.

### 1.3 Blast slots, a second criterion

The user's ruling (2026-10-29): avoiding chain death is "a second or third optimization criteria rather than a
requirement"; fusions and advanced fusions "are critical enough to build quickly that placing turrets near them is
usually the acceptable penalty. The blast will kill any construction unit anyway, so optimizing that comes down to
limiting how much of our economy it takes with it. The balance with stand-up time is critical too, so in many cases the
danger is not enough to dissuade building it right next to the active con turret farm." So a frame slot is chosen
first by stand-up time (the turrets in reach of it), among slots within a few seconds of the best by the metal of ours
its death blast would take (the falloff arithmetic of D-base-layout: what stands inside 606 for an advanced fusion,
240 for a fusion, 210 for a storage), and only then by distance. A fusion or an advanced fusion goes beside the active
farm; the layer's part is the side of the farm whose blast holds the least of the economy (a bar's end rather than its
middle, away from the other fusions and the plants). Spacing buys nothing between the big reactors at any pitch a
block allows: one advanced fusion's blast kills another at one footprint apart (Armada's 8,300 falls within 190 of
surface; Cortex's 9,800 within 48, and two neighbours' blasts within far more), and three touching fusions in a row put
9,800 on the next against 3,800, so the reactors stand where stand-up time puts them and the report carries the bill. An
energy storage keeps 90 from turrets where a slot allows and is otherwise placed; converters touch, as the people's
do. The gantry's ring (turrets 122 or more from its wall, reach to 472) costs nothing in stand-up time and is kept.
The report states the bill of each big frame as it is placed ("afus_12 placed: 59 turrets in reach; its blast holds 31
turrets and 2 converters, 8,100 metal") so the player sees what it accepts; the layer refuses no slot for its bill.

### 1.4 What the player says

Nothing new is required of it. Its marks name the three sites when it wants to: a mark for the plant (`plant_yard`,
as it writes `avp_yard` today) sets the plant district's place and facing; a mark for the block (`block`) sets the
block's centre when the advanced plant is placed; without marks the layer chooses as §1.1 says. A `queue` step with a
place is an anchor into the district that holds the place; a bare step goes to its kind's district (a solar to the
stand while the stand has slots, else the nearest district with a solar slot; a turret to the block, else beside the
nearest plant; a fusion to a blast slot). The report gains one line a plant, its street's state ("street from
plant_12: open for everything it makes" / "closed for Bulls by armsolar_3041; reclaim it to open"), so the player can
act when a street has been closed by what the layer could not prevent (an enemy frame, an ally's building, a wreck).

### 1.5 Where the code lives

A new shared crate, `crates/layout`, with no engine or Jev dependency: districts, lattices, streets, blast slots,
the reclaim-for-ground list, the street check over a ground model it is given; unit facts (footprints, blasts,
health, movement footprints) from the glossary and the unit definitions the host already sends. `crates/bot` consumes
it through `Plan::Slot(def, site, facing)` in `place_planned`, asking the shim for the slot `exact` (the territory bot's
way, T-SITES). `bots/territory` replaces `farm_slots` with the block district (a change to both regimes, as a shared
crate is; the bot's planner keeps deciding what and when). `beside_builder`, `BACK_FIELD`, `TURRET_LINE`,
`GENERATOR_HOME`, `LAB_YARD` and the 320 lane are deleted, not kept behind a flag.

## 2. Steps, in order, each with its judge

1. **The stand and the queue.** `crates/layout` with the stand district; the lists order every step whose slot is in
   reach with `queue: true`. Judge: `run/base_layout.py pool` on the new games against the pool, gaps (0.6 s), stands
   (one or two), the plant's clock (Hellontoast's 0:54 on the same sequence).
2. **The ground budget, the street rule and footprint packing.** The buildable ground computed with allies' structures,
   claims and lanes taken out; the plant district with apron and street; the ground check before every site, allied
   structures and allied streets in it; solars and converters at footprint in rows. Judge: `pen` and `pen --allies`
   (nothing penned at any size a plant makes, ours or an ally's, through the game, in the 1v1s and in an 8v8 against
   BARb on Glitters), nearest-neighbour spacing (solars 80, converters 48), the lane's contents (nothing within 100 of
   a door), the report's street lines when an ally closes one.
3. **The block.** Laid at the advanced plant; frame slots by turrets in reach; blast slots; reclaim-for-ground.
   Judge: `block` (turrets in reach when a frame is placed, frame seconds against his 46 / 5.5 s and 53 / 35 s; the
   sparse lattice's 96 before the player's `block dense`, 48 after; the chain bill reported, not bounded), and the
   economy marks the territory bot is already judged by.
4. **The territory bot on the shared crate, if the crate looks simple then** (the user: "may adopt the crate in this
   arc if it looks simple, and may be delayed if not"). T-FARM replaced; judged by its standing marks (terr202-207's
   reference rows) not moving down.
5. **Water.** The sea kinds' depth table, the passable set over water, the shipyard's street, the naval turret's bars;
   the pen check given the water depth. Judge: a game on Cape Violet or SailAway, the stand's gaps and the pen check at
   sea, and nothing of ours on land that the budget said was water's.
6. **Transport of turrets**, when the hand exists: the block follows the frames. Not designed here.

Each step is a batch of player games on Comet against BARb hard_aggressive (the arena target), the instruments run on
their records, and a ledger row; nothing moves to the next step while the judge of the last is unmet.

## 3. What this does not do

It does not decide what to build or when (the player's lists, the territory bot's program). It does not place for
allies; it reads their buildings as obstacles and their plants' lanes as streets, and never reclaims theirs. It does
not model wrecks or standing units as obstacles in the street check beyond what the ground model already carries,
which is where the 8v8 block-ins the grid does not show must come from; if they persist after step 2 the check gains
the wrecks and the standing units' positions from the samples. It does not decide when the block densifies (the player's word). It leaves the facing of a plant toward the enemy unless the player marks otherwise.

## 4. Decisions taken (the user, 2026-10-29)

- **Pitch: sparse back-fill.** The 96 sub-lattice first (no turret within another's blast, 68 in reach of a frame),
  the 48 lattice filled in on the player's word once anti-air is robust; the people's 48 is the end state.
- **Chain death is a second or third criterion**, never a requirement: stand-up time first, the blast's bill among the
  slots within a few seconds of the best, the bill reported; fusions and advanced fusions beside the active farm.
- **The territory bot adopts the crate in this arc if it looks simple** when step 3 is done, and is delayed if not.

## Status

Design written 2026-10-29; nothing built. The research it rests on: `docs/briefs/derived/base.md`,
`docs/studies/2026-10-29-base-layout.md`, `run/base_layout.py`.
