# Target design: the muzzled ball — sim fidelity, scenario replication, micro options

Written 2026-09-24 at the user's direction, before any code (the user: "The stout ball muzzling is definitely a problem
now"; the project agreed as phases, the first two to an Opus agent in a worktree). Branch `muzzle-sim`, worktree
`/home/christian/workspace/playground/bar_bots-muzzle`.

**Status (2026-09-24).** Phase 1 built and measured (`muzzle-stout-shapes`): on flat ground a ball of 24 Stouts is
not muzzled (under 0.5% of in-reach seconds, full rate of fire) and beats a line abreast; the live 14-18% comes from
something the flat mirror duel lacks (`docs/studies/2026-09-24-muzzled-ball.md`). Phase 2 built and measured:
`run/engagement.py` cuts a scenario, `duel --scenario` fights it (health by hurting after spawning, facing by a short
move: the cheat sets neither); the three engagements replay stably (margin sd 0.01-0.09 over 8), and only bank-1 27:34
(out-ranged by Fatboys) muzzles us, 23%. The simulator cannot take the files yet (no per-unit health or orders, no
reader, no hulls): phase 3's list. Phases 3 and 4 not started.

## The problem, as measured
- The engine refuses a shot whose line (or a cannon's arc) crosses an allied hull: every weapon keeps `avoidFriendly`,
  BAR overrides it nowhere; a unit with its target in range and its shot refused stops and points, it does not step
  aside (`docs/knowledge/mechanics.md`, K-engine-a-shot-is-refused-across-a-friend). Sight is terrain-only.
- Live games (the fire instrument, `run/fire.py`; `docs/harness/record-format.md`): Stout balls sat muzzled 12% of
  their in-reach seconds in bank-1 (0.45 shots per in-reach second against the Blitz's 4.24), 17% in 2v1b-hard,
  24% in 2v1b-hard_aggressive; friendly fire 13.2% of all damage in bank-1, three quarters of it the Stout.
- The simulator (`crates/combatsim`) has no hulls and no line-of-fire test: it cannot see any of this, so it cannot
  rank a formation or a footwork rule against another on the thing that is now losing fights.

## What exists
- `crates/combatsim`: range, speed, arrival order, mixed forces, ground; `Micro`/`Focus`/`Intent` in `scenario.rs`, its
  own policies in `policies.rs`; judged by `combatsim validate` against the engine duel tables of 2026-09-19
  (`docs/data/duels-2026-09-19/`, `docs/studies/combat-sim.md`).
- `target/release/duel` (`crates/arena/src/bin/duel/`, `docs/harness/duels.md`): N of one type against M of another,
  spawned by the AI cheat on flat sites 1440 x 640, ranks of eight 56 apart, 1100 apart, Fight orders at the enemy's
  centre; both sides are the duel director (the bot's brain and `micro.rs` are not involved); `duels.csv` with a
  dispersion probe; `run/duel_ab.py` compares two arms on the same pairings.
- `crates/bot/src/brain/micro.rs`: the 10 Hz footwork (flee, focus, kite, march, follow, raw, the D-gun fan) that the
  live bot runs; `run/micro_ledger.py` judges a batch's fighting (exchange, time under fire, deaths in reach).
- The shim's `EVENT_WEAPON_FIRED` and `EVENT_ENEMY_DAMAGED` (`Event::WeaponFired`, `Event::EnemyDamaged`), which the
  bot's recorder turns into `shots`, `dealt`, `ff`, `xo`, `xi`; the duel director does not record them yet.

## Phases (each ends with a validation run and a commit; the design here is the ground truth across compactions)
1. **The harness sees muzzling.** The duel director records per-duel: shots per in-reach soldier-second by type,
   muzzled seconds (the `run/fire.py` definition: in reach, no shot for max(3 s, 2 x reload)) by cause (a friend on
   the line within a hull's width, the target at the reach's edge, clear), friendly fire, and the exchange. New
   columns in `duels.csv`, `pairs.csv`; a `--formation` knob (ranks of N, line abreast, spacing) so the same pairing
   can be fought in several shapes. Validation: a Stout-against-Stout batch in ranks of eight against a line
   abreast, the muzzled share and the margin side by side; the numbers go into a `docs/studies/` page.
2. **Scenario replication.** `duel --scenario FILE`: types, positions, facing, health and the first orders per side, on
   a named map at a named place, spawned as written (the cheat gives units; health set by damage or by the cheat's
   options if it has one; facing by a short move if it has none), then the sides' orders. A tool
   (`run/engagement.py <record> <clock> [--radius]`) cuts a scenario from a live record's `s` line at the clock (own
   units and enemies in sight with positions and health), so a recorded engagement is replayed in the engine and in
   the sim from one file. The three to cut: 2v1b-hard 15:10 (the D6 fight, 30 Stouts, won; `run/matches/1790261454-2v1b-hard/00`),
   2v1b-hard 13:50 (the trade 4.0k against 1.4k), bank-1 27:34 (Fatboy splash, 14 of a packed ball in 11 s;
   `run/matches/1790257668-bank-1/00`). Validation: each scenario fought in the engine `--reps 8`, the spread of
   outcomes recorded; the sim's outcome on the same file beside it.
3. **Sim fidelity** (after 1 and 2): hull radii from the unit table, the line-of-fire test against friends before
   each simulated shot, stop-and-point when refused, splash on friends (`collideFriendly`), cannon arcs where they
   differ from rays. Judged by the phase 1 and 2 tables: the muzzled share and the margin per formation, ordinal
   agreement first, magnitude second.
4. **One micro implementation.** The footwork decisions of `micro.rs` as pure functions over a world-view trait, called
   by the bot's tick and the sim's step; the sim's own `policies.rs` retired where it duplicates them. Then the
   options for a muzzled ball, searched in the sim and checked in the engine by phase 2: a wider line at contact
   (`line` footwork), spacing by hull width, the rear rank stepping out to a flank, target spread across the front,
   a range-mixed front. Validation: `duel_ab.py` on the phase 2 scenarios with the rule on and off; then one live
   game read with `run/fire.py` and `run/micro_ledger.py`.

## Constraints
- The worktree has hard-linked copies of `run/engines` and `run/data` (the AI installs into its own
  `run/data/AI/Skirmish/WReason/0.1`; the main tree's arena is never touched). Ports 9500-9599 (the main tree uses
  9100 and 9200). Never `~/.local/share/BeyondAllReason`; never `pkill -f`/`pgrep -f`; never `git add -A`; commit on
  the branch with the attribution line; every brain change registers a heuristic (`docs/heuristics.md`) and a claim
  (`docs/knowledge/`), every batch a row in `docs/experiments.md`.
- The live records under the main tree's `run/matches/` are read-only inputs (they are git-ignored working data).
- The sim will not match the engine's pathing, collisions and turn rates: the target is ordinal agreement between
  options, with the engine table as the judge, never the sim alone.
