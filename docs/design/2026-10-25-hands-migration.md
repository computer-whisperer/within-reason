# The hands' migration to the group engine (2026-10-25; draft 1 ruled on the same day, in build)

The micro engine's group (`docs/design/2026-10-24-formation-drills.md` §2, D14-D19; `crates/micro/src/group.rs`) is
drilled and measured. The flagship's hands declare only two kinds of group to it (Hunt and Rove, D15); every other
pianist group's soldiers are loose soldiers to the lane, ordered one by one by the hands, and no formation forms for
them. This proposes how the hands attach, what dies in them, and what the player and Jev get to read. Nothing here is
built; each section ends with what needs the user's word. The order of chunks is at the end.

**The engine's own open item, carried here (item 19, D19 rejected 2026-10-25):** the formed band and the drag measure a
member against a slot it can only trail by the engine's following distance (100-140 for a Pawn at full pace), so a
formed body reads as "behind" by chance of where its last set-down left it, and a lap of the circuit varies 190-213 s
run to run. The lead (D14's open decision) is the fix: the band measured from the body's own trail, or the slot order
placed the following distance ahead. It does not block chunk 1 (words and record) and should precede chunk 2, since
the words would otherwise call a following body "behind". **Built as D20 of the drills design (2026-10-25, after
chunk 1): the slot order placed the type's lead ahead, the lead learned on the march.**

## 1. What the hands do today that the group replaces

`crates/bot/src/brain/pianist/groups.rs`, `execute.rs`, `micro.rs`:

- **Per-unit orders, re-issued.** Each `GroupTask` is turned into one engine order per member (`rejoin_orders`,
  `keep_groups`): a Fight or Move to the task's point, an Attack on the party's unit, a Stop for a hold. The lane sees
  these as standing orders and prices each soldier against the task's commitment (`note_commitments`).
- **Joiners.** A newcomer is sent to the body's nearest member (`admit`, `nearest_of_body`), is "joining" until within
  `ADOPT_RADIUS` of the body's place, then takes the task's order; the `join_in` allowance holds newcomers beside their
  plant until enough stand (`waiting`, `release_waiting`).
- **The body's shape.** `Group::body` sorts the core toward the goal and names a front, a tail, the body's place (the
  0.6-th nearest), the arrived count within `ARRIVED` (300), and under the `stragglers` layer the farthest fifth over
  `STRUNG_OUT` (600). The picture says "its front N to go, K of M arrived, its tail S short", the joiners on the way,
  the stragglers.
- **Gather** (`gathering`): the group closes up on its front as a move. **Shell** (`shelling`): the long-reach members
  at a standoff, the rest between. **Stall** (H-HANDS-STALL): the centre's best distance to go not improving.
- **Routing** (`routes.rs`): `walker_of`, `snap_for`, `reachable_for`, `seconds_to_spot` for a group's walker (its
  slowest class) decide where a point snaps and whether it is reachable; the engine paths each unit.

## 2. The mapping

| `GroupTask` (the hands) | `GroupKind` + route (the lane) | Commitment (as today's `note_commitments`) | Note |
|---|---|---|---|
| `Hold { picked }` | `Hold`, route `[the body's place]` | `All` if committed, else `Priced { turrets: [], commander: false }` | the lane's Hold keeps the ranks where they stand; a hold ending an order is the same |
| `Move { fight: false }` | `Go`, route `[to]` | `None` | the way is the lane's (D17); the hands' point is the host's one point |
| `Move { fight: true }` | `FightTo`, route `[to]` | `All` | contact arc on the way (H-MICRO-FORM) |
| `Move` while `gathering` | dies | | §2 of the drills design ruled it: the pace and the joiners do its work; "gather" leaves the menu |
| `Move` while `shelling` | **open** (§5) | | the two-tier placement by reach has no word in the group yet |
| `Engage { target: Some(t) }` | `Attack(t)`, route `[at]` | `Priced { commander: metal >= party }` | the lane retargets as the target's sightings move (`retarget`, re-plan at 128) |
| `Engage { target: None }` | `FightTo`, route `[at]`, re-declared as `at` moves | same | the hands keep following the party (`sent` follows `at`); a re-declared route the same within 48 is taken silently |
| `Hunt` | `Hunt` (as today) | | |
| `Rove` | `Rove` (as today) | | |
| `Follow { ward }` | `Go`, route `[follow_step(ward's place)]`, re-declared as the ward moves | `Priced { turrets: [], commander: false }` | no new kind: the hands already compute the beside-point; the lane's re-plan at 128 and its silent take of a route the same within 48 make a creeping point cheap. Alternative, a `GroupKind::Follow(UnitId)` tracking a friendly as `Attack` tracks a foe: cleaner, more surface. **Open** (§5) |

Air groups stay the hands' entirely: the lane commands ground soldiers only (`is_soldier`: a movement class; the
domains design keeps air and ground apart).

Footwork: the player's rules for the group (`footwork_of`), as the loose soldiers get today; Hunt and Rove keep `raw`.

## 3. What dies in the hands when a kind is attached

Per the "delete first" rule, each chunk below removes the hands' version before the lane's is wired:

1. `rejoin_orders` and the standing re-issue in `keep_groups` for ground Move / Engage / Hold / Follow (the lane
   drops a host's per-unit order to a declared member anyway: H-MICRO-GROUP). The hands declare and re-declare the
   group each tick (`set_group` is idempotent on the same order) and issue nothing to its members.
2. The joiners' walk to the nearest member (`admit`'s `nearest_of_body`, the `joining` map's targets): a newcomer
   added to the members gets a slot and walks to it, and is `Cause::Joining` in the state until formed (the circuit
   drill: 8 of 8 joining at a median 50 s with the body on the march). `waiting` / `join_in` stay the hands': whether
   to release is a decision about the plant, not the walk.
3. The gather move (`gathering`, its menu entry, `rally`), by the ruling.
4. `Group::body` for ground groups: the front, tail, place, arrived count and stragglers are replaced by the state
   (anchor, heading, formed, exceptions with causes, arrived, legs_left). `body` remains for air groups and for the
   picture's sentence about an undeclared group (none, after the migration). The `stragglers` layer dies with it:
   an exception `Away` or `Stuck` with its seconds is the straggler, named, not a fifth by distance.
5. H-HANDS-STALL: the state's pace at 0 while not arrived, not engaged and not stopped is the stall; the words in §4.
6. `ARRIVED` (300) for ground groups: `state.arrived` (the anchor at the way's end and every non-exception in its
   slot). The picture's `reached` registers (named places the body came within 300 of) keep their rule on the
   anchor's position.
7. `routes.rs` for groups: `snap_for` and `reachable_for` on a group's destination are the lane's ground's now
   (`Ground::stand`, `Ground::way`); they stay for builders, the commander and the economy. The group state should
   say when a route's end is on no ground the body can walk to (today `way` is the straight line and the engine does
   what it can): a `way: Option<f32>` (the planned way's length, None when there is none) in `GroupState`, so the
   picture can say "no way on foot to X for this group" instead of walking the body into a cliff.

Not touched: the loss wake, the losses words, the split / detach / join moves (membership is the hands'), the
transfer tool, the hunts' and rovers' event words, the commander's handling, every builder.

## 4. The words: what the player and Jev read (the flagship's wording is the user's)

Observations only, no claims about his state (the 2026-10-17 ruling). Proposed per group, in the picture's entry
and the register line:

- On the march: "advancing to {place} in ranks of {files}, {formed} of {members} formed, {behind} behind (the farthest
  {s} s), {joining} joining, {cut_off} cut off by the ground; walking at {pace share} of its pace; {legs_left} points
  to go, {to go} on its way". `files` under six says the ground narrowed it ("in a column of {files}" when 1-2).
- Held: "held {s} s at {at}: {cause words}" when the pace is 0 and nobody arrived; the causes from the exceptions
  (behind, away, stuck, cut off). This replaces the stall warning.
- At contact: "engaged at {at}: the arc on {party}, {firing} firing" (the state's `engaged` word today).
- Arrived: "arrived at {place}, {formed} of {members} in rank" (today's "K of M arrived").
- No way: "no way on foot from here to {place} for this group" (a fact for a pick to react to; the hands do nothing).

Jev's packet: the same facts, as today's body words are given, so the attack / retreat / shell calculus the user
named reads one number set per group (formed share, pace share, exceptions) instead of a front, a tail and a fifth.

## 5. Rulings (the user, 2026-10-25, on draft 1: "Your leans are reasonable, so let's begin implementation")

1. **Shell: (b).** Two groups the hands declare from one pianist group by reach; the player's group stays one in the
   menu. Chose-because: no engine surface, the split is the hands' own data. Rejected-because: (a) loses the tiering
   the standoff is for; (c) is the shape word D15 deferred, surface for one task.
2. **Follow: the re-declared Go.** Chose-because: the hands already compute the beside-point, and the lane's re-plan
   at 128 with its silent take of a route the same within 48 makes a creeping point cheap. Rejected-because: a
   `Follow(UnitId)` kind is more surface for one task; reopen if the creeping point shows in the words as re-plans.
3. **The words as §4**, with two departures made in the build: no held duration (the state carries none; the
   exceptions' seconds are the time dimension), and the contact words name the target unit (the party is already the
   entry's `doing` line).
4. **The player's `lane` tool gains no `files` word in these chunks.**
5. **Gather dies** in chunk 2 (§3's item 3; the §2 ruling of the drills design).
6. **Measurement:** the first live measure is a pianist game against BARb hard_aggressive on Comet Catcher.

The questions as they were asked, kept for the record:

1. **Shell.** The standoff's two tiers (long reach at the standoff, the rest between) have no group word. Options:
   (a) a Go to the standoff for the whole group and let the contact arc place by reach when fired on (loses the
   tiering); (b) two declared groups the hands make from one pianist group by reach (the player's group stays one in
   the menu); (c) a `GroupOrder` shape word (`Shape::ByReach`) as D15 deferred. I lean (b): no engine surface, the
   split is the hands' own data, and the words say "its {n} long-reach soldiers at the standoff, {m} between".
2. **Follow.** The re-declared Go (no surface) or a `Follow(UnitId)` kind (clean, tracks the ward in the lane).
3. **The words' phrasing** in §4, and whether the brief's game-experience layer gets a paragraph on reading them.
4. **The player's `lane` tool**: whether it gains a `files` word (a column by choice, for a pass) now or later.
5. **Gather** dies with chunk 3, by the §2 ruling; confirm since the menu loses a move the player has used.
6. **Measurement**: the first live measure is a pianist game against BARb hard_aggressive on Comet Catcher (the arena
   target), the group line in the record and `run/view_match.py` showing it; the drill numbers do not transfer.

## 6. Chunks, in order

1. **Words and record first (no behaviour change):** `GroupState.way`; the record's per-second group line (the drill's
   per-second `group` object: anchor, heading, pace, full_pace, files, room, formed, exceptions by cause, legs_left,
   arrived, engaged) written by `recorder.rs` for every declared group, and `run/view_match.py` reading it; the
   picture's words in §4 for Hunt and Rove groups (the two declared today) as the first live reading.
2. **Hold, Go, FightTo:** §3's deletions (1, 3, 4, 5, 6 for these tasks) then the mapping; one pianist game.
3. **Attack and FightTo-at-a-party** (Engage): the retarget path live; one pianist game with raids.
4. **Follow**, by the ruling in §5.2.
5. **Shell**, by the ruling in §5.1.
6. **`routes.rs` cut** to what builders and the economy use, and the hands' `body` cut to air groups.

Each chunk: delete, wire, one pianist game on the arena target reviewed with `bar-review`, the ledger row, and the
heuristics rows for H-HANDS-GROUP-BODY, H-HANDS-STRAGGLERS, H-HANDS-STALL, H-HANDS-JOINERS retired or amended.

### Chunk 1, built 2026-10-25

- `GroupState::way` (`crates/micro/src/group.rs`): the elmos left along the planned way from the anchor, `None` when
  a leg of the route has no way on the group's ground (`Ground::way_found`, which `way` wraps; the leg is then walked
  straight at its point as before). The duel's per-second `group` carries it too.
- The record: the brain's journal carries the lane's group states each tick (`Journal::groups`), the recorder writes
  them in every sample as `groups` keyed by the lane's group name with the drill's object by unit id
  (`docs/harness/record-format.md`); the viewer draws them under the `hands` layer (the anchor, its heading at the
  pace's share, a line of the state) from `viewer/app.js` `drawRanks`.
- The words: `picture.rs` `formation_words` builds §4's sentences from a state, and the picture adds them as the
  group entry's `formation` for every group whose lane state exists.
- **A correction to the proposal:** the first live reading was to be Hunt and Rove groups, the two kinds declared
  today. The lane's `group_orders` skips both (`lib.rs`: a hunt and a rove are the lane's by other means and form no
  ranks), so they have no state, no record line and no words: nothing of chunk 1 is live until chunk 2 attaches a
  kind. The words and the record line are tested on synthetic states (`picture.rs`, `recorder.rs`); the engine's
  `way` on the walled map (`group.rs`) and in a drill run (the ledger).

### Chunk 2, the build decisions (2026-10-25, written before the deletions)

1. **Declared:** every ground group whose task is `Hold` or `Move` and that is not shelling (shell is chunk 5: a
   shelling group's soldiers stay loose soldiers with the per-unit commitment, as today). Air groups, `Engage` and
   `Follow` stay the hands' until their chunks. The members declared are the group's members less the newcomers
   `waiting` beside their plant (`join_in`): the lane drops a host's order to a declared member, and the wait is the
   hands' order.
2. **The mapping as §2,** with one addition: `Hold` carries `at`, where the lane's anchor stands for the hold (the
   ranks hang from it; the lane's `Group::new` puts a Hold's anchor at its one point, so a hold about a body walks
   nowhere). It is the anchor as it is when an arrival ends in a hold or Jev picks a hold of a declared group (the
   re-declared Hold is arrived at once; the slots are re-dealt by least travel where the body stands), the body's
   place when a group forms or the hold is picked for one not yet declared (a newcomer's own position for a group of
   one), the centre of the soldiers given when a transfer forms a group. Commitment per §2's column; footwork the
   player's (`footwork_of`). (First written as "the walk's `to`": migration-2-1 showed a hold about the hunters'
   centroid walking its anchor a body's depth onto the point.)
3. **Dies, for these groups:** the per-unit Move / Fight / Stop in `execute_group` for Hold, Go and FightTo; the
   joiners' walk (`joining` is not kept for a declared group: a newcomer admitted is a member with a slot from the
   next tick, `Cause::Joining` in the state until formed; a newcomer under `join_in` waits as today and is declared
   when released); the arrival by members (`to_go` under `ARRIVED` becomes `state.arrived`); the picture's body words
   (front, tail, "K of M arrived", reinforcements, stragglers; replaced by the `formation` line); `reached` and `met`
   measured from the anchor instead of the body's place.
4. **Dies, for every group:** the gather move (`Order::Gather`, `gathering`, the menu's `gather`, the `together`
   end of a walk) by the ruling; the stall rule H-HANDS-STALL whole (`PROGRESS_STEP`, `STALL_FRAMES`, `best_to_go`,
   `progressed`, `stall_warned`, `stalled_seconds`, the picture's `progress` line and the wake): for a declared
   group the state's pace at 0 while not arrived and not engaged is the "held" line of §4; an air group walking loses
   the stall word (it never fired for one in the reviewed games).
5. **Kept:** `Group::body` and its words for air groups and the undeclared tasks until their chunks; `waiting` and
   `join_in`; split, join (the absorbed soldiers become members: slots, not `joining`, when the body is declared),
   transfer; the loss wake; the registers' rule (`ARRIVED` on the anchor).
6. **The reading:** a declared group's `at` is the anchor's place words; its `doing` for a walk is "advancing to
   {place}" or "walking to {place}" and the `formation` line carries the rest (§4); Jev reads the same entry.
7. **The measure:** one pianist game on the arena target (Opus 5.5 medium as the player, BARb hard_aggressive, Comet
   Catcher), reviewed with `bar-review`; the record's `groups` and the viewer's ranks are the instrument; the formed
   share, the pace share and the exceptions over the game's marches are the numbers kept, against the drills'
   (a formed share near one on a straight at full pace after D20).

### Chunk 2, built 2026-10-25 (after D20)

As decided above. `Group::declared` (`groups.rs`) names the groups the lane runs; `note_commitments` (`brain/micro.rs`)
declares them with the kind, the one-point route, the player's footwork and the §2 commitment, the waiting newcomers
left out; `GroupTask::Hold` carries `at`. Gone: the per-unit Move / Fight / Stop for these groups (`execute_group`,
`hold_orders`), the joiners' walk for them (`admit`, `release_waiting`, the join move), the stall rule whole
(`PROGRESS_STEP`, `STALL_FRAMES`, `best_to_go`, `progressed`, `stall_warned`, `stalled_seconds`, the `progress` line,
the wake), the gather move whole (`Order::Gather`, `gathering`, the menu's `gather` and its words, the player's prompt
line), the body words for a declared group (its `at` is the anchor's place, its walk's `doing` is the place alone and
the `formation` line says the rest; no reinforcements or stragglers line). Arrival for a declared walk is the lane's
`arrived`, read a second after the order at the earliest (the state before that is the last task's); `reached` and
`met` are measured from the anchor. Kept as decided. Tests: a declared group's newcomer is a member with no order of
the hands'; `join_in` on a declared group waits as before and its release is a membership; the undeclared paths
(an air group in the tests) keep theirs. The pianist game: the ledger row.

**Found in the game's first minutes (migration-2-1) and fixed the same hour (the engine):** a group re-declared as
a rove kept its hold's `Group` and state in the lane (the lane skips hunts and roves, so nothing updated them), the
record and the picture read a formed, arrived hold for 27 s while the Rovers roamed 4,000 away, and the advance
picked after planned from that anchor and walked it alone with every member "away". Now a hunt or a rove drops the
formation and its state (`Lane::set_group`), and a new order for a body standing over `REJOIN_WAY` from the anchor
sets the anchor down on the body (`Group::tick`). A second reading of the same minutes: after the hunt's end at 2:32
the hold declared about the hunters' centroid did order them (eight Fights a second, the record's `cmd` rows), but
its anchor walked a body's depth from the ranks' centroid onto the point, dragging the slots and their leads off the
hunters, who read "joining" for three seconds; a Hold's point is the anchor's place now (`Group::new`), and the hands
pass the anchor when they have one (an arrival, a picked hold of a declared group).

**From the game's minutes 6-14 (the record's `groups`, read while it ran), three more engine changes:** Jev alternated
`go_spot_63` and `fight_to_spot_63` every second for a minute (the pick rework's alternation among like-rated courses),
and each re-declaration re-dealt and re-planned the body; a walk and an advance on the same route are one march now
(`Group::update`). A set-down among the base's buildings put the anchor on a cell with 16 of room and the eleven
soldiers squeezed to a column of one at a pace of 15; a set-down anchor stands on the nearest cell with the ranks'
room (`Ground::stand`). Arrived read true with every member an exception; it needs a member in the body. Seen and
left: the hands' groups join and split constantly (sixty `join_group_G` and `send_N` picks in eight minutes), so the
body G carried three to five joiners from across the map at all times; that is the pick's, not the lane's.

**The game (migration-2-1): WON 20:32.** The record's `groups` is the instrument: 52 groups declared; the main body
marched 372 s at a formed share of 0.28 and a pace share of 0.82 (the drills' straights 0.93-0.96), its exceptions
mostly joiners (4,675 member-seconds) and away (1,780). The review read 55% of the joining as whole-body re-forms after
an attack or a join: an attacking group is undeclared in this chunk, so the lane drops its formation and the hold or
walk after it starts a fresh group whose every member is a joiner until formed (about 12 s each time); chunk 3 removes
that boundary. The rest is the picks' churn (join 66, send 56 in twenty minutes: a join or a split every nine seconds),
which carried joiners from across the map. The march itself held where the body was together (group_C 0.61 at 0.91 of
pace; the small groups that stayed together 0.61-0.70; the first minutes' holds formed to the member); the median
formed share at contact was 0.46 against the drills' 0.94. The five engine
fixes above are not in this game's binary; the next game measures them. The ledger row carries the numbers; the
review's findings are its amendment.
