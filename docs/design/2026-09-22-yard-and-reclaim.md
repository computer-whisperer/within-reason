# Yards, stuck units, and removing our own buildings

Written 2026-09-22 late night, from the user's reading of hands-2-bulldogs: "death by stupid build order, plus a side
of troops getting stuck coming out of the tier two factory", and "providing a way for the player to deliberately kill
or eat our own structures and units ... We should make sure the player is aware of death blast radius however."

## What happened (the record, `run/matches/1790117628-hands-2-bulldogs/00`)

- The first two Bulls left the Advanced Vehicle Plant within seconds. The player then queued solar collectors at its
  marked place `avp_side`; one finished at 11:00 about 165 elmos straight off the plant's front, four more around it.
- Bulls three to seven, finished 11:42 to 14:24, stood in the yard five to seven and a half minutes each, within
  thirty elmos of the exit. The plant built nothing from 14:24 to 19:04 with metal and energy full. They left at 19:10,
  when the enemy destroyed the solars and then the plant.
- The engine raised 156 move-failed events on Bulls and 249 on constructors. The brain counted them into its
  per-minute log (`stuck_cells`); the picture said nothing; the scorecard's `stuck_s` counts the commander only.
- The build order: seven constructors by 9:30, one plant, no soldier between the sixth Flash (about 4:30) and the first
  Bull (9:36). The search's answer was a tight chain to the thing asked and said nothing about what should stand beside
  it or after the horizon (K-plan-single-goal-strips-the-rest; the user's own experience with an annealer of this shape).

## Decisions

1. **The engine tells us the yard.** The protocol gains what the engine already knows: `UnitDefInfo.footprint`
   (xsize, zsize in 8-elmo squares), `UnitDefInfo.death_blast` and `self_destruct_blast` (`Blast { radius, damage }`
   from the def's explosion weapons, None when it has none), `self_destruct_seconds`; `OwnUnit.facing` (the engine's:
   0 south +z, 1 east +x, 2 north -z, 3 west -x; a factory's units leave through its front, `Factory.cpp`
   `SendToEmptySpot`: `exitPos = pos + frontdir * (radius + unit radius)`). A new `Command::ReclaimUnit { unit, target,
   queue }` (the engine's `COMMAND_UNIT_RECLAIM_UNIT`). The record's header carries footprint and blasts; a factory's
   `finished` event carries its facing. GUI play needs `run/install_to_bar.sh` re-run (docs/harness/pitfalls.md).
2. **A factory's exit lane is kept clear, in every mode.** `Brain::lanes()`: for every factory of ours, finished or
   not, a `Lane { from: centre, to: centre + front * (half_depth + LANE_DEPTH), half_width: half_width + LANE_MARGIN }`
   (`LANE_DEPTH` 320 elmos, `LANE_MARGIN` 48: a Bull is about 40 across, and the engine's empty-spot search walks the
   front). `BuildSite` gains `keep_out: Vec<Lane>`; the shim's `find_build_site` rejects a found site whose centre lies
   within a lane (its ring walk already rejects metal spots this way). Every Build the bot issues with a site goes
   through `Brain::build_site_for`, which fills `keep_out`; extractors are exempt (a spot is where it is). Not decided
   here: keeping a new factory's own future lane clear of what already stands (`LAB_GAP` is the only rule today).
3. **Stuck units are tracked and said.** `Brain::stuck: HashMap<UnitId, Stuck { since, at }>`: a mobile unit of ours
   enters on a move-failed event and leaves when it has moved `STUCK_FREE` (80 elmos) from where it failed or dies.
   The picture: a group entry gets `stuck` ("3 of its 5 soldiers have not been able to move for 4 min, at plant_1763's
   exit"); a factory actor entry gets `yard` ("blocked: 5 Bulls have stood in its exit lane for 3 min; in the lane:
   armsolar_31002, armsolar_31640"), naming the buildings of ours whose centre lies in the lane by `<def>_<id>` so the
   player can name them back. The player's report says the same on a `yards` line and wakes the player once per
   blockage. `run/floor.py` gains `yard_min` (minutes any factory's yard was blocked) and `stuck_s` becomes the
   unit-seconds of any own mobile unit stuck, not the commander's move failures.
4. **The player can remove what we own.** A `remove` tool: `{"reclaim": ["armsolar_31002", ...], "by": "constructor_N"
   (optional; else the nearest builder not on a list)}` puts `reclaim <handle>` steps at the front of that builder's
   list (the list machinery makes it happen and keeps the builder off the menu; `Task::Reclaim { target }` is done when
   the target is gone); `{"destruct": ["armsolar_31002"]}` sends the engine's self-destruct. Handles are
   `<def>_<id>` for any unit of ours, or an actor name (constructor_N, plant_N, commander). The answer names, for each
   unit, its self-destruct blast radius and damage and every unit of ours inside the radius with its health; a
   destruct that would kill something of ours is refused unless `"accept_losses": true`. The `units` glossary tool
   states the death and self-destruct blasts. The guide says a reclaim returns most of the metal and a destruct none,
   and that the commander's blast is the game's largest.
5. **A search can ask for the economy.** `target` gains the goal kind `income:N by M:SS` (metal a second at that
   time): short of it, the score loses `TARGET_SHORTFALL` times a minute of the missing income. Extractors are asked
   as units already (`armmex:12 by 8:00`). The guide says so, and says a target order is a chain to the thing asked
   and nothing else: the economy and the army around it are the player's to add.
6. **The claim.** K-plan-target-order-is-a-chain (openings.md): the four games' orders and the user's experience;
   status observed; the brief sentence rests on it.

## Steps

1. Protocol and shim (decision 1; `keep_out` in `BuildSite`); the recorder and the record-format document.
2. `brain/yards.rs`: lanes, stuck tracking, the lane test in `build_site_for`; the picture's and the report's words;
   `floor.py`.
3. The `remove` tool, `reclaim` list steps, `Task::Reclaim`, the glossary's blast words, the guide.
4. The `income` goal; the guide's chain sentence; the claim.
5. Tests; one check game (Bulls again, the same seed) read for: no building in a lane, stuck words in the picture,
   whether the player used `remove` or `income`.

## Status (2026-09-22 late night)

Steps 1-4 built. The scorecard's `stuck_s` is unit-seconds stuck (hands-2: 3,187 against hands-1's 12) and `yard_min`
reads lanes from the header's footprints and the `finished` events' facings, so older records show `-`.

Step 5, yard-1-bulldogs: WON at 17:10 with 13 Bulls; no building in either plant's lane, `yard_min` 0, `stuck_s` 44
against hands-2's 3,187, metal full 0.7 % against 50 %; the player's one search asked four goals with an income goal
(the guide's sentence heard on the first try); `remove` untried, nothing to remove. Left open: the advanced-solar
step the player skipped (an energy stall 8:45-10:00), and a new factory's own future lane against what already stands.

**Observed 2026-09-22 late (escalate-4-hard-aggressive, `run/matches/1790125719-escalate-4-hard-aggressive`).** Two plants ordered by two builders within a minute
(7:57 the commander's `armvp plant_site`, 6:42 a constructor's `armvp spot_36`, both sites chosen at order time) went up
at (1568, 2734) and (1664, 3054), both facing south: the second stands on the edge of the first's lane (96 elmos off
its axis, exactly `half_width + LANE_MARGIN`), and the first's Blitzes could not leave (ten move failures in minute
10, `yard_min` 0.8). The lane rule sees only factories that stand; a site chosen while another factory's order is
still walking to its place is not checked against that factory's lane, nor is a new factory's own lane checked
against what stands. Candidate (not built): count pending factory orders as lanes (the engine faces our buildings
south, facing 0, in every recorded game), check a new factory's own south lane against standing buildings, and widen
`LANE_MARGIN` past a factory's half-width. `remove` had its first use here: the blocked plant reclaimed by the
commander, asked 10:44, gone 11:22 (the first ask went to a constructor's list and waited on its current build).

**Decided 2026-09-23 (after escalate-4 and fixes-1).** Pending factory orders have lanes (facing south, the engine's facing for every building of ours in the records), and a new factory's own south lane is kept clear of every building of ours standing or ordered by the mirrored north-pointing lanes in `keep_out` (`yards.rs` `own_lane_keep_out`). `LANE_MARGIN` stays 48. The factory option's words now say what of that type is already under way.

**Decided 2026-09-30 (player-29-hard, the user: "we should work to tag and handle all structures that have units leave them through a dedicated side to be placed or rotated carefully").** The law: a factory is placed with the facing whose exit lane lies inside the map, over ground its units can walk, and clear of our buildings; a site with no such facing is not a site. `BuildSite.keep_out` became `placements: Vec<Placement { facing, keep_out }>`: the bot offers a factory all four facings, the one pointing most toward the enemy's start first (its units leave toward where they go), each with its own keep-out (the standing lanes, the mirrors of our buildings behind that facing, the mirrors of unwalkable cells and of the cells past the map's edge, which now count as blocked), and the shim tries them in order, keeps the first with a site and builds with its facing. Anything but a factory offers one placement, facing 0, with the standing lanes. The yard wake names a lane that runs off the map or over unwalkable ground and says only removing the factory frees the units, instead of "nothing of ours stands in the lane" (player-29 17:22: the player read that and reclaimed the two stuck Lugers). A pending factory's lane is drawn with the facing the shim tries first. The protocol changed: GUI play needs `run/install_to_bar.sh` re-run.

