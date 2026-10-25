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

## 1. Where groups are in the stack today (at the arc's start, 2026-10-24 evening; the lane's row is §2's since the night's build)

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

## 2. The target: the lane owns the group as a formation it follows (ruled 2026-10-24 night)

The user, after the baselines and the replays: "The fundamental goal here should be making the group of units act
predictably and coherently with each other, flagging outliers as such. Right now whenever a group gets stretched out
by either terrain or unit mix, the subsequent battle is with a complex series of engagements as new units trickle in
and mix with the units already fighting. Making the micro engine responsible for groups of units, with expectations
of it managing certain properties of the group in a closed loop, enables a simpler attack/retreat/shell cost calculus
for Jev and Claude [and] directly resolves the issues we started today's work with. Exactly how we do that is up in
the air, but my initial thought was to try and make actual organized formations a thing we can follow." The anchor
model below was proposed in answer and ruled ("Go ahead").

**The object.** A group is one object with a position, a heading, a speed and a shape, and a member's only job is to
be in its slot. The object is an anchor: a virtual point that walks the group's route at the group's pace. Slots are
offsets from the anchor: ranks of `FILES` across the heading on the march, narrowed to the corridor the ground
allows; the contact arc at contact, as today. Each member's order is its slot placed a lead ahead and re-issued at a
cadence, so the engine's own movement does the following. The host never orders a member; it orders the group.

**The properties the loop holds** (the contract the hands and the duel director rely on):

1. Pace: the anchor moves at the slowest type's speed in the declared group (the fraction 1.0, ruled by the bench the
   night it was built: a lagging unit of that type regains its slot because the anchor is held back by its formed
   members, not because the pace leaves a margin; 0.9 cost a pure column 4 s and met no bar the 1.0 missed).
2. Shape: the template across the heading, never wider than the standable ground across the route.
3. Coherence: every member within a tolerance of its slot, or an exception.
4. Integrity at contact: the anchor stops at reach, the formed body takes the arc, and the loop falls silent for any
   unit with a target in reach (every order there costs shots: form6, form7).
5. The report: each member formed, or an exception with a cause (slower, stuck, fled, blocked, joining) and a time to
   its slot. This replaces the hands' arrival test, the gather, the joiners and the stragglers, and it is what prices
   a fight: the formed body, with a joiner counted when it is in a slot.

**Outliers and the anchor.** The anchor is held back by the members of its body: a member behind its slot drags the
pace, to a stop at the exception threshold behind; past the threshold it is reported as away and is formed again
when it is back inside the tolerance (two thresholds with hysteresis, so the report never flickers). **The anchor
never outruns its body** (amended 2026-10-24 night on the first circuit's evidence, D12: under the law as first
written, which released a member past the threshold and let the anchor go on, the anchor walked the crow line
through ground the Pawns went round, every member fell past the threshold within twenty seconds, and the anchor
walked the whole lap alone at full pace with twelve members a map behind it; a same-speed member released behind a
full-pace anchor can never close, so the release was permanent for everyone. Chose: every member that has once been
within the threshold of its slot holds the anchor by its lag however far behind it is, a fleeing member excepted, and
a joiner holds nothing until it has come that near. Rejected: the release as written, because it made the common
case, a whole body delayed by terrain, a group of nobody; and a formed-share threshold for walking on, because it is
another arbitrary number.) What the group will not wait for, the host takes out of the group: whether to wait for a
real straggler stays Jev's, from the words.

**Reopened rulings, with the evidence** (decisions are fluid at an arc boundary, 2026-07-18):

- `2026-09-25-formation-as-a-transform.md` ("the formation as a transform of the order, not a control loop") is
  reversed. Chose-because: the bench shows the transform holds width at arrival and nothing else (a passage files the
  body and the ranks return only in the last six seconds; a mixed column is one column a speed; §5 D8, D10), and any
  repair needs an anchor that moves and orders re-issued at a cadence, which is a loop. Rejected-because, what the
  September evidence was: a 150-elmo lead that braked units through the engine's end-of-path cap, and orders costing
  shots at contact. The first is a parameter (the lead law uses that very cap on purpose), the second holds only at
  contact, where the loop is silent.
- The routing arc's "routes for accounting, the engine executes" (2026-09-20) is narrowed: the host passes the route
  with the group order and the lane places the anchor and the slots along it; the engine still paths each unit to its
  slot. The lane does not depend on `terrain`; the hosts (the hands, the director) have the fields.
- Bodies by chaining (`form::bodies`) go: the host declares the group and the lane never splits a declared group. The
  raider/line separation (the pros keep them apart) becomes the host's choice, two groups or one.

**The API** (one call per decision; the route from the host's field, the goal its last point):

```rust
// crates/micro
pub struct GroupOrder { pub kind: GroupKind, pub route: Vec<Vec3> }   // Go | FightTo | Attack(UnitId) | Hold
Lane::set_group(name: &str, members: Vec<UnitId>, order: GroupOrder, footwork: Footwork, commitment: Commitment)
Lane::clear_group(name)
Lane::group_state(name) -> GroupState {
    anchor: Vec3, heading: (f32, f32), pace: f32,           // the formation's place, way and speed this tick
    shape: Shape,                                           // ranks on the march; the arc at contact
    formed: Vec<UnitId>,                                    // within the tolerance of their slot
    exceptions: Vec<(UnitId, Cause, f32)>,                  // out of slot: why, and seconds to the slot
    arrived: bool,                                          // the anchor at the goal and the formed in their slots
}
```

**The actuator and its numbers.** The engine has no speed command (the AI interface's set-wanted-max-speed is marked
unused; `Spring.SetUnitMoveGoal`'s speed is a synced Lua call we cannot ship into a lobby game), so the loop closes
through goal placement. The actuator, measured in D11 before anything was built on it, is the slot itself: a unit sent
every tick to a point moving at the pace settles at the pace, trailing the point inside its brake zone
(K-engine-a-slot-moving-at-a-pace-paces-the-unit; a Blitz holds 52-56 behind a slot at 54), and the mixed column
whose slots all move at 0.9 of the Janus's speed arrives as the formation it set out in (98 x 320 against the nominal
96 x 320, ramp 1.8 s against 17.1). The lead law as first written (a goal a fixed distance ahead of each unit, held by
the end-of-path cap) was the wrong reading of the source: the cap does not bite on a straight walk. Five numbers, each
with a drill: the pace fraction (0.9 held the Janus in its slot where 1.0 did not), the cadence (every tick; a 1 s
cadence held the per-second mean and is an arm, not the default), the slot tolerance and the exception threshold (the
gather and the terrain cases: the exceptions' count against the arrival), and the catch-up: a unit far from its slot
runs at full speed and brakes into it. The alternative kept on the bench as a control: no pacing
and a form-up point short of contact, which arrives when the paced march does but leaves the group a column for the
whole walk, with no position for Jev until it forms.

**Sequencing.** This arc: the group API, the anchor and the pacing arms in the lane and the duel director, judged by
§5's bars with no model. The arc after: the hands declare groups, their soldier upkeep (`groups.rs`, about 1,200
lines) is deleted, and the group state becomes words ("group_A: 11 in formation at X heading east at 75; 3 out of
it: 2 Blitzes 400 ahead (faster), 1 Stout stuck at Y"), judged in games. The commitment ladder (`brain/micro.rs`
`note_commitments`) moves into the group order's `commitment`, the hands' mapping the audited default until Jev's
menu carries the choice.

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
- **D9. The target drill, `into`** (the user, 2026-10-24: "how much of the army group is able to engage with an enemy
  at once"; the geometry of D8 is a prediction, the shots are the fact). The walk with an enemy structure standing on
  the goal (`--target`, default `armshltx`, the Experimental Gantry: 16,000 health and no weapon, so the ramp is
  unconfounded by deaths; `armllt` or `armbeamer` for the turret variant, where the cost of arriving in pieces is in
  the duel's fire columns) and the fight order to the goal. Two targets tried first and rejected the same night: the
  advanced fusion (8,300 health) is a bomb, its death blast 10,600 damage over 1,280, and in the mixed column's
  batches the Blitzes alone had it down before the Janus arrived and the blast wiped the army (66,727 damage dealt
  by a side with no weapon, 8 of 8 timeouts); the fortification wall (8,900 health) is never fired at, since a fight
  order ignores walls (K-mech-a-fight-order-ignores-walls). The gantry's own blast is 1,330 over 420 and it has the
  health to outlast the ramp. The target is given to side `y` at the goal and given again the tick after it dies, so the
  fire never runs out of a target; its deaths are counted. Every unit's first shot is recorded (once a second, from the
  shots the fire instrument already counts): `first_shot_s` from the first order, and the **shot ramp**, the seconds
  from the first unit's first shot to the median unit's and to the last's (`shot_ramp_half_s`, `shot_ramp_all_s`),
  which is D8's ramp measured instead of predicted. Contact for the shape is the first shot. The drill ends when every
  living mobile unit has fired (`engaged`) or at the time limit. The lane on side `x` meets the target at its
  horizon (700) and forms as it would on a party, which is the lane's shape at contact under the measure.
- **D10. Terrain cases** (the user, 2026-10-24: "I would also make sure to add cases here where the army has to
  navigate around terrain"). Two ways to place a drill off the strip. `--from X,Z --to X,Z` runs any drill between two
  chosen points on any map: the army spawns at `from` in its formation facing `to`, the strip chooser is not consulted,
  and the ends' ground height is read from the heightmap. `--around R` asks the chooser for a detour instead of a strip:
  pairs `--distance` apart as the crow flies whose walking distance for the force's movement class (the terrain crate's
  field over the passable cells, `terrain::Field`) is at least `R` times the straight line (default 1.3), both ends on
  dry passable ground holding the formation (its extent plus the margin, relief 48) and clear of the commanders, ranked
  by detour, `--sites` of them taken apart from each other; when none is found the message says the best detour the map
  offers at that distance. Three new columns: `straight`, `walk` (the field's distance) and `route_s`, the walk over the
  slowest type's speed, which is the arrival the engine's own path would give if it took the field's; read against
  `arrive_last_s`. The heading for the shape (D8) becomes the body's: the centroid's displacement over the last second
  when it moved 10 or more, the axis to the goal in force otherwise, in the director and the reader alike (on a strip
  the two agree). What the terrain cases show is the file at the passage and the time after its exit until the body is
  abreast again, in the per-second `shape`; the first cases come from the maps on disk under `run/data/maps`
  (Quicksilver, Comet Catcher, Great Divide, Cape Violet, Altored Divide, Isidis Crack among them), chosen by the
  chooser, not named from memory.

- **D11. The lead law's kill-test, `--lead L`** (2026-10-24 night, before the anchor is built on the law; the user:
  "Go ahead"). On a walk every re-issue (`--reissue S`, fractional seconds from here on; 0.1 is the lane's tick) sends
  each unit to the point `L` ahead of its own position along the axis, or to the goal itself when that point is past
  it. One unit, the lane off, a move order: what the engine does to a unit whose goal is always `L` ahead. The two
  readings of the source it discriminates: under the end-of-path cap (`speed <= L x turnrate x 0.001438`, which
  `ChangeSpeed` applies only while the unit's heading differs from its waypoint's direction, and a straight walk may
  keep them equal) a Blitz holds 31 / 55 / 78 / 101 / 101 elmos a second at L = 40 / 70 / 100 / 130 / 200 and a
  Stout 49 at 100; under the brake alone (`v^2 / (2 x maxdec)`, 42 elmos for a Blitz at full speed, the move goal's
  radius `SQUARE_SIZE` = 8) the Blitz holds 99 / 101 / 101 / 101 / 101. At a 1 s cadence under the cap the speed
  decays with the shrinking distance within the second (a mean near 0.54 x L a second, against the 0.1 s cadence's
  ripple under 8%). L = 200 at 0.1 s is the control for the orders' own cost (the chase instrument found none at 3 Hz
  with far goals; 10 Hz is untested). Read from `speed_share`, the per-type share and the arrival. If the cap does
  not bite on a straight walk, the fallback is the same law with its condition met, a lead placed a few elmos off the
  unit's heading each tick, measured the same way before the anchor is built on it. Item 2 below stands: `--lead` is
  the one deliberate loop of the director's, and an arm.
  **Measured the same night (36 duels, ledger row lead-*):** neither reading stood. The cap does not bite on a straight
  walk (0.95 at L = 70, 0.98-0.99 at 100-200 every tick) and the brake alone under-reads (a steady 72 at L = 40, the
  brake over the goal distance less about seven frames of latency); a 1 s cadence is a sawtooth (43-66 at 70). The
  fallback was not the off-heading lead but the slot itself: **`--pace P`** sends each unit every re-issue to its own
  slot, its spawn place carried along the axis by `--lead` plus `P` times the seconds since the first order (from the
  turn, for a reverse), stopping at the goal, the formation translated rigidly. Measured (24 duels, ledger row pace-*):
  the lone Blitz settles at 52-56 behind a slot at 54 and 72-83 behind one at 75, the per-second mean holding at a 1 s
  cadence too; the Stout at 54 swings 34-77; the mix at 49 arrives 34 / 36 / 37 as the formation it set out in (98 x
  320, ramp 0.9 / 1.8), at 54 the Janus never regains its slot (31 / 33 / 36, 194 deep). The anchor rests on this:
  K-engine-a-slot-moving-at-a-pace-paces-the-unit.

- **D12. The circuit with a feeder, `--drill circuit`** (the user, 2026-10-24 night, after the group's bench: "take a bot
  lab producing pawns and have them auto-join a group. Said group should be on a circuit of locations throughout the
  map, so this can test a real deployed army and feeder line"). A deployed army (`--ours`, spawned in its formation
  beside a lab; `armpw*0` for none, when the group forms from the lab's first unit) walks a loop of points across the
  map `--laps` times (default 1) while the lab (`--lab`, default `armlab`) builds `--feed name*count[+...]` under the
  resource cheat (fed energy, as a gather), each new unit adopted into the army as it appears within 400 of the lab,
  and so into the group: the lane's membership is the army's living mobile units, and a joiner is an exception with
  the cause Joining until it first reaches a slot, walking straight to the slot across the map rather than round the
  circuit, which is what a feeder line is. The circuit is `--circuit X,Z;X,Z;...` (the first point the lab's) or, by
  default, the four corners of the rectangle inset a quarter of the map's width and height, south-west first (the
  lab's), then south-east, north-east, north-west (the commanders stand along the north edge; a corner within 900 of
  one is pushed away from it), each moved to the nearest cell the force can stand on and reach from the lab (one field
  over the standable cells from the lab's corner; a corner nobody can reach from there is replaced by the nearest
  reachable cell), the lab's corner on a flat square. The route is the loop from the lab's neighbour round to the lab,
  `laps` times over, as one order: a group order re-declared the same is taken silently, so the anchor turns at each
  corner without stopping and the formation never re-deals on a corner. Lane off, the baseline: each unit is given the
  whole route as queued fight (or move) orders, the hands' way today, a joiner from the lab the same. Measured, both
  arms alike: each deployed unit's passes of the lab's corner (within 400 after having been further: hysteresis, so
  the start counts for nothing), its `laps`-th pass its arrival (`arrive_first_s` / `median` / `last` from the first
  order; `laps` the median deployed unit's passes at the end, where a timeout shows a partial walk), against the laps
  on foot over the type's speed (`route_s`); `produced` and `joined` (fed units that came within 300 of the deployed
  body's centroid before the end) and `join_median_s` (a circuit: seconds from appearance to joining the body); under
  the group `formed_share`, the formed members over the members, averaged over the seconds, which is the coherence the
  user asked for in one number, and the per-second `group` series (pace, formed, exceptions by cause) for how the
  joiners and the corners show. Ends when every deployed unit has walked its laps and (under the group) the group
  stands at the route's end with no exceptions, or at the time limit (600 s). What it is for: the group's turning at
  corners, its re-dealing when a member joins (a re-deal of the whole formation on each joiner is the risk: watch the
  formed count at each appearance), whether a joiner at full speed ever closes on a group at full speed (only by the
  corners' cuts and the straight walk across, so the lap and the lab's rate set the answer), and the terrain between
  corners under the group, which the strip never showed.
  **Built the same night, and what the first three runs taught** (ledger rows circuit-*): the route the host gives
  must be the walked one (the corners alone sent the anchor through ground the Pawns went round: §2's law amended
  above, the director now passes the field's way, a point every 256, and the lane-off baseline keeps the corners as
  the hands' way); the deployed army must spawn on standable ground (two Pawns of a formation spawned off the lab's
  square never moved, and under the amended law held the anchor for nine minutes), so the lab's square is 640 and the
  army spawns 240 from the lab on it; and the amended law needs the host's one rule, which the director keeps as Jev
  would from the state's words: an away or stuck member no nearer its slot for 20 s is taken out of the group
  (`STRAGGLER_PATIENCE`, said and counted as `left_behind`). Found beside it: a written `name*0` read as a plain name
  (the batch's count, 22 Pawns), the game swallowing one match's `/cheat 1` (pitfalls.md), and a stuck unit reported
  as away (stuck is said first now).

- **D13. Route clearance** (the user, 2026-10-24 night, after watching the circuit demos: "I regularly see artifacts
  where a formation rounding a sharp corner gets bogged down and creeps around it rather than leaving enough margin to
  maneuver... For route clearance, we should definitely add some automatic allocation for an army rounding a point.
  Once we have that working, we should put effort into optimizing formation redirections as a whole"). The host's
  route is planned over a field priced by room: each cell's clearance is its distance to the nearest impassable cell
  or the map's edge (`terrain::clearance`, an exact distance transform), and a cell costs the flat rate where the body
  has the room it wants and up to six times that at a wall (`terrain::clearance_costs`), so the cheapest way rounds a
  corner at about the wanted distance and walks a corridor narrower than that down its middle rather than along one
  wall. The room wanted is the formation's half-width plus a hull: for the group's six files at 64, 224. The field's
  cell route hugged every corner and edge before this (the shortest path is tangent to the obstacle), so the inner
  file's slots lay inside the wall at each corner and the formation crept round. Chose a priced field over a dilated
  obstacle map because the dilated map has no way through a corridor narrower than the formation and would need a
  second plan for the fallback; the priced field is one plan with the fallback in it. The corners themselves are
  chosen with that room. The lane-off baseline keeps the corners alone, as before. Measured by the circuit (§5): the
  route's least room is in the match's message and the meta line; the corners' cost in the per-second series. The
  wheel at corners and the shape to the corridor are the redirection work after this, in the user's order.

- **D14. The re-deal at the turn** (the user, 2026-10-24 night, after the clearance demo: "How do you think we should
  handle wheeling? One improvement I can see directly from that last match is that turns could easily include slot
  reallocations. A 90-degree or 180-degree turn could be very cheap with bots"; "Go ahead with the re-deal, all four
  pieces"). Before this the slots were rigid offsets from the anchor and a member's slot index was sticky from the deal
  to the next membership or route change, so at a corner every slot rotated at once about the anchor (the front rank's
  centre) and each member chased its own slot round: the outer flank's slot moved 294-674 at the circuit's three big
  corners (82-149 degrees), 1-5 of the 13-20 body members stayed formed, the anchor was held at 27-61 of full pace for
  4-16 s, and the body took 4-51 s to be formed again; the same for Pawns, Flashes and Stouts (every one turns in
  place in the engine, at 200, 90 and 56 degrees a second: the turn's cost was the slot jump, not the turn rate). Re-run
  on the logged positions, a least-travel deal of the slots with the anchor set down so the new ranks lie over the body
  put every body member within 128 of a slot at eight of the nine corners (16 of 18 at the ninth) and 9-17 of them
  within the 64 drag band: a turn is a re-labelling of who stands where, and an about-face is free (the rear rank is
  the front). Four pieces, built together: (1) **when**: the slots are re-dealt when the anchor passes a route point
  whose heading change is `REDEAL_TURN` (30 degrees) or more, when the membership changes (as before), when the route
  changes (as before) and when the anchor is set down after contact; between those the deal is sticky, so it never
  flickers; (2) **where the anchor goes**: at such a turn it is set down so the body's ranks' centroid lies on the
  body's centroid (the body: members in the body, not away, not fleeing; the slot count for the centroid is the body's,
  not the members' with the joiners, or the anchor would stand a rank ahead of the body), and walks on to the next
  point from there; (3) **the deal**: an exact least-cost assignment on squared distance (Kuhn-Munkres; twenty units,
  microseconds) in place of the sort-and-swap, for the march's slots and the contact arc's alike, so a joiner takes the
  slot the body can spare rather than the body re-sorting around it; (4) **the layout grows at the back**: full ranks
  of `FILES` from the front and the remainder in the last rank, centred, in place of ranks as even as possible (13 were
  5 / 5 / 3 and 12 were 6 / 6, so a joiner moved every slot); now a joiner adds a slot behind and the body's slots
  stand, the partial rank's few shifting half a spacing at most. Chose the re-deal over the wheel (an arc at the
  corner of radius at least the half-width with the pace capped at R / (R + w / 2) through it, or a smoothed heading)
  because the wheel keeps each member's place in the rank, which is the human drill's constraint and not ours, and
  pays for it in pace and in room outside the corner; the re-deal costs neither. Rejected a gain rule (re-deal
  whenever a fresh deal would halve the worst slot distance) for the angle at a route point: more general, less
  predictable, and the corners are where the gain is. What this does not touch: the vehicles' extra price along the
  lap, which is not at the corners (the anchor's slow seconds after 30 s: Pawn 37 = 18 at corners + 10 after small
  bends + 9 elsewhere; Stout 112 = 30 + 39 + 43; Flash 74 = 28 + 8 + 38), the small bends slowing every vehicle to its
  turning speed and the passages; a smoothed heading or a straighter route is the follow-up, unread. Measured by the
  circuit re-run for all three types (§5).

  **Built and measured the same night, in six rounds (ledger rows circuit6-* to circuit12-*): the four pieces as
  approved, then what the traces (`WITHIN_REASON_GROUP_DEBUG=1`, the group saying each re-deal, turn and slot jump on
  stderr) changed about where two of them are placed, and two findings beside them.**
  - *The carry.* Set down on the body at the point itself, the anchor left the body's centroid half its depth short
    of the corner (the body stands behind its front), so the new ranks lay that far inside the new leg and the next
    point, 256 on, stood nearly abreast of the anchor: the heading swung 60-85 degrees a second after every turn and
    the body was knocked out of slot a second time (circuit6: Pawns 233 s against the sticky slots' 204). The anchor
    now carries on past each intermediate point by the body's measured depth behind it (the mean of the body's
    members along the heading, so the trail they walk behind their slots and the ranks they actually hold are in it;
    from the rank count alone the centroid was still a hundred short), at most half way to the next point, and the
    body's centroid turns at the point; the first point has no leg into it and is not carried. After the setting-down
    the heading is taken again from where the anchor stands (taken from where it turned, the ranks swung 25-35 degrees
    more a tick later).
  - *The route.* The field's way is a chain of cell moves at multiples of 45 degrees; thinned to 256 it was a
    staircase of 26-45 degree bends at most points (13-15 of 58-61 at 20 degrees or more), each snapping a four-rank
    body's rear corners (370 from the anchor) by about 150, with no re-deal under 30 degrees; the per-second log's
    headings were kept to a tenth, which made 26-degree turns out of none (now a hundredth). `sites::circuit` now
    straightens each leg: from each point the furthest later one the body reaches in a straight line with the room it
    wants all the way (a sample a cell), so a leg bends only where the ground bends it (Quicksilver's loop: 19 points
    from 58, the four corners among them). The re-deal threshold is 20 degrees, not 30: a 23-degree snap moved 17 of
    20 out of the band (circuit9), and a re-deal costs the body nothing.
  - *Set down on the members, not the trail ahead of them* (tried and rejected). The body walks a trail behind its
    slots (below); set down on the members, the slots stand on them and the body re-forms the trail in a few seconds
    as it walks off at full pace. Setting the anchor down the trail ahead instead, to keep the trail across the turn,
    put 19 of 20 out of the band at a 130-degree corner (the trail added to the re-deal's travel: pace 21, formed 1),
    where on the members the same corner kept 20 formed with the pace rising 70 to 87 (circuit10 against circuit9b).
  - *The body is those who have stood in a slot.* A joiner held the anchor from within 300 of its slot, so one
    arriving 280 behind stalled the body to a pace of 16, twice a lap (circuit9); it holds nothing until it has been
    formed once (`formed_once`, the one book; `in_body` is gone).
  - *The drag starts at the formed band (128), not 64.* The finding behind the vehicles' price along the lap and a
    tenth of every Pawn lap: a unit sent every tick to a point moving at the pace settles behind it by a distance of
    the engine's (72-86 for a Pawn at 80, every rank the same, about twice that for a Stout by its turn rate), and the
    drag from 64 made that natural trail alone hold twenty Pawns at 0.90 of full pace on a straight walk (the worst of
    more members lags more: twelve at 0.98; the layout is not it, ranks of 5 / 5 / 5 / 5 walked 0.90 too:
    walk-redeal-pawn20, walk-evenranks-pawn20). From 128, the band holds nothing it calls formed. Open after this: a
    Stout's trail is past the band, so a Stout following perfectly is dragged and reported out of slot; the answer is
    a lead, the slot order placed the type's following distance ahead of the slot so the unit stands in the slot,
    which needs the turn rate from the defs or an online measure, and is the next decision for the user.

- **D15. The control surface cut** (the user, 2026-10-24 night, after the re-deal: "audit and make cut or extend
  decisions on the current control surface of the micro engine (what claude, jev, and the other bots have to control
  units)"; the proposal ruled "Agreed with the proposed order. The flagship's current state is not a problem: focus on
  building a good engine and we will attach the hands to it when it is ready"). The surface found, by layer: the
  engine's commands (Move, Fight, Attack, Stop, Guard, FireState, MoveState; nothing to cut); the lane's per-unit API
  (`note_standing_orders`, `set_commitments` with a `Commitment` and a `Footwork` per soldier); the group API
  (`set_group`, `clear_group`, `group_state`; kinds Go, FightTo, Attack, Hold; a route; a state); the hosts (the hands
  with 1,200 lines of their own body model, the duel director, the distilled bot per unit with the lane off by
  default, the territory bot with no soldier in the lane). Since the group build (39a4238) only declared groups form,
  and only the director declares them: the flagship's soldiers walk as a blob under the three reflexes; the user ruled
  that acceptable until the hands attach. **Cut:** (1) `Footwork` is the three reflexes, flee, fan and kite. `form`
  is gone (a declared group forms by definition; a host that wants the reflexes without the group declares none, the
  `old` arm) and `march` is the order's (`GroupOrder::march`; false puts the slots at the route's end at once, the
  `ranks` arm). The player's `lane` tool offers the three. (2) `Commitment` is the threat set alone (None, Priced,
  All). A hunt and a rove are kinds of group: `GroupKind::Hunt(HuntOrder { quarry, leash, lost_frames, since })` and
  `GroupKind::Rove { within }`. The lane runs a hunt group raw, every member an Attack on the quarry each tick, the
  leash measured for each member from where it stood when the lane first ran it under the hunt (the hands' rule since
  2026-10-16: from the hunters' centre, a detachment cut from a strung-out body tripped it on its first tick);
  `since` is the frame the host began the hunt, so a hunt picked again on the same quarry after an end is a new order
  and not the silent re-declaration; the end event names the group. `View::rove_within` is cut: the region is the
  order's. (3) `set_commitments` stays as the per-unit surface for loose soldiers (the hands' until their migration,
  the distilled bot's) and dies with the migration; the hands declare their hunting and roving groups through
  `set_group` from here, which attaches two kinds to the API that exists and is not the migration. (4) Rot:
  `docs/code-decisions.md` §5 carried the deleted form-up march and said `march` was read by nothing; the `lane`
  tool's text offered `form` and `march` as rules that did nothing in the flagship. **Deferred to the assault drill's
  evidence (D16):** a shape in the order (files, a standoff), contact words in the state (who is in reach, who is
  muzzled, the arc's fill), a fire state. Chose the fold over keeping Hunt and Rove as commitments because a
  commitment that also means "the lane owns this unit" is two things under one name, and a group of one declaration
  is the whole surface Jev's words will map onto.
- **D16. The assault drill, `--drill assault`** (the user, 2026-10-24 night: "scale up the micro drills to a move and
  fight scenario. Marching to and attacking into various defensive structure under different formation control
  modes"). `duel --drill assault --ours FORCE --theirs DEFENCE [--distance 2500] [--order fight|move]
  [--lane off|old|ranks|on] [--formation ranks6@64] [--from X,Z --to X,Z | --around R]`: side `x` spawns `distance`
  from the goal in its formation facing it (the strip chooser, or D10's placements); side `y` is the defence
  `--theirs` names as a force (`armllt*2+armbeamer`), stood in one rank across the approach centred on the goal at
  `DEFENCE_SPACING` (128) in list order, given once (an `into` target is given again; a position falls once). The
  first order is the walk's (the Fight to the goal; under the lane the group's FightTo). The duel ends when `y` is
  wiped (`taken`), `x` is wiped (`wiped`) or at the time limit; the stalemate rule does not apply (a defence and a
  body out of its reach are quiet by nature). Measured by the duel's own columns (value left on both sides, the
  margin, survivors, the fire instrument: in-reach seconds, shots, muzzled seconds by cause, friends on the line) and
  the drill's (the formed share, the shape at contact, `first_shot_s`, the shot ramp to half and all, `taken_s` from
  the first order to the last defence's death, `lost_before_shot`: units of `x` dead before their own first shot, the
  price of arriving in pieces). **The reference is the bound, not a TAS:** the plain duel of the same forces
  (`duel --ours FORCE --theirs DEFENCE`, the sides `SEPARATION` 1,100 apart, both attack-moving) is the fight with the
  approach nearly removed, and the assault's margin against it is the approach's price; TAS scripts for two positions
  after, when the slice says which. **The first slice,** baselines before any lane change (D7's rule): forces
  `armflash*12` (1,320 metal) and `armstump*10` (2,250); defences `armhlt` (440 metal, reach 620), `armguard` (1,250,
  reach 1,220, area fire, outranges every tier-1 soldier) and `armllt*2+armbeamer` (360, reaches 430 and 490, the
  defended expansion); distance 2,500 (outside every reach); the four lane modes; 8 duels an arm: 24 arms and 6
  bounds, on Quicksilver's strips. The slice is chosen so the formation matters (twelve to twenty units) and the
  metal ratio runs from near even (the Blitzes against the Guardian, 1.06) to six to one (the Stouts against the
  expansion); what it shows decides the second slice (counts, mixes, terrain by `--around`).

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
| 8 | circuit: a deployed army round the map while a lab feeds it (D12) | laps against the walk at the type's speed, the formed share, the joiners' time to the body, the formed count at each joiner and corner | the lap within 1.1 x the walk on foot over the type's speed; the formed share over 0.9 with no joiner; a joiner in the body within its own straight walk plus 10 s; no formed member knocked out of its slot by a joiner |

Pacing arms for drills 1-3 once the group API exists: none (the march as today), the paced slot every tick and every
second (D11: the lead law as first written does not pace; the slot moving at the pace does), slow guard on the slowest
unit. Each against the baseline, 8 repetitions, two seeds.

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

**The target drill (D9, 2026-10-24 night; batches `*-duel-drill-into-{gantry,llt}-*`, 48 duels, every one engaged):**
the walk with a structure on the goal, every unit's first shot recorded. A first pair of batches against the advanced
fusion is superseded (its death blast wiped the column, D9).

| target | arm | shot ramp half / all s | geometry's ramp (D8) | target deaths | our value left | seconds |
|---|---|---|---|---|---|---|
| gantry | 12 Blitz, lane off | 1.6 / 11.6 | 0.8 / 1.4 | 1.1 | 0.53 | 55 |
| gantry | 12 Blitz, lane on | 0.8 / 2.4 | 0.6 / 1.2 | 0 | 1.00 | 21 |
| gantry | the mix, lane off | 5.1 / 20.4 | 7.1 / 18.2 | 0.5 | 0.90 | 39 |
| gantry | the mix, lane on | 5.2 / 21.4 | 7.4 / 17.8 | 0.9 | 0.94 | 41 |
| laser turret | the mix, lane off | 5.6 / 21.2 | 7.4 / 18.7 | 12.8 | 0.63 | 40 |
| laser turret | the mix, lane on | 5.9 / 20.1 | 7.4 / 17.9 | 14.8 | 0.65 | 40 |

Units firing each second from the first shot, the mix on the gantry, one duel: lane off 2, 5, 6, 6, 5, 6, 8, 8, 9, 8,
9, 8, 8, 9, 9, 9, 10 (the six Blitzes by +1, the Stouts from +6, the Janus from +16); lane on 1, 6, 6, 6, 6, 6, 10, 9,
10, 9, 8, 10, 10 (the Stouts from +6, then ten).

9. **The ball blocks its own rear.** Twelve Blitzes with the lane off reach the gantry 138 deep and 73 wide, which the
   geometry reads as everyone firing within 1.4 s; half are, at 1.6 s, but the last fires 11.6 s after the first,
   standing behind friends with no line of fire. The gantry dies under them in the meantime and its blast (1,330 over
   420) takes 47% of their value. With the lane on the same twelve arrive in ranks (125 x 370) and all fire within
   2.4 s; nothing dies. This is the shape instrument's muzzled-by-a-friend finding
   (K-engine-a-shot-is-refused-across-a-friend) in ramp terms, and the first measured gain of the lane's shape at
   contact on this bench.
10. **For a column arriving in pieces the formation at contact changes nothing.** The mix fires half at 5 s and all at
    20-21 s with the lane off or on, on the gantry or the turret: the stringing is the whole story. Against the
    geometry's 7 / 18 the half comes sooner (the Stouts' and Janus's longer reach) and the last later (the last unit's
    turn into a slot at contact costs it 2-3 s), so D8 predicts the ramp to within a few seconds and the shots decide.
11. **The turret prices the pieces.** A laser turret that dies in about a second to a dozen units and is given again
    lives 13-15 times in the 20 s it takes the mix to engage, and takes 35-37% of the column's value; the lane on
    does not change that.

The bars for drill 4 (`into`) from these: a pure column's shot ramp to all within 2.5 s of its first shot (the lane's
2.4) and no target death before everyone has fired; a mixed column's shot ramp to half and to all are the measures a
pacing arm must shrink from 5 and 20 s, read with the turret's cost (value left 0.63-0.65).

**Terrain cases (D10, 2026-10-24 night; batches `*-duel-drill-around-*`, Quicksilver, 32 duels, all arrived):** the
chooser's detour at 2,000 as the crow flies is (1878,5475) to (3848,5128), 2,749 on foot (1.37; the map offers no
more at that distance with ends holding the formation). The same pair with `--from/--to` reproduces it.

| arm | first / median / last s | route_s (the field) | at contact, deep x wide | ramp half / all s |
|---|---|---|---|---|
| 12 Blitz, lane off | 26 / 28 / 29 | 27 | 325 x 44 | 1.6 / 3.2 |
| 12 Blitz, lane on | 26 / 28 / 31 | 27 | 364 x 291 | 1.8 / 3.6 |
| the mix, lane off | 26 / 35 / 54 | 51 | 1386 x 218 | 8.8 / 25.7 |
| the mix, lane on | 26 / 35 / 56 | 51 | 1353 x 326 | 8.7 / 25.0 |

The series, one duel each, every three seconds (deep x wide): twelve Blitzes with the lane off 257x247, 310x216,
197x185, 206x45, 262x33, 296x36, 347x25, 343x32, 336x52; with the lane on 69x338, 341x157, 347x119, 375x112, 390x55,
361x69, 398x23, 369x201, 400x381, 317x339. The mix with the lane off reaches 1522 deep at 30 s and is 376x154 when
the Janus arrive at 54 s.

12. **The field's walk predicts the arrival.** The slowest type's time over the field's distance (`route_s`) is the
    last arrival to within a second for the Blitzes (27 against 28-29) and three for the mix (51 against 54): the
    terrain crate's distances are the engine's path, so a group's arrival on terrain can be said before it is
    ordered.
13. **A passage files the body and nothing re-forms it on the way.** The Blitz column is a file 25-45 wide and 340
    deep from 12 s to the goal with the lane off; with the lane on it files the same (23 wide at 21 s) and the width
    comes back only in the last six seconds (201 then 381), at the march's ranks at the destination. The lane's
    formation is a destination's, not a route's: between the passage and the goal the body is a file either way, and
    the contact shape 364 x 291 is the destination's ranks reached, not a body that marched abreast.
14. **The detour stretches a mixed column further than the strip did:** 1,386 deep at contact against 923, the ramp
    to all 25 s against 17, since the slow types lose the same share of a longer walk and the passage adds its queue.

Other maps probed at 2,000 and 1.3 with one duel: Great Divide offers 2,755 on foot (1.38, (301,1238) to (648,3208)),
Altored Divide 3,462 (1.73, (2928,5316) to (1928,7048)), Comet Catcher nothing (the best 1.20; a flat map).

The two maps' baselines (`*-duel-drill-around-{great,altored}-*`, 64 duels, all arrived), 8 duels an arm:

| map, detour | arm | first / median / last s | route_s | at contact, deep x wide | ramp half / all s |
|---|---|---|---|---|---|
| Great Divide, 2,755 (1.38) | 12 Blitz, off / on | 25 / 27 / 30 and 26 / 29 / 34 | 27 | 471 x 55 and 693 x 211 | 1.5 / 4.7 and 2.4 / 6.9 |
| Great Divide | the mix, off / on | 25 / 34 / 51 and 26 / 35 / 53 | 51 | 1286 x 305 and 1260 x 479 | 8.6 / 23.8 and 8.7 / 23.3 |
| Altored Divide, 3,462 (1.73) | 12 Blitz, off / on | 32 / 34 / 36 and 32 / 35 / 40 | 34 | 392 x 56 and 501 x 144 | 1.8 / 3.9 and 1.6 / 5.0 |
| Altored Divide | the mix, off / on | 32 / 45 / 62 and 33 / 46 / 62 | 64 | 1384 x 356 and 1363 x 336 | 11.4 / 25.6 and 12.0 / 25.2 |

The Blitz series on Great Divide (deep x wide every three seconds): lane off 256x177, 280x182, 273x87, 279x35,
270x49, 281x29, 273x54, 286x25, 287x30; lane on 95x355, 434x122, 552x70, 861x37, 900x305, 792x400, 800x385, 896x292,
936x247, 829x227, 632x256, 174x401.

15. **On a passage the lane's march costs depth and time.** With the lane off the Blitz column files through Great
    Divide's passage 270-290 deep and arrives at 30 s; with it on the same column is 861-936 deep from 12 to 27 s and
    arrives at 34, the per-unit slots at the destination pulling the units through at different times, and the width
    at contact (211) is the destination's ranks half re-formed. Altored Divide the same way, 392 x 56 against
    501 x 144 and 36 against 40 s. Items 12-14 hold on both maps (route_s within 1-3 s of the last arrival; the mix
    24-26 s to all either way).

The bars for the terrain cases: the last arrival within `route_s` plus 3 s; the width at contact no less than the
formation's; and the measure a route-following arm is judged on is the seconds between the passage's exit and the
body's width back at the formation's, read from the series, with the arrival kept within the bar.

**The pacing kill-tests (D11, 2026-10-24 night; batches `*-duel-lead-*` and `*-duel-pace-*`, 60 duels, all arrived):**
a lone Blitz sent every tick to a point a fixed lead ahead of itself holds 0.95 of its speed at 70 and 0.98-0.99 at
100-200, a steady 72 of 101 at 40; the end-of-path cap the design leaned on does not bite on a straight walk, and a
1 s cadence is a sawtooth (43-66 at 70). A slot moving at the pace does pace: the Blitz settles at 52-56 behind a
slot at 54 (22 s over 1,500 against 14) and 72-83 behind one at 75, the Stout at 54 swings 34-77 (mean 0.77 of 75),
and the mixed column translated rigidly at 49 arrives 34 / 36 / 37 over 2,000 with every type within 35-37, 98 deep x
320 wide at contact (the two ranks' nominal 96 x 320), ramp 0.9 / 1.8 s, against the unpaced 18 / 25 / 35, 923 deep,
7.1 / 17.1; at the Janus's full 54 the Janus (0.95) never regains its slot and the column arrives 31 / 33 / 36, 194
deep. §5's bar for the mixed column (the ramp shrunk, the width kept, the last arrival within 3 s of the slowest
type's walk) is met by the first arm, walking at the Janus's pace; the bars for the lane's own arm are these numbers.

**The group built and benched (H-MICRO-GROUP, 2026-10-24 night; `crates/micro/src/group.rs`, the lane's `set_group`
/ `clear_group` / `group_state`, the duel director as the first host, `--lane on` the paced group and `--lane ranks`
the group with its march off; batches `*-duel-group-*`, ledger rows group-* and group2-*):** the mix under the paced
group at the fraction 0.9 arrives 36 / 38 / 40 with every type within 36-40, 102 x 352 at contact, ramp 1.3 / 1.9,
twelve formed every second at the pace 49, against the unpaced 18 / 25 / 35, 932 x 183, 7.2 / 17.2; `ranks` gives the
old march's end state (the mix 18 / 25 / 36 at 924 x 347; the Blitz column 18 / 19 / 20 at 94 x 339); the pure Blitz
column paced costs 4 s for nothing (22 / 22 / 23 against 18 / 19 / 19), the fraction's price with no slower type to
hold slots for; the reverse holds the formation through the turn (recovery 6.0 s against 4.0, the anchor slowed from
49 to 41 for four seconds). Two faults found and fixed the same night: the gather's anchor left the lab with its first
unit (the host's protocol was missing, not the lane's: the director now holds the group at the rally, `AT_THE_LAB`
along the axis, until the whole list stands there formed, then gives the order, which is what Jev would do from the
state's words), and a fighting group's march orders were Moves, so units walked on to slots on the gantry's footprint
and the shot ramp to all was 6.0 s (Fights now, as the old march gave). The 12 v 12 Blitz-Stout fight is lost either
way and read worse under the group at 0.9 (y's value left 0.68-0.78 against 0.58-0.66); the fraction 1.0 arm
(`WITHIN_REASON_PACING=fraction[,cadence]`) and the Fights are the next row's question. The lane keeps no formed /
exception books at contact (the slots are the arc's then); the flagship's hands declare no group yet, so H-MICRO-FORM
is inert there until their migration.

**The second bench the same night (ledger row group2-*):** the fraction 1.0 beats 0.9 everywhere, because the drag
rule holds the anchor to a lagging member and a margin on the pace buys nothing: the mix 33 / 35 / 37 (every type
34-37, twelve formed every second, the anchor at the Janus's 54 after a dip to 44 during the spin-up), 135 x 352,
ramp 1.4 / 2.5, which meets §5's bar for the mixed column; the Blitz column 20 / 20 / 21; the gantry under Fights at
1.0 first shot 21 s, shot ramp 0.8 / 1.8 (the bar 2.5 met; 1.2 / 2.8 at 0.9; 6.0 under Moves). `Pacing::fraction`
is 1.0 from here. The gather holds at the rally and forms the list as it comes (one joiner at a time, each an exception
for about four seconds), then walks as one; its first run ended the hold with the last unit still walking from the lab
(`arrived` ignores exceptions beyond the threshold), so the director's condition asks for no exceptions too. The
Blitz-Stout fight stays worse under the group than unpaced (y's value left 0.71-0.79 against 0.62), a matchup lost
either way; the regression check is the body-shape study's own arms (Stouts against Fatboys and Bulls at 5,400, E3
900 against the director) against their numbers (+0.462, +0.366, +0.292 under the old lane). **Run the same night
(ledger row regress-*):** Fatboys 8-0 at +0.47, Bulls 8-0 at +0.27, E3 24-0 at +0.36, the attack-move arms at -0.55,
-0.32 and +0.31 as the study had them. The contact arc under the group loses nothing the old march had. **Caveat and
the clean run (the same night):** the user, watching the demos, saw the sweep's Bedbugs standing into the next duel;
every fight match since the synced spawner carried them (the sweep judged a wave gone before the synced give had set
its bombs down; `docs/harness/pitfalls.md`), so those margins were contaminated by about a tenth either way. With the
sweep fixed: Fatboys 8-0 at +0.40 +- 0.09 (the study's +0.462 +- 0.113), Bulls 8-0 at +0.34 +- 0.07 (+0.366 +- 0.064),
Fatboys attack-move 0-8 at -0.56 +- 0.11 (-0.515 +- 0.082): within the study's spreads.

**The circuit with a feeder (D12, 2026-10-24 night; ledger rows circuit-*; `duel --drill circuit --ours armpw*12 --feed
armpw*8 --laps 1` on Quicksilver, the default loop 15,450 on foot, 178 s for a Pawn at full speed).** The first run
under the group was the law's failure in one picture: the director gave the anchor the four corners, the anchor walked
the crow line east from the lab through ground the Pawns had to go round to the north, within twenty seconds every
member was past the exception threshold, and the anchor, released from them all, walked the whole lap alone at 87
with twelve Pawns a map behind it (formed share 0.07; the units arrived 168 / 169 / 172 s, as a stream). Two things
changed on that evidence: the host passes the walked route (the field's way from corner to corner, a point every 256;
§2's rule as ruled, which the director had not kept), and the law is amended so that the anchor never outruns its body
(§2 above). The second runs found the harness faults listed under D12 (a swallowed `/cheat 1`, `name*0` as a plain
name, two Pawns spawned on broken ground that the amended anchor then waited nine minutes for, which is the amended
law's one hole and the host's rule now covers). **The third run, clean (circuit3-on against circuit3-off):** the group
walks the lap as a body, formed share 0.81, every one of the eight fed Pawns joins the body (median 57 s from the lab),
nothing left behind, and arrives 252 / 253 / 255 s as 339 x 288 with a ramp of 2.1 / 3.9 s; the lane-off baseline
(the corners as queued orders, every unit on its own path) arrives 168 / 170 / 172 s as a stream 3,778 deep and 3,822
wide, spread 1,774, ramp 2.7 / 43.4 s, and not one fed Pawn reaches the body. The group's price is the time: 1.42 x
the walk at full speed against the stream's 0.96 x, and the per-second series says where it goes: the anchor stopped
dead for 8 s at the first corner's passage and 4 s on the third leg (members behind through a file, the drag to zero),
and at the corners the turn rotates the whole formation about the anchor at once, so that at the third corner 15 of 17
members were behind their slots and the pace fell to 57-62 for several seconds; the start costs too (the slot deal
against the spawn formation, 8 formed and the pace 6-20 at 10-25 s). Against §4's bars: the lap is not within 1.1 x
the walk (1.42); the formed share is 0.81, not 0.9; every joiner joins; nothing is knocked out of its slot by a joiner
that the series shows (the formed count does not dip at an appearance). What the circuit asks of the group next, in
the order the series ranks them: a wheel at the corners (the slots should turn with the route's local direction, not
jump with the anchor's heading at a point), the shape narrowed to the corridor at a passage, and the slot deal at the
start.

**Route clearance (D13, the same night; ledger row circuit4-clearance-on):** the user, watching the two demos, named
the corner creep and asked for automatic room for an army rounding a point. With the route priced by clearance (the
formation's half-width plus a hull, 224; Quicksilver's loop then has 204 of room at its narrowest, one spot) the same
drill arrives 200 / 202 / 204 s against 252 / 253 / 255: 1.15 x the full-speed walk (176 s on the priced loop) against
1.42 x, with the formed share 0.83 against 0.81, the speed share 0.91 against 0.76, every fed Pawn joining (median 51 s),
nothing left behind, 286 x 288 at the end with a ramp of 2.2 / 3.3 s. The anchor's seconds under 0.8 of full pace fall
from 159 to 49 and its stops from 13 s to 1; within 600 of the four corners the slow seconds are 7 / 4 / 4 / 4 against
8 / 33 / 9 / 13. What remains is the wheel: at the third corner the turn still knocks 13 of 14 members out of slot for
a few seconds (8 away, 5 behind), the redirection work the user put next. The lap bar of 1.1 x is near; the formed
share's 0.9 is not met.

**The re-deal at the turn (D14, the same night; ledger rows circuit6-* to circuit12-*, walk-redeal-*):** built in
six rounds, each measured (D14's amendment has the mechanisms and what the traces changed). The end state on the same
drill: twenty Pawns with no feed walk the lap in 188 / 189 / 191 s, 1.085 x the walk on foot (the lap bar met), at a
speed share of 0.94 and a formed share of 0.94, 8 slow seconds a lap, the three big corners (118-132 degrees) turned
with 18-20 of 20 formed and the pace never under 66; fed, Pawns 196 / 198 / 199 (the sticky slots' 204, 1.13 x),
Stouts 246 / 249 / 251 (270), Flashes 186 / 189 / 190 (200), every fed unit joined. The formed share with a feed fell
to 0.75-0.77 (0.79-0.83) for a reason the bench found and the band must answer: a unit sent every tick to a point
moving at the pace settles behind it by a distance of the engine's (72-86 for a Pawn at 80, every rank alike, about
twice that for a Stout by its turn rate: `walk-redeal-pawn20` 0.89 of full speed, `walk-redeal-pawn12` 0.97, the old
even layout the same), so with the drag from 64 a body of twenty Pawns was held at 0.90 of full pace by nothing but
following, and with the drag moved to the formed band (128) it walks at full pace and trails at the band's edge, a
Stout past it. The formed-share bar (0.9) is therefore not a formation question but a lead question: the slot order
placed the type's following distance ahead of the slot, so the unit stands in the slot and the band means what it says
(a lead measured from the units as they walk, or from the turn rate in the defs, which the protocol does not carry).
That is the next decision for the user, with the shape to the corridor and the slot deal at the start behind it. Beside
the group: the field's way thinned to 256 was a staircase of grid bends, straightened by line of sight (19 points from
58); the per-second log's headings to a hundredth; the game swallows `/cheat 1` now and then, both sendings (four of
nine matches), so an army of which nothing appeared in three seconds is given again under a fresh `/cheat 1`
(pitfalls.md).

## 6. Open for the user

Ruled 2026-10-24 night (§2): the group API's shape, the gather's fate (it dies as a move; the pace and the joiner
behaviour do its work and its words become the group state), the commitment ladder's destination (the group order).
Still open:

- The numbers §2 names, set by the bench, not by hand: the pace fraction, the lead and the cadence (D11 first), the
  slot tolerance and the exception threshold.
- Whether the raider/line separation, the host's choice from here on, becomes a word to the player or a default of
  the hands.
- The maps for drill 6 beyond Quicksilver, Great Divide and Altored Divide (Cape Violet's shore, Isidis Crack).
