# Rove approval: a group on a course leaves it to rove only on a pick

**Status.** Proposed 2026-10-07 after human-26 1:57. The decision row for the switch (source `lane`, kind `group`)
is built (0e48a29). The pending-approval mechanism below was superseded the same evening by the user's question
("if we already have a scout mechanic, should we just lean into using that instead?") and ruling: "rove being the one
odd low level control action that allows the player to bypass jev on a unit control operation seems like it may be
a standing flaw. Go ahead and see how it plays out." Built as §"The build": the lane's `rove` is deleted, the whole
group roves on the hands' `rove` move (a pick, beside `scout`), any lane setting takes a roving group back. The
first game: rove-move-1 (docs/experiments.md): won 23:52, the move offered at 1,905 gates and never picked. The words
were the cause: replayed offline (docs/studies/2026-10-07-rove-words.md) the recorded words rated 0.15 median where
the packet asked for roving and the `plain` words 0.65, with no lift where it did not ask; the plain words are in
the menu since. May be rolled back.

## The rework (2026-10-08): roving is a task, the way back is a pick

Human-27 1:43-2:34 (the first Rascals' 50 s at spot_18 while the packet asked for roving) showed the 10-07 build's
shape as lingering complexity: two moves for one behaviour (`scout` splitting one soldier into a fresh lettered
group flagged roving on the spot, needing two soldiers; `rove` setting a wish flag the next look turns into a state,
a `lane` decision row and a reset to a hold), a roving group with no menu, and the way back a counter on the
player's footwork tool, so Jev could put a group on rove but never take it back. The user, 2026-10-08: "lingering
complexity on the scout/rove system that needs to be weeded out of the hands menus"; the rework below agreed, the
check run first (`docs/studies/2026-10-08-roving-menu.md`: a roving group with a menu is not pulled off its rove
under the recorded packets, 0 of 48 gates, and a take-back paragraph brings it home at every gate, 48 of 48).

- **Roving is a group task.** `GroupTask::Rove { since }` beside hold, move, engage, hunt and follow. The pick sets
  it in `execute` like every task; the micro maps it to the rove commitment as it maps a hunt. `wants_rove`,
  `roving`, `lane_set_at`, the deferred switch in `keep_groups` and the `lane` decision row are deleted: the pick's
  own `plan` row is the record (the record format's `lane` row is retired, readers keep it for the records of
  2026-10-07). `rove_log` stays: the lane's findings for the picture.
- **One move family.** `rove` is the whole group; `scout` is renamed `rove_1`, in the detachment ladder's shape and
  words ("1 corfav of group_A (its fastest) leaves it as a group of its own and roves: ..."), the roving words shared
  with `rove`. A group of one has `rove` and needs nothing else. The lane tool's refusal names both.
- **A roving group keeps a menu**: its `stay` line says what its soldiers are looking at; `hold`, the walks, the
  joins and the follows are on it; no party-aimed move (attack, shell, send): its sightings stay the lane's
  business (its hits and sightings are no events, as before). The `news` layer closes it like any actor with a
  course, and the party in its entry is not news for it, so it is asked only on a packet or list change, a course
  change, and the re-ask interval. The lane tool is footwork only: no counter, no take-back.
- **Words.** The entry's `doing` reads "roving: its Rascal at ... is on its way to look at spot_62 ..." (no "in
  code, beyond your hands' reach"); the entry's `lane` line says "a pick of yours brings them back". The brief says
  the way back is a pick: name the group's new job in the packet.

Rejected: keeping `scout` as a no-menu group (the player-9 pull-home it guarded against is answered by the news
layer and the check above, not by hiding the group); the lane counter as the way back (a bypass in the other
direction, and a footwork tool doing control).

## The build (2026-10-07, the ruling; superseded by the rework above)

- `lane` refuses `rove`. Roving is the group menu's `rove` move (`Order::Rove`, allowed for a group of one, every
  ground group) beside `scout` (one soldier, a group of two or more). The move's words carry the fight it is near,
  what it leaves, the undo and the turn-cost words, like every move.
- The pick sets `Group::wants_rove`; the next look switches the group (task to a hold, the lane takes the soldiers,
  the `lane` decision row and the `recent` line). `scout` groups are made wanting and roving at once, as before.
- The way back: the `lane` tool counts its settings per name (`Shared::lane_set`); a roving group whose count has
  grown since it was picked (`lane_set_at`) stops roving and holds where each soldier stands. So `on`, `raw` or a
  rule list given after the pick takes it back; one given before does not.
- What scout alone could not do, and why the move is whole-group: one soldier a pick, never the last of a group,
  never a group of one (the first Rover out of the plant); the players used `rove` on whole groups of two to four.
- What the player loses: certainty. The packet says which group roves and when; Jev weighs it against the group's
  course. Jev picked `scout` on its own twelve times in the recorded games, so the move is one it takes.
- Not built: the pending flag, the urgent event, the force word, the tool's "asked once" reply.

## The superseded mechanism (kept for the record)

## The case

human-26, seat 1 (team 3, the south-east base). An enemy Rover shows at the G8 extractor at 1:51. Jev sends the Rover
group after it at 1:54, the commander at 1:55, a second Rover joins at 1:56. The player, woken at 1:51 by the raid,
reads one radar dot, decides the commander handles it, and sends `lane group_A_t3: rove` to find his base; the turn
takes 9.3 s of realtime and the setting lands at 1:57 (frame 3510). The switch resets the group's task to a hold and
hands both Rovers to the lane whole: one evades the armed raider (rove never stands in a reach, whatever the odds),
the other is sent 4,190 elmos north to look at spot_12. The G8 extractor dies at 2:03; the commander kills the raider
at 2:06. The record had no decision row for the switch; the pianist's log showed it only as `rove` lines.

The same shape in the other recorded games (`run/rove_switches.py` over every v3 Jev log: 51 switches into
roving, 12 of them the hands' own `scout` pick): of the 39 the player set with `lane`, 16 were on an idle hold and 23
on a course (12 following, 8 walking or advancing, 3 attacking a party). The three attacks were chases of a raider
the group could not catch (going-opus-1 12:51 and 12:57, same-opus-1 11:19), and the player said why each time
("standing with no orders", "stalled under march"). So the switch mid-course is common and usually deliberate, and
it always lands on a picture older than the turn that made it.

## The principle it serves

One decider (2026-09-25/26): the player gives standing instructions, Jev picks each actor's next action from a menu
the hands compose, code executes. `lane` and `queue` are the two bypasses: an order from the player that code carries
out with no pick. For a group with nothing to do the bypass is harmless. For a group on a course it is a decision
taken over Jev's head on a stale picture, and it is the only path by which an engaged group leaves its engagement
without a pick. The ask: a unit that is part of a current action needs Jev's approval once before it roves.

## The mechanism (proposed)

The lane's `rove` becomes a move on the group's menu while the group has a course, and stays a switch when it has none.

1. `roves(group)` splits into *wanted* (the lane says `rove`, or the group was made by `scout`) and *roving* (in
   force). In `keep_groups`: wanted and not roving and the group idle (`Hold { picked: false }`), or made by `scout`:
   switch now, as today. Wanted and not roving and the group on any other task (a walk, an advance, an attack, a
   hunt, a follow, a picked hold): `group.rove_pending = Some(frame)`, and an event `group_A's lane is set to rove`
   that counts as urgent and as the actor's news, so the next gate asks it and every gate asks it until it is picked.
   Not wanted and roving: switch back now (a hold, asked as idle), unchanged.
2. The group's menu carries one more move while pending, `group_A.rove`, with the menu's own words for leaving:
   "group_A leaves its attack on party_1 to rove, as the player's lane sets: from then on its soldiers look at what we
   know least, kill what they find unguarded and keep out of every reach, in code, beyond your hands' reach, until
   the lane is set otherwise; leaves party_1; this undoes its last pick; [the turn-cost words]". The mirror of
   `scout`, which already sends one soldier to rove on a pick. No decode read (as `scout`); the `same` layer applies.
3. `execute_group` for `Order::Rove` sets `group.roving = true`, ends a hunt as the switch does today, and the
   `plan` row records the pick; the `lane` row records the switch on the next look. Approval is once: after the
   pick the lane owns the group. A later `on` then `rove` while on a course asks again.
4. What the player sees: the `lane` tool's reply says, for a group with a course, "group_A is attacking party_1: your
   hands are asked once whether it leaves that to rove"; the group's entry carries `lane: "set to rove: your hands are
   asked whether it leaves its attack on party_1; not picked yet (12 s)"`; the report's "what your hands did" carries
   the pick and the switch. The packet is where the player says why ("Rovers rove unless a raider is at our
   buildings"); Jev reads it beside the move, which is the lever: the order meets the current picture through Jev.

## Ways to regret it, and what the design does about each

- **A pending rove Jev never picks.** No time cap and no default (the no-live-heuristics rule). The player is told
  "not picked yet" every report and can rewrite the packet or take the group back. Decision for the user: whether
  the tool keeps a force word (`"rove_now"`) for the player when it means it. Recommendation: not at first; count
  unpicked pendings over the next games (the two rows make it countable) and add the word if they are many.
- **Jev refuses to end a futile chase** (the three recorded attack cases). The move's words carry the course's own
  diagnosis ("the party outruns it: it reaches the party only where the party stands still", the stall words), and
  the packet carries the player's reason. If Jev still keeps the chase, that is a words problem, found in the log.
- **A separate yes/no question instead of a move.** Rejected: it needs its own bar (a new threshold), its own place
  in the gate's log and the viewer, and a parallel path into execution. The move rides the pick, the worlds, the
  split, the `plan` row, the thrash words and the viewer as they are.
- **The `openers` layer hides the move**: an attacking group's `.change` may say no and its moves are then not
  asked for up to `RE_ASK` (20 s). Hence the pending flag counts as the actor's news at every gate until picked.
- **A half state in the micro engine.** While pending the group's `Commitment` stays its task's; `Rove` only after
  the pick. Nothing of the lane changes.
- **Cost.** One more move on one menu and that actor asked each gate while pending; bounded by the player's action.
- **Hunt detachments** (group_Q1, group_S1): the setting is by name, so a pending rove on a hunting group says "leaves
  its hunt of party_57"; the pick ends the hunt as a hold does.

## Not proposed

A reason field on the `lane` call (`{"group_A": {"rove": "to find his base"}}`): the packet is the place for reasons,
and a second place for them is the pattern we keep paying for.

## What it touches

`B/micro.rs` `roves` (split into wanted and in force), `P/groups.rs` `keep_groups` (the pending flag, the event, the
switch for an idle group), `P/menu.rs` (the move and its words), `P/execute.rs` (`Order::Rove`), `P/picture.rs` (the
`lane` entry's words), `strategist/mcp.rs` (the tool's reply), `docs/heuristics.md` H-MICRO-ROVE and H-HANDS-LANE,
`docs/code-decisions.md` §2 (the switch for an idle group stays a code decision; the switch for a group on a course
leaves the table), `docs/harness/record-format.md` (unchanged: the pick is a `plan` row, the switch a `lane` row).
