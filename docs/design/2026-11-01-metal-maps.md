# Metal maps: places without spots, extractors in the layout (2026-11-01)

Status: proposed 2026-11-01 night; step 0 (the shim) built 2026-11-02 (`crates/ai-host/src/metal.rs`; judged by
speedmetal-probe-2, fmp-probe-1, metal-step0-comet-check); step 1 (the coverage, the distilled bot) built 2026-11-02
(`crates/layout/src/coverage.rs`, distilled `free_site`; judged by fmp-distilled-hard-1, metal-step1-comet-check);
steps 2-4 not started. The user's ask: "let's work on making metal maps play decently. I
have never tried it due to fears of the potentially large [number of] metal spot[s] breaking the mex waypoint system.
Let's review how those maps work, how our systems work with them, and what needs to change." The user's two rulings on
the shape (the same night): "we should make sure we can handle maps without a convenient grid of mexes to be used as
waypoints", and "we should have mexes participate in the layout engine when necessary. We already have to carefully
plan out energy converter placements in each of our bots, and mexes should probably be treated mostly the same. There
are also maps with large regions with no mexes to waypoint with, so the solution to that problem may be common there."

## 0. What is measured (read and probed 2026-11-01, HEAD ca72f42; ledger speedmetal-probe-1)

**The game.** BAR's spot finder (`upstream/Beyond-All-Reason/common/upgets/api_resource_spot_finder.lua`) declares a
metal map by a name list (l.33-40: SpeedMetal BAR V2, Oort Cloud V2, Asteroid Mines V2.1, Cloud9 V2, Iron Isle V1,
Nine Metal Islands V1) or when any contiguous strip of metal is longer than six extractor radii (l.241, l.352-354).
It then sets the rules param `mex_count` to -1 and publishes no spots (l.207-223); there is no `isMetalMap` param.
The snap widget (`cmd_extractor_snap.lua:184`), the area-mex command (`cmd_area_mex.lua:276`) and the mex denier
(`cmd_mex_denier.lua:40-44`) switch themselves off: extractors go anywhere, duplicates are allowed.

**The engine.** The metal map is one byte per 16-elmo square times the map's `maxMetal`
(`rts/Map/MetalMap.cpp:82`). An extractor claims every square whose centre lies within the map's `extractorRadius`
(`rts/Sim/Units/UnitTypes/ExtractorBuilding.cpp:128-146`) and earns `extractsMetal` times the square's metal for each,
but only the depth above what another extractor already claimed (`MetalMap.cpp:98-113`, `RequestExtraction`): a
second tier-1 extractor over a claimed square gets nothing; a tier-2 (depth 0.004 over 0.001) gets the difference,
which is why upgrading in place works. A mex's centre sits on a square corner (footprint 4 by 4, 64 elmos), so the
squares it claims span 32 elmos at radius 24 and 80 at radius 30, and the nearest square a neighbour 64 away could
contest is 40 from the centre: on today's metal maps touching mexes share nothing. Sharing bites only past a radius of
about 41 (Comet's 120, the engine default 500), where the game's spot list pins mexes anyway. The engine's own spot
analyzer (`rts/Sim/Misc/ResourceMapAnalyzer.cpp`, `maxSpots` 10,000, `minIncomeForSpot` 50) has no metal-map case: it
packs spots greedily one radius apart until the cap.

| map | size (elmos) | `maxMetal` | radius | metal squares | squares a mex claims | metal/s a tier-1 mex |
|---|---|---|---|---|---|---|
| Comet Catcher Remake 1.8 | 8,192 x 6,144 | 0.75 | 120 | 960 of 196,608 (0.5 %), 80 spots | the patch (about 12 within reach) | 2.3 |
| SpeedMetal BAR V2 | 13,312 x 2,048 | 10 | 30 | 50,863 of 106,496 (47.8 %): the end plateaus and the middle rows full, strips between | 12 | 30.6 (measured: 63.2 with two and the commander) |
| Full Metal Plate 1.7 | 12,288 x 12,288 | 7.5 | 24 | all 589,824 at value 79 | 4 | 2.32 (the client's +2.4) |

(`smf_metal.py` in the session's scratchpad reads an `.smf`'s metal map; the maps' `mapinfo.lua` give the scale and
radius.) The two metal maps are different games: SpeedMetal floods a dozen mexes' worth from the second minute,
Full Metal Plate is a carpet of hundreds each worth a Comet spot. In both, metal is free and energy, build power,
footprints and roads are the economy. BARb thins the engine's packing to about 80-240 pseudo-spots (CircuitAI
`src/circuit/resource/MetalManager.cpp:144-182`) and is known to drown in metal and overbuild converters (BAR issue
1764).

**Ours, on SpeedMetal (the distilled bot against NullAI, 1.3 minutes).** The shim reads `mex_count` -1 as none and
falls back to the engine's list (`crates/ai-host/src/engine.rs:152-166`): 10,000 spots. Its nearest-spot assignment
of every metal square (`engine.rs:178-198`) is squares times spots: hello took 7,444 ms (Comet 5 ms); in a lobby game
that holds the engine's init. `SPOT_KEEPOUT` 100 round every spot (`engine.rs:763`) refused every solar on the
plateau with `possible_at_exact=true`: today no bot of ours can build anything but extractors on metal ground. Nothing
panicked. What breaks at thousands of spots in the flagship: `brain/routes.rs:119-128` builds one full-grid Dijkstra
field per spot per movement class (10,000 x 426 KB a class on SpeedMetal); the `map` tool
(`brain/briefing.rs:953, 1028-1063`), the report's per-cell spot lists (`strategist/report.rs:72,105,113`) and the map
sheet (`terrain/src/sheet.rs:111-182`) are O(spots) text to the model; the picture's spot scan
(`pianist/picture.rs:1285-1326`), the stand's taken list (`brain/stand.rs:39-40`), the region partition
(`buildorder/src/regions.rs`) and the start search (`start.rs:42-56`) are O(spots x units) a second or O(spots^2).
With zero spots nothing panics and no bot ever orders an extractor: siting is "snap to a listed spot, offset inside
its squares" (`brain/economy.rs:274-302`, `layout::extractor_site`; territory `hands.rs:2083`, distilled
`economy.rs:725-740`, template `lib.rs:54-60`). `Terrain::metal` (`bot-protocol/src/messages.rs:197-200`) already
carries the raw metal map to the bot; only the viewer reads it.

**The two jobs a spot does for us.** Income site, and place: the player's `spot_N` vocabulary, the hands' walk and
build moves, the lane's rove goals, the route fields, the picture's free and enemy places. On a metal map the first
job has no spots and the second has ten thousand; on a normal map's empty quarter (Glitters' middle, Great Divide's
plateau edges) the second job has none. The design separates them.

## 1. The design

### 1.1 The shim: what a metal map publishes

- A map is a metal map when the game says so (`mex_count` -1) or, for a game without the finder, when the engine's
  list reaches its cap or the metal squares cover more than a tenth of the map. The engine's list is never published
  as spots: on a metal map `metal_spots` is empty.
- `Hello` gains `metal_fields`: the connected regions of metal squares (8-neighbour, merged across gaps under one
  extractor diameter), each with a name (`field_N`, west to east), its metal-weighted centre, its bounding box, its
  squares' total worth, and its *room*: about how many extractor footprints fit (its squares over sixteen; the
  ground's slopes and the buildings on it not counted). Built 2026-11-02: both metal maps are **one field**, since
  SpeedMetal's strips connect its plateaus and highway even square to square (checked at gaps of 1, 2 and 4 squares),
  and Full Metal Plate is uniform; fields tell islands apart (Nine Metal Islands, untested), and on a connected map
  the cells' metal share (§1.2, step 2) is the summary that tells places apart. Fields are the income summary, not
  the waypoints: "field_0: room for 3,178 at 30.6 each" (SpeedMetal), "room for 36,864 at 2.37" (Full Metal Plate).
- `MapInfo` gains `extractor_worth`: metal a second a tier-1 extractor draws on full metal here (squares claimed at
  the radius x 255 x `maxMetal` x `extractsMetal`): 30.6, 2.32. The brief and the sheet say it; nothing derives it from
  the spot list.
- The square assignment is bucketed by a grid of the extractor radius (O(squares)); hello on SpeedMetal under 50 ms.
- `SPOT_KEEPOUT` applies to published spots only, so none on a metal map. What keeps the yard open there is the
  layout's streets and exit lanes (`docs/design/2026-10-29-base-layout.md`, §1.2), the same as for any building.
- `Terrain::metal` stays the coverage source for the bot (§1.3); the record's header keeps `metal_spots` (empty) and
  gains the fields, so the viewer and the scripts know a metal map when they read one.

### 1.2 Places without spots

- **The place layer** is the set of things a packet, a menu, a list, a rove or a route can name: home, the passages,
  the marked places, the metal spots or fields, and *every cell of the grid*. Today the cell grid is a fixed 8 by 8
  whatever the map (`scout.rs` `CELLS`, `mcp.rs:281-310` `/ 8.0`, `sheet.rs:84-89` `input.grid`, `report.rs:42`):
  1,024 by 768 on Comet, 1,664 by 256 on SpeedMetal, 1,920 by 1,280 on Glitters Extended. **Cells become a fixed
  1,024 elmos square** (the user, 2026-11-02: "a fixed elmo count rather than a subdivision quantity. Larger maps
  should have larger cell counts"), columns `ceil(width / 1024)`, rows likewise, one grid definition shared by the
  scout, the report, the sheet, the `mark` tool and the picture. Comet's columns stay A-H and its rows go from 768 to
  1,024 (six rows); the generated sheets follow, the brief's few prose cell names are re-read.

  | map | elmos | cells at 1,024 |
  |---|---|---|
  | Great Divide | 3,072 x 4,096 | 3 x 4 = 12 |
  | Comet Catcher | 8,192 x 6,144 | 8 x 6 = 48 |
  | Quicksilver, Boreal Falls | 7,168 x 7,168 | 7 x 7 = 49 |
  | Center Command | 8,192 x 4,096 | 8 x 4 = 32 |
  | All That Glitters | 6,144 x 10,240 | 6 x 10 = 60 |
  | SpeedMetal | 13,312 x 2,048 | 13 x 2 = 26 |
  | Full Metal Plate, Supreme Isthmus | 12,288 x 12,288 | 12 x 12 = 144 |
  | All That Glitters Extended | 15,360 x 10,240 | 15 x 10 = 150 |
  | SD-129, DWorld | 14,336 x 14,336 | 14 x 14 = 196 |

  Single letters reach column X at 24,576 elmos, BAR's largest maps. What a grid of 150-196 cells asks of the
  consumers: the report's scouting lines are already runs down a column over the cells that hold something
  (`report.rs:43-80`), so they grow with content, not with cells; the scout's per-cell look (`scout.rs:97-110`) walks
  every tile per cell, O(cells x tiles) a glance, and is bucketed once; the picture offers cells by the packet's names
  and the group's neighbours (below), never all of them; the sheet's per-cell text is generated and read by the player
  once before the game. Great Divide's twelve cells are coarse for a 1v1, and there the spots and the marked places
  carry the fine names, as they do today.
- **Cells as destinations.** The menus offer `walk_to_<cell>`, `advance_to_<cell>`, `rove_within_<cells>` for the
  cells the packet names and the cells next to the group's own, the way `spot_N` moves are offered for the places the
  packet names (`diet::spot_runs` learns cell names); a list step takes a cell as its place; the lane's end for a
  cell is the cell's centre snapped to the nearest passable ground of the group's class (`Lane` already stops short on
  a blocked end, H-MICRO-STAND). The marked place missing from the group menus (the follow-ward arc's open item) is
  the same change.
- **Rove goals** are places, not spots: on a metal map the field centres and the cells of a region; on a spot map the
  spots as today (`brain/micro.rs:271`).
- **Route costs bounded by use, not by the count of places.** The per-spot fields (`routes.rs:52-56, 119-128`) are
  replaced by fields on demand: a field is computed on the routes thread the first time a place is asked for (10 ms a
  field, the comment at `routes.rs:52`) and kept in a bounded cache (home, the enemy's bases, the passages, the
  marked places and the last few dozen asked), evicted least recently used. "Nearest place by travel time" for a unit
  reads the fields the cache holds; a place with no field yet answers by the straight line this tick and has its field
  the next. On Comet the cache holds every spot within a minute of the survey, as today; on SpeedMetal it never holds
  ten thousand. `regions.rs` and `start.rs` (the opening planner) run over spots on a spot map and over the home
  field's room on a metal map (§1.3).

### 1.3 Extractors in the layout engine

- **Coverage, not a list.** `crates/layout` gets a `Coverage`: the metal squares from `Terrain::metal` with the map's
  scale, the radius, and a *claimed* layer (the extractors seen, ours, allied and his, each claiming the squares within
  its radius at its depth, the engine's rule). A tile's extractor worth is the unclaimed metal within the radius of
  its centre. A spot is a local maximum of this field, a metal map a plateau: one siting rule serves both.
- **The extractor want.** `Want::Extractor` becomes a want with a score like any building's: footprint 4 by 4 on
  passable ground, the street rule and the exit lanes respected, the chain sum counted, and its own criterion the
  worth of the tile, ties to the nearest the builder (or the stand). On a spot map the worth peaks only on the
  patches, so the stand's extractor slots are today's, and the game's pin (it refuses a mex off its published spot)
  is honoured by snapping to the published spot and offsetting inside the patch as `extractor_site` does now. On a
  metal map the want places a mex where a converter would go, on metal: for the opening, the stand lays the first
  mexes with the first solars from one standing place with the orders queued (the base-layout design's §1); after the
  stand, `Plan::Extractor(place)` asks the layout for the best tile within the place (a field, a cell, "near home") and
  the shim is given it `exact`.
- **Spacing is the score's.** On SpeedMetal and Full Metal Plate the claimed layer never overlaps a legal neighbour, so
  the rule reduces to "the next free tile of metal along the street grid"; on a map with a large radius and spread
  metal the same rule keeps the second mex off the first's squares. No spacing constant.
- **Duplicates.** The mex denier is off on metal maps; the claimed layer is our own denier (a tile whose worth is
  zero is not a site).
- **The planner** (`crates/buildorder`: the LP's rings, the annealer, the opening search) takes its extractor options
  from the coverage: a ring's worth is its unclaimed metal over its room in footprints, so on a metal map the ring
  nearest home has hundreds of equal options at `extractor_worth` each and the search is about build power and energy,
  which is the game there.

### 1.4 What the player reads

- The map sheet and the `map` tool: on a metal map no spot list; the fields with their room and worth, the
  `extractor_worth` line ("an extractor here draws 30.6 metal a second, thirteen Comet spots; metal is not what is
  short"), the cells with their metal share. Cells are the vocabulary for places on every map.
- The report: per cell, the metal share and the extractors ours, allied, his, in place of the spot list; free places
  as "room" not names.
- The picture: on a metal map `places` are the fields, the cells with ours or his, and the packet's names; the
  per-second scan is over extractors, not spots.
- The brief: a metal-map section in the experience brief (fact layer: the extraction rule and the two maps' numbers;
  derived: the carpet, energy as the constraint, converters worthless, build power the lever, the fight for the
  highway on SpeedMetal), and the income words made conditional ("extractors on metal spots are the income" is a
  spot map's sentence). The spending line and the frontier tables are per map already.

### 1.5 The heuristic bots

The same `Coverage` and want through the shared crate: territory's `free_spot` and its scenario of every spot become
the layout's site and the planner's rings over coverage (its LP is already over rings, not spots); distilled's
`free_spot` likewise; template's nearest-spot loop too. Their energy and converter placement is where the user says a
mex now belongs; nothing written for them goes into `crates/bot`.

### 1.6 Where the code lives

`crates/ai-host` (detection, fields, `extractor_worth`, the bucketed assignment, the keep-out); `crates/bot-protocol`
(`Hello::metal_fields`, `MapInfo::extractor_worth`); `crates/layout` (`Coverage`, the extractor want and site);
`crates/terrain` (the square cell grid, the sheet); `crates/bot` (routes on demand, places and cell moves, the picture
and report, the brief); `crates/buildorder` (rings over coverage); `bots/*` last. The record's header and
`docs/harness/record-format.md` carry the fields.

## 2. Steps, in order, each with its judge

0. **The shim's gaps** (§1.1): detection, no engine fallback, fields and `extractor_worth` in `Hello`, the bucketed
   assignment, the keep-out off. Judge: hello under 50 ms on SpeedMetal; the distilled bot builds a solar on the
   plateau; Comet's hello and spot list unchanged to the elmo. **Built and judged 2026-11-02:** SpeedMetal hello
   8 ms (was 7,444), three solars and a plant on the plateau in two minutes (speedmetal-probe-2); Full Metal Plate
   hello 73 ms, 2.37 a mex (fmp-probe-1); Comet's 80 spots identical to lane-thread-lockstep-1's header, hello 4 ms
   (metal-step0-comet-check). The bots still build no extractor on a metal map: that is step 1.
1. **Coverage and the extractor want** (§1.3), consumed first by the distilled bot (the user's project to iterate).
   Judge: on Full Metal Plate against BARb hard, extractors built per minute and income at 5, 10, 15 minutes against
   BARb's, no refused extractor site, the exit lanes clear (the yard instruments); on Comet the stand's extractor
   slots unchanged. **Built and judged 2026-11-02:** `layout::Coverage` (the metal map with the claims by depth,
   `worth_at`, `best_site` in rings round the builder, stopping at the first ring with a tile of full worth) and the
   distilled bot's `free_site` over it, the claims rebuilt every tick from what stands; `MapInfo::metal_scale` added
   for the absolute worth. Full Metal Plate against hard: extractors 14 / 32 / 47 / 57 at 4 / 7 / 10 / 15 minutes,
   income 40 / 113 / 147 at 5 / 10 / 15 against BARb's 26 / 50 / 111, no refused site (fmp-distilled-hard-1); Comet
   5 extractors by 3:00 on the spot path, income 13.5, as before (metal-step1-comet-check). The distilled bot sites its
   other buildings by the shim's search, not `layout::choose`, so the exit lanes are its own list and the yard
   instruments were not read; the flagship's `Plan::Extractor(place)` over the coverage comes with step 2's places.
2. **Places without spots** (§1.2): square cells, cell destinations, rove goals as places, route fields on demand.
   Judge: a flagship game on SpeedMetal with the pianist alone on a fixed packet reaches 20 minutes with `late` 0,
   memory flat, and the groups walk to named cells; on Comet a lockstep game's route choices unchanged (the
   route A/B script). **Build plan (2026-11-02), one commit each:** (a) `terrain::Grid`, the one grid definition
   (columns `ceil(width / 1024)`, rows likewise, the map divided equally so a cell is 1,024 on the usual sizes and
   never under 768; names `A1`..`X99`, parse, centre, rect, words, neighbours), consumed by `World`, the scout (its
   per-cell look bucketed into one pass over the tiles), the `mark` tool, the sheet, the report's runs (rows of two
   digits), the briefing's grid words, the sketch's labels, the record header and `player.md`; (b) route fields on
   demand: a cache keyed by walker and place, filled by a worker thread, bounded in bytes, the straight line until
   a field lands; `walk_to_spot_as`/`seconds_to_spot` become `walk_to_as`/`seconds_to` over a position; (c) cells as
   places: a cell the packet or a list step names is a `Place` (name the cell's, `cells` the one cell, so `rove_<cell>`
   is a region move already), and each group's menu offers its four edge-neighbour cells as well (the open question
   of §5: diagonals are two steps); rove goals on a metal map are the fields' centres and the cells the rover's class
   reaches, seen as the scout's look says; (d) the flagship's extractor over the coverage: `Brain::coverage` rebuilt
   each tick as the distilled bot's is, `Site::Tile` in the pianist, the `extractor` list step and the builder's
   `build_<extractor>` move siting by `Coverage::best_site` within the named place (anchor the place, reach a cell's
   half-diagonal; `nearest` from the builder), the engine told `exact`; the stand takes `Want::Extractor` from the
   coverage when the spot list is empty.
3. **What the player reads** (§1.4). Judge: a player game on Full Metal Plate and one on SpeedMetal against BARb
   hard: the packets name cells and fields, the hands build the carpet, the post-game review finds no spot words.
4. **Territory and template** (§1.5). Judge: the territory benchmark on Full Metal Plate against NullAI (extractors at
   5/10/15/20 as the standing numbers are on Comet).

Each step is a commit with its ledger row; the design is amended where a step finds it wrong.

## 3. Rejected

- **BARb's pseudo-spot grid** (a few hundred `spot_N` generated by the shim): every consumer keeps working, but a
  grid of hundreds on SpeedMetal is a tenth of the economy the map allows, the player reads a list that is not the
  map, and the keep-out and the per-spot fields still need taming at hundreds.
- **Fields as the places.** Full Metal Plate is one field; the waypoint problem needs the grid.
- **Capping the per-spot fields** (the first N spots): arbitrary, and still ten thousand spots in every list.
- **A spacing constant for mexes on metal maps.** The coverage score gives the spacing where it matters and nothing
  where it does not (§0: touching mexes share nothing at radius 24 or 30).

## 4. Decisions taken

- 2026-11-02 (the user): the cell is a fixed elmo count, 1,024 to start, so larger maps have more cells; checked
  against the largest maps played (Glitters Extended 150 cells, SD-129 196), §1.2.

## 5. Open

- Whether cells next to the group's own are enough as offered destinations, or Jev needs a cell's neighbours two
  deep on a map of 150-196 cells.
- Allied extractors on a metal map are claimed only when seen; a tile under an unseen allied mex is refused by the
  engine, not by us (the `no site` path already re-plans).
- Metal maps with a large extractor radius (the engine default is 500) where sharing does bite: none installed; the
  rule covers them untested.
- Energy on SpeedMetal: thirty mexes draw 90 energy a second before the first lab; the brief's spending words, or
  the planner's energy line, decide whether the carpet waits for power. A player question, not a code rule.
