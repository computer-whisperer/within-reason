# The army's control loop, from the properties down (2026-11-15)

Target design for the rebuild of group movement in `crates/micro`. Written before the first deletion, as the arc's
ground truth. Nothing here is built. It replaces `group.rs` and `army.rs` as a base; the earlier designs it
supersedes are listed in §8 and stay in the tree as history.

## 0. The rulings

The user, 2026-11-15, after the read of one engagement (`docs/experiments.md`, the `c5-repro ...` row: the tier-3
army of vents-barb-8v8-1 milled at the place it was ordered to for 116 seconds under one order):

- "Look at this from a controls perspective. If the micro engine can fail this badly when under a fixed control
  state (move to c5) and only unit movement, then it probably wasn't built very well to begin with. Control loops
  like this need to be thought through very carefully from the beginning and, when a new scenario occurs you didn't
  think of when designing it, you must re-evaluate the assumptions you used to design it. Don't patch this -- look
  at the control loop and see where the core architectural mistakes were made."
- On the review that answered it: "Go ahead with the design update first. Each point we see as a theoretical issue
  from simply looking at the implemented control loop should be addressed with the design update. This is a
  variation on the rip-out-and-replace ruling, where fine tuning a bad base is pointless until you rework the base.
  In which case, you might as well rip out the fine tunings that depended on the bad base and let future experience
  guide fine tunings on the new base."

Rulings of his that bind this design, from earlier arcs (the words are in the designs named):

1. "The army being a real object in the engine was the intent since day one" (2026-11-09).
2. The cap is so that no group inherits one movement domain: companies are small and move independently; no
   company is slaved to another's anchor (`2026-10-25-companies.md`, after companies-1; again 2026-11-11).
3. Almost no hidden state: "the state we are trying to manipulate is completely in the game ... Two units with
   the same position, velocity, etc that are not fully interchangable is a gauge orbit we can take advantage of
   removing" (2026-11-10).
4. A group is "that set of unit slots in that location" the player can point at for thirty seconds (2026-11-11).
5. Nothing in the engine holds a unit back from an active engagement (2026-11-11).
6. On restricted ground the posture is relaxed: "the best thing the units can do is push through it as fast as
   they can"; nothing there sends a unit to align (2026-11-10).
7. The formation's value: legible control, units free to fight without chasing, a layout that holds when the
   enemy is met; the march in ranks is judged good from watching it (2026-11-09, 2026-11-11).
8. Formations should separate from each other where they can; overlap is not forbidden (2026-10-29).
9. "Reduce the control system complexity ... Whatever we build should start out simpler and more likely to work
   in all scenarios" (2026-10-24).

## 1. The loop as it is

One declared group is partitioned into companies of at most the cap. Each company is a `group::Group`: an anchor
(a virtual lead point) that integrates along a planned way at the slowest type's speed, slowed by the lag of its
worst member; slots as offsets from the anchor in a `facing` frame; each member sent to its slot. The order (one
point from the flagship) is copied to every company. Above the companies `army.rs` sorts soldiers into them, merges
two that fit, and composes a report; it controls nothing.

Around that core, each of these writes a company's anchor, way, way's end, facing or deal from inside the tick:
the planner on a ground rebuild (any structure change on the map, at most every 5 s), on a standing formation
coming onto the way (every 2 s), on an end that moved; `keep_off` (the end moved beside whichever formation
outranks this one now, ten times a second); `retarget` (attack, party, ward); `close_on` and `guard_at` (contact);
the turn at a corner of 60 degrees (anchor set down on the body, slots re-dealt); the re-aim on a new order; the
pass (anchor set on the forming-up ground) and `place_forming` (moved on past another formation); a membership
change (every slot re-dealt). `Group` holds 53 fields, nine of them books by unit.

## 2. The theoretical issues, from the loop alone

Each is a property of the structure, visible without running it. The last column is where this design answers it.

| # | Issue | Why it is wrong in a control loop | Answer |
|---|---|---|---|
| T1 | One order, N controllers, no allocator: who stands where is settled by companies moving each other's ends | Each loop's setpoint is a function of the other loops' outputs; no resting arrangement is defined, so none can be converged to | §5.2, §5.3 |
| T2 | The priority between companies depends on their state (standing first, then size, then name) | A priority that inverts as the state moves re-opens every earlier settlement | §5.3: order of decision is fixed |
| T3 | Other formations enter the plan by their pose this instant, read a tick late | Planning round where a controlled body is, not where it will stand, is the reciprocal dance; a one-tick-late simultaneous update oscillates | §5.4: only places that stand by decision are priced |
| T4 | Every solver (way, end, deal, partition) is run again from scratch on its trigger, with no term for the answer in force | A solver is a discontinuous map of its inputs; in a feedback path it must carry hysteresis on its own value | §4 (P6), §5.1 |
| T5 | A change anywhere on the map re-plans every company | No relevance test between a disturbance and what it can affect | §5.4 |
| T6 | The outer decisions run as fast as, or faster than, the inner loop settles (end placed 10 Hz, re-plan 0.2 to 0.5 Hz, orders 1 Hz; a re-lay takes about 9 s) | Cascades need the outer loop slower than the inner one's settling; otherwise the inner loop is always in transient | §4 (P6): dwell |
| T7 | Nine or more writers of one company's reference, ordered by call order | No single owner of the setpoint; interactions between writers are untested by construction | §5.1: one writer, at decisions |
| T8 | The anchor is dead-reckoned and re-synchronised with the body by events (re-aim, set-down, placement by the dense cluster) | The loop's main state is open-loop; each event is a jump | §5.5: the frame is seated from the units when it is founded, and the hold of §5.6 keeps it by them since (D10: seated at every order as first written, a body of two speeds stalled) |
| T9 | The pace is fed back from the worst member | The most outlier-sensitive statistic there is; influence of one unit unbounded | §5.6 |
| T10 | Each kind of outlier is excused by a latch (away, once formed, waited, stuck clock, cut off, joiner reach, pass body, given up) | State the game does not have; two identical armies differ; no analysis of the latches together | §5.6: an order statistic and a velocity test, both memoryless |
| T11 | The pace type marches at its own top speed | The reference sits at the actuator's saturation: a pace-type member behind its slot cannot close | §5.6 (stated limit, with the knob) |
| T12 | A membership change re-deals the whole company; the partition founds and merges companies during an order | A disturbance of one unit moves many; the set of controllers is itself state | §5.7 |
| T13 | A new order clears the deal and re-aims; a place moved 49 elmos is a new order | The response is not continuous in the command | §5.8 |
| T14 | `arrived` is the anchor's index on the current plan plus the latches | The outer loop measures the inner loop's books, not the plant | §5.10 |
| T15 | The group's place in the report is the largest company's anchor | Flips between companies; a controller variable published as a measurement | §5.10 |
| T16 | Contact is a set of modes that move the reference (the close walks the anchor at the nearest enemy and turns the ranks to it every tick; the party's end hops) | An argmin selector in the reference path is discontinuous; a second goal generator beside the order | §5.9 |
| T17 | The halt and the arrival quantify over every member | Same fault as T9: one member decides | §5.9, §5.10 |
| T18 | The pass has its own books and is reset by every re-plan; forming-up grounds are negotiated between companies as they arrive | T4, T7 and T1 again in the second posture | §5.5 (a gap in the way), §5.4 (grounds at plan time) |
| T19 | Allied buildings are in no ground; the field model is ours and his | The plant model omits what a team game puts on the place | §5.4 |
| T20 | The bench drills one formation on a static field with fixed members; its bars are lap times | The design was validated only inside its own assumptions; no drill states a closed-loop property | §9 |
| T21 | No design stated what the loop must guarantee; each stated a mechanism | Nothing to check a new scenario against, so each got a rule | §4, §11 |
| T22 | The pitch is read each tick from the largest blast radius within 1,500 of the body | Every slot moves the tick an enemy with an area weapon comes or goes: a sensed quantity stepping the reference with no dwell | §5.11: the pitch is a decision |
| T23 | A slot that stands is sent once; a member the engine stops short of it has no order and gets none (`resend_due` needs the point to have moved) | The innermost loop has no retry at rest, so rest is not reached by everyone | §5.6 |

## 3. The plant, and the assumptions this design makes about it

Stated so that a scenario that breaks one is recognised as breaking the design, not as needing a rule.

- A1. A unit goes to the point it is sent to by the engine's own path, at up to its type's speed, and brakes into
  it. There is no speed command. A blocked move may end short and leave the unit with no order.
- A2. Orders are budgeted (100 a second a seat, `2026-11-10-order-budget.md`): about one order a unit a second at
  most, so fine following by goal placement is not affordable for a whole army. Authority over a member's speed
  comes from where its goal is and from the frame waiting, not from a stream of orders.
- A3. Units collide and push; the engine resolves it. The design keeps slots from overlapping and does not model
  collisions; where units must cross each other is a cost the bench measures, not a thing the loop reasons about.
- A4. The field changes: buildings and wrecks appear and vanish at any time, ours, allies' and his. Soldiers join
  and die at any time (C5: 45 joined and 32 left one group in 117 s).
- A5. The order changes at up to once a second and may alternate (the pick is a draw).
- A6. The host hands the engine only the soldiers with the group (`deal.rs`: inside a gate of fifteen seconds of
  their own travel). A member may still be several hundred elmos from the body.
- A7. Members of one type are interchangeable. Members of different types differ in speed and reach only.
- A8. Other bodies stand and move on the field that this group does not control: allies' units, our other groups,
  soldiers under a reflex (flee, fan, kite), units on a hunt or a rove.

## 4. What the loop must guarantee

These are the contract. A mechanism is in the design only if it serves one, and each has a drill (§9).

- **P1 It converges.** Under one order, on a field that does not change, with members that do not change, the
  army comes to rest in one arrangement that is defined at the order, in about its travel time; after that no
  order goes out.
- **P2 A disturbance moves only what it touches.** A soldier joining moves itself. A soldier dying moves nobody.
  A building that blocks nothing moves nobody; one that blocks a company's way or place moves that company.
- **P3 The response is continuous in the order.** The same order again changes nothing. A place moved by `d` moves
  each member's goal by about `d`. Alternating between two orders costs the travel between them and no more.
- **P4 The report is of the units.** Progress and arrival are functions of where the soldiers are and what was
  ordered. No re-plan, re-deal or restart changes them.
- **P5 The state is small and of two kinds.** One number a company that moves continuously, and decisions. A
  decision is a pure function of the order and the units at the moment it is taken. Restarting the engine on the
  same units and order changes each member's goal by a bounded distance.
- **P6 Decisions dwell.** A decision once taken stands for at least its dwell (longer than what it disturbs takes
  to settle), unless it has become impossible. A decision is replaced by a better one only when the gain exceeds a
  stated margin.
- **P7 Something always moves.** At every moment a company's frame advances, or it waits for a member that is
  moving toward its slot, or an enemy stops it, or it is at its end (a follower's also stands behind its ward).
  Nothing waits on a unit that stands.
- **P8 Nothing is held back from a fight** (ruling 5): a member with an enemy in reach fires; no rule delays a
  member to improve the fight.

## 5. The design

### 5.1 Structure

Three objects, one owner each.

- **The army** (one per declared group) owns every decision: the facing, how many companies and who is in which,
  each company's block at the place, each company's way, the deal within each company. It takes decisions at
  events only (§5.11) and writes them once.
- **A company's frame** is a rectangle of slots (ranks of six, as today) with a centre and the army's facing. On
  the march its centre is at a distance `F` along its own way. `F` is the only continuous state. It never
  decreases under an order and it is advanced by one law (§5.6), limited by clamps that are functions of the
  units and the enemy (§5.9).
- **The report** is computed from the units and the order each tick (§5.10) and reads no frame.

Between decisions nothing writes a frame's way, end, facing or deal. Inside a tick no company reads another
company's state.

### 5.2 The place: one layout for the army

The order names a place `P`. The army's resting arrangement is a **layout**: one block (a company's rectangle) per
company, laid on a lattice about `P` in the army's facing, each block where its slots stand on the ground.

- The lattice cell is the largest block's width and depth plus the margin between formations (48). Rows of up to
  four blocks abreast, the front row centred on `P`, further rows behind it.
- A cell is taken when the block stands there whole on ground joined to its centre and overlaps no block another
  group holds; otherwise the next cell outward is tried. When too few cells near `P` take a block whole, the cells
  where the most slots stand are taken and the slots off the ground are snapped beside, as `Ground::stand` does
  today for one block.
- It is a pure function of (`P`, facing, the companies' sizes and pitches, the ground, the other groups' blocks).
  The same inputs give the same layout whoever arrives first and wherever anyone is.
- A block is **held** from the moment of the order, whether its company has arrived or not. Other groups' layouts
  are laid round held blocks; the group ordered earlier keeps its blocks. Nothing re-packs when a block is given
  up.

Not slaving (ruling 2): no company's position is an offset from another's while they move. Each has its own
destination, way and pace. What they share is a decision taken once at the order.

### 5.3 Who goes where

At the order the companies are paired with the blocks by the least total travel from where their frames are (at
most a handful of companies). The pairing stands for the order. With a least-travel pairing the straight lines do
not cross, so on open ground companies that set out abreast arrive abreast without meeting.

The order in which the army decides for its companies is the block index. It does not change with their state.

### 5.4 The way

Each company has its own way from where its frame is to its block (ruling 2), planned over the group's ground as
today (`Ground`: the members' common ground, priced by room).

- In the price, beside the terrain and the structures: the blocks other groups hold and this army's other blocks
  (a place that stands by decision, at the present `STANDING_COST`), and a band about the ways already planned for
  this army's earlier companies (the present `MARCHING_COST`), so siblings take lanes beside each other where the
  ground has room and share the way where it has none (ruling 8). No formation's pose enters a plan.
- The structures are ours, the enemy's seen or remembered, wrecks, **and the allies'** (the snapshot's `allies`
  with no speed).
- **A way stands** until one of: a new order; a structure now on a cell of the remaining way (then it is planned
  again); a ground change after which a fresh plan is shorter than the remaining way by a fifth and by 200 elmos
  (then the fresh plan is taken). A ground change that does neither changes no way. The ground itself is rebuilt
  as today.
- A block that no longer stands whole after a ground change is laid again (the next free cell), and its company's
  way with it. No other block moves.
- A route of several points (the bench's circuits) is one order: each company's way passes each point by its
  block's offset in the layout, and the facing is the order's throughout. An order with the march off (the
  bench's `ranks` arm) is a way that is one gap: every frame is set on its block at once.
- **Tight stretches** (ruling 6) are found on the way when it is planned and again on the remaining way at a
  ground change: a stretch is tight where fewer than four fifths of the block's slots stand on ground joined to
  the frame's centre. Each gets a forming-up ground: the first place beyond it on the way where the block stands
  whole. Where several of an army's companies have a tight stretch in common, their forming-up grounds are laid at
  the plan, one beyond the other on the way, the company nearest the stretch furthest on. Nothing is negotiated
  on arrival.

### 5.5 The frame on its way

- The frame's centre walks its way; the facing is the army's and does not turn at the way's bends, so a bend
  translates every slot by one vector and the deal stands. (Units walk at one speed in any direction.) There is no
  carry, no set-down and no turn on the march.
- **A tight stretch is a gap in `F`**: a frame that comes to one is set on the forming-up ground beyond it, and its
  members go there on one order each by the engine's path. The waiting rule of §5.6 is what holds the frame there
  until its body is through; there are no books of the pass. A group that fights passes relaxed only when nothing
  hostile is within 1,000 of the stretch when the frame reaches it (the rule of 2026-11-10, kept); otherwise the
  frame walks the stretch in ranks at whatever pace §5.6 gives.
- **Seating.** When a frame is founded (a re-form, a newcomer's company), it is put where its body is: the
  median, by coordinate, of where each member's position says the centre would be. A company whose block does not
  stand where its body is (it was founded in a yard) starts in a gap: its members are sent to the first ground on
  the way where the block stands. After its founding `F` only advances, a new order's way starts where the frame
  is, and by §5.6 the frame cannot leave the body. (As first written and built the frame was seated again at
  every order; D10 took that out.)

### 5.6 The pace, and whom the frame waits for

`F` advances at the slowest type's speed times the hold.

- **The hold** is computed each tick from the members' lags (how far each is behind its slot along the travel):
  take the members that are *live* (under the formation's order, and either within the band of their slot or
  moving); take the lag of the member four fifths of the way up that list; the hold is 1 while that lag is under
  the band (128), falls to 0 at the stop distance (300, wider for large hulls as today), as the drag does now.
- So the frame waits for the body and not for its last fifth (T9), a member that stands holds nothing (P7), and
  nothing remembers who was once formed, away or stuck (T10). A joiner walking in, a member in a reflex and a
  member penned by a wreck are each handled by the same two tests.
- If a whole company is blocked and standing, nothing holds its frame, the frame runs on, and its members' goals
  become their places at the next forming-up ground or the end: the company degrades to the engine's own move
  order and re-forms where it arrives. That is the worst case, and it is what a plain order would have done.
- **The goal a member is sent to** is its slot, carried ahead along the travel for the slowest type by the
  present lead (300) times the hold, shrinking to nothing over the last lead of the way. Other types are sent to
  the slot itself and trail it by their own following distance, inside the band.
- **A member with no order** (the engine ended its move) that stands further than the band from its goal is sent
  there again, on the march and at rest alike, at a cadence that doubles while it does not move (one second to
  eight). Today a slot that stands goes out once (T23).
- **Stated limit (T11).** The slowest type marches at its top speed, so one of them that falls behind inside the
  tolerated fifth does not close until the frame slows or stops. The alternative is a pace under the top speed,
  which costs the whole march that share always (the bench of 2026-10-24: 0.9 cost a column a tenth of its time).
  The base keeps 1.0; the straggler drill (§9) measures what it costs, and the pace share is the knob.

### 5.7 Members coming and going

- **The deal** within a company is the least-total-squared-travel assignment of its members to its slots
  (`form::assign`). That assignment does not change when every slot moves by one vector, so a march and a moved
  place never call for a new one.
- **A joiner** takes the nearest free slot of the army and nobody else moves. With no slot free, a block is added
  at the next free cell of the layout and a frame founded on the joiners; no existing block moves.
- **A death or a departure** frees its slot. Nobody moves.
- **Swap on gain.** Once a second the army computes each company's best deal for where its members stand; it is
  taken only when it lowers the total by a quarter and at most once in five seconds a company. This is what lets
  two members blocking each other's way to their slots exchange them (ruling 3), and it is the only re-deal
  outside a decision.
- **Re-form.** The companies' number and membership are decided when the group is founded, when the cap changes,
  and when a company's worth of slots stands free (not while a company is at contact, at most once a dwell): the
  fewest blocks that hold the members, laid at the body, every member dealt to every slot by least travel.
  Between re-forms no company is founded by nearness or merged, and a member changes company only by taking a free
  slot as a joiner. A group that comes back from a hunt or a rove is founded again on its units.

### 5.8 A new order

1. The same order (kind as today's classes, the place within 48, the same party or ward) changes nothing. A hold
   whose point lies inside the layout in force names no new place: the layout stands.
2. Otherwise the layout is laid at the new place, the companies paired, and the ways planned from where the
   frames are. The deals stand and no frame jumps: a member's goal moves by what its frame's travel turns, and
   by nothing at the moment of the order (D10).
3. **The facing** changes only when the approach to the new place (the last 400 elmos of a way planned from the
   body's centre) is more than 60 degrees off the facing's axis in either direction, the place is further than
   the layout is wide, and the last change is ten seconds old. A reversal keeps the facing (the army backs away
   facing what it faced). A change of facing re-deals each company on its body, once. A change wanted inside the
   dwell is taken when the dwell ends if the order still stands.

Under one order there is at most one re-deal by decision, at its start. Under orders that alternate every second
there is at most one in ten seconds.

### 5.9 Contact

Contact limits `F`; it does not move the goal (T16).

- **The front's guard**: a frame of a group that fights does not step to where a slot of its front rank would be
  nearer than the stop (half the company's shortest reach, as today) to an armed enemy in sight.
- **The halt**: it does not step while every slot whose member is armed has an armed enemy in sight within that
  member's reach (in by the present margin to halt, out by the present margin to walk again). The test is on the
  slots, so no straggler decides it (T17), and the members walk into slots that are in reach.
- When the enemy dies or leaves, the frame walks on along its way to its block. A walk (`Go`) stops for nothing;
  a hold stands in its layout.
- **An attack** on a unit or a party is an advance whose place the engine keeps on the target: the unit, or the
  party's member nearest the army's centre (kept until it dies, leaves sight, or another is nearer by 150). The
  place is moved when the target has moved 128, by the rule of §5.8, so it costs what a moved place costs.
- **A follower** stands in the row behind its ward's layout, and its frames do not step past the rear of the
  ward's body on the march. Its place is laid again when its ward is ordered.
- **Gone: the close.** Today an advance at its end walks at the nearest armed enemy within 700, up to 600 past
  the end, turning its ranks to it. In this design an advance ends at its place; what stands beyond it is said in
  the report and attacking it is an order. This changes a rule of `2026-10-28-contact-cleanup.md` §5 and is
  decision D3.

### 5.10 The report

Computed from the units, the order and the layout; valid whatever the frames are doing.

| field | what it is |
|---|---|
| `place`, `centre` | the order's place; the median by coordinate of the members' positions |
| `members`, `at_place` | how many, and how many stand inside the group's blocks at the place (grown by the band) |
| `arrived` | `at_place` is four fifths of `members` or more |
| `to_go`, `speed` | the median over members of the way left to their own slot at the place; the median of their speeds |
| `formed`, `exceptions` | members within the band of their slot; the others with a cause read this tick (coming: moving; standing: not moving; reflex: under flee, fan or kite) and seconds to the slot |
| `engaged`, `in_reach`, `halted` | as today's `engaged`; members with an armed enemy in reach; companies at the halt |
| `passing` | members through, of companies in a gap |
| `companies` | each one's centre (median), block, members, formed, at place, way left |

`anchor`, `heading`, `pace` and `legs_left` leave the state. Their readers (§8) take `centre`, `speed`, `to_go`.

### 5.11 Decisions, in one table

| decision | a function of | taken when | stands until |
|---|---|---|---|
| facing | the order's place, the body's centre, the ground | founded; an order approached more than 60 degrees off its axis | 10 s, then the next such order |
| re-form (companies, who is in which) | members' positions, the cap, pitch | founded; cap changed; a company's worth of slots free | 10 s |
| layout (blocks at the place) | place, facing, companies, ground, other groups' blocks | each new order; for one block, when it no longer stands | the order |
| pairing (company to block) | frames, blocks | with the layout | the order |
| way, tight stretches, forming-up grounds | frame, block, ground, held blocks, sibling ways | with the layout; blocked; beaten by a fifth and 200 | blocked or beaten |
| deal | members' positions, slots | re-form; facing; a joiner (its own slot); gain of a quarter | 5 s between gains |
| pitch | hulls, blast radii about | at a seating; when the wanted pitch has differed for 10 s | 10 s |
| party member aimed at | seen members, the army's centre | the order; it died or left sight; another nearer by 150 | that |
| ranks or relaxed at a stretch | hostile things within 1,000 | the frame reaches the stretch | through it |

The continuous state is `F` a company and one bit for the halt. By unit there is one book: its slot.

## 6. Why it holds

- **P1.** After the order's decisions nothing discrete changes on a static field with fixed members. Each `F` is
  non-decreasing and bounded by its way. It stops only for a live member moving to its slot, whose lag falls while
  the frame stands, so each wait is finite. At the end the slots are the block's, fixed at the order. The engine's
  collisions on the last stretch (A3) are the part no argument covers; the stand drill measures it.
- **P2.** By §5.7 and §5.4: a joiner is one assignment; a death is none; a ground change is tested against each
  way and block and touches only those it blocks.
- **P3.** By §5.8: an equal order is no event; a moved place is a translation, under which the deals are
  unchanged; the facing has a dwell.
- **P4, P5.** By §5.10 and §5.11. On a restart the army re-forms on the units: the same layout function, the
  least-travel deal on where they stand.
- **P6.** Every row of §5.11 has its "stands until".
- **P7.** By §5.6: a member that stands is not live.
- **P8.** No rule sends a member anywhere but its slot, and a member in reach fires on the way and where it stands
  (move orders, hold position, as today). The frame's waiting is the march's coherence, not a rule about fights:
  at contact the halt puts the slots in reach.

## 7. Scenarios, read against the design

Kept as the list to extend: a new scenario is added here and read against §3 and §4 before anything is built for
it (§11).

| scenario | what happens |
|---|---|
| C5: ninety in four companies ordered to a point among buildings, allies building on it, soldiers joining and dying, a building changing every five seconds | four blocks laid once in a row about the point; each company walks its own way to its own; joiners take free slots; a building change that blocks nothing changes nothing; four fifths in the blocks is `arrived` |
| a block of the layout falls inside a turret's reach beside the place | it stands there and the report says the fire; the layout does not read threat (a limit, §10) |
| the order alternates between two places every second | the ways are re-planned each second from where the frames are; no deal changes and no frame jumps; the units dither between two goals, which is the order's doing; a body of two speeds gets there as one of a single type does (the rows `alternate` and `alternate-mixed`) |
| a hundred through a wall's door | each frame is set beyond the door on its own ground, laid at the plan; everyone goes through on one order; each frame walks on when four fifths of its live members are up |
| a quarter of a company penned in a yard | the frame waits for none of them once they stand; the report names them standing |
| two groups ordered to one place | the later one's blocks are laid beside and behind the earlier one's |
| a building is begun on a standing block | that block is laid again at the next free cell; one company moves |
| the army is told to hold where it is on the march | a layout at the body's centre; frames paired to the nearest blocks |
| the engine is restarted mid-march | re-form on the units; goals move by under a block |
| an enemy stands 400 beyond the place, out of reach | the army stands at its place and the report says what is in sight; closing on it is an order (D3) |
| large hulls (a Thor's block is 936 wide) | the lattice cell is that wide; four abreast is over 3,000 across (a limit, §10) |

## 8. What goes, what stays

**Deleted with the base** (each was an answer to a fault of the structure, and its drill result stays in the old
designs as history): the anchor's dead reckoning, the re-aim, the set-down on the body and the placement by the
densest cluster (D12, D22, D23); the turn and re-deal at a corner and the carry past it (D14, D18, H-MICRO-FACING);
the learned lead of the faster types and its fourth-power share (D19, D20); `away`, `been_formed`, `waited` and the
stuck clock, `cut_off` and its walk search, the joiner's reach (D17, D22, D24, the fixes of 2026-11-09); the close,
`pinned`, the party's per-company end (the contact cleanup §5, §5.2); `keep_off`, `placed`, the footprints by
pose in the way's price, `way_blocked` and its cadence (§5.3 there, H-MICRO-GROUND-FORMATIONS); the plan-time stand
with the at-end exception and `way_skips` (§5.1 there, H-MICRO-STAND as a rule of one block); the pass's books,
`place_forming`, the deal by arrival (the restricted-ground design §1.3); the partition by nearness, the merge and
`JOIN_NEAR`, the main body as the largest company (`army.rs`); `arrived` as the anchor at its end with half the
body and nobody walking in; `FRONT_MIN` in the hands. `group.rs` and `army.rs` are deleted before the rebuild, with
their 65 tests; the new tests are of §4's properties and of the pure functions.

**Kept, because they are facts of the plant or rulings and not tunings of the old base:** ranks of six and the
pitch from the hulls and the blast radii (`form.rs`, H-MICRO-FORM-HULLS, H-MICRO-FORM-SPACING; the pitch becomes a
decision with a dwell); the least-travel assignment; the ground, its room price, `stand`, `snap` and the structure
overlay (`ground.rs`, less `stamp` by pose); the order's sending rules (`take_slot`: the near tolerance, the
re-send to a member the engine stopped, now at a slowing cadence while it does not move; H-MICRO-NEAR); hold
position for members (H-MICRO-FORM-HOLDPOS); the relaxed posture as a rule and its 1,000 (H-MICRO-RELAX); the
guard's stop and the halt's margins as numbers; the band and the stop distance of the drag; the cap and the
hands' deal of soldiers to groups (`deal.rs`); hunts, roves and the reflexes, untouched.

**Readers to move** (the engine's group interface is used by `crates/bot/src/brain/micro.rs` and
`pianist/groups.rs`, `bots/distilled/src/lane.rs` and `army.rs`, `crates/arena/src/bin/duel/director.rs`):

- The hands end an advance on the report's `arrived` (H-HANDS-ARRIVAL rewritten; its row in
  `docs/code-decisions.md` with it); the hold after it keeps the order's place; the `reached` register, the
  follow point and the shell's screen point read `centre`; the picture's words for a marching group come from
  `to_go`, `speed`, `at_place` and the exceptions' causes.
- WRDistilled's strike body and column are declared as today and change behaviour with the engine
  (`crates/micro` is shared); its debug line reads `centre` and `speed`. Its benches are run again after the build.
- The duel director logs the new report and, beside it, the same measures from the engine's truth (§9).

**Superseded as designs, kept as history:** `2026-10-24-formation-drills.md` §2 and D12 to D24,
`2026-10-25-companies.md` §2 and §3, `2026-10-28-contact-cleanup.md` §5 to §5.3, `2026-10-31-facing-and-footprints.md`,
`2026-11-10-restricted-ground.md` §1.3 (its rule stands, its mechanism goes). `2026-11-11-groups-as-slots.md`
stands; this design is the lane it left unchanged. Claims this design is expected to close or re-open are read
again at the build (`docs/knowledge/formations.md`: K-group-five-companies-told-to-one-goal-do-not-settle,
K-group-the-ways-price-walks-round-a-pass).

## 9. The bench: properties before laps

Built first and run on the present engine for the record, then on the new one. Every measure is taken by the
director from the engine's truth, beside the lane's own report.

| drill | what is done | what is read | P |
|---|---|---|---|
| stand | 24, 48 and 96 to one point: open ground; among buildings (the C5 scenes); through a door | seconds from the first member at the place to `arrived` and to the last order sent; each member's distance from its slot at rest; orders after rest | P1 |
| disturb | on the march and at rest: add a soldier; kill one; a building begun off the way, on the way, on a block | members whose goal moved more than the band | P2 |
| thrash | the same order every second; the place moved by 49, 100 and 300; two places alternated every second for 30 s | distance walked against net displacement; deals changed | P3 |
| report | every drill above | `arrived` and `at_place` against the same counts from truth | P4 |
| restart | the lane dropped and declared again mid-march and at rest | the largest move of any goal; the arrival's delay | P5 |
| pen | 10, 25 and 40 % of a company walled in | the rest's arrival; what the report says | P7 |
| straggler | one and three pace-type members set 300 behind on a long walk | seconds until within the band | T11 |

The standing drills (walk, circuit, the door, the yard, the canyon, pair, assault, the T2/T3 rows) are run before
and after as observations. Their old bars were set on the old base and are not bars for the new one.

## 10. Limits of the base, stated

Left for experience, each visible in a drill or the report: a pace-type straggler does not close on the march
(§5.6); a company with more than a fifth of its live members far behind and moving waits for them; the layout
reads no threat, so a block may be laid inside a turret's reach; one row abreast is wide for large hulls; a block
may travel at an angle to its ranks and meet the enemy side-on; holes left by the dead are filled by joiners or at
a re-form, not by closing up; the faster types trail their slots by their following distance; units crossing on
the last stretch into adjacent blocks are the engine's to resolve; allies' and other groups' mobile units are in
no plan; an advance does not close on what stands beyond its place.

## 11. How this design is amended

A fault seen in a game is first read against §3 and §4: which assumption about the plant it breaks, or which
property. If an assumption, §3 changes and every section that leaned on it is read again. If a property, the
change is a row of §5.11 (a decision with its function, its moment and what it stands until) or a term of the one
law of §5.6 or a clamp of §5.9, with a drill in §9. A rule that writes a frame's way, end, facing or deal from
inside the tick is not an amendment of this design.

## 12. Build order

1. The property drills and the truth-side measures in the duel director; run on the present engine.
2. Delete `group.rs` and `army.rs`. Build the layout (pure), the frame, the army's decisions, the report; the
   lane's glue; allies in the ground. Unit tests of §4 and of the pure functions.
3. The property drills, then the standing drills as observations; the ledger rows; one table before and after.
4. The hands, WRDistilled's reader, the picture's words; `docs/heuristics.md`, `docs/code-decisions.md`,
   `docs/knowledge/formations.md`, `docs/harness/duels.md`.
5. Pianist-only smokes; WRDistilled's benches; a flagship game on the user's word.

## 13. Decisions for the user

Each changes something he ruled or judged, or is a choice the bench cannot make.

- **D1 The army lays one block a company at the place and each company goes to its own.** This gives the army a
  controller. It is not the slaving of companies-1: nothing ties the companies while they move.
- **D2 No turn on the march; the facing is set by the order and only for a move across it.** Blocks will sometimes
  travel sideways and can meet the enemy side-on. It buys: one deal an order, and nothing re-laid on the way.
- **D3 The close goes.** An advance stands at its place; what is in sight beyond it is reported, and attacking it
  is Jev's order.
- **D4 The hands end an advance when four fifths of the group's soldiers stand in its blocks** (a code decision of
  the flagship, H-HANDS-ARRIVAL).
- **D5 The frame waits for four fifths of its live members, and for nobody who stands.** One share, used for the
  wait, the tight test and the arrival.
- **D6 The layout's shape: rows of up to four abreast, the front row centred on the place.**
- **D7 The halt is read on the slots, not the members.**
- **D8 An army goes through a shared door together, its forming-up grounds laid at the plan.**
- **D9 The learned lead and every other tuning listed in §8 go with the base**, to come back on evidence.
- **D10 (2026-11-15, after the first game on the hands) A frame is seated on its body when it is founded and not
  at an order.** Chose-because: seated at every order, a body of two speeds under an order that changes every
  second did not arrive (after a seat half the members are ahead of their slots; a faster member's point is its
  slot, so it was sent back; the engine of before arrived in 30 s), and with the seat at founding alone the 21
  property rows read the same. Rejected-because, of the seat at every order: its purpose was that the frame never
  be a dead-reckoned thing apart from its units (T8); the hold of §5.6 already keeps it by them, and the seat's
  jump is itself a discontinuity in the order (P3) for every member whose point is not carried ahead. The user,
  on "Make the re-seat apply only when the army has no order" and on measuring the faster type's re-send
  distance: "Yes to both -- apply it and try another test game" (a bare assent).

## 14. As built (2026-11-15, branch `army-loop`)

The user on the design: "Agreed, let's proceed with this. Do it in a worktree so the main bot is available for lobby
games." A bare assent on D1 to D9: a checkpoint, open again at any time.

The engine is `crates/micro/src/army.rs` (the decisions, the frame, the report), `layout.rs` (the block, the layout)
and `group.rs` (the host's types alone). Where the build departs from the sections above, with the reason:

- **§5.4, the band about sibling ways is not built.** On open ground the planner straightens a way to a line whatever
  the band, and where the ground has two corridors a band sends an army's companies down different ones, so the army
  arrives from two sides. Companies set out from separate parts of the body to separate blocks; where the ground has
  one corridor they share it.
- **§5.4, a block that stands is priced at 20 a cell (`ground::BLOCK_COST`), not the old 600.** At 600 a company walked
  a thousand elmos round a block of its own army that nobody stood in yet (the first bench run: 3,320 of way for a
  straight 2,140). At 20 a block of four ranks costs about what 800 of flat ground do: walked round when the way
  round is about its own width longer, through otherwise.
- **§5.7, a re-form makes companies of compact parts of the body where it stands**, the body halved across its longer
  side until the parts are companies (`army.rs` `split`), each frame seated on its own members. The design laid the
  blocks on the lattice at the body: a column of ninety-six would have walked up to 700 sideways to line up before it
  set out, and near the map's edge a block of that lattice lay off the map. Where the companies stand apart is the
  layout's at the place.
- **§5.8, an army with no facing in force reads one**: the approach to the order's place, or with the place too near
  for one, the facing in which its members are nearest slots of their own (sixteen directions tried, then refined).
  Without it an army made again where it stood took the direction from its centre to the place, a few elmos apart,
  and re-laid everyone (the first restart drill: 48 of 48 moved).
- **§5.11, the pitch changes at a seating only** (an order's decisions, a re-form). The row's second trigger, a pitch
  wanted for ten seconds, is not built.
- **§9, the events' measure is read on the positions**: a member moved when its displacement differs from the
  median displacement by more than the band, four and ten seconds after the event. Read on the points the members
  were last sent to it counted five to fifteen with nothing happening, because the points go out at different
  moments on a march.
- **§8, the readers.** The flagship's record and the duel's per-second log carry the report's keys and, for the
  scripts and the viewer that read the old ones, `anchor` (the group's centre), `heading` (its facing), `pace` (its
  soldiers' median speed) and `way` (the way left) with their nearest meaning.
- **§4 P7, measured.** Members boxed out of free slots by the ranks before them stand short and move for no order
  (four of a hundred in one of two runs; K-group-a-member-boxed-out-of-its-slot-stands-short). They hold nothing, so
  the property stands; they are the residue of §3's A3.

**The first readings (the ledger's row base-\* to loop3-\*).** P1, P2 at rest, P3 and P5 hold where the engine of
before did not; the numbers are in `docs/heuristics.md` under H-MICRO-GROUP. Two of the decisions show their cost on
the bench, each in a fight the other choice wins (K-group-the-stands-shape-decides-what-his-turrets-do):

- **D6.** The row of four puts a company inside his turrets' reach at C5 (55 to 72 % of the army left, 95 to 98 %
  in rows of two) and takes a wide turret line in a minute where rows of two do not take it in five.
- **D3.** With no close, companies that have killed what their slots reach walk on to their blocks and stand while
  turrets beside them live.

Both are one gap: the layout is a function of the place, the facing, the companies and the ground, and the enemy is
not among its inputs (§10 said so of threat). By §11 the answer is a change of that function's inputs and a row of
§7, not a rule in the tick. It is the user's to rule; the default stays rows of four, and `WITHIN_REASON_ROW=2` is
the bench's arm.

**A third reading (2026-11-15, the ledger's row orders1-\* to alternate-mixed).** §5.8 seats every frame on its body
when an order is decided. The property drills were all of one type and passed; a body of two speeds under an order
that changes every second does not arrive (K-group-a-frame-seated-at-every-order-stalls-its-faster-type): after a
seat half the members are ahead of their slots, and a faster member, whose point is its slot itself, is sent back.
With the seat kept for a frame's founding (a re-form, a newcomer's company, a lane made again) and dropped for one
that walks an order, the body arrives as on the engine of before and the 21 rows read the same; the hold of §5.6 is
then what ties a frame to its body between decisions. Decided the same day (D10) and built: §5.5 and §5.8 are
amended above, and the bench's arm is gone with the question. §9 gains the rows `walk-mixed` and `alternate-mixed`.

**A fourth reading (2026-11-15, the ledger's rows fix-\* and loop-pianist-smoke-2).** With D10 built the 23 rows
hold and the hands played a whole game on it with no fault. Two things stand for the user:

- **The faster type's stride.** §5.6 sends a member faster than the pace its slot itself, which costs an order for
  each cell or two the slot travels on either engine (K-group-a-faster-type-costs-an-order-a-cell). Sent its slot
  carried a stride ahead and again a stride on (the bench's arm `WITHIN_REASON_STRIDE`), a body of two speeds takes
  1.10, 0.87, 0.72 and 0.56 orders a member a second at 32, 48, 64 and 96 against 1.56, with arrival, pace, depth and
  an assault's outcome unchanged. A term of the goal law by §11; not the default until he rules.
- **A party.** In the game's first clash a third of the group had anything in reach
  (K-group-a-block-meets-a-party-with-a-third-of-its-guns): the guard of §5.9 stops the frame at its nearest slot
  and the halt wants every slot. The engine of before shows the same off the positions. It belongs with D3, D6 and
  D7 above: what the army does about an enemy at or beside its place.

