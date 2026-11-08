# What holds the turret farm is the player's call (2026-11-08)

The user, after econ-flagship-15: "The high level decisions here are things that the player should be making. The
layout engine exists to take the burden of exact 2d layout and placement, but the player should be able to make the
call for what deserves the turret farm or not. Let's find a way to make that elegant rather than adding more
heuristics." And the caution the design must keep at its centre: "the one thing that genuinely makes different
structures take different build area is what it takes to manage the scarcity of high-bp surface area. It's why we
convince the player to place solar farms elsewhere: a build turret area full of fusions is more valuable than a build
turret area full of basic solars. In that last game, there was plenty of space near the existing turrets to build more
turrets, more fusion, and more converters. Instead the constructors built new converters one at a time far out of
range." Ruled 2026-11-08: the rework, with a default list kept. This is the design, written before anything is deleted;
§5 lists the decisions it still needs.

## 0. What the code does today (read 2026-11-08, HEAD 6985f9cd)

**Who decides where a frame goes.** A list step without a place goes through `economy.rs` `place_planned`: the stand's
slot for its kind while the opening stand has one, else `band_for`, which asks `purpose_of` what kind of thing the
building is and hands the layout layer a centre and a radius: a generator within 300 of its builder, a frame within the
block's reach of the block's centre, a turret the block's lattice, a plant beside its builder. `purpose_of`
(`a344fec3`, 2026-10-29) sorts by unit data: a factory is a plant, a static builder a turret, a converter under 500
metal or a wind turbine a generator, any other energy maker under 500 metal a generator, a defence or radar a place,
everything else a frame. The layout layer (`crates/layout` `choose`) then scores a site by that purpose: a generator
for touching its own kind under a bank cap, a frame for the seconds it takes to stand under the build power in reach,
a turret for the frame slots and frames in its reach, a plant for the turrets in reach and its street.

**The block.** `layout/block.rs` lays it once beside the first advanced plant or at a `block` mark: a centre, a pitch,
a reach, and two rows of frame slots sized for the advanced fusion, growing by tiers of two rows when fewer than two
slots are free, to `MAX_TIERS` 3, eighteen slots in all; the rows are reservations only a frame may take. The block line
counts the free slots ("3 of 18 frame slots free in 3 tiers of rows"), and the sentence that a fourth tier would stand
past the turrets' reach so "a second block wants a `block` mark" prints only under two free.

**What this did in econ-flagship-15** (`run/matches/1794162747-econ-flagship-15`). The advanced converter costs 380
metal and 21,000 energy, so `purpose_of` classes it a generator and bands it beside its builder, although the
base-layout design (§1.1) and the layout crate's own `Purpose::Frame` doc name it a frame. Of the 66 advanced converter
frames begun from 15:00, 33 had no construction turret in reach: a median 132 from their builder and 971 from the
block's centre; 24 of them decayed. The 33 in reach were a median 572 from the centre, there because the builder was.
Fifty-four advanced converter frames decayed over the game. Meanwhile the block line said 3 of 18 slots free from
17:36 while the ground said otherwise (free ground not under any building of ours, before slopes and streets):

| econ-15 | Turrets | Block line said free | Free ground in reach of 30+ turrets | Of 10+ turrets |
|---|---|---|---|---|
| 15:00 | 57 | 7 of 18 slots | 17 advanced fusions or 38 converters | 73 or 164 |
| 17:00 | 77 | 4 of 18 | 24 or 54 | 68 or 154 |
| 19:00 | 82 | 3 of 18 | 25 or 57 | 67 or 152 |

The player at 15:30: "the hands pick where a converter goes, and I have no way to keep them inside the block"; at 18:16
"the block has only 3 of its 18 frame slots free, so frames are what's short", and it cancelled seven turret lists. Both
readings were of the code's making. The same rule put econ-14's advanced converters at their builders' spots ("constructors
that walked to their own spots put advanced converters there, where each built alone at under 1 % a second").

**Why kinds are the wrong shape here.** Three things the purpose decides, the band, the score and the bank caps, each
stands in for something the player can say or the engine can measure. What a structure asks of the farm's ground is its
build time per footprint square, the power the ground absorbs for the one time it is built (from the record's unit
definitions, footprints in 8-elmo squares as the shim reports them):

| Kind | Build time per square | Seconds under one turret |
|---|---|---|
| advanced fusion | 2,174 | 1,565 |
| advanced converter | 547 | 175 |
| fusion | 450 | 270 |
| advanced extractor | 233 | 74 |
| construction turret | 147 | 26 |
| advanced solar | 124 | 40 |
| basic converter | 72 | 13 |
| solar | 26 | 13 |

A fusion is worth seventeen solars of the farm's ground. The player knows this (the late-economy article teaches it);
the engine does not need to, and the user ruled that it should not: the pricing is the player's, the geometry the
engine's.

## 1. The design

The player says what deserves the farm, as a standing list and as a word on a step. The engine owns the two-dimensional
work: where on the farm a frame stands so that the most build power reaches it, where a turret stands so the farm grows
as one piece, what the streets and the blast chains allow. Nothing in the engine sorts buildings into kinds to decide
where they belong; the only kinds it knows are the ones that are geometry: a plant has a street, an extractor has a
spot, a construction turret has a reach, a defence stands at the player's place.

### 1.1 `holds`: the standing list

The `block` tool, which today sets the pitch, gains `holds`: the kinds whose frames go on the farm's ground when a
step names no place, in `tend`'s vocabulary and with `tend`'s shape (a list, most wanted first; `[]` holds nothing).
`{"holds": ["advanced_fusion", "fusion", "advanced_converter", "storage"]}` is the default and is what the block line
says every report, as `tend`'s order is said, so the player plays under a rule it can see and changes with one call.
A kind not held goes beside its builder, a short walk, which is where the people put their solars and basic converters
and where a frame that stands in thirteen seconds under its own builder loses nothing. Turrets are not a `holds` kind:
they are the farm. Plants and extractors are not: they have their own ground.

The vocabulary gains one word, `advanced_converter`, and `converter` then means the basic one: the two differ by a
factor of eight in build time per square and the whole of econ-14's last three minutes turned on the difference. `tend`
takes the same word in the same place (default order: turret, advanced_fusion, fusion, advanced_converter, converter,
extractor, generator, storage, building, advanced_plant, plant), so the two lists are read against one vocabulary.
`tend_category` already sees `d.converter`; the split is by capacity (600 a second is advanced, 70 basic), the figure
the conversion gadget itself uses.

### 1.2 The word on a step

`block` is a place word on any build step: `armmmkr block start` puts the frame on the farm whatever the list holds,
`armsolar block` likewise (the player may want a solar there in the opening, when the ground is plentiful). Any other
place keeps a held kind off the farm: `armfus home`, `armmmkr spot_12`, a marked place. The grammar already lets a
building take a place (`execute.rs` `at_the_orders_site`, the `places` the picture builds: home, the marks, the spots);
the farm's centre joins that list under the name `block` once the block is laid, which also makes `block` what the
`block` mark is called when the player lays it by hand. The queue tool's text says so in the sentence that lists the
places a step may carry.

### 1.3 The engine: the farm is ground in reach, not slots

The farm's ground is the free ground within reach of its construction turrets, read from the turrets standing and
framed (the layout world's `workers` already carry each turret's position, reach and power). A held frame's site is the
free lattice point on that ground where it stands soonest, the `Purpose::Frame` score as it is today (build time over
the power in reach plus the builder's own, the streets kept, the blast chain within its band), with the reservations of
frame rows gone: the rows were a hand-drawn stand-in for "where the turrets will gather", and the turrets have gathered.
The gantry's slot and apron stay as they are, a street matter laid with the block.

A construction turret's site is the lattice point, on the block's pitch, that adds the most free ground to the farm
within reach of itself and of the turrets already standing, adjacent to the farm so it grows as one piece (today the
turret score counts frame slots and frames in reach; the slots go, the frames stay, and free farm ground in reach
joins). That is exact placement, and it is the engine's. The pitch stays the player's (`sparse`, `dense`).

What is not held keeps the band beside its builder (300) and the layer's tie-break of touching its own kind, which is
tidiness, not a decision; the bank caps that stopped a bank at six or eight go, since the chain sum (§1.3 of the
base-layout design) already prices a bank that would kill itself and the player can read the blast words.

### 1.4 The report: room, in footprints

The block line stops counting slots. It says what the farm holds and how much room it has, in the footprints of the
held kinds at two power lines: "holds: advanced_fusion, fusion, advanced_converter, storage (`block` changes it); room:
about 24 advanced fusions or 54 advanced converters in reach of 30 or more turrets, 68 or 154 in reach of 10 or more".
The figures are the free-ground count of §0's table, which the engine computes from the same world it places over, so
the line and the placement cannot disagree the way the slots and the ground did.

When a held frame finds no room above the lower line, the hands' line says so and where it went: "armafus_1234: no room
on the block in reach of 10 turrets; placed at its edge with 4 in reach" or, on the player's word (§5), the step is
skipped and said. The decision that follows, a second `block` mark, a thinner `holds`, more turrets, is the player's,
and the words give it what it needs. The sentence about tiers and "a second block wants a mark" goes with the tiers.

### 1.5 What goes

`economy.rs`: `purpose_of`, `band_for` as a kind sorter, `BAND_FRAME`, `BANK_CAP` and `BANK_CAP_GENERATOR`,
`Purpose::Generator { cap }` as a decision. In their place one question: held, by the list or by the word, or not;
then the engine's own kinds (plant, turret, extractor, place) which are geometry. `layout/block.rs`: `frame_rows`,
`frame_slots`, `tiers`, `grow`, `MAX_TIERS`, `ROW_SLOTS`, `free_frame_slots`, `Reserved::Frames`; `choose.rs`: the
`frame_slots` field of the world and its use in the turret score, the "in a frame row" tie-break. `brain/block.rs`:
`grow_block`, `block_free_slots`, `block_grow_failed`, the tiers words. The test `kinds_have_their_purposes_and_bands`
is rewritten as the holds test. The base-layout design's §1.1 reservation paragraph and §2 step 3 are superseded by
this document and say so.

## 2. Steps, in order, each with its judge

1. **`holds`, the place word and the vocabulary.** The `block` tool takes `holds`; `block` is a place; `advanced_converter`
   joins `tend` and `holds`; the block line says the list; the default list stands in the role prompt's words. No
   change to the engine yet: a held frame takes today's frame band (the block's reach), the rest the builder's band.
   Judge: an econ game in which every advanced converter frame begun after the block is laid has turrets in reach, and
   the player's turns show the list read (it changes it, or says why not).
2. **The farm as ground.** The rows, slots and tiers deleted; the frame score over the free ground in reach; the room
   line; the no-room words. Judge: in the same game the room line against the free-ground count read from the record
   (§0's method), and no frame decaying for want of reach while the room line said there was room.
3. **Turrets grow the farm.** The turret score over free farm ground; contiguity. Judge: the farm's turrets within 500
   of its centre as a share (econ-13 97 of 139, econ-12 35 of 128) and the turrets in reach of the fusions at the
   median against thebluegecko's 46-53.

Each step is a commit with its registry rows (H-ECO-BLOCK, H-ECO-LAYOUT amended; a new row for `holds`) and its ledger
row. Step 1 supersedes the one-line fix of classing the advanced converter by capacity, so that fix is not made
separately.

## 3. What this does not do

It does not price the ground. The table of §0 is in the late-economy article for the player to read, not in the engine.
It does not move solars or storages the player never mentioned: unheld kinds stand where they stand today. It does not
touch `tend` beyond the one word, nor the harness's 45-second turn cap and the session replacement that cost econ-15 its
17:01 and 17:36 turns, which is a separate matter (the econ-15 ledger row).

## 4. Decisions taken (the user, 2026-11-08)

- The player decides what holds the farm; the engine does the two-dimensional placement.
- A default `holds` list, stated in the block line every report: advanced_fusion, fusion, advanced_converter, storage.
- `advanced_converter` is a word in `tend` as well as `holds`; `converter` means the basic one; `tend`'s default order
  gains the entry after fusion.
- A held frame that finds no room above the lower power line is placed at the farm's edge and said.
- A `start` step whose frame would have no construction turret in reach is built as a plain step (the builder stays
  on it) and the hands' line says so: a frame left there only decays.
- The room figure's two power lines are 30 and 10 turrets.
- (2026-11-08 night, after econ-flagship-16) The turrets keep every frame in reach alive whatever the `tend` order
  (H-ECO-NANO-GUARD's keep-alive, default on, `tend` `keep_alive: false` off), so the start rule's reach test is the
  right test again: a frame a turret reaches is a frame a turret will hold.

## 5. Decisions this design still needs

None open. (The five questions of the first draft were ruled as §4 records.)

## Status

Written 2026-11-08 after econ-flagship-15, before any code; the user agreed the shape and ruled §5 the same evening. Step 1 built 2026-11-08 (`holds`, the `block` place word, `advanced_converter` in both vocabularies, the start-stays rule, the block line's holds words; H-ECO-BLOCK and H-ECO-LAYOUT amended); judged in econ-flagship-16 (its ledger row): the placement half met, 42 of 42 advanced converter frames with turrets in reach (econ-15 55 of 95), but by the place word on every frame step, not the list, which the player never changed nor mentioned; and the decays went on (14 of 42 advanced converters, 16 of 49 turrets) with 5 to 29 turrets in reach of each, none told onto it: a `start` leaves a frame to turrets that are each on the nearest frame of the top-ranked kind in their reach, and a new frame farther off than those gets none of them, and a 0-3 % frame is gone in 10-17 s (K-mech-frame-decay). The turret count was the player's own call at 14:35 ('most were decaying before the turrets got to them'). So the start rule of §4 is the wrong test: a turret in reach is not a turret that will come, and what a start needs is a turret free for it, or the frame fed until one is. Answered the same night by the keep-alive (§4's last ruling; H-ECO-NANO-GUARD), judged met in econ-flagship-18 (7 decays, none in a turret's reach, two `tend` calls) after econ-flagship-17: decays 33 -> 10, turrets 33 -> 95, three misses from the hands' notion of a builder being on a frame (fixed: the share alone), five while the player had it off. econ-17 also showed ten constructors penned for 2,525 constructor-seconds, three advanced constructors inside the dense block and two tier-1 constructors inside rings of their own solars: step 2's farm as ground wants lanes out, and the generator band wants the builder's way out kept, both open. Steps 2 and 3 not begun.
