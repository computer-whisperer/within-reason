# Army movement as the micro engine's: groups in the lane, and the drill bench

2026-10-24. Follows `2026-10-24-pick-rework.md` (the pick is one draw, the t-1 words out; what the gate's removal
exposed is the idle body re-asked, the gather arrival and detachments by draw) and the discussion of the same day on
group control. Design written before the first line of code, as the arc's ground truth.

## 0. The rulings (the user, 2026-10-24)

- "I think we need to deeply rework group control. A group of units scattered across the map is exactly that, and the
  simplified model of blob + tail is part of the problem."
- "I would put weight here on reducing the control system complexity of this in general. What we have now sounds like
  a built-up set of heuristics and filters that sometimes take actions and decisions that are not generally robust.
  Whatever we build should start out simpler and more likely to work in all scenarios."
- "My thoughts are turning towards formation control as a critical part of this: keeping the units in formation
  trivially modellable and the units out of formation as the exceptions to build filters and rules for."
- "We may turn this project into an army movement and formation arc, tested with no LLM and only various low level
  layers engaging on various maps with something akin to military drill scenarios. As one example: given a lab
  producing units and an enemy defensive structure: gather units coherently and attack into it."
- "Agreed on this being an expansion of the micro engine. Rather than adding abstraction layers, this should be a pure
  test of the micro engine (with group controls built in rather than on top) in standard scenarios. Let's start with
  any needed refactoring and micro arena upkeep or construction needed to make the bench tests accurate for what we
  are trying to do."

The regime: `crates/micro` is the one place where code deciding live is already law (the micro engine's exception to
the flagship rule, `docs/code-decisions.md`). The drills run with no model. The exit into the flagship is words: the
hands read the lane's group state, the moves stay Jev's.

## 1. Where groups are in the stack today

| layer | what it holds | what it decides |
|---|---|---|
| the hands, `crates/bot/src/brain/pianist/groups.rs` | membership (the player's: a plant's output joins the group named for it), one task (hold, move, engage, hunt, follow, rove), the body as a sort along the goal axis (front = the nearest member, tail = the farthest, "the body's place" = the 60%-th member), `joining` (newcomers walking to the body, by origin), `waiting` (the player's `join_in`), the stragglers layer (off in the arena target line), `reached`, `met`, losses | the standing-order upkeep: arrival (the 60%-th member within 300) ends a move in a hold nobody picked; joiners re-sent to the nearest core member when it has moved 150 and adopted within 400; an attack follows its party by 150-steps; the gather (everyone to the snapped front, ended by the arrival test or the whole length under 600); the stall warning; commitments and footwork handed to the lane per unit by task kind (`brain/micro.rs` `note_commitments`: advance = everything, picked hold = mobiles only, walk = nothing) |
| the lane, `crates/micro` | per unit: the host's last order (`standing`), a commitment, a footwork setting, a claim; bodies **inferred** every tick: units under the same order (points within 48, or one attack target), of one class (raider or line), chained within 400 (`form::bodies`) | the march: the host's order rewritten once into ranks of six at the destination and a form-up rank three seconds ahead (nothing issued between the host's orders); at contact (an armed enemy within 700 of the body's centre) the arc at reach round a target, the walk to the slot, standing in reach; flee, fan, kite per unit; hunts; rovers |
| the duel director, `crates/arena/src/bin/duel` | an army as a unit set with a spawn formation; per unit a Fight at the enemy's centre; a `micro::Lane` per army under `Commitment::All`; idle units re-sent every 2 s | when the fight is decided (wiped, timeout, stalemate), the instruments (fire, shape, speed, queue, per-second log) |

What no layer has: a pace (every unit walks its own line at its own speed), a group object the lane is told rather
than infers, an "arrived" the lane reports, a formation state (who is in a slot and who is not, and why) anyone can
read. The engine side (read 2026-10-24 from `RecoilEngine/rts`): the AI interface has no speed command (the
set-wanted-max-speed command is marked unused); a unit's speed is capped at its goal distance times its turn rate
while its current waypoint is its goal (`GroundMoveType::ChangeSpeed`, the `atEndOfPath` rule), which is what slowed
the old lane's near steps (K-engine-a-short-move-order-brakes-the-unit's measurement, with its mechanism); vehicles
slow only for turns over 90 degrees, to a tenth of their speed while they turn; the one native pacing is slow guard
(`MobileCAI::ExecuteGuard`: a guarder within 200 of a moving guardee takes the guardee's speed as its wanted maximum
and a goal of its own position plus the guardee's velocity times 2.13 s; BAR's `modrules.lua` guard block).

## 2. The target (the arc after this one): the lane owns the group

The host (the hands, or the duel director) declares a group once per decision and gives per-unit orders only to
units in no group:

```rust
// crates/micro
pub struct GroupOrder { pub kind: GroupKind, pub goal: Vec3 }   // Go | FightTo | Attack(UnitId) | Hold
Lane::set_group(name: &str, members: Vec<UnitId>, order: GroupOrder, footwork: Footwork, commitment: Commitment)
Lane::clear_group(name)
Lane::group_state(name) -> GroupState {
    anchor: Vec3, heading: (f32, f32), pace: f32,           // the formation's place, way and speed this tick
    shape: Shape,                                           // ranks on the march; the arc at contact
    formed: Vec<UnitId>,                                    // within a hull or two of their slot
    exceptions: Vec<(UnitId, Cause, f32)>,                  // out of slot: why (slower, stuck, fled, blocked, joining), seconds to its slot
    arrived: bool,                                          // the anchor at the goal and the formed within their slots
}
```

The lane keeps the formation up: slots as today (ranks of six on the march, the arc at contact), **pacing** by one of
the arms the bench measures (the lead law: a unit's goal placed ahead of it by `pace / (turn_rate * k)` so the engine's
own cap holds it at the pace, re-issued at a cadence; slow guard on a pacesetter; or no pacing and a form-up point
short of contact), the joiners (members far from the shape walk to their slot), and the state. The bodies-by-chaining
inference goes; the hands' standing-order upkeep for soldiers (arrival, joiners, the gather, the follow steps) goes
into the lane or into words. The hands' words become the group state: "group_A: 11 in formation at X heading east at
75; 3 out of it: 2 Blitzes 400 ahead (faster), 1 Stout stuck at Y" and the moves stay what they are.

Not built in this arc. Written here so the bench is built to judge it.

## 3. The bench (this arc)

`duel --drill walk|reverse|gather` fights no one: one army (`x`, `--ours`, a mixed force as written), one site, one
order, measured. Reuses the engine start, the sites, the sweep, `duels.csv`, the per-second log and the lane arms
(`--lane off|old|on`, `WITHIN_REASON_DISABLE`, `WITHIN_REASON_FORM_SPACING`).

**Decisions taken here** (frozen for the arc, surfaced for reopening):

- **D1. Drills are the duel binary's.** A separate binary would copy the engine start, the sweep, the CSV and the
  log, which are four fifths of a harness.
- **D2. One army, side `y` empty.** The judge's "wiped" and "stalemate" do not apply to a drill; it ends on
  `arrived` (every living unit within `DRILL_ARRIVED` = 300 of the goal, the hands' own arrival number) or `timeout`.
  The idle re-send at the enemy's centre is off; `--reissue S` re-sends the drill order to every living unit every
  S seconds (the hands' pattern), default 0 = once.
- **D3. The site is a strip `--distance` long** (default 2,000) plus the formation's depth and the arrival radius with
  margins, dry and passable for every listed unit, within 48 elmos of relief (`sites::Footprint::strip`; a walk needs
  crossable ground, not the duel's firing-range 16). The army spawns at the west end facing south (the engine's cheat
  gives no facing), so every walk begins with a quarter turn; the clock starts at the first order and every arm
  shares the turn. **Found building it (drill-smoke, 2026-10-24):** at width 640 Quicksilver's longest dry passable
  west-east strip is 3,200 and Comet Catcher's 1,920, so the standard walk is 2,000 on Quicksilver (the duel's own
  map) and Comet hosts 1,300 at most; and the duel's commanders in the north-west and south-east corners left no
  2,540 strip 900 from both, so a drill starts both commanders along the north edge (`script::DRILL_BOXES`). The
  message for a map with no strip says the flattest one's relief and place against the commanders, or the longest
  strip of that width.
- **D4. The order is `--order fight|move`** (default fight, the hands' advance). `reverse` gives the same order back
  to the start point at `--turn-at S` seconds (default 12: a 2,000 walk takes a Blitz 20 s, so the fast units have
  not arrived) and ends when every unit is back within 300 of it.
- **D5. `gather` gives a lab** (`--lab armvp` by default, `armlab` for bots) at the west end and a build list
  (`--produce armflash*6+armstump*4+armjanus*2`, the opener's mix) under the resource cheat
  (`Command::GiveResource`, new: the engine's `COMMAND_CHEATS_GIVE_ME_RESOURCE`, honoured as the unit cheat is,
  locally hosted with one player; found 2026-10-24 to give metal only, an engine bug, so a gather runs
  `--economy fed` for its energy: the first baselines had the lab energy-starved from 30 s on), each new soldier of the team adopted into the army as it appears within 400 of
  the lab and sent the drill order to the rally `--distance` east. Ends when every produced unit has arrived, or at
  the time limit. Measures the stream: seconds from a unit's appearance to its arrival, and the formation's spread
  as the joiners come in.
- **D6. The instruments are the per-second log** (positions of every unit each second, already written for
  scenarios, now for drills too, with a first `meta` line naming each unit's type by index, the strip's ends and the
  arms) read offline by `run/drill_table.py` (`--types` for each type's own arrival and speed share), plus summary
  columns appended to `duels.csv` so comparisons work without the logs: `drill, arrive_first_s, arrive_median_s,
  arrive_last_s, spread_arrival, nn_arrival, speed_share, spin_up_s, recover_s, produced, join_median_s`. Appended at
  the end: old readers take columns by name. Built 2026-10-24 (`drill.rs`, `director.rs` `drill_orders` /
  `drill_sample` / `drill_meta`, `report.rs`, `run/drill_table.py`; `Command::GiveResource` in the protocol and the
  host); the duel binary's and the drill's tests pass; the smoke numbers are in `docs/harness/duels.md`.
- **D7. The baselines are run before any lane change**: the lane off and on, pure and mixed columns, two spawn
  shapes. Their numbers are the bars the group arc is judged against.
- **D8. The depth and width measure** (the user, 2026-10-24, after the baselines: "I would measure the columns by how
  much of the army group is able to engage with an enemy at once, while maintaining speed. A column is much more
  dangerous if it is moving in formation abreast rather than single-file"; "Go, start with the depth and width
  measure"). Each second the units are projected onto the body's heading (on a strip, the axis from the end behind to
  the goal in force; on terrain, to come, the centroid's velocity): the body's **depth** is the leading unit's
  projection less the rearmost's, its **width** the extent across the heading. Against an enemy front the units come
  into reach in depth order at their own speed, so each unit's depth behind the leader over its type's speed is the
  time until it is firing beside the leader: the **ramp to half** is the median of those times over the army, the
  **ramp to all** the largest. No bar in any of it: positions, the axis and the glossary's speeds. Logged every second
  (`shape: [depth, width, ramp_half_s, ramp_all_s]` on the per-second line) and summarised in `duels.csv` at
  **contact**, the second the first unit arrives at the goal in force (for a reverse, the first unit back at the
  start after the turn), which is when a front at the goal would be met: `depth_contact`, `width_contact`,
  `ramp_half_s`, `ramp_all_s`. Read beside `speed_share`: a body that forms by stopping shows it there. A gather's
  contact shape is the stream's first unit alone and says nothing; its measure is the next drill's.

**What the bench must get right to be accurate for movement** (each a fault found in the survey or a property of the
engine, not a guess):

1. The quarter turn at the start is shared by every arm and reported (`spin_up_s`: seconds until the median unit
   reaches 90% of its type's speed), never hidden in the arrival time.
2. No order of the director's between the drill order and the end but the explicit `--reissue`: the lane's march is
   a transform of the host's order, and a re-send is a different host.
3. A drill ends on arrival or the time limit alone.
4. Flat sites only in this arc: `DuelView::passable` is `None` and `snap` is the identity, so a chokepoint or a cliff
   drill needs the terrain in the view first (drills 6 and 7 below).
5. Speeds from positions a second apart under-read a turning unit slightly; the type's maximum is the engine's
   `speed` from the definitions, so `speed_share` is a ratio of like measures only on straight walks. Said in the
   reader.
6. The engine starts no unit until it has a path (`WAIT_FOR_PATH` is 0 in this Recoil: it does start; the path
   arrives later), and twenty units ordered on one frame request twenty paths. Whatever that costs is in the
   measurement, as it is in a game.

## 4. The drills, in order, with the bars the baselines will set

| # | drill | what it measures | the bar proposed (the baselines replace it) |
|---|---|---|---|
| 1 | walk: a column 3,000 east | arrival first / median / last, spread and nearest friend at the median arrival, speed share per type, spin-up | last arrival within 1.15 x distance over the slowest type's speed; nearest friend 55-90 at arrival (the pros' 67-70); no unit more than 20 s ahead of the median |
| 2 | reverse: the column turned back at 20 s | seconds from the turn until the median speed is back at 90% of the type's; spread after against before | recovery within the per-unit reversal time from the defs (Blitz 4.5 s, Stout 5.9 s, Janus 7.4 s) plus 3 s; spread not grown |
| 3 | gather: a lab producing into a rally | per unit, seconds from appearance to arrival; the formation's spread as joiners come | arrival within the walk at the type's speed plus 5 s; the body's spread unchanged by a joiner |
| 4 | attack into a structure (exists: the E3 and F3 scenarios, the TAS as the reference) | margin, in-reach while engaged, muzzled, friend on the line | the body-shape study's numbers |
| 5 | two bodies abreast into a party | the same, and the bodies' separation | not before 1-3 |
| 6 | a chokepoint and a cliff on other maps (Great Divide's passages, Cape Violet's shore) | 1 and 2 on terrain | needs the terrain in the duel's view |
| 7 | the raid answer from the stream while the bodies hold their course | the raid scenario's score with a producing lab behind the picket | after the group API |

Pacing arms for drills 1-3 once the group API exists: none (the march as today), the lead law at 1 Hz and at 0.5 Hz,
slow guard on the slowest unit. Each against the baseline, 8 repetitions, two seeds.

## 5. Baselines

Run 2026-10-24 evening on febecda+ (the bench as committed with this note), Quicksilver, the 2,000 strip, twelve
units in ranks of six at 64, 8 duels an arm (`--reps 8 --parallel 2 --sites 1 --duels-per-match 4`), the lane off
and on; the table is `run/drill_table.py --types` over the batches `*-duel-drill-{walk,reverse,gather}-*-{off,on}`
(1792872060 to 1792872625). Every one of the 128 duels arrived. A first set of sixteen (1792870531 to 1792871133)
found two bench faults and is superseded: its gather ran on a lab starved of energy (the engine's resource cheat
gives metal only, D5), and its speed share counted units standing at the goal.

| drill | arm | first / median / last s | spread | nearest friend | speed share (per type) | spin-up | recover |
|---|---|---|---|---|---|---|---|
| walk | 12 Blitz, lane off | 18 / 19 / 19 | 51 | 24 | 1.00 (0.95) | 3.0 | |
| walk | 12 Blitz, lane on | 18 / 19 / 20 | 121 | 65 | 1.00 (0.95) | 3.0 | |
| walk | 12 Stout, lane off | 24 / 25 / 25 | 71 | 40 | 1.00 (0.96) | 3.0 | |
| walk | 12 Stout, lane on | 24 / 25 / 26 | 119 | 66 | 1.00 (0.96) | 3.0 | |
| walk | 6 Blitz + 4 Stout + 2 Janus, lane off | 18 / 25 / 35 | 359 | 40 | 0.99 (Blitz 0.95, Stout 0.96, Janus 0.94) | 3.0 | |
| walk | the mix, lane on | 18 / 25 / 36 | 387 | 67 | 0.99 (the same) | 3.1 | |
| walk | the mix, `--order move`, off / on | 18 / 25 / 35 | 359 / 378 | 40 / 68 | 0.99 | 3.1 | |
| walk | the mix, `--reissue 2`, off / on | 18 / 25 / 35 | 359 / 386 | 40 / 65 | 0.99 | 3.1 | |
| reverse (turn at 10 s) | 12 Blitz, off / on | back at 18 / 19 / 20 | 44 / 117 | 25 / 64 | 0.92 (0.88) | 3.0 | 4.0 |
| reverse (turn at 10 s) | the mix, lane off | back at 16 / 19 / 19 | 77 | 40 | 0.89 (Blitz 0.88, Stout 0.83, Janus 0.75) | 3.0 | 4.0 |
| reverse (turn at 10 s) | the mix, lane on | back at 18 / 19 / 20 | 119 | 65 | 0.87 (Blitz 0.88, Stout 0.83, Janus 0.67) | 3.0 | 4.0 |
| gather (fed) | the mix from an armvp, off / on | 32 / 133 / 265 | 94 / 124 | 25 / 65 | 0.99 (every type 0.97-0.99) | 16.4 | |

Gather: 12 produced in all 16 duels; join median 25 s (appearance to arrival 2,000 away); per type the Blitzes arrive
75-103 s, the Stouts 181-205, the Janus pair at 265 (the lab builds the list in order: a Blitz every 14 s, a Stout
every 24, a Janus every 26).

What the baselines say:

1. **The engine walks a pure column perfectly.** Full speed from 3 s after the order, arrival at the theoretical
   time (1,700 over the type's speed plus the spin-up), the same to the second whether the order is a fight or a
   move and whether it is given once or every 2 s. There is nothing for a controller to gain on a straight walk of
   one speed; the re-issue costs nothing either (the goal is unchanged, so the engine's path is unchanged).
2. **A mixed column under one order is three columns.** Blitz, Stout and Janus arrive at 18, 25 and 35 s, the
   spread 359 at the median arrival, each type at its own full speed: a 17-second string-out over 2,000, which is the
   stringing the games show. Neither the engine nor today's lane holds them together. This is the quantity the
   group API's pacing arms exist to shrink, and the bench measures it as first-to-last arrival and spread.
3. **The lane's one effect with nobody in sight is the shape at arrival.** Lane off ends in a ball (nearest friend
   24-40, spread 44-77); lane on ends in ranks (nearest friend 64-68, spread 117-124), the pros' 67-70, at a cost of
   one second at most. That is the march's rewrite into ranks of six at the destination doing its work.
4. **A reverse costs 4 s and 8-13% of the walk.** The median unit is back at 90% speed 4.0 s after the turn in every
   arm (the Blitz's computed floor is 4.5 s from a standing turn; a moving column does slightly better because the
   front units' turns overlap the rear's travel). The slow types lose the most: Janus 0.75 of its speed over the
   drill against the Blitz's 0.88, and 0.67 with the lane on (8 duels each; the lane's rewrite at the turn may cost
   the slowest unit a lateral move, to be checked with more repetitions before it is a claim).
5. **The gather is a stream.** Each unit walks alone from the lab at full speed; nothing waits, nothing forms until
   the goal, where the arrived stand in a ball or in the lane's ranks while the rest come in. The first unit is at
   the goal 32 s in, the last 265 s in.

The bars, set from these (replacing §4's proposals for drills 1-3): a pure column of twelve arrives first to last
within 2 s, at 1,700 over its speed plus 3 s, with a nearest friend of 55-90 at the median arrival; a mixed column's
first-to-last gap is the measure to shrink from 17 s, and an arm counts as pacing when the gap halves without the
last arrival slipping past the slowest type's 35 s by more than 3 s; a reverse recovers within 4 s and keeps a
nearest friend of 55-90 after the turn; a gather's joiners arrive within their type's walk plus 3 s of appearing
(Blitz 20 s, Stout 26, Janus 34), and the body at the goal keeps 55-90 as they come.

**Re-read with the depth and width measure (D8, 2026-10-24 night),** derived by `run/drill_table.py` from the same
sixteen batches' per-second logs (the director's own columns agree with the derivation to the elmo on a check duel:
88 x 115, ramp 0.6 / 0.9):

| drill | arm | at contact, deep x wide | ramp half / all s |
|---|---|---|---|
| walk | 12 Blitz, off / on | 88 x 116 / 122 x 371 | 0.6 / 0.9 and 0.6 / 1.2 |
| walk | 12 Stout, off / on | 82 x 193 / 68 x 372 | 0.9 / 1.1 and 0.9 / 0.9 |
| walk | the mix, off / on (move and reissue the same) | 923 x 187 / 927 x 335 | 7.1 / 17.1 and 7.2 / 17.2 |
| reverse | 12 Blitz, off / on | 105 x 98 / 87 x 359 | 0.7 / 1.0 and 0.6 / 0.9 |
| reverse | the mix, off / on | 313 x 121 / 136 x 326 | 2.7 / 3.1 and 1.2 / 1.4 |
| gather | (the stream's first unit; says nothing) | 1463 x 25 | 14.5 / 14.5 |

And the series, one duel each, every two seconds: twelve Blitzes with the lane off go 64 x 318 at 2 s, 87 x 167 at
10 s, 94 x 105 at 20 s; with the lane on 64 x 330, 159 x 379, 157 x 362; the mix with the lane off 99 x 317, 507 x
255, 994 x 180; with the lane on 106 x 318, 512 x 332, 997 x 336.

6. **One goal point funnels a line into a file.** Every unit walks its own straight line to the same point, so the
   width falls in step with the distance left: 320 at the order, 167 halfway, 105-116 at the goal for twelve
   Blitzes, while the depth holds near the two ranks' 88. The "ball" at arrival is this funnel's end. The hands'
   advance is one point for the whole group; in the games the lane's march (H-MICRO-FORM, on unless disabled)
   rewrites it into ranks at the destination, so the funnel is what any unit outside the lane's bodies does, and
   what the order alone would do (K-engine-one-goal-point-funnels-a-line-into-a-file).
7. **The lane's per-unit destinations hold the width the whole way** (335-381 against the formation's 320), at a
   cost in depth (64 to 160 for the Blitzes, the ramp to all 0.9 to 1.2 s): the march's ranks at the destination
   are what keeps six abreast, and it is the only thing in the stack today that does.
8. **The mixed column's depth grows at the speed difference,** 47 elmos a second, to 923 at contact: half the army
   is 7 s behind the leader when the leader meets a front, the last 17 s. A pacing arm is judged on these two numbers
   with the width kept.

The bars for drills 1-3 in these terms: a pure column's ramp to all at contact within 1.2 s and its width at
contact no less than the formation's at the order; a mixed column's ramp to half and to all are the measures to
shrink from 7.1 and 17.1 s with the width kept and the last arrival within 3 s of the slowest type's walk; a
reverse's shape at the first unit back within the walk's.

## 6. Open for the user

- The group API's shape in §2 (one call per decision; what `GroupState` carries).
- Whether the gather survives as a move or becomes the group state's words (the hands' gather is a scripted
  operation with two cliffs that contradict each other, `2026-10-24-pick-rework.md` §5).
- The maps for drill 6.
- The commitment ladder keyed to the task kind (`brain/micro.rs` `note_commitments`), a code decision layer older
  than the no-live-heuristics ruling, reviewed after the group API.
