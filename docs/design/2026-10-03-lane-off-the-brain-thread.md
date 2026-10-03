# The control lane off the brain's thread

Status: proposed 2026-10-03, not built. The user's concern: "The main thing to be concerned about are degradations in
actual realtime gameplay. Here it's merely an inconvenience, but if it falls behind in a human game it can actually
cause issues."

## 1. What is measured

- The bot is one thread. `main.rs` reads a tick, calls `Brain::decide`, writes the commands. The shim sends the next
  tick only when the last was answered (`ai-shim` `tick_due`, `has_credit`), so a slow `decide` makes every later
  tick late, and the lane (`crates/micro`: flee, kite, fan, form, hunt, rove) runs inside `decide`, after the brain.
- `decide` runs the whole brain (`think`: the trackers, the pianist's picture, menus and pass, the report) on the
  frames due every `BRAIN_FRAMES` (15), the pianist's pass once a second (`interval_frames` 30), and the lane on
  every tick.
- player-50 (lockstep, minutes 26-30, 94 groups, about 250 units): a game second cost 1.6 s of wall, about 0.8 s in
  Jev calls (not on this thread in realtime: the worker, `spawn_worker`) and about 0.8 s outside them, of which
  0.5-0.6 s was this thread's own work (a 25 s `perf` sample: builder menus 9%, the threat grid 5%, JSON 5%,
  passages 4%, the rest spread; `docs/experiments.md`, player-50).
- The human games so far were small and kept up: ticks of 18 to 105 ms at the median with 38 to 105 units, 216-283 ms
  at the 95th percentile, 728 ms at most; lateness 3-6 frames at the 95th percentile, 19 frames at most (human-11 to
  -13, the record's `ms` and `late`). The median grows with the unit count. No human game has reached player-50's
  size; at it the pass would hold the lane for half a second or more, once a second, in the fights.

## 2. The design

Two threads, split at the lane.

**The fast thread** (the main loop) owns the socket, the recorder and the `Lane`. For every tick it: takes what the
brain has sent since the last tick (orders, standing orders, commitments and footwork, a fresh view), runs
`Lane::tick` over them, answers the shim. In realtime it never waits for the brain.

**The brain thread** owns the `Brain` as it is, less the lane. It is handed each tick's snapshot and events (the
events of ticks it did not get to are carried, as `carried_events` does today; it always works on the newest
snapshot) and runs `think`. What it produces goes to the fast thread stamped with the frame it was computed from.

**What crosses.**

| From | To | What | Today |
|---|---|---|---|
| brain | lane | the brain's orders of a think (the lane may rewrite them: the march) | `commands`, in the same call |
| brain | lane | standing orders, commitments, footwork | `note_standing_orders`, `note_commitments` |
| brain | lane | the view's changing part: remembered enemy buildings (finished ones), known enemy soldiers and the blip type, the rove goals, the disabled rules | `BrainView` reading `&Brain` |
| brain | lane | the view's fixed part, once, behind an `Arc`: unit definitions, the simulator's stats, the terrain grid, the passable cells, the class fields `snap` uses, home | the same |
| lane | brain | hunts ended, rove events, rules fired, milling | `Output`, applied in `Brain::micro` |

The lane's `View` is implemented over that snapshot (`LaneView`), with no borrow of the brain.

**Lockstep stays as it is.** With `WITHIN_REASON_LOCKSTEP` the fast thread waits for the brain's result on the ticks
the brain is due at, so the arena's games run in the same order as today (the world held while the brain and Jev
work) and stay comparable with the ledger.

**What changes in a realtime game.** The brain's orders reach the engine one to several ticks after the snapshot
they were computed from, as Jev's answers already do; the lane rewrites them against the positions of the tick they
arrive at. The lane's own reactions keep the tick rate whatever the brain costs. The record's `ms` becomes the fast
thread's; the brain's time is recorded beside it.

## 3. Rejected

- **Threads inside the pass** (the menus per actor in parallel). It divides the cost by the cores and bounds nothing:
  the lane still waits for the pass, and the brain holds channels and locks that are not shareable as they stand.
  May still be worth doing afterwards for the pass's own cadence.
- **The pass alone on a worker.** The picture and the menus read the whole brain; a worker would need a copy of it
  every second.
- **Fewer actors** (the hunt splinters are most of player-50's 94 groups). Wanted for cost and for the churn, and
  not a bound either.

## 4. Steps

1. `LaneView`: the snapshot type and its `View`, filled from the brain and used on the one thread. No behaviour
   change; the lane's tests and one lockstep game.
2. The two threads with the lockstep wait. An arena game against the same seed.
3. Realtime without the wait. Measured in a realtime arena game (`--realtime`) grown to player-50's size: the
   record's `late`, the fast thread's `ms`, the brain's.

## 5. Open

- Whether the pass's log line (about 1 MB a second, JSON 5% of the profile) is written in full in realtime.
- A think that takes longer than its interval: the brain then runs back to back on the newest snapshot; whether the
  pass should say so in the per-minute line beside the late ticks.
- The lane's hunt and rove events reach the brain a think later than today.
