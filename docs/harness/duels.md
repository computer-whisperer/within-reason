# Duels — unit against unit

`target/release/duel` spawns N units of one type against M of another on flat ground, sends them at each other and
records who is left. It exists to fill the matchup table the brain and the LLM commander choose a unit mix from
(`docs/knowledge/units.md`, K-units-duel-*).

```
duel (--units a,b,c | --ours a,b --theirs c,d | --pairs a:b,c:d)
     [--reps 4] [--budget 1200 | --count N] [--parallel 2] [--sites 3] [--duels-per-match 45] [--time-limit 240]
     [--sweep-waves 3] [--spacing 56] [--formation X[/Y],...] [--lane off|old|on[/...]] [--speed 50] [--map NAME]
     [--label TEXT] [--base-port 9500]
duel --scenario FILE [--reps 8] [--parallel 2] [--time-limit 240] [--lane X[/Y]] [--speed 50] [--label TEXT] [--base-port 9500]
     [--script FILE [--script-delay S]] [--theirs director|track] [--economy bare|fed] [--seeds a,b,c]
duel --pairs chaser:runner --count N --chase FRAMES [--lane on] ...   the chase instrument (below)
duel --scenario raid [--ours armfav*4+armflash*2] [--theirs armflea*1+armpw*1] [--reps 8] [--lane on|off] [--time-limit 180] ...
                                       the raid scenario (below): a picket body against scripted raiders
duel --report DIR [duels.csv ...]      rebuild the tables in DIR (from its own duels.csv, or merge the files named)
run/engagement.py <match dir> <MM:SS> [--radius 900] [--at X,Z] [--enemies sight|known|truth] [--orders 20]
                  [--after 30,60] [--out FILE] [--track [--track-seconds 60]] [--script FILE]
                                       cut a scenario file from a live record (with the enemy's track, our orders)
run/scenario_table.py <batch dir> ... [--by-seed]   one markdown row per scenario batch (margins, value left at 30/60 s)
```
`--seeds a,b,c` works for every kind of batch: each duel is fought `--reps` times under each engine seed.
`--units` runs every pair from the list, each unit against itself included; `--ours/--theirs` the cross product;
`--pairs` exactly those. Unit names are the game's internal ones (`armpw`). A side may also be a **mixed force**,
`armflash*8+armstump*6` (2026-09-25): spawned as written, in list order, so the first type stands in the front
rank; a plain name against a mixed force takes the count that matches its metal, two plain names take the batch's
sizing. Two buildings are never paired.

Output, in `run/matches/<stamp>-duel-<label>/`: `duels.csv` (one row per duel, appended as they finish), `pairs.csv`
(per ordered pairing and pair of formations: counts, wins / losses / draws, mean margin, mean seconds, then the fire
instrument's columns below), `matrix.csv` and `matrix.md` (row unit against column unit, mean margin x 100, formations
pooled), `batch.json`, and one directory per engine start with `engine.log` and the replay. The first ten columns of
`pairs.csv` keep the 2026-09-19 layout (`combatsim` reads them by position); `duel --report` and `run/duel_ab.py` read
batches from before the fire instrument too, leaving its columns empty (nan in `duel_ab.py`).

## How it works
- **Both teams are our AI**; the `duel` process is the bot for both (`crates/arena/src/bin/duel/`): the two shims connect
  to its socket, and one director sees both sides whole. The heuristic brain is not involved. The shim runs in lockstep
  (`WITHIN_REASON_LOCKSTEP`), so orders land on the frame they were decided for at any game speed; results at speed 50
  and 200 agree (below).
- **Spawning** is the AI interface's cheat `COMMAND_CHEATS_GIVE_ME_NEW_UNIT` (`Command::GiveUnit`). The engine honours
  it without `/cheat` because the game is hosted locally with one player (`CAICheats::OnlyPassiveCheats`,
  `rts/ExternalAI/AICheats.cpp:34`); it would silently do nothing in a multiplayer game. The engine delivers the new
  unit's events from inside that call, so the shim issues it after letting go of its instance table
  (`engine::Spawner`) — issued under the lock, the first spawn deadlocks the engine.
- **Sites** are chosen from `Hello.terrain`: 1440 x 640 elmo rectangles, every cell dry and within the least agile
  listed unit's slope limit, at most 16 elmos of relief, 900 from each other and from both commanders
  (`sites.rs`). Quicksilver gives two. Each site runs its own sequence of duels, so a match fights two at a time.
- **A duel.** The armies appear 1100 elmos apart (beyond every tier-1 weapon; artillery reaches 710) in their formation
  (by default ranks of eight, 56 elmos apart, further ranks behind; `--formation` below). The axis runs west-east because spawned units always face south: both sides
  start side-on. After one second both get a Fight order (attack-move) at the enemy's centre; idle units are sent again
  at wherever the enemy is now. Buildings just stand; the mobile side comes to them.
- **Formations** (`--formation`, 2026-09-24). `X` or `X/Y`: `X` for the first army of each pairing, `Y` (default
  `ranks8`) for the second; each is `ranksN` (ranks of N across the approach, further ranks behind, and the whole army
  attack-moves at the enemy's centre, so it closes into a ball: the tables before 2026-09-24 were fought this way at
  eight) or `line` (one rank; each unit attack-moves at the enemy's centre shifted by its own place in the rank, so
  the line advances as a line: `director.rs` `line_abreast`), optionally `@SPACING` (else `--spacing`). Several
  shapes, comma-separated, fight every pairing in each (`--formation ranks8,line,line/line` is three duels per pairing
  and repetition), on the same sites and in the same matches; `duels.csv` names them in `form_x` / `form_y` and
  `pairs.csv` splits by them. The site rectangle grows to hold the largest army of the batch in its formation (the
  separation plus the deepest formation and 120 behind each end, the widest rank plus 120 each side; never less than
  1440 x 640), which fixed the old flaw of rear ranks standing on unchecked ground. Wide shapes need a flat map: a line
  of 24 Stouts at 56 (1564 x 1528) found no site on Quicksilver, Comet Catcher, Gasbag Grabens or Feast of Hades
  (Full Metal Plate hangs the shim: 10,000 metal spots), and two on Mithril Mountain v2.0.1.
- **Sizes.** By default equal metal: about `--budget` a side, with the whole-unit counts whose totals differ least
  (26 Pawns against 10 Thugs). `--count N` gives N against N instead. Energy cost and build time are ignored.
- **Order of play.** Repetition `r` of a pairing puts the first unit at the west end when `r` is even and gives it
  team 0 when `r / 2` is even, so four repetitions cover every combination. The plan is shuffled with a fixed seed.
- **The lane** (`--lane`, 2026-09-25). `X` or `X/Y`: whether the bot's control lane (`crates/micro`: flee, fan,
  kite, the body's shape of H-MICRO-FORM and the step out of a turret's reach) drives the first army of each pairing, and the second
  (default `off`): `off` is the director's orders alone (the harness before 2026-09-25), `old` the lane without
  H-MICRO-FORM (the bot's lane as it was), `on` the whole lane. The director holds a `micro::Lane` per army, notes
  its own Fight orders as the standing orders, commits everything (`Commitment::All`), and appends the lane's
  commands after its own each tick; with the lane on, idle units are re-sent at the enemy's centre only every 2 s
  and when the centre has moved 150 (as the pianist's follow does), since a unit standing at contact is idle by
  design. `duels.csv` names the modes in `lane_x` / `lane_y`; `pairs.csv` splits by them. Run a batch in each mode
  and compare with `run/duel_ab.py <without> <with>` (the lane is not part of its pairing key). This replaced
  `--spread` (a copy of the retired H-MICRO-SPREAD's block, deleted 2026-09-25: the lane itself now runs here).
  `WITHIN_REASON_MICRO_DEBUG=1` prints the lane's claims to the terminal. `WITHIN_REASON_DISABLE=<rule IDs>` switches
  lane rules off (H-MICRO-FORM-SPACING, H-MICRO-STEP-OUT, ...), and `WITHIN_REASON_FORM_SPACING=factor,max` sets the
  spacing against area weapons (`micro::FormTuning`: twice the blast radius, at most 160, by default); `batch.json`
  records both (`disabled`, `form_spacing`; from 2026-09-28). At every start the duel prints the blast radius the
  definitions carry for the Fatboy, Bull, Stout and Blitz (`duel: blast radius from the definitions: armfboy 150, ...`).
- **Scoring.** Decided when one army has nobody left (`wiped`), after `--time-limit` game seconds (`timeout`), or after
  60 s without damage to anyone (`stalemate`: anti-air against anti-air). `value_left` is the surviving share of an
  army's metal, each survivor weighted by its health. **Margin** = own `value_left` minus the enemy's: +1 is a flawless
  win, -1 a wipe without scratching them. `damage_taken` sums the engine's damage events (hit points, overkill
  included). `contact_seconds` is the time to the first damage. `spread_x` / `spread_y` are how far each army's
  units stood from their own centre (root mean square, elmos) at the frame the first damage was seen: a spawned
  tier-1 bot army at spacing 56 fights at a mean of 49 (Centurion) to 93 (Grunt) of these, four times that within
  one type, so an army's fighting formation is an outcome and not the spacing it was spawned at (the probe `docs/studies/combat-sim.md` asks for first).
- **The fire instrument** (2026-09-24; `fire.rs`, the definitions of `run/fire.py` so a duel and a live game read
  alike). Once a second from the advance order on, for every armed unit: is an enemy of the other army within its
  type's reach plus 20 (`UnitDefInfo::reach`)? If so it is an in-reach second, and the unit's shots in that second
  (`Event::WeaponFired`) count. A unit in reach for 3 s or two reloads, whichever is longer, without a shot is
  muzzled that second, by cause: a friend of its army within 24 of the line to the nearest enemy in reach and nearer
  than it (`line`), that enemy within 40 of the reach's edge (`edge`), else `clear`. Damage events to an army's units
  are booked by attacker: its own units (`ff`, friendly fire), the other army's (the other army's `dealt`). Columns,
  each `_x` and `_y`: `reach_s`, `shots`, `muzzled_line`, `muzzled_edge`, `muzzled_clear`, `ff`, `dealt` (hit points).
  `pairs.csv` adds `shots_per_reach_s`, `muzzled_share` (muzzled over in-reach seconds), the three causes' shares of
  the muzzled seconds, `ff_share` (ff over ff plus dealt) and `exchange` (dealt over dealt by the other). In a mirror
  (same unit, same formation, same lane) both armies' fire counts in the one row. The progress line prints each
  army's muzzled share. `xf_x` / `xf_y` (2026-09-25) is the victim table, friendly fire by `shooter>victim:damage;...`,
  the record's `xf`.
- **The shape instrument** (2026-09-25; `fire.rs` `Shape`, the definitions of `run/replays/shapes.py` so the numbers
  compare with the pro survey's, `docs/knowledge/formations.md`). At the first damage and ten seconds later, over
  each army's living soldiers: every soldier's distance to its nearest friend, and of the soldiers with an enemy
  within reach + 20, how many have a friendly soldier within 24 of the segment to the nearest one and nearer than
  it. Columns `nn` (the median nearest-friend distance over both samples), `fol_in`, `fol_blocked` (counts, summed
  over both); `pairs.csv` gives `nearest_friend` (the mean of the duels' medians) and `friend_on_line` (the pooled
  share); `duel_ab.py` prints both and the friendly-fire share beside the exchange.
- **The chase instrument** (`--chase N`, 2026-09-25; `director.rs` `chase_orders`, `sample_speed`). The second army
  runs from the first (a Move 1,500 along the line away from the chasers' centre, inside the map, re-issued every
  2 s) and the first is sent a Fight at the runners' centre every N frames (0: only when a unit is idle, the plain
  duel's rule); with `--lane on` the lane drives the chasers over that order instead. `speed_x` / `speed_y` are
  each army's mean speed before the first damage, elmos a second per unit-second, to set beside the type's
  maximum. Measured (chase-fav-*, chase-flash-*): a far goal re-sent every 10-60 frames costs a Rover nothing
  (168 of 168) and a Blitz nothing (99 of 101); the lane's formation slot 150 ahead re-issued every 64 elmos cost
  them 25% and 13% (K-engine-a-short-move-order-brakes-the-unit).
- **The queue instrument** (2026-09-25; `fire.rs`, `run/queued.py`'s definitions: `docs/design/2026-09-25-queued-rear.md`).
  Once a second: a soldier with an enemy within 700 is *engaged* (`engaged_s`); engaged with no enemy in reach while
  a friend within 300, in reach and nearer its enemy, fired this second, it is *queued* (`queued_s`); queued with
  that friend within 40 of its line to its nearest enemy, *blocked* (`blocked_s`): behind a firing front rather
  than spread out with nothing to shoot. `duel_ab.py` prints both shares of engaged seconds. The pros' blocked
  share is 8% (geometric), ours in the player games 11-22%.
- **Clearing.** Survivors self-destruct. BAR's "Self-Destruct Resign" gadget (`luarules/gadgets/game_selfd_resign.lua`)
  cancels a team's first two attempts to destroy 95% of its units at once, which a large surviving army beside one
  commander is, so an order not carried out after 8 s is given again (never sooner: a second order while the 5 s
  countdown runs cancels the first). Then crawling bombs (`corroach`) are spawned on a 200-elmo grid over where
  units died and set off, `--sweep-waves` times, to destroy the wrecks. Without this, results drift within a match:
  80 duels on one site took 27 s -> 41 s each as wrecks piled up, and Rocketeer-against-Incisor read -0.10 instead of
  -0.36 (wrecks stop rockets). With three waves both stay flat over 80 duels (batches `wrecks`, `wrecks-swept`).

## Scenarios: a recorded engagement fought again (2026-09-24)

`run/engagement.py` cuts a scenario from a match record at a clock: every armed unit of ours (the `s` line's `own`,
health in percent, no commander, nothing being built) and of the enemy's within `--radius` of the contact (halfway
between our units with an enemy within 800 and the enemies with one of ours within 800, or `--at`), with a heading for
each unit that moved in the second before, and our units' last move / fight / attack order of the 20 s before the clock.
The enemies are those in sight (`--enemies sight`, absolute hit points), also radar contacts of known type (`known`, at
full health), or the opponent's ground truth (`truth`: `truth-<ai>.jsonl`, written when the match ran with
`WITHIN_REASON_OBSERVE=1`; every enemy unit, seen or not, health in percent). It also writes how the engagement went
live (`source.live`: each side's value left 30 and 60 s later, ours from the record, theirs from the truth file), to set
beside the replay. The file format is the doc comment of `crates/arena/src/bin/duel/scenario.rs`; the files cut so far
are in `docs/data/scenarios-2026-09-24/`.

`duel --scenario FILE` fights it `--reps` times on its map, at its place, one repetition after another on the one field
(swept between, as a duel's site is); side 0 is `x` in `duels.csv`, side 1 `y`, and the commanders start in the two
corners of the map farthest from the place. The AI interface's cheat gives a unit at a point and nothing else (no
health, no facing, and no gadget of the game answers an AI's Lua message), so the director prepares a scenario:
1. The units to be hurt (below 99.5%) are given first, both sides; each is told to hold fire and to stop as soon as it
   is seen (hold fire alone keeps a target the weapon took in the frames after it appeared: K-engine-hold-fire-keeps-a-target).
2. Two seconds later (units given close together push each other apart) each gets a hurter from the other side's team,
   a Rover (`armfav`: a beam laser, 35 a shot, one a second, no spray, no area), 100 or 130 from where it stands, in the
   direction whose line to it passes furthest from everybody, with 45 of room round it. Each hurter shoots while its
   target is above the file's health and holds while not, re-shooting if it heals past it; one that has not hurt its
   target for 8 s is walked up to 70 of it. When all are down (or after 90 s) every hurter destroys itself where it
   stands (blast 22 across, 56 damage; the wreck stays) at once.
3. Then everyone else is given at full health, and when all are there both sides are set to fire at will and get
   their first orders: a unit with a heading is sent 24 along it first, then its order from the file (fight, move, or
   attack a unit of the other side), else a fight at the other side's centre; from a second later idle units are sent
   at the other side's centre, as in a duel.
The fight is then scored like a duel, except that `value_left` is over the metal each side started the fight with,
each unit weighted by its health then (`metal_x`/`metal_y` are that), and `health_err_x`/`_y` give the mean distance
between the health the units started with and the file's (a share of full health). A unit killed while it was
prepared is given again whole in step 3, and counts in that error. Measured on the first four scenarios: 0.00 to
0.02 for three, 0.058 for our side of bank-1 27:34. What went wrong on the way there, each fixed and measured:
the Pawn's sprayed gun hurt and killed its target's neighbours (mean error 0.15); held units kept firing at the
hurters they had taken as targets; hurters given with their targets were crushed by the tanks pushed into them
(killed by a Stout that never fired); and a unit left hurt for a minute and a half healed back to whole.

### Scripts, tracks, seeds and the per-second log (2026-09-28)

For the hand-built TAS of one engagement (`docs/design/2026-09-28-tas-micro.md`, `docs/studies/2026-09-28-tas-e3.md`).
- **`--script FILE`**: timed orders for side 0 (ours), JSON rows `{"t", "unit", "cmd", "x", "z" | "target", "queue",
  "state"}` (the format is `scenario.rs`'s doc comment: move, fight, attack, stop, guard, firestate; `unit` an index
  into side 0, a list, or `"all"`; an attack's `target` an index into side 1). Side 0 then gets no order from the
  director (only fire at will and the heading step at the start) and its lane is off; a row goes out on the
  director's first tick at or after `t + --script-delay` seconds from the first orders. Ticks are three frames, so
  a row lands up to 0.1 s after its time; the rows of one tick go out in file order in that tick's one batch of
  commands, which the engine carries out in that order on one frame. A dead unit or dead target skips the row for
  that unit (the log says which). `run/engagement.py --script FILE` writes the record's own orders as a script (an
  order repeating the unit's last dropped, a unit's later orders of one tick queued, a point further than 1,500 from
  the place pulled in to 1,500: a retreat home walked into a commander and ended the match, smoke-e3).
- **`--theirs director|track`**: who orders side 1. `director` is the plain duel (the file's first orders, idle units
  attack-move at our centre). `track` walks the positions the file's `sides[1].track` recorded (`run/engagement.py
  --track`: from the truth file every 2 s, or from the record's sight rows): each mobile unit is moved to its first
  recorded place later than now whenever that changes (and is more than 16 from the last place sent); it dies when
  it dies in the engine, and stands when the rows run out. Checked on E3 (val-e3-record-track): the Centurions stood
  within 15-90 of their recorded places at 5, 10, 20, 30, 45 and 59 s. `ai` (BARb on side 1) is not built.
- **`--economy fed`**: each team is given a `freefusion` (850 energy a second) beside its commander at the start, so
  turrets fire at their rate (K-units-towers-fire-at-the-rate-energy-arrives). On E3 (two light turrets, a 25 s
  fight) it changed nothing measurable (val-e3-director +0.297 ± 0.076 fed, +0.330 ± 0.056 bare).
- **`--seeds a,b,c`**: every duel `--reps` times under each engine seed (`FixedRNGSeed`); a match takes the seed of
  the first job waiting and fights only that seed's jobs. Without it the seed is the match's index plus one, as before.
- **The per-second log**: `seconds/m<match>-s<site>-d<seq>-rep<r>[-seed<s>].jsonl` beside `duels.csv` (named in its
  `log` column), one line a second from the first orders: `t`, `left` (each side's value left), `x` and `y` (every
  living unit as `[index in the file, x, z, health]`), `shots` per unit and `dealt` as `[attacker, victim, damage]` that second (from
  2026-09-28 evening; the first batches, val-* to tas-e3w-v0-director, carry `[attacker, damage]`), each team's stored
  `energy`, and the script rows that went out (`orders`: row, frame, units sent, units skipped).
- **An attack on a unit out of sight does not hunt it.** Queued, the engine drops it when it comes due (tas-e3-v3:
  Centurion 3 lived in 11 of 24); given as the unit's current order it ends at once and the unit stands (tas-e3w-w-v9:
  the Blitzes stood 430 from Centurion 7 for a minute). Send a fight to where the unit was seen instead. Buildings
  once seen can be attacked from out of sight (the turret assaults).
- Readers: `run/scenario_table.py` (one row per batch; `--position` counts the duels in which the named units of side
  1 all died, and those with our value ahead), `run/tas_read.py <log>` (one duel's log as a timeline),
  `run/front_rank.py` (in reach, stood still, blocked and first target by rank, from logs with the victim).
- `duels.csv` gains `seed`, `left30_x/_y`, `left60_x/_y` (each side's value left 30 and 60 s after the first orders,
  the end's value for a fight decided sooner: the live outcome a scenario file carries is scored at the same two) and
  `log`. `batch.json` records the script, the delay, `theirs`, the seeds, the economy and the command line.

## The raid scenario: a picket body against scripted raiders (2026-09-26)

`duel --scenario raid` (`crates/arena/src/bin/duel/raid.rs`; `docs/design/2026-09-26-threat-response.md` §3) is the
cheap loop for the hunt primitive and the raider rules: no BARb, no Jev, one flat site. Ours (`x`, `--ours`, default
four Rovers and two Blitzes) stand at a station in the middle of a strip of four extractors 600 apart, a constructor
guarding the far one; theirs (`y`, `--theirs`, default a Tick and a Pawn) appear 1,400 beyond the station past the
far end and follow a director script: the constructor first, then the extractors far to near (a move to 120 short of
the target with the attack queued, so it lands when the building is in sight), running 500 from any soldier of ours
within 350 and coming back three seconds after the last one has gone. Three minutes (`--time-limit`), or every raider
dead, or nothing of ours left on the strip (`raid_over`). The site is 2,600 by 400 of flat ground (Comet Catcher, the
default map here, offers a handful; `sites::choose` finds them).

The picket answers as the bot's standing raider rule does: a raider within 250 of a building of ours and within 1,200 of
the station, with no hunt on it yet, gets the two nearest pickets faster than it (Rovers after a Tick, Blitzes or
Rovers after a Pawn). **Lane on**: they go as a hunt, `micro::Commitment::Hunt { quarry, leash_from, leash: 900 }`
set on the lane, which sends an attack by id every tick and reports the end (H-MICRO-HUNT); the leash is measured from
where the hunters stood when the hunt began. **Lane off**: the hands' engage as it was before the hunt, a fight-to-point
at the raider's place re-issued every two seconds, judged by the same ends in the director (the quarry dead, out of
sight six seconds, a hunter past the leash, under a third of its health). Either way a hunter whose hunt ended walks
back to its place at the station, and a raider still at a building starts the next hunt. `duels.csv` carries the
score in its last four columns: `extractors_lost`, `first_kill_s` (seconds from the first orders to the first raider
killed), `hunters_lost` (pickets dead), and `hunts` (each hunt's quarry, end and timing: `armflea:leash@3s+7s` began 3 s
in and ended by the leash 7 s later; `open` never ended). The harness prints a `raid f=N: 2 hunters after ...` line at
each hunt's start; `WITHIN_REASON_MICRO_DEBUG=1` adds the lane's claims, drops and ends.

## Checks made (2026-09-19)
| Check | Batch | Result |
|---|---|---|
| Position and team slot | `bias`: Pawn, Blitz, Thug each against itself, 16 times | west won 25, east 22, 1 draw; team 0 won 21, team 1 27: no bias visible at this n |
| Wrecks | `wrecks` / `wrecks-swept` | drift without sweeping, none with (above) |
| Site and speed | `speed200`: 3 pairings x 30 at speed 200 on two sites | margins per site -0.46 / -0.43, -0.35 / -0.37, -0.21 / -0.22; same as speed 50 (-0.43, -0.36) |

| Formation spacing | `sp56`..`sp160`, `t1-matrix-wide` | decides area-damage matchups: Pawn against Mace -0.45 / -0.02 / +0.09 / +0.19 at 56 / 100 / 120 / 160. Run both `--spacing 56` and `--spacing 100` before believing a row |
| Order-level spread | `micro2-off` / `micro2-on` (2026-09-20) | 28 tier-1 pairings x 6 duels an arm, `--spread 110` for the first army: metal killed per metal lost 1.09 -> 1.30, mean margin +0.146, and `spread_x` at contact 81 -> 131 while theirs stayed at 94. All of the gain is the raiders' (`docs/studies/micro-combat.md`) |
| Large armies at wide spacing | `scouts-wide` | 48 of 48 spawned, 57-Tick armies included. Units are claimed within the formation's own extent + 150; a fixed 600 radius missed rear ranks, and the first wide matrix had 112 `spawn_failed` of 1092 (deleted, not used) |

Fixed 2026-09-24: the site rectangle used to leave 170 elmos behind the front rank whatever the army, so rear ranks of
big or widely spaced armies stood outside the ground that was checked for flatness; it now holds the batch's largest
formation (Formations, above).

| Scenario preparation | four scenarios x 8 (2026-09-24) | mean distance from the file's health 0.00-0.02, 0.058 once; spread of the margin over 8 repetitions 0.01-0.09 (sd) |
| Stout ball against a line | `muzzle-stout-shapes` (2026-09-24) | 24 against 24 Stouts, `ranks8`, `line`, `line/line`, 8 each on Mithril Mountain: the ball is not muzzled (0.0-0.4% of in-reach seconds, 0.79-0.84 shots a second of 0.83 possible) and beats the line (-0.13 +- 0.03 for the line, 1-7); `docs/studies/2026-09-24-muzzled-ball.md` |

## Cost
An engine start is ~30 s. The full 23-unit table (2184 duels) took 568 s of wall time on two engines at `--speed 200`
with nine other engines busy on the machine: ~4 duels a second (a duel is ~25 game seconds plus ~10 of clearing). One engine uses ~3 GB and about two cores.

## What a duel is not
Without `--lane`, no micro beyond the `line` formation (no kiting, no retreat, no focus fire beyond the engine's own
targeting); no terrain, no support (radar, repair, turrets behind the line), one army size. Both sides charge: a
unit that would normally hold at its range and be approached is tested as an attacker too. See the caveats in
K-units-duel-caveats.
