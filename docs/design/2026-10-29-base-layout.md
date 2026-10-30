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
  the base that holds it, centred so that the plant's slot is in it: bars of turret slots (the people's: four turrets
  wide, pitch 48; or pitch 96, the user's decision, §4) alternating with frame rows for the things the block builds
  (advanced converters, fusions, the later plants, the gantry), each frame slot having the most turret slots within
  400 of it. The block has blast slots (§1.3) and its own streets: every plant placed in it gets the plant district's
  street, and a gantry's slot is reserved with a 7-square street from the first bar. Solars and other T1 structures
  standing on block slots are listed as **reclaim-for-ground** in the order the block grows into them: the plan says
  which solar goes when the slot it stands on is next.

Extractors stay on spots, exactly as now. Defences stay at the player's places (the line 650 ahead and the defence
120 toward the enemy go; a defence without a place is refused as now).

### 1.2 The street rule

No placement may close a street. Before a site is asked of the engine, the layer runs the ground check: with the
candidate footprint added to the ground model, every plant's door still reaches the open with clearance for that
plant's largest product (the micro engine's ground carrying the structures by footprint, `terrain::clearance` over it,
a `Field` from the door with the clearance bound). A site that fails is moved to the next slot. This is the one rule
that holds when the plan is disturbed (spots, slopes, losses, the player's own marks), and it is what the fencing-in
instrument measures after the fact.

### 1.3 Blast slots

From the falloff arithmetic (D-base-layout, "The chain bill"): a fusion's slot is in the ring 190 to 448 from the
turrets that build it and one footprint from any other fusion; an energy storage at least 90 from any turret and 180
from another storage; converters touching each other are accepted (the people's; a converter's death takes
converters, not turrets); an advanced fusion's death takes every turret within 606, so its slot is where that costs
least, at a block's edge, and the policy for it is the user's (§4): accept the bill as the people do, or place it by
aircraft or constructors beside a block that reaches it with few turrets. The gantry's slot keeps turrets 122 or more
from its wall (its 1,330 kills a turret within 121) and builds with the ring to 472.

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
2. **The street rule and footprint packing.** The plant district with apron and street; the ground check before every
   site; solars and converters at footprint in rows. Judge: `pen` (nothing penned at any size a plant makes, through
   the game), nearest-neighbour spacing (solars 80, converters 48), the lane's contents (nothing within 100 of a door).
3. **The block.** Laid at the advanced plant; frame slots by turrets in reach; blast slots; reclaim-for-ground.
   Judge: `block` (turrets in reach when a frame is placed, frame seconds against his 46 / 5.5 s and 53 / 35 s; pitch
   as decided; the chain bill as computed), and the economy marks the territory bot is already judged by.
4. **The territory bot on the shared crate.** T-FARM replaced; judged by its standing marks (terr202-207's reference
   rows) not moving down.
5. **Transport of turrets**, when the hand exists: the block follows the frames. Not designed here.

Each step is a batch of player games on Comet against BARb hard_aggressive (the arena target), the instruments run on
their records, and a ledger row; nothing moves to the next step while the judge of the last is unmet.

## 3. What this does not do

It does not decide what to build or when (the player's lists, the territory bot's program). It does not place for
allies or read their buildings except as obstacles. It does not model wrecks as obstacles in the street check beyond
what the ground model already carries. It does not choose between pitch 48 and 96 or the advanced fusion's policy
(§4). It leaves the facing of a plant toward the enemy unless the player marks otherwise.

## 4. Decisions for the user before step 3

- **Pitch**: 48 (the people's; 274 turrets in reach of a frame; a train once two adjacent turrets die) or 96 (68 in
  reach, still above his frames' median of 46 to 53; no turret-to-turret chain; a vehicle street between every two
  rows; four times the flat ground for the same count).
- **The advanced fusion**: accept the bill inside the block as the people do, or keep it to a block's edge placed by
  aircraft or constructors with few turrets in its 606.
- **Whether the territory bot adopts the crate in this arc** (step 4) or keeps T-FARM until the flagship's block is
  judged.

## Status

Design written 2026-10-29; nothing built. The research it rests on: `docs/briefs/derived/base.md`,
`docs/studies/2026-10-29-base-layout.md`, `run/base_layout.py`.
