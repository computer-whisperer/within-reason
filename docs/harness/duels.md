# Duels — unit against unit

`target/release/duel` spawns N units of one type against M of another on flat ground, sends them at each other and
records who is left. It exists to fill the matchup table the brain and the LLM commander choose a unit mix from
(`docs/knowledge/units.md`, K-units-duel-*).

```
duel (--units a,b,c | --ours a,b --theirs c,d | --pairs a:b,c:d)
     [--reps 4] [--budget 1200 | --count N] [--parallel 2] [--sites 3] [--duels-per-match 45] [--time-limit 240]
     [--sweep-waves 3] [--spacing 56] [--formation X[/Y],...] [--spread 0] [--speed 50] [--map NAME] [--label TEXT]
     [--base-port 9500]
duel --scenario FILE [--reps 8] [--parallel 2] [--time-limit 240] [--speed 50] [--label TEXT] [--base-port 9500]
duel --report DIR [duels.csv ...]      rebuild the tables in DIR (from its own duels.csv, or merge the files named)
run/engagement.py <match dir> <MM:SS> [--radius 900] [--at X,Z] [--enemies sight|known|truth] [--orders 20]
                  [--after 30,60] [--out FILE]     cut a scenario file from a live record
```
`--units` runs every pair from the list, each unit against itself included; `--ours/--theirs` the cross product;
`--pairs` exactly those. Unit names are the game's internal ones (`armpw`). Two buildings are never paired.

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
- **Micro.** `--spread N` changes the orders the **first** unit of each pairing is given: instead of one attack-move
  at the enemy's centre, each of its units is sent to its own point in a block around the enemy, N elmos between
  neighbours, files across the approach and ranks behind (`director.rs` `loose_block`, a copy of the bot's
  `brain::micro::loose_block` — change both together). The second army always gets the plain blob order. Run a batch
  with and without it and compare with `run/duel_ab.py <without> <with>`; this is how H-MICRO-SPREAD was measured
  (`docs/studies/micro-combat.md`).
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
  (same unit, same formation) both armies' fire counts in the one row. The progress line prints each army's muzzled
  share.
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
No micro beyond `--spread` and the `line` formation (no kiting, no retreat, no focus fire beyond the engine's own targeting), no terrain, no mixed armies, no
support (radar, repair, turrets behind the line), one army size. Both sides charge: a unit that would normally hold at
its range and be approached is tested as an attacker too. See the caveats in K-units-duel-caveats.
