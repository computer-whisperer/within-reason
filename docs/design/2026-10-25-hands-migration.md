# The hands' migration to the group engine: a proposal (2026-10-25, draft 1, for the user's ruling)

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
the words would otherwise call a following body "behind".

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

## 5. Open for the user

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
2. **Hold, Go, FightTo:** chunk 3's deletions (1, 3, 4, 5, 6 for these tasks) then the mapping; one pianist game.
3. **Attack and FightTo-at-a-party** (Engage): the retarget path live; one pianist game with raids.
4. **Follow**, by the ruling in §5.2.
5. **Shell**, by the ruling in §5.1.
6. **`routes.rs` cut** to what builders and the economy use, and the hands' `body` cut to air groups.

Each chunk: delete, wire, one pianist game on the arena target reviewed with `bar-review`, the ledger row, and the
heuristics rows for H-HANDS-GROUP-BODY, H-HANDS-STRAGGLERS, H-HANDS-STALL, H-HANDS-JOINERS retired or amended.
