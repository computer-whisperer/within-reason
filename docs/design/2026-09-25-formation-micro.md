# Target design: formation micro — slots for a group's units, the lane in the duel harness, and the lane's shut-offs

Written 2026-09-25 by Claude (Fable 5.1) before any code, at the user's direction: improve the micro engine and
demonstrate the improvement in real engine fights; investigate why the player games switched the lane off. Branch
`micro-formation`, worktree `bar_bots-micro`. Supersedes phases 3 and 4 of `docs/design/2026-09-24-muzzle-project.md`.

## What is true today

- **The evidence** (`docs/studies/2026-09-24-pro-fight-shapes.md`, `docs/knowledge/formations.md`): the pros give one
  point per unit before contact (70% of group orders; ours 2 of 44), on a nearly straight line 64 elmos apart (two
  hulls; the hull is 32); their Stouts and Blitzes stand 67-70 from the nearest friend at contact, ours 25-40; their
  bodies are 6 soldiers, ours 13; they mix raiders and line units in 10% of cores, we in 61%; their raiders fight at
  0.94 of their reach, ours at 0.86; a friend is within a hull of the line of fire for 22% of their units in reach,
  43% of ours (Stouts 24% against 53%). A line at contact trades +5 points over a blob; orientation and type layering
  do not matter measurably. The engine refuses a shot across an allied hull and the refused unit stops and points
  (K-engine-a-shot-is-refused-across-a-friend); live Blitz muzzling is almost all a friend on the line; the Stouts'
  shells land on the Blitzes in front (bank-1: friendly fire 13% of all damage, the Stout three quarters of it).
- **The lane** (`crates/bot/src/brain/micro.rs`, `docs/design/2026-09-20-micro-lane.md`): every tick, per soldier
  with an enemy within 700 of its reach, flee, fan, focus, kite, else the standing order; march (`march.rs`) and
  follow (`pianist/groups.rs`) act at the group level. Every intent source (raid, contact answers, waves, squads, the
  pianist's groups) gives a group one point (`Fight { to }` per member, the same `to`), which the engine turns into a
  crowd. The lane has no notion of a slot. It is `impl Brain` and reads fifteen brain fields; nothing outside the bot
  can run it.
- **The duel harness** (`crates/arena/src/bin/duel/`, `docs/harness/duels.md`): both sides are the director;
  `send_idle` re-sends every idle unit at the enemy's centre every tick; `--formation` sets the spawn shape and the
  `line` advance; `--spread` is a copy of the retired H-MICRO-SPREAD's block (`loose_block`); the fire instrument
  gives muzzled seconds by cause, friendly fire in total (no victim table), and the exchange; nothing measures
  spacing or the friend-on-the-line share at contact as `run/replays/shapes.py` defines them; no mixed armies.
- **The shut-offs** (read from the records, `strategist-0.jsonl`, `jev-0.jsonl`; the scratch scripts are not kept):
  - 2v1-hard_aggressive 7:13 "flee pulled groups out of Pawn fights": in the minute before, the groups that left
    fights were sent home by the hands (`move_to->home` five times, Jev's fall_back rule); the lane fled 14 times, on
    single units and on group D, which stayed in its fight. Two D units at 7:04-7:12 were re-stepped every 6-9 frames
    to points zigzagging within 50 elmos: the flee step chooser mills.
  - 2v1-hard_aggressive 11:44 "follow re-sends groups after fleeing parties": right. Group F (11) followed a party
    2,000 elmos east 9:21-9:46, K (4) 3,500 elmos 11:04-11:32 and died to Warriors; follow re-sent every 2 s while the
    party stayed in sight 300-600 ahead. There is no leash.
  - 2v1b-hard 17:23 "small groups step back instead of joining": the stepping back is the hands' fall_back (groups
    of 1-2 always outweighed); the lane then flees them all the way home (a walking group flees everything, by
    design) at half speed: after a lane step Stouts move 28-30 elmos/s and Blitzes 25-40 against 48-55 on a far move
    and 67-100 on a fight order (three games, n = 62-550 per cell). A 100-elmo step re-issued every 32 elmos keeps the
    unit in the engine's braking zone.
  - 2v1b-hard_aggressive 5:51 "groups only moved after lane raw": the lane fired no rule and gave no order to groups
    A and D in the minute before; they held because Jev chose hold. The hands' engage orders at 5:52 came with the
    same turn. Misreading. 14:13 (H raw), bank-1 3:46 (B), 9:05 (E), 2v1b-hard 10:09-10:29: no lane activity on those
    groups in the minute before; precautionary or misread.
  - bank-1 8:53 (I, J raw): J's six Blitzes at 44 elmos from Pawns were fled 12 times a second and re-taken by focus
    the next tick (a fresh flee claim is offered to focus at once); J never left. Chatter, not a retreat.
  - bank-1 14:13 (L, Q): Q walked home at 39 elmos/s under flee steps (the braking defect). L, holding (standing
    order Stop), got `Attack` + `Stop` from focus every second: `queued(Stop)` is a plain Stop that cancels the
    Attack in the same batch.

## The design

### 1. The lane becomes a crate (`crates/micro`), the brain and the duel director both drive it
`micro::Lane` owns the claims, standing orders, commitments, footwork, memories, the threat grid and the milling
counters; `Lane::tick(&mut self, view: &dyn View, tick: &Tick) -> Output` returns the commands, the rules that fired
(for the journal) and the debug lines. `View` is what the lane reads of its host: unit definitions, a type's reach /
damage a second / speed / D-gun reach, the passable mask and grid spec, snapping to reachable ground, home, the
remembered armed buildings, whether an enemy is still known, the radar blip's type, and which rule IDs are disabled.
`Footwork` moves into the crate (the bot re-exports it). The brain's `micro.rs` keeps what reads brain state
(`note_standing_orders`, `note_commitments`, `footwork_of`) and implements `View`; `brain/threat.rs` moves into the
crate. Chose-because: the director must issue the orders the bot issues for the check to mean anything, and a copy
(the `--spread` precedent) drifts; the sim's phase-4 "pure functions over a world-view" is this, without the sim.
Rejected: making the bot crate a library and building a `Brain` in the director (the brain's constructor and state
are the whole bot).

### 2. H-MICRO-FORM: slots (the formation behaviour)
A **body** is a set of soldiers under the same group order (Fight or Move to points within 48 of each other, or
Attack on the same target), chained within 400 of each other, of one class: **raiders** (reach <= 250) and **line
units** (reach > 250) under one order are two bodies (K-form-pros-fight-with-pure-bodies). A body of more than 12
splits by its order across the heading into sub-bodies of at most 12, each its own body. Bodies of one are left to
their order. Every tick, for every body:
- **Heading** `h`: from the body's centroid toward the order's goal (an Attack target's position when known); with
  armed enemies within 700 of the centroid, toward their centroid instead, so the line faces what it meets.
- **Slots**: one rank across `h`, `SPACING` = 64 (two hulls), centred on the anchor = centroid + `h` x `LEAD` (150),
  never beyond the goal (when the goal is nearer than the lead the rank is centred on the goal). Snapped to
  reachable ground. Chose one rank because a second rank at 64 has a friend within 24 of its line for any target
  more than about 10 degrees off the axis (the checkerboard offset is 32, the hull test is 24), and the evidence says
  a line trades no worse and the pros' bodies are one rank deep in practice (core 190 x 165 for 6).
- **Assignment** by least total travel: units sorted by their coordinate across `h` take slots in that order (no
  crossing), then pairwise swaps while any swap shortens the total (CustomFormations2 solves the same assignment).
- **Advancing**: a unit not in contact gets `Fight` (Move for a Move order, Move for an Attack order) at its slot, as
  a claim (`Rule::Form`); re-issued when the slot has moved more than 64 from what was sent and 15 frames have passed.
  Slots move with the centroid, so the body walks together and the slow are not left behind.
- **Contact**, per unit: an enemy within its reach minus 20 -> it **stands**: `Fight` at its own position (an Attack
  order at the target instead, when its group order is an Attack and the target is in reach), re-issued only if it
  drifts more than 48 from where it stood. A **raider** with the nearest enemy nearer than 0.8 of its reach steps
  back along the line to it to 0.92 of its reach (K-form-pros-raiders-fight-at-reach), a Move so it keeps shooting
  on the move; a raider in reach between 0.8 and 1.0 stands. A unit not in reach while its body has enemies within
  700 closes on the nearest enemy to 0.92 of its reach (`Fight` at that point), so a line's ends curl in rather than
  stand out of reach (the muzzled-ball study: the line lost because its ends were out of reach). A Move order has
  no contact behaviour: a walking group walks.
- **Place in the lane**: after flee, fan, focus and kite; it replaces "the standing order" as the terminal behaviour
  for units in a body (a unit in no body is released to its standing order as today). Focus queues the unit's form
  order behind its Attack instead of the raw standing order. H-MICRO-FORM has its own ID under H-MICRO-LANE; the
  pianist's `Footwork` gains `form` (default on; `raw` clears it).
- **Deleted**: the duel director's `--spread` and `loose_block` (superseded by running the lane in the director).

### 3. The lane's defects (part C), fixed in the crate
- **Steps brake** (2v1b-hard 17:23, bank-1 14:13): a flee step chooses its cell within 100 as today, then the order
  goes to the point 2.5x further along the same direction (clamped to the map and snapped), and is re-issued after
  64 elmos, not 32. Judged by the step-speed table above, re-run on the live game.
- **Flee/focus chatter** (bank-1 8:53): a unit under a Flee claim younger than `CLAIM_FRAMES` (30) stays with flee;
  it is not offered to fan, focus or kite that tick.
- **Focus cancels itself on a holding group** (bank-1 14:13): no order is queued behind focus's Attack when the
  standing order is a Stop; the release re-issues the Stop.
- **Follow has no leash** (2v1-hard_aggressive 11:44): an engaging group re-sent after its party stops following
  when the party has drawn it more than 900 from where the engagement began; it holds, the hands' report says
  "party_N drew group_K 900 from where it engaged: it holds", and the picture shows it (pianist `groups.rs`).
- **The flee chooser mills** (D at 7:04): a claimed unit keeps its step while the step's cell is still within 20% of
  the least threat in reach of it; only a clearly better cell replaces it.
Not the lane's, said plainly: the fall-backs of small groups (Jev's rule and the packet), the holds at 5:51.

### 4. The harness measures what the survey measured
- `duel --lane X[/Y]` per side: `off` (today), `old` (the lane without H-MICRO-FORM), `on` (the lane). The director
  holds a `Lane` per army, notes its own Fight orders as standing orders, commits everything, and appends the lane's
  commands after its own; with the lane on, `send_idle` re-sends only every 2 s and when the enemy's centre has moved
  150 (as follow does), so the lane's holds are not fought by the director's re-sends every tick.
- Mixed forces: a side is `name*count+name*count` (spawned in list order, so the first type is the front rank); a
  plain name against a mixed force takes the count that matches its metal.
- New columns in `duels.csv` (each `_x`/`_y`): `nn` (median nearest-friend distance over the body at t0 and t0+10,
  soldiers only), `fol_in` / `fol_blocked` (units with an enemy within reach + 20, and those of them with a friendly
  soldier within 24 of the segment to the nearest one and nearer than it, summed over t0 and t0+10: `shapes.py`'s
  definitions), `xf` (friendly fire by shooter type -> victim type, `a>b:dmg;...`). `pairs.csv` and `run/duel_ab.py`
  carry nearest-friend, friend-on-the-line share, friendly-fire share and the exchange.

## Judged by
1. Duel batches, `--reps 8`, arms `off`, `old`, `on` on the same sites and seeds (`run/duel_ab.py`): Blitz pack
   against Pawns; Stout body against Stout body; Stout body against a Cortex bot mix; a Blitz-and-Stout force
   against a Cortex mix (the user's case); the two scenarios (2v1b-hard 15:10 truth, bank-1 27:34 truth). Read:
   exchange, muzzled seconds by cause, friendly fire (and its victims), nearest-friend spacing, friend-on-the-line
   share at contact. The claim to beat: spacing 64 and one rank take the friend-on-the-line share from ours (43%)
   toward the pros' (22%) and the exchange does not fall.
2. One live heuristic-bot game against BARb hard_aggressive on Comet Catcher (`form-1`), read with `run/fire.py`
   and `run/micro_ledger.py` beside bank-1 and the 2v1b games; the step-speed table beside the three games above.
3. The defect fixes: the step-speed table; no Attack+Stop pairs in the record; follow leash lines in the hands' log.

## Decisions taken here (the evidence does not settle them; stated so they can be reopened)
- One rank, not a checkerboard block (the hull geometry above). If a body of 12 at 64 (700 wide) proves too wide on
  Comet Catcher's ramps, the split size drops to 8.
- Sub-bodies of a large body are separate bodies side by side, each centred on its own units, not an echelon.
- Raiders hold at 0.92 of reach (the pros' median is 0.94; the engine's range test and ours disagree within 20).
- The follow leash is 900 (the engage/fight-in-sight horizon is 600; a party that has walked the group 900 from its
  start is running, not fighting).

## Status
- 2026-09-25: written; nothing built.
