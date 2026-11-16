# The game tape — format version 4

`game.tape`: one file of a whole game as the engine had it, both sides, every seat, made so that a viewer can show any
moment at once. Where the match record (`record-format.md`) is the bot's fair view and its decisions once a second, the
tape is the truth ten times a second: every unit's place and attitude, health, build progress and head order, every
projectile, the features, the ground and its craters, both sides' orders and resources. The two name a unit by the
engine's id and a moment by the game frame, so a reader of both joins them (`rowOf` in `viewer/scene/tape.js`).

Why it is shaped so: `docs/design/2026-11-15-game-tape.md`.

## Making one

    run/tape.py <match dir>              the match's own demo; game.tape is written beside its records
    run/tape.py <demo.sdfz> [--label L]  any demo; makes run/matches/<unix time>-tape-<label>/ (--records for the
                                         per-team records as well)

`run/tape.py` plays the demo in the headless engine with the widget `run/replay-template/LuaUI/Widgets/tape_dump.lua`
(through `run/replay_match.py --tape --no-records`), which writes the raw stream `tape.raw`; `target/release/tape pack`
makes the tape of it and (`--check`, on unless `--no-check`) reads the tape back against the raw stream: every unit
alive at every sample with its place, attitude, health, build progress, order and aim, every order, projectile and
burst, the tables and the ground. `--aim` adds every weapon's direction; `--pieces` every unit's moving pieces (turrets,
legs, arms, doors: below); `--keep` keeps the raw stream.

The widget also copies the files a viewer draws with out of the engine's archives, as they are: the models (`.s3o`),
their textures (`.dds`), the map's `.smf` and `.smt`. They go to `run/data/tape-assets/<name>`, named by content (the
base64 of the file's MD5, made safe for a file name) and shared by every tape; the tape's header says which copy a path
is (`assets`). The cache is git-ignored working data and belongs to its worktree: a tape made in one tree needs that
tree's cache (or the files copied over, `cp -n`). BAR's art is CC BY-NC-ND: nothing of it is converted or committed.

Other tools: `tape info <game.tape>` (what it holds, the bytes by chunk kind), `tape audit <game.tape>` (the waste
audit's dump, `audit.jsonl`, from a tape of format 3; `-o -` to standard output, as `run/audit.py <match dir>` reads
it; `docs/harness/audit.md`),
`tape check <tape.raw> <game.tape>`,
`tape grow <tape.raw> [--cuts N]` (the growing tape checked: the stream cut at N places inside records, the tape grown
through each and read again from its new footer by one reader that keeps its slices, as the page does; on the dist358
duel, 25 cuts, every slice the same as the stream so far, the grown file 3.5 MB where the compact tape is 2.3).

**In an arena game** (2026-11-15 night; `--no-tape` turns it off) the tape is made while the game is played, with no
second engine run: the arena's host is a spectator with full view, and `crates/arena/src/capture.rs` puts the same
widget (its only copy is the replay template's) and its marker into the match directory. While the game runs, a
follower thread reads the raw stream every few seconds (at most half a core: it waits as long as its last read took)
and appends each slice the stream has passed to `game.tape` (`tape::pack::Growing`, below), moving the widget's asset
copies into the cache as they come; after the game the raw stream is packed whole into a compact `game.tape` that
replaces the growing one in one rename, and removed. The raw stream is written out just past each slice's end (frame
`300k + 1`). A stream that ends inside a record (one still being written, an engine killed) reads as far as its last
whole record. On a duel at speed 50 the widget costs about 2 s at load and 11 % of the game's time (1.29 against
1.17 ms a frame); its own Lua work was 2.3 s of a 15-minute game. The arena's games are not repeatable even without
it (two runs of one seed part at the sixth truth sample), so whether it moved the AIs' timing cannot be told from one
game.

**What the engine gives a widget in a headless replay, and what it does not** (measured 2026-11-15, engine
2026.07.04, game test-31836 and test-31908):
- Projectiles have no event for a widget: the widget asks every frame which exist (`GetProjectilesInRectangle`) and
  reads a new one's start. A beam that lives one frame is caught (5,248 beams in the duel below, lives of 1 to 3
  frames). The engine reuses a projectile's id as soon as it is gone: within two frames never in the duel, 335 times
  in the 10v8's 281,852. One gone and made again between two looks is still on the list, so every listed projectile's
  owner and weapon type are read each frame and a change is an end and a beginning (`same_frame_reuse` in the
  header's `capture`; the 10v8's tape of 2026-11-15 was made before this check).
- **No explosion or muzzle-fire event reaches a widget in a headless engine**: BAR's `gfx_explosion_lights.lua`
  forwards them from its unsynced half, and BAR skips unsynced gadgets when headless (`Headless: skipped unsynced
  gadget` in the log). A reader draws a burst where a projectile ended and where a unit died instead; the tape's burst
  rows (kinds 0 and 1) are there for a capture in a client with a screen.
- A shell (`Cannon`) and a bomb (`AircraftBomb`) fly by `place + velocity * t + gravity * t * (t + 1) / 2`, `t` in
  frames (the engine adds gravity to the speed, then the speed to the place): within 0.08 elmos of the engine's own
  places over 41,000 checked. A laser bolt (`LaserCannon`) does not: it stops where it hits and lingers while it
  fades (the formula was 3,700 elmos out), so bolts are sampled, like missiles, torpedoes, flame and the D-gun's ball.
  `tape pack` prints the check (the widget writes every formula-path projectile's place every 15 frames for it), and a
  projectile the formula misses by more than 4 elmos flies by those samples.
- **A headless engine sleeps after every frame to hold the game's speed factor** (`Game.cpp`, under `HEADLESS`), so a
  demo, whose factor is 1, replayed at twice game speed however idle the machine was. The widgets send `setmaxspeed`
  and `setminspeed` first (`replay_dump.lua` too, from 2026-11-15): the duel's 18.7 minutes simulate in 24 s, and the
  minutes a lobby demo's players spent loading pass at once (Erebos: frame 0 after 7 s where it had taken 4.6
  minutes).

## The raw stream (`tape.raw`)

A run of records: a byte of kind, a little-endian u32 of length, the payload. It is the packer's input and nothing
else reads it; `tape_dump.lua` documents each kind where it writes it. `J` is a JSON object whose `t` names it (`header`,
`unitdef`, `weapondef`, `featuredef`, `asset`, `map_files`, `ev`, `cmd`, `res`, `stats`, `over`); the rest are packed
numbers: `U` unit samples, `V` vitals, `O` head orders, `S` what each ally team sees, `A` aim, `P` new projectiles, `Q`
projectile places, `E` projectiles gone, `X` and `B` explosions and muzzle fire, `G` the ground, and from version 3 what
the waste audit reads (`docs/harness/audit.md`): `W` a builder's build power in use and a factory's line, `R`
experience and full health now, `H` the damage a unit took, every `activity_frames` for the units that changed (`R`
and `H` also at a unit's death: its last second's), `fmetal` rows (a wreck's metal as it is reclaimed, every 15 s), the
`res` rows' excess, sent and received and the engine's team statistics, and the unit types' fields the audit's
`def` rows have. A sample is stamped with the frame of its `GameFrame`, a call-in's row with the engine's frame then
(until version 3 with the last `GameFrame`'s); a unit's row is written only when the unit changed. The players' marks
on the map are `ev` rows of kind `mark` (a point with its words, a line, an erase with its radius; the player, the
team, the ally team, whether a spectator made it).

**The moving pieces** (raw version 4, with the marker `tape.pieces`: `run/tape.py --pieces`; the design and what it
costs: `docs/design/2026-11-15-game-tape.md` section 9). A unit type's pieces once, in a `pieces` row (names in the
engine's order, each one's parent, its offset from the parent at rest), kept in `DEFS` with the type. Every 6 frames the
widget reads every piece's model-space matrix (`GetUnitPieceMatrix`), and for each piece whose matrix or whose parent's
changed works out its turn and place against its parent: the script's own angles (pitch = asin(-R12), yaw = atan2(R02,
R22), roll = atan2(R10, R11), which `CQuaternion::FromEulerYPR` turns back into the engine's rotation; yaw and roll unwound
so that a spin keeps counting) and its place. A script turns and moves at set speeds, so each number runs in straight
stretches: a keyframe (`K`: frame, unit, piece, three angles in 16384ths of a radian, the place in sixteenths of an
elmo) is written only where one bends by more than half a degree or a quarter of an elmo, where a piece stops, before
a stretch that starts from rest, and at a unit's death and the capture's end. Before a piece's first keyframe it stands
as that keyframe has it (a unit's creation script poses its pieces at once), after its last where that left it.

A record of a kind the packer does not know (a widget newer than the build: the arena copies the widget from the
checkout but runs the binary it was started as) is passed over by its length and counted, the rest packed; the
arena then keeps the raw stream for a newer build to pack whole (`tape pack`).

The engine's Lua numbers are 32-bit floats, exact to 2^24: whatever the widget packs must fit. Raw version 1 wrote a
head order's builder target and queue as one number (65536 x the unit + the queue), which lost the queue whenever the
unit's id was over 127; version 2 (2026-11-15 night) writes them apart, writes `S` as two masks of sixteen bits, and
puts `max_units` (`Game.maxUnits`) and `ally_teams` (the ally teams `S` covers) in the header. Version 3 (2026-11-15
night, stage 4c) adds the audit's numbers above and mends three faults of the capture found by comparing the audit
made from a tape with the audit widget's: a unit that had just died, which the engine still lists, was met again by
the sweep for units no call-in told of and stood to the game's end (a ghost: 122 in the 10v8's tape, 11 in the dist358
duel's, 13 in the flagship game's); a unit's first commands, given before the call-in that tells of its making, named
a unit the tape did not know and were lost (they are held until it is met); and the damage of a killing blow was
dropped with the unit. The packer now names a killer that died first (by the last unit that held its id) and takes a
unit's build progress as it died from its `destroyed` row. A widget's mistakes
show only in the engine's log (`run/tape.py --keep`, then `infolog.txt` in `tape-capture/`: "Error in GameFrame()" and
the widget is gone); `luac5.1 -l -p tape_dump.lua | grep GETGLOBAL` lists every global it reads, where a local used
above its declaration shows up.

## The file

Little-endian. `magic "WRTAPE\1\0"`, then chunks, then a footer.

    chunk    kind[4]  flags:u32 (bit 0: the payload is zstd)  raw_len:u32  stored_len:u32  payload
    footer   the offset of the INDX chunk: u64, then "WRTAPEND"

| Chunk | Payload |
|---|---|
| `HEAD` | JSON: `format` ("within-reason-tape"), `version` (3 with what the audit reads, else 2; a tape that grows has `growing`), `raw_version` (the raw stream's: 1 when absent), `max_units` (BAR's 32000 when absent), `ally_teams` (when the tape says what each saw), `engine`, `game`, `map` {name, width, height (elmos), gravity, low, high, wind_min, wind_max, tidal}, `teams` [{team, ally, side, name, ai, gaia, color [r, g, b], start [x, y, z], controller}] (the name and `controller` from the demo's start script, `tape pack --seats`: in a replay the engine names every seat UNKNOWN), `spots` [[x, z, metal]], `sample_frames` (3), `slice_frames` (300), `linger_frames` (60), `vitals_frames`, `activity_frames`, `check_frames`, `position_scale` (4), `angle_scale` (4096), `aim_scale` (1024), `aim` (1 when weapons' directions were captured), `last_frame`, `slices`, `winners`, `units`, `features`, `projectiles` (counts), `assets` {path in the engine's archives, lower case: the copy's name}, `map_files`, `capture` (the widget's own account: seconds by part, rows by kind), `path_check` {weapon kind: {rows, worst}} |
| `DEFS` | JSON: `units`, `weapons`, `features`: the types the game used, each with the engine's `id`. A unit type: name, human, model {path, tex1, tex2, min, max}, xsize, zsize, height, radius, speed, mobile, air, building, factory, builder, health, metal, energy, build_time, build_speed, build_distance, extracts, sight, radar, sonar, jammer, cloak, tech, icon, weapons [weapon type ids], options [unit type ids it builds], wreck, death (the weapon type of its death blast, -1 none). A weapon type: name, kind (the engine's: Cannon, BeamLaser, LaserCannon, MissileLauncher, ...), range, speed, gravity, aoe, size, beamtime, duration, reload, salvo, tracks, turn_rate, wobble, dance, damage, paralyzer, shield, shield_radius, model, color, color2, thickness, core_thickness, flare, smoke_trail. A feature type: name, model, xsize, zsize, height, radius, metal, energy, reclaimable, blocking, geo, tree |
| `GRND` | the ground at the start: columns u32, rows u32 (corners of the 8-elmo squares: width / 8 + 1), low f32, step f32, then a u16 a corner, row by row from the north, each the difference from its western neighbour; a height is `low + step * value` |
| `SLIC` | a slice, below; one every `slice_frames`, in order |
| `UNIT` | a row a unit life, in the order created; **the row number is the tape's id of a unit** (the engine reuses its ids) and everything in the tape names a unit by it, 1-based where 0 means none. Engine id, type, team, mobile, the frames created, finished and destroyed, the rows of its killer and builder, full health, health and build progress at birth, where it first stood and how it was turned, where it died, the teams it was given to |
| `FEAT` | a row a feature (tree, rock, wreck): the engine's id, type, team, the frames it appeared and went, place, front and up directions, metal, energy |
| `TEAM` | JSON: `frames` [f], `columns` (the names: metal, store, pull, income, spend and the same of energy; from format 3 also metal's excess, sent and received and energy's), `teams` {team: [the columns a second, null before a team first appears]}, from format 3 `stat_columns` and `stats` {team: [[frame, metal used, produced, excess, energy used, produced, excess, damage dealt, received, units made, lost, killed] as the engine's statistics changed]} |
| `FMET` | (format 3) JSON: [[frame, the feature's index in `FEAT`, whole metal]] as a wreck's metal changed |
| `EVNT` | JSON: the rare events as the widget wrote them: chat, the players' marks, the end with its winners |
| `GPAT` | every change to the ground after the start, in order: frame, column, row, columns, rows, then the u16 heights. The ground at a frame is `GRND` with the patches up to it |
| `INDX` | u32 n, then for every chunk before it: kind[4], offset u64, length u32 (the chunk whole), a slice's first frame u32. Not compressed |

The tables stand after the slices so that a tape being written can grow (append a slice, then the tables, the index and
the footer: `pack::Growing`, below).

**One rule gives the state at a frame: everything stamped with that frame or an earlier one.**

**A slice** holds the state at its first frame (the keyframe) and everything stamped after it up to and including
`first + slice_frames`, so a slice's last frame is the next one's first and either answers for it. Given the `UNIT`
table it is decoded alone. Its sections, each a run of columns (a u32 length, then varints; signed values zigzag):
- **keyframe**: every unit alive at `first`: row, health, build progress (thousandths), paralysis, flags (1 cloaked),
  its head order, and for a type that moves its place and attitude; each weapon's direction when captured.
- **motion**: for each unit that moved, its samples in `(first, first + slice_frames]`: the frame step, the place
  (quarters of an elmo; a second difference, so a steady pace writes zeros) and the attitude (heading, pitch and roll
  in 4096ths of a turn, each difference the short way round). A sample exists only when the unit moved or turned, so
  between two samples further apart than `sample_frames` the unit stood still until the last `sample_frames`.
- **vitals**, **activity**, **aim**: a row when it changed: (frame, row, ...). Activity is the head of the order
  queue: the engine's command id (negative: a build of that type), how it points (0 nowhere, 1 at the unit or feature
  with engine id `a`, 3 at the place a, b, c), the engine id of what a builder is building, the queue's length. A
  feature's engine id here and in orders is its own plus the unit limit (`max_units` in the header; BAR's 32000 when
  the header does not say). Repair, reclaim, resurrect and capture take five numbers when a builder working an area
  is on one target (its id, then the area: `BuilderCAI.cpp`); the widget writes that as the target (`how` 1) since
  2026-11-15 night, and before kept its first three numbers as a place, which the reader sorts out (`aim_of`: an
  area's centre is a click on the ground, else the numbers are the id and the area's first two).
- **orders**: every command given, either side: frame, the unit's row, command id, shift, up to six parameters.
- **projectiles**: every projectile in flight at a frame of the slice, or gone under `linger_frames` before it: the
  frames it began and ended, engine id, weapon type (-1 a piece of a dying unit), owner row, team, where it began
  (eighths of an elmo), velocity (1024ths of an elmo a frame), gravity (65536ths), its path (0 the formula, 1 a beam
  from where it began to its target, 2 its samples, which follow), how it was aimed (1 the ground at a place, 2 a
  unit's row, 3 a feature, 4 a projectile) and at what.
- **bursts**: explosions and muzzle fire as the game reported them (none from a headless capture).
- **sight** (version 2, when the header lists `ally_teams`; a slice of version 1 ends with its bursts): for each unit of
  the keyframe the ally teams that have it in sight and those that have it on radar (a bit `a` for ally team a), then
  a row when they changed: (frame, row, in sight, on radar). Looked at every `vitals_frames` (half a second); before
  a unit's first look, its own side alone.
- **work**, **rank**, **hurt** (format 3, from a raw stream of version 3): for each unit of the keyframe its build
  power in use (hundredths of its build speed) and a factory's line, then a row when they changed; the same of its
  experience (f32) and full health; then the damage each unit took, (frame, row, f32), at the samples and at its death.
  Before a unit's first row: no build power, no experience, the full health it was made with.
- **pieces** (format 4, from a raw stream of version 4 captured with them): a track for each piece not at rest
  somewhere in the slice, by row and piece: its numbers at the slice's (or its unit's) first frame, at each keyframe
  inside and at the slice's last frame, so that a slice is read alone; straight between two points.

An attitude's three angles are made of the unit's front and up directions by `attitude` in `tape_dump.lua` and turned
back by `read::attitude` (a test holds the two together): the heading about the vertical (0 facing south, +z), the
pitch (the front's rise), the roll (the up's turn about the front from where it would be with no roll). Against the
engine's own vectors (the duel captured both ways, 379,447 samples at the same frame and unit): the front turned back
is off by a median 0.011 degrees and at worst 0.063, the up by 0.018 and 0.079; a step is 0.088. A model's own x axis
is the up crossed with the front (the engine's `CMatrix44f(pos, -rightdir, updir, frontdir)`, `SolidObject.h`).

**A tape that grows** (`tape::pack::Growing`): the ground at the start goes in once (its height scale 400 elmos wider
each way than the heights seen so far, for the craters to come); then each time the raw stream has passed a slice's
last frame, the slice is appended and after it the header (with `growing: true`, and `last_frame` the written slices'
end), the types, the tables, the index and the footer. The file only grows, so a reader holding an older footer still
reads whole chunks; the superseded header and tables are dead bytes until the compact tape replaces the file. The index
lists the chunks in a packed tape's order (header, types, ground, slices, tables), the order a reader hands them in.

## Reading one

`crates/tape` is the one reader: `tape::read::Tape` takes the index, then any chunk, then slices, and answers
`frame(f)`: the state at a frame as flat arrays. `crates/tape-wasm` is the same code behind a C interface for the page
(`run/build_viewer.sh` builds it to `viewer/scene/tape.wasm`; `run/view_match.py` does at its start when the crate is
newer), loaded by `viewer/scene/tape.js`.

- **units**, sixteen f32 each, by type so that a type is one run: place x y z (elmos), front x y z, up x y z, health
  as a share of full, build progress 0 to 1, team, flags (1 cloaked, 2 its type moves, 4 finished), row, type,
  paralysis as a share of full health. `ranges`: (type, first, count) per type.
- **projectiles**, twelve f32 each: where it is, where its tail is (a frame behind) or a beam's far end, weapon type,
  path, frames since it began, frames it lives, owner row, team.
- **bursts**, eight f32 each: place, frames since, weapon type (the unit's type for a death), kind (2 a projectile's
  end, 3 a unit's death; 0 and 1 the game's own), the owner's or the dead unit's row.

- **pieces** and **pieceAt** (format 4): for each unit with a piece not at rest, a block of `piece_stride` transforms
  (the most pieces a type has), twelve floats each (`pieces::compose`: a rotation by columns and a translation, so that
  a vertex v of the model at rest is drawn at R v + t), its pieces composed through their parents; and beside `units`
  each unit's block's number, or -1.
- **seen**, a u32 a unit beside `units`: the ally teams that have it in sight (bit `a`) and, sixteen bits up, those
  that have it on radar; every bit set on a tape that does not say.
- **doing**, ten f32 a unit with a head order or a build under way: its place in `units`, its row, the command, how
  it points (0 nowhere, 1 a place, 2 a unit, 3 a feature), the place it points at this frame (a unit's where the unit
  is), the target's row or feature number (-1), the row it is building (-1), its queue's length.
- **commands**, eight f32 a command given in the last `COMMAND_FRAMES` (45: a second and a half) by a unit alive with
  an aim: the unit's place in `units`, the command, the place it points at, frames since, 1 for shift, in the order given. The fading
  lines of the game's spectator view: the page draws a unit's latest frame of them as one chain, as BAR's
  `gui_commands_fx` does.
- `orders_of(row)`: every order given to a unit in the slices loaded (the page keeps the twelve it used last, wherever
  they lie; the unit panel reads the minute it lists with `wantSpan` and keeps to it), eight f32 each: frame, command,
  shift, how, place, target.

A command points at a place when it has three or more numbers (a build's too), at a unit or feature when it has one
and is one of attack, guard, repair, reclaim, manual fire, resurrect, capture, load, load onto or set target; a place
off the map is no aim. `tape orders <game.tape>` counts, a game second at a time, every head order by what it points
at, and lists any place off the map (none on the three games of 2026-11-15 once the five-number orders were read).

Between samples a unit's place runs straight and each angle the short way round; projectile, burst, order and vitals
rows have their exact frames.

## The page

`run/view_match.py` serves `game.tape` by byte range and the asset copies at `/assets/<name>`; a directory with a tape
and no record of ours (a bot of `bots/`, a taped demo) is a match the browser lists and opens. `viewer/scene/`:
`tape.js` (the reader and its fetches: the footer, the index, the tables, then the slice under the playhead and the
next, twelve kept; never from the browser's cache; `refresh()` reads a growing tape's footer again and on a new index
calls `wr_reindex`, which keeps the slices read, and hands the tables in again, or reads afresh when the file shrank:
the compact tape at the game's end), `assets.js` (S3O, DDS, SMF and SMT as they are), `terrain.js`, `units.js` (each type's model once for all its units; a vertex carries its piece, renumbered to the engine's order by name, and a unit with a block of piece transforms has each vertex moved by its piece's, read from a float texture), `effects.js`, `lines.js`
(lines over the scene: a set number of CSS pixels across at any distance, not depth-tested, so an order reads through
a hill), `scene.js` (the camera, the drawing, picking, and a 2D canvas over the scene that the page paints with the
same functions as the 2D map), `gl.js`. See `record-format.md`, Viewer, for what the page shows.

Tests: `cargo test -p tape`; `tape pack --check` on every tape made; `node viewer/test/scene.js <match url> <out dir>
[seconds ...]` (the real page in a headless browser: checks the scene as it opens, seeks, saves screenshots of the
whole map, the busiest ground and the same turned, times seeks, drives the wheel, a drag, a shift-drag, a hover and a
click, checks the clicked unit's panel, the lines with every order and reach on, the overlay's size, the sides'
numbers of a game with no record, and with a record a unit of ours's panel; and fails on any page error). **Run it in both browsers**: Chromium by default, Firefox with
`SCENE_BROWSER=firefox` (over WebDriver BiDi); the user's browser is Firefox. The first build of this test reset the
camera before every screenshot and so passed while the scene opened blank in every browser (the camera fitted while
the canvas was still hidden, its distance infinite): the user found it in Firefox the same night.
`node viewer/test/muzzle.js <match url>` checks the moving pieces against the engine's own shots: where a shot began
must lie on one of its owner's pieces as composed, and nearer than at rest (the dist358 duel, 308 shots: a median of
1.0 elmos, 90 % under 2.2, against 5.5 and 18.1 at rest, a light laser tower's 0.1 against 12 to 26; the Erebos 10v8,
401 shots: 0.5 against 6.2; Chromium and Firefox the same to the tenth). The tail of the spread is the fast shots,
already well out of the gun when the widget first sees them, not the pieces: an Alaris tank's shotgun pellets begin
20 elmos from its nearest piece whether the pieces are composed or at rest; rocket and anti-air missile shots likewise.
`node viewer/test/live.js <match url> <out dir>` watches a game being played (start the arena first): it waits for
the tape, checks that it grows while the page watches with the playhead on its newest frame, and that at the game's
end the page takes the compact tape and draws from it. Both browsers again. `viewer/test/driver.js` is the two tests'
browser driver.

## Measured (2026-11-15)

| Game | Length | Capture | Raw stream | Tape | Of which |
|---|---|---|---|---|---|
| dist358 duel, TitanDuel (WRDistilled against BARb), 504 unit lives, 11,762 projectiles | 18.7 min | 46 s in all, 24 s of it the game (the widget's own share 1.5 s) | 15.2 MB | 2.5 MB | motion 1.1, projectiles 0.3, the ground 0.3 |
| apex-v0511-ladder-0-1, Comet Catcher (the flagship against Apex), 870 unit lives | 21.1 min | 81 s in all | | 4.1 MB | |
| the lobby 10v8 of 2026-11-10, Erebos Lakes, 12,529 unit lives (5,374 at once), 281,852 projectiles, 4,803 features | 29.2 min | 17 min in all: the engine 15.4 (the game itself, at one worker thread), the read-back check 1.4 | 547 MB | 79.3 MB | motion 39.5, projectiles 9.1, aim 8.9, keyframes 6.1, orders 5.6, head orders 5.1 |

(The Erebos 10v8 at the first encoding, places in eighths of an elmo and the attitude as two directions: 156 MB, 103 of
it motion and 21 aim. Heading, pitch and roll in 4096ths and places in quarters brought motion to about 40 % of that:
`docs/design/2026-11-15-game-tape.md` section 6.)

In the page (`viewer/test/scene.js`, headless Chromium on this machine's GPU, a Radeon RX 6900 XT): a seek to a slice
not loaded, timed from the seek to the frame drawn, 100 to 155 ms on all three games, the 10v8 with its 5,200 units
included (the test looks every 100 ms, so that is a ceiling); a frame's script 0.6 ms on the duel and 2.4 ms on the
10v8 (276 model draws); the first frame after opening waits for the models and textures (the faction textures are
22 MB each, the 10v8's map tiles 70 MB). On the software renderer the same seeks take 0.4 to 2.4 s on the 10v8.
With the lines and the overlay (2026-11-15 night) the 10v8 on the GPU: seeks 105 to 231 ms, a frame's script 1.5 ms
with 4,742 units (Firefox: seeks 106 to 330 ms, a frame 6 to 7 ms); on the software renderer seeks 0.5 to 3.3 s.
The slowest slice to decode: 2 ms on the duel, 18 ms on the 10v8. With the moving pieces (2026-11-16): the 10v8's seeks
105 to 271 ms in Chromium and 108 to 329 ms in Firefox, a frame's script 3.6 ms in Chromium; the slowest slice to
decode 43 to 71 ms on the 10v8 over two runs, 2 ms on the duel.
