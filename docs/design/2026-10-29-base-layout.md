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

### 1.1 Constraints, then bands: how a site is chosen

(Rewritten 2026-10-29 on the user's word, after the additions of §1.0, the banks, the roll-off and the chain sum showed
that every concern since the first draft was a criterion, not a shape: "rewrite 1.1 and 1.3 as constraints and bands".)
Nothing is drawn in advance but reservations. The lattice is the 16-elmo grid with its 48 and 96 sub-lattices over land
and water, its passable set the ground budget of §1.0. A building's site is the free lattice point that is best in the
order below, each level a band: a site within the band of the best at one level goes to the next level, and the first
level that separates the candidates decides. Bars of turrets, banks of generators and plants sharing a turret row then
come out where the ground allows them and bend where it does not, which is what the 8v8s and the island maps need; and
the report prints the chosen site's reasons, which a drawn pattern could not.

**Level 0, the constraints** (a site that fails any is not a candidate):

- The footprint stands: slope and depth for its kind over every square (F-map-build), no spot or extractor footprint
  under it, nothing standing, framed or ordered on it, the 16-grid snap.
- No street closed: ours (every plant's apron of 100 and its street, as wide as the plant's largest product, §1.2) and
  allies' (their plants' lanes at the same width), checked with the footprint added to the ground model; one square of
  margin round allied footprints.
- The reservations kept: the stand's slots for its kinds while the stand is being built; the frame rows and the gantry's
  slot and street at the block's centre once the advanced plant is placed; a street's cells never.
- The phase's sub-lattice for turrets: the 96 sub-lattice while the block is sparse, the 48 lattice after the player's
  `block dense` (the territory bot: when its program wants more turrets in reach than the sparse slots give). At 96 the
  sum of every dead neighbour's blast on a turret is zero; at 64 a dead row behind puts 297 on the next (under its 560)
  but every neighbour dead puts 1,189, so a hole spreads though a front does not; at 48 a dead row puts 920 and the
  block is a train.
- The kind's own band of distance: a converter or generator step goes no farther from its builder than a short walk (the
  people's banks are where the constructor was working); a frame no farther from the block's centre than the block's
  reach; anything the player anchored within the anchor's district.

**Level 1, the purpose** (the score the kind exists for; the band is the user's "within a few seconds of the best"):

- A frame (a fusion, an advanced fusion, an advanced converter, an advanced solar wanted at speed, a plant, the
  gantry, a storage): the build power in reach of the site, so stand-up time; the band a few seconds of the frame at
  that power.
- A construction turret: the pads and frame slots within its reach, weighted by what they will build; a turret that
  reaches two pads is worth more than one that reaches one (the roll-off gap of about a second a unit, 12 to 17 percent
  of a plant's cycle, is what a one-plant turret idles; the people's turrets reach two or more plants 72 percent of the
  time, ours 48).
- A plant: the turret slots and turrets its pad would share with the tier's standing plants (co-location: the people's
  same-type pairs 304 apart median, ours 647), and the facing's street cost (the shortest street to the open).
- A generator or basic converter: adjacency to its own kind, so banks grow where one stands (the pool's solars in
  clusters of up to 7, thebluegecko's 10, ours all singletons), under the kind's bank cap (§1.3: converters six, winds
  by pitch or six; solars and the other generators eight since 2026-10-30, the people's clusters of 7 to 10: block-2's
  bank of eighteen was a wall 700 long the commander walked round) and with a vehicle street of three cells (48)
  between banks; meeting at a corner is packing, not a gap.
- An extractor: its spot, exactly, as now. A defence: the player's place, as now.

**Level 2, the chain sum** (the user's second criterion): among sites within the band, the one whose own death would
take the least of ours by the blast sum (§1.3), and that adds the least to the sum its neighbours already bear.

**Level 3, the builder's walk** (since 2026-10-30 the way round what stands, one flood from the builder a choice, not
the line: a site across a bank read as near while the commander rounded the bank), then **level 4, distance to home**;
a tie after that is the lattice's order. A generator's level-1 band is one neighbour, so touching one beside the
builder and touching two across the base both reach the walk, which decides; founding a bank scores worse than touching
one by more than the band (the user's ruling, 2026-10-30, on block-2's commander).

**The stand** is the one place the levels are pre-solved: at the start, from the start position, the spots and the
terrain, the layer solves the commander's first slots (two extractors, solars, the plant with its facing and street)
within the arm's reach (145 plus the buildee's radius, about 190 to a solar's centre), so that the opening list's
steps are ordered as one queue (`queue: true`) and the commander never idles between them; the pool's 0.6 s from a
structure finishing to the next order and one or two stands up to the plant are the marks.

**The block's centre** is the one global decision: the player's `block` mark, else the flattest ground within the base
that holds the frame rows and the gantry's street beside the advanced plant's slot, found once when that plant is
placed; after that every slot in it is chosen by the levels like any other. Banks on that ground are listed for
reclaim-for-ground as the block's slots reach them, the way his seventeen solars went.

### 1.2 The street rule

No placement may close a street. Before a site is asked of the engine, the layer runs the ground check: with the
candidate footprint added to the ground model, every plant's door still reaches the open with clearance for that
plant's largest product (the micro engine's ground carrying the structures by footprint, `terrain::clearance` over it,
a `Field` from the door with the clearance bound). A site that fails is moved to the next slot. This is the one rule
that holds when the plan is disturbed (spots, slopes, losses, the player's own marks), and it is what the fencing-in
instrument measures after the fact.

### 1.3 The chain sum, the second criterion

The user's ruling (2026-10-29): avoiding chain death is "a second or third optimization criteria rather than a
requirement"; fusions and advanced fusions "are critical enough to build quickly that placing turrets near them is
usually the acceptable penalty. The blast will kill any construction unit anyway, so optimizing that comes down to
limiting how much of our economy it takes with it. The balance with stand-up time is critical too, so in many cases the
danger is not enough to dissuade building it right next to the active con turret farm." And the rule for the sum (the
user, the same day): "a two-dimensional grid does not require blast > hp to spread. Two adjacent structures may blow,
chain to a third, and continue through the block." So the chain criterion is the sum of every dead neighbour's blast on
a cell against its health (the engine's falloff to the collision surface, F-fight-death), computed by
`run/base_layout.py chain` (the study's §9 table), and it enters the choice at level 2, never as a refusal:

- *What the sum says by kind.* Solars and advanced solars never chain at any pitch (146 and 283 from every neighbour
  dead against 340 and 1,130). Wind turbines chain through a touching bank (a dead row behind puts 245 on the next,
  against 196) and are safe at one footprint between (127). Basic converters chain at every pitch to 96 (182 on the next,
  against 167), advanced converters at every pitch to 128 (610 against 445). Storages chain each other under about 260
  between centres. Turrets: 48 a train, 64 front-proof but not hole-proof, 96 nothing. Fusions touching chain by a
  dead row (9,813 against 3,800), one footprint apart only as a hole (1,324 behind, 4,702 all round). Advanced fusions
  chain at any pitch a block allows (56,195 from a dead row one footprint away; Armada's dies at one footprint from a
  single blast).
- *What follows for the levels.* The reactors stand where level 1 puts them, beside the active farm, and spacing buys
  nothing between them; level 2 only picks, among the sites within a few seconds of the best, the side of the farm
  whose blast holds the least of the economy (a bar's end rather than its middle, away from the other reactors and the
  plants). Bank caps are level-0 constraints for the kinds that chain: converters six to a bank with the 3-square street
  between banks as the firebreak, winds six or at pitch 96, storages apart from each other by 260 and from every bank
  (a storage's single blast kills solars within 137 of its wall, turbines 168, converters 174); solars and advanced
  solars have no cap. The gantry's ring (turrets 122 or more from its wall, reach to 472) is kept because it costs no
  stand-up time.
- *The bill is reported, not bounded.* Each big frame's placement line states it ("afus_12 placed: 59 turrets in
  reach; its blast holds 31 turrets and 2 converters, 8,100 metal"), so the player sees what it accepts.

### 1.4 What the player says

Nothing new is required of it. Its marks name the three sites when it wants to: a mark for the plant (`plant_yard`,
as it writes `avp_yard` today) sets the plant district's place and facing; a mark for the block (`block`) sets the
block's centre when the advanced plant is placed; without marks the layer chooses as §1.1 says. A `queue` step with a
place makes the place the step's level-0 distance band (the site is chosen by the levels within it); a bare step is
chosen by the levels with its kind's own band (a solar beside the solars its builder is near, a turret where it
reaches the most pads and frames, a fusion where the most build power reaches it). The report gains one line a plant, its street's state ("street from
plant_12: open for everything it makes" / "closed for Bulls by armsolar_3041; reclaim it to open"), so the player can
act when a street has been closed by what the layer could not prevent (an enemy frame, an ally's building, a wreck).

### 1.5 Where the code lives

A new shared crate, `crates/layout`, with no engine or Jev dependency: districts, lattices, streets, blast slots,
the reclaim-for-ground list, the street check over a ground model it is given; unit facts (footprints, blasts,
health, movement footprints) from the glossary and the unit definitions the host already sends. `crates/bot` consumes
it through `Plan::Slot(def, site, facing)` in `place_planned` (the layer: reserve, then choose by the levels of §1.1), asking the shim for the slot `exact` (the territory bot's
way, T-SITES). `bots/territory` replaces `farm_slots` with the block district (a change to both regimes, as a shared
crate is; the bot's planner keeps deciding what and when). `beside_builder`, `BACK_FIELD`, `TURRET_LINE`,
`GENERATOR_HOME`, `LAB_YARD` and the 320 lane are deleted, not kept behind a flag.

## 2. Steps, in order, each with its judge

1. **The stand and the queue.** `crates/layout` with the stand district; the lists order every step whose slot is in
   reach with `queue: true`. Judge: `run/base_layout.py pool` on the new games against the pool, gaps (0.6 s), stands
   (one or two), the plant's clock (Hellontoast's 0:54 on the same sequence). **Built 2026-10-29** (H-ECO-STAND):
   `crates/layout` (the build grid, footprints, the ground a footprint stands on by F-map-build with the engine's own
   height tolerance now sent per definition, the extractor's site in its patch, a factory's lane, `stand`), the
   bot's `stand.rs` (the stand solved when the commander's list first plays, from its leading steps up to the first
   factory), `Plan::Slot` asked of the shim exactly, and the pianist's queue made a queue (`Queued`, a deque per
   builder) so the lists issue the stand's steps in one tick. Judge pending: the first game is `stand-1`.
2. **The ground budget, the street rule and footprint packing.** The buildable ground computed with allies' structures,
   claims and lanes taken out; the plant district with apron and street; the ground check before every site, allied
   structures and allied streets in it; solars and converters at footprint in rows. Judge: `pen` and `pen --allies`
   (nothing penned at any size a plant makes, ours or an ally's, through the game, in the 1v1s and in an 8v8 against
   BARb on Glitters), nearest-neighbour spacing (solars 80, converters 48), the lane's contents (nothing within 100 of
   a door), the report's street lines when an ally closes one; the share of turrets in reach of two or more plants
   (the pool's 72 percent against our 48) and same-type plant pairs within 500. **Built 2026-10-29** (H-ECO-LAYOUT):
   `layout::choose` (the levels of §1.1 as written, the bands as constants, the street check of §1.2 as a windowed
   flood over the ground a plant's widest product stands on, `street.rs`), the bot's `layout_world` (ours, allies'
   with their facing now sent by the shim, his known buildings, the plants' aprons, the turrets' reach, the frames,
   the stand's free slots as reservations, the refused sites), every kind's purpose and band in `economy.rs`, the
   player's place as a band, the picture's `street` line per plant, the stand's own slots under the same apron and
   street check; `beside_builder` and the 320 lane deleted with their constants. The ground budget of §1.0 is implicit
   in the candidate set (what stands, allies' footprints with a margin, the aprons, the spots, the ground) rather than
   computed as a map; no "no ground for a district" line yet, since no district is drawn. Judge pending: `layout-1`.
3. **The block.** Laid at the advanced plant; frame slots by turrets in reach; blast slots; reclaim-for-ground.
   Judge: `block` (turrets in reach when a frame is placed, frame seconds against his 46 / 5.5 s and 53 / 35 s; the
   sparse lattice's 96 before the player's `block dense`, 48 after; the chain bill reported, not bounded), and the
   economy marks the territory bot is already judged by. **Built 2026-10-30** (H-ECO-BLOCK): `layout::lay_block` (`block.rs`): when our first advanced plant (a
   factory of glossary tier 2 or above) is ordered or stands, the block's centre is found once, at the player's `block`
   mark or on the flattest ground within 600 (doubled twice) of the plant that holds two frame rows (144 deep, three
   advanced-fusion slots each, between a bar of turrets through the centre and one 192 either side) and, when the
   advanced constructor builds a factory larger than the plant, the gantry's slot with its apron toward the enemy;
   clear of everything standing but our own banks (listed for reclaim-for-ground), off the spots and every apron,
   closing no street, among sites equally flat (4 elmos) the least of ours on it, then the nearest the plant; under a
   mark the nearest point to it that holds. From then on the world carries its rows as reservations a frame may take
   and nothing else, the gantry's slot as its kind's, and its sub-lattice for construction turrets (96 sparse, 48 after
   the player's `block` tool says `dense`); a turret's and a frame's bands are the block's reach (the turret's 400)
   from its centre. Among frame sites equally quick to stand (within the 3 s band) a row slot wins and the hands' line
   says "in a frame row" (block-5, 2026-10-30: the rows admitted frames but did not draw them, and both frames went
   beside the rows on the builder's walk, one of them onto the turrets' lattice). The frame's hands' line counts the turrets in reach and the chain bill by kind ("its blast holds 3
   armnanotc and 1 armmakr, 850 metal of ours"); the report gains the block's line (centre, pitch, turrets on it,
   frame slots free, the gantry's slot, the banks on its ground to reclaim) or, when no ground held one, why. The
   judge `run/base_layout.py block` reads the 96 bin and the share of turrets on one lattice. Judged in `block-5`
   (block-1 to block-4 found the fixes above or never played).
4. **The territory bot on the shared crate, if the crate looks simple then** (the user: "may adopt the crate in this
   arc if it looks simple, and may be delayed if not"). T-FARM replaced; judged by its standing marks (terr202-207's
   reference rows) not moving down. **Delayed 2026-10-30 (the user's word)**: read against the code it did not look
   simple. The farm is woven into the planner's turret capacity per plant (`plant_rings` → `LpOptions::plant_rings`),
   the hands' core (constructors sent home by `FARM_CORE`, the anneal's walk fields prepared at the farm's slots),
   the silos' siting and the dead-slot ledger, and the block needs an anchor without a tier-2 plant (the farm origin
   as its mark, the first factory as its plant). A scoped adoption (turrets on the lattice, big buildings as frames,
   cheap ones as banks, the origin, core and capacity kept) is half a day plus a two-arm bench, for an architectural
   gain and no claim on the marks. To be taken up after step 5 shows the crate holds at sea.
5. **Water.** The sea kinds' depth table, the passable set over water, the shipyard's street, the naval turret's bars;
   the pen check given the water depth. Judge: a game on Cape Violet or SailAway, the stand's gaps and the pen check at
   sea, and nothing of ours on land that the budget said was water's. **Built 2026-10-30**: the depth table is the
   engine's, not a table of ours: the shim sends every kind's `min_water_depth`, `max_water_depth` and `floater`
   (`UnitDef_getMinWaterDepth` and kin; the protocol's `UnitDefInfo`, the record's `unit_defs`), and `Ground::stands`
   applies the engine's own build rule (`GameHelper.cpp` `CheckTerrainConstraints`): every square's ground at a height
   in [-max, -min], within the kind's height difference of the centre unless the kind floats and the square is under
   water. So a land building stands on land alone (BAR's say max 0), a tidal or shipyard in water 20 or 30 deep, a
   tier-1 extractor anywhere, and the same `choose` places them all: the hands' nearest-water search for the glossary's
   `on_water` kinds is deleted, and the level-3 walk is now the builder's walk to within reach of the site (a shore for
   a water site), not to the site. A shipyard's street runs through water its widest product floats in (`Plant.sea`,
   the ship's move-class depth), whatever the land's passable set says. A block at a shipyard is of the sea kinds (the
   naval construction turret, the underwater fusion, the underwater gantry). The study's `pen` blocks water for land
   factories and shallows for shipyards; `water` lists our buildings against the depth under them. Not built: the
   ground budget over water as a map (§1.0 is still implicit in the candidate set), sea constructors' walks (the walk
   flood is the land classes'), the naval turret's bars as a separate lattice (the block's lattice serves). Judge
   pending: `water-1` on Cape Violet.
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

Design written 2026-10-29; §1.1 and §1.3 rewritten the same day as constraints and bands on the user's word; steps 1 and 2 built the same night (step 1 judged in stand-1: the plant at 0:58, one queue; step 2 in layout-1: banks touching at 80, three plants 181 and 202 apart, nothing penned), step 3 built 2026-10-30 and judged in block-5 the same day (the block laid at 6:25 on the flattest ground 746 from the plant, 7 of 7 turrets on the 96 lattice, the second advanced fusion standing in 186 s with 7 turrets in reach against 185 said; the frames went beside the rows rather than in them, fixed: a row slot wins among sites as quick to stand), step 4 delayed 2026-10-30 (the farm is not a simple swap; see its note), step 5 built 2026-10-30 (the engine's depths and build rule in `Ground::stands`, the shipyard's street at sea, the sea block; judge pending: water-1 on Cape Violet), step 6 not. The research it rests on: `docs/briefs/derived/base.md`,
`docs/studies/2026-10-29-base-layout.md`, `run/base_layout.py`.
