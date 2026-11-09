# WRDistilled on the base layout engine as it stands (2026-11-09)

Regime: `bots/distilled` (a heuristic bot: code decides). The user, 2026-11-09: "Let's update the distilled bot to use
the new base layout engine. As usual with that bot, I will let you decide how exactly to do that."

## 0. What the bot does today (read at 9dc41d9c)

The bot has asked `crates/layout` for its sites since 2026-11-03 (D-LAYOUT, D-STAND; `siting.rs`), so the shared
crate's changes since reach it already: the street baseline, the way-out veto, the plant's footprint on its products'
ground, the searches' speed. What it does not use is what the engine gained for the flagship after that date, and it
still carries the siting it had before the engine beside it:

| The engine has | The flagship | The bot today |
|---|---|---|
| The block and the farm (`lay_block`, `World.lattice`, `farm_frame`, `Farm`; farm-holds steps 2-3) | laid at the first advanced plant; turrets on its sub-lattice growing the farm, held frames where the most power reaches | none: reactors and advanced converters within 600 of the back field, a ring turret within 400 of its frame |
| A frame's purpose by what holds the farm, not by unit data | `held` | `purpose_of` sorts by unit data; the advanced converter (380 metal) is a generator in banks of 6, econ-flagship-15's fault |
| Sites the engine refused (`World.refused`) | the pianist's refused sites, 90 s | none; a refused slot is the best again next ask, so the old code's rotations (`turret_attempts`, `back_field(attempt)`) stand in for it on the fallback path only |
| Each mobile builder's own ground for the way-out veto (`Builder.passable`) | its movement class's | `None`: judged on ground open everywhere |
| The stand's unbuilt slots as reservations | yes | none |
| One fallback when the layer has no site (the engine's search at the band's centre, the plants' lanes kept out) | `fallback_placements` | a second siting system: each job's own point and search (the pre-layer rules), also what `layout=off` plays |

Beside placement: the bot's construction turrets help one frame, the dearest project's, when it is in reach, else
guard a plant in reach; a turret with another frame in reach and no plant stands idle (the audit of 2026-11-06:
construction turrets idle 37 % with the bank full). A farm is worth nothing under that rule.

## 1. The change

1. **One path.** Every building but an extractor is asked of the layer. The per-job site rules in `economy.rs`
   `order`, `back_field`'s attempts, `turret_attempts`, `exit_lanes`, `facing_enemy` and the `layout=off` arm go.
   When the layer has no site, the flagship's fallback: the engine's search round the band's centre, every plant's
   lane kept out (`layout::lane`), a plant facing the enemy.
2. **The world as the flagship feeds it.** Refused sites (an order with a slot that failed: the slot for 90 s),
   each builder's ground (`terrain::passable` of its class, found once a class), the stand's unbuilt slots while the
   commander holds the stand, the block's lattice and farm frame.
3. **Purposes by the job.** Plant; turret; frame (fusion, advanced fusion, advanced converter: the flagship's default
   `holds`; the silo at the back field); generator (solar, advanced solar, converter, in banks); place (anti-air,
   radar).
4. **D-BLOCK** (`block.rs`). Laid once when the first advanced plant stands or is going up (`layout::lay_block`;
   first written with no mark, the flattest ground beside the plant, and changed after the first rows to the back
   field as the mark: see Status), tried again every 30 s while no ground holds it, laid again when its plant is gone. Sparse pitch (96). With it laid a frame's and a ring turret's band is the farm's (the centre out
   to the farthest turret's reach, the flagship's `farm_band`); a plant's turret stays within 400 of its plant, on
   the lattice. Without it the bands are today's. The yard (`YARD_WALK`) is measured from the block's centre.
   Arm `block=on|off`.
5. **The turrets' work.** A construction turret works the frame in its reach that ranks first (a construction
   turret's frame, then the dearest), else guards a plant in reach; told again when the answer changes.
6. **Said.** `block:` when laid or refused; `site:` with the layer's words for every frame, turret and plant; the
   minute line's `farm=` (turrets on it, room at ten turrets in advanced fusions).

## 2. Judges

- The econ bench (seeds 1-8, 20:00) against the same-day baseline at HEAD: income at 5/10/15/20, the bank, the
  turrets in reach of each reactor at its placing (the `site:` words), turret idle share not measured without the audit.
- The map bench (Comet 16 a side and twelve maps, both opponents) against the same-day baseline: no loss beyond the
  rows' scatter; `no layout site` counts by kind.
- The layout crate's and the bot's tests.

## 3. Not in this change

The way-out watch (`layout::way_out_state`: freeing a penned unit by reclaiming the cheapest closer) is the
flagship's report to its player; for the bot it would be a new rule that takes its own buildings apart, and no bench
has shown the bot's units penned since `product_walk`. The dense pitch. Water kinds for the block.

## Status

Built and judged 2026-11-09 (the rows dist268-dist343 in `docs/experiments.md`; the rules and tables in
`bots/distilled/README.md`: D-LAYOUT amended, D-BLOCK, D-TURRETS).

- **Econ bench** (Comet, eight seeds, income at 5 / 10 / 15 / 20): the baseline 35 / 123 / 327 / 753; the block off
  36 / 135 / 377 / 874; the block on 34 / 128 / 399 / 918. The gain is the turrets' work and the purposes; the block
  adds 30 to 60, inside the scatter.
- **Duels** (Comet 16 a side and twelve maps, both opponents, 224 games): 145-45-34 against the baseline's 133-44-47.
  No duel reached an advanced plant: the block plays in none of them.
- **The block's mark.** The unmarked laying put the block 2,300 from home in two of four games on Flooded Valley
  (the flattest ground within 2,400 of the plant) and the first reactors went up there with no turret in reach. The
  bot has no player to move a block, so it lays its own at the back field's mark: Flooded Valley back at the
  baseline's income, Comet, Onyx and Avalanche unchanged, Hooked with no block (no ground at the mark; the bands of
  before).
- **A fault the turrets' work brought, fixed.** A converter raised inside three seconds read as a dropped order (51 in
  eight games); a building standing at the order's slot is the order done.
- **Tried and not kept.** The ring's turrets within reach of their frame, on the lattice, instead of over the farm's
  band: 922 against 933, level; the flagship's shape stays.
- **Left.** The farm's turrets spread (a median 1,170 from the block's centre on Comet's econ row, a quarter placed
  with no frame in reach): the layer's turret score is the free ground shared with the standing turrets, and the bot
  places no frame to follow them as a player would. A frame is placed with 19-21 turrets in reach at the median
  against thebluegecko's 46-53. The bank at 20:00 is unchanged (44,000). The block is unjudged in a duel. The rule
  that kept the commander's solars within 700 of home went with the engine-search siting on 2026-11-03 and was not
  put back.
