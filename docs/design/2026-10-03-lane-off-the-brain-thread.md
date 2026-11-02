# The control lane off the brain's thread

Status: proposed 2026-10-03; built 2026-11-01 (the user: "Let's fix the core problem with blocking micro engine ticks
in realtime mode"). The user's concern of 2026-10-03: "The main thing to be concerned about are degradations in actual
realtime gameplay. Here it's merely an inconvenience, but if it falls behind in a human game it can actually cause
issues."

## 1. What is measured

- The bot was one thread a session. `main.rs` read a tick, called `Brain::decide`, wrote the commands. The shim sends
  the next tick only when the last was answered (`ai-host` `tick_due`, `has_credit`), so a slow `decide` makes every
  later tick late, and the lane (`crates/micro`: flee, kite, fan, form, hunt, rove, the groups' march) ran inside
  `decide`, after the brain.
- `decide` ran the whole brain (`think`: the trackers, the pianist's picture, menus and pass, the report) on the frames
  due every `BRAIN_FRAMES` (15), the pianist's pass once a second (`interval_frames` 30), and the lane on every tick
  (every 3 frames, 10 Hz).
- The games with people, from the records' `ms` (the slowest `decide` of each second) and `late` (the most frames a
  tick of the second waited for the previous answer; one tick is 3 frames):

  | game | map | units at most | `ms` median / p95 / max | seconds with a late tick | `late` p95 / max |
  |---|---|---|---|---|---|
  | 1792971120-human seat 1 (2026-10-25) | All That Glitters Extended | 319 | 42 / 168 / 1,253 | 17 % | 3 / 27 |
  | the same, 35-40 min | | 313 | 60 / 214 / 414 | 27 % | 4 / 21 |
  | 1792969892-human seat 0 | Comet Catcher | 72 | 20 / 150 / 465 | 20 % | 2 / 12 |
  | 1792812445-human seat 0 | Comet Catcher | 126 | 11 / 163 / 462 | 16 % | 2 / 13 |

  At 300 units the pass cost 170-210 ms once a second at the 95th percentile, so the lane missed one or two of its
  ticks in a fifth to a quarter of all seconds, and the slowest seconds held it for a dozen ticks.
- Two network calls run in place on the tick thread. The gate and the pick go through a worker in realtime already
  (`pianist/mod.rs` `Worker`), but the `fuse` layer's decode call does not (`decode_and_fuse`): in barb-ladder4-30-1
  it was made 833 times in 40 minutes, median 98 ms, 95th percentile 316 ms, longest 461 ms; each one, in a realtime
  game, holds the lane for one to five ticks.
- Earlier: player-50 (lockstep, minutes 26-30, 94 groups, about 250 units): a game second cost 1.6 s of wall, about
  0.8 s in Jev calls and about 0.8 s outside them, of which 0.5-0.6 s was this thread's own work (`docs/experiments.md`,
  player-50).

## 2. The design

Two threads a session, split at the lane (`crates/bot/src/session.rs`).

**The fast thread** (the session's own) owns the socket, the recorder and the `Lane`. For every tick it: hands the tick
to the brain thread along with the lane's report of the previous tick; takes what the brain has answered (in lockstep,
waits for this tick's answer; in realtime, whatever answers have arrived, none included); applies each answer to the
lane in order (the standing orders, the declared groups, the fixed and changing parts of the view); runs `Lane::tick`
over the brain's commands; records; answers the shim. In realtime it never waits for the brain.

**The brain thread** owns the `Brain` as it was, less the lane. It runs `decide` on every tick it is handed. When
several ticks have piled up (realtime, the brain slower than the tick) it folds them into one: the newest snapshot,
every event in order, and `late` set so that `Tick::due` is the think frame among them when there was one, so the
brain's schedule (`due() % BRAIN_FRAMES`) sees the think it owes and not a missed one. In lockstep nothing piles up.

**What crosses.**

| From | To | What | Carried as |
|---|---|---|---|
| fast | brain | the tick | `Arc<Tick>` |
| fast | brain | the lane's report of the previous tick: hunts ended, rove events, rules fired, the milling counters, every declared group's `GroupState`, every rover's `RoveReport` | `LaneReport` |
| brain | fast | the brain's commands of a decide (the lane may rewrite them: the march) | `BrainResult::commands` |
| brain | fast | the soldiers' orders among them, for the lane's standing orders (the brain knows who is a soldier) | `BrainResult::standing` |
| brain | fast | the groups declared to the lane (`declare_groups`, now a list rather than calls on the lane), when the brain re-declared | `BrainResult::declared` |
| brain | fast | the view's fixed part, once, when the faction is adopted: unit definitions, each type's `Stats`, the terrain, the grid, home, the disabled rules, the label | `Arc<LaneFixed>` |
| brain | fast | the view's changing part, with every think: remembered enemy buildings, known enemy soldiers, the blip type, each enemy's stun, the rove goals per type that roves | `Arc<LaneScene>` |
| brain | fast | the brain's journal (its notes, rules, intent) and its own wall time | `BrainResult::journal`, `decide_ms` |

The lane's `View` is `SceneView`, the fixed and changing parts together; nothing borrows the brain. The brain reads
the lane through the report: `Brain::lane_groups` and `Brain::lane_roving` are what `self.lane.group_state` and
`self.lane.roving` read before (the picture, the groups' words, the screen point of a shelling hold).

**Lockstep is the same game as before.** The fast thread waits for the brain's answer at every tick, so the order is
today's: think, standing orders, declared groups, the lane, all on the tick's own snapshot, the world held while the
brain and Jev work. One difference: a hunt's end and a rover's event reach the brain with the next tick (3 frames)
rather than in the same `decide`.

**What changes in a realtime game.** The brain's orders reach the engine one to several ticks after the snapshot they
were computed from, as Jev's answers already do; the lane rewrites them against the positions of the tick they arrive
at. The lane's own reactions keep the tick rate whatever the brain costs. The record's `ms` is the fast thread's tick
(in lockstep it includes the wait, so it stays the whole `decide`); `brain_ms` beside it is the slowest brain `decide`
whose answer landed in the sample's second.

## 3. Rejected

- **Threads inside the pass** (the menus per actor in parallel). It divides the cost by the cores and bounds nothing:
  the lane still waits for the pass, and the brain holds channels and locks that are not shareable as they stand.
  May still be worth doing afterwards for the pass's own cadence.
- **The pass alone on a worker.** The picture and the menus read the whole brain; a worker would need a copy of it
  every second.
- **Fewer actors** (the hunt splinters are most of player-50's 94 groups). Wanted for cost and for the churn, and
  not a bound either.
- **Making the brain cheap enough instead** (the econ-flagship arc cut the menu's layout searches from 8,986 s to
  1,262 s of wall over a game). Worth doing for its own sake; it bounds nothing, since every new reading of the game
  lands on the same thread, and a network call on it (the decode, 100-460 ms) is never cheap enough.

## 4. Judged by

1. The bot's and the micro crate's tests; a lockstep drill batch unchanged (the drills run the lane in the duel
   director, not the bot: they cannot change).
2. A lockstep arena game: runs to its end, `late` 0 throughout, `ms` of the order of before.
3. A realtime arena game (`--realtime`): `late` stays at 0 while `brain_ms` runs at the brain's cost; the per-minute
   line's late ticks are the brain's own folds.

## 5. Open

- The decode call still holds the brain thread (not the lane) for 100-460 ms a call in realtime: the hands' pass is
  that much later. A worker for it, as the gate has, is the next step if the brain's lateness shows in a game.
- Whether the pass's log line (about 1 MB a second) is written in full in realtime.
- A think that takes longer than its interval: the brain runs back to back on the newest snapshot (the fold); the
  per-minute line counts the folds as late ticks.
