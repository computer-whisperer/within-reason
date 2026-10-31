# Contact cleanup: ranks through contact, no flee in a group (2026-10-28)

The micro engine's contact behaviour is cut back to one rule a reader can follow. From airstun-1's replay
(`run/matches/1793155434-airstun-1`, the ledger row "airstun-1") and the user's reading of it: "the actual go-to-combat
action results in the formation immediately breaking once units get in range. The circle formation is probably just
hurting us at this point, and units randomly fleeing in various directions at various points makes it hard to tell
what the actual orders are at any given time"; "It is a lot harder to figure out what is wrong with a diffuse pile of
units doing unintelligible things, versus a block of units that clearly goes the wrong way or retreats at the wrong
time"; then "the 'arc' intent has been running in game as a circle reality for awhile now. When a group of units
reaches an enemy turret, they usually get ordered into a circle around the structure -- causing many units to cross
the diameter to the other side. This also tends to backfire randomly, where a group of units will move into a circle
when no enemy is there ... We should probably start by refactoring and cleaning things up. The arc behavior can be
removed entirely ... holding ranks through contact should definitely be tried."

## 0. What the code did (read 2026-10-28, HEAD 1139b10)

- **Contact began on ghosts.** A fighting group was engaged when any threat source stood within 700 of its members'
  centre (`lib.rs` `formation_tick`), and the sources are not only what is in sight: every armed building the host
  remembers, in sight or not; radar blips taken for the soldier type seen most; soldiers seen up to 10 s ago at their
  last place, fading (`threat_sources`). The group then stopped and formed on whichever of those it chose.
- **The shape was a ring.** The target was the host's (an Attack), else the one most members' claims named, else the
  nearest soldier in the open, else the nearest soldier, else the nearest building; its party the foes chained within
  150; the slots on the stadium at the body's shortest reach less 20 round it, the near half first and every further
  unit round the sides and the far side (`form.rs` `contact_slots`: the unit test asserts 8 of 25 Stouts "beyond the
  target from the body"), pitched up to 60 degrees away from other buildings' reach. A lone turret is a point, so its
  stadium is a circle and the body walked across its reach to fill it: the user's "cross the diameter".
- **The shape moved with the target.** Each change of target moved every slot; in airstun-1 the engaged target changed
  once every 14 s of contact (group_W 20 times in 196 s), and a body attacking a party had four fifths of its members
  within 1,307 of its centre against 300 on a fight-to march (`run/group_shape.py`).
- **Contact ended in a set-down.** When contact cleared the anchor was put down on the body's densest cluster and the
  slots dealt again (`Group::reanchor`), because the ring's places were not the march's; the formed and exception books
  were not kept during contact.
- **Members fled one by one.** H-MICRO-FLEE ran on members of declared groups: a member heading into an unpriced threat
  or a fight it would lose stepped to the least threatened cell within 100, for at least a second, and the group
  reported it `fled`. airstun-1: 420 member-seconds fled in 22 groups; the player cut a group's footwork to `fan` at
  17:19 "so flee doesn't drag the body in and out of turret reach".

## 1. The rule now

A declared group's shape is its march's at all times: ranks of `files` across its heading at its spacing, hung from
the anchor (`form::slots`), dealt by least travel. Contact changes one thing, the pace.

- **The halt.** A group that fights (FightTo, Attack, Hold; never a Go) stops its anchor while more than half its
  members stand (`Pacing::halt_share`, 0.5, from the bench, §3), and a member stands while an armed enemy in sight (a
  known type, not being built, not stunned; or the Attack's named target) is within its reach less `STAND_INSIDE`
  (20), kept until it is beyond reach plus `STAND_KEEP` and for `FORM_FRAMES` at least, as before. A standing member
  gets no order (it fires at will under the order it has; an order costs shots), or an Attack on the group's named
  target when that is the nearest enemy to it. Every other member walks to its slot: while the anchor walks the block
  closes on what its front is fighting, and once it stands the slots stand and the ranks hold where the block stopped.
- **The march resumes** from where the anchor stands as soon as half or fewer stand: no set-down, no re-deal, no
  target.
  Nothing out of sight, on radar only or remembered starts a halt, so nothing that is not there stops a block.
- **The books are kept through contact**: formed, exceptions, the drag (a standing member behind its slot holds the
  anchor as any member does), arrival. `engaged` in the group's state is the nearest armed enemy a standing member has
  in reach, for the record and the hands' words.
- **No flee in a declared group.** The group's members are under fan and kite (their group's footwork) but never flee;
  a retreat is the group's order, walked as a block, the hands' or the player's like any other move. Loose soldiers (in
  no declared group: the distilled bot's, a fresh unit before its group takes it) keep flee and the threat set
  (`Commitment`) it reads; a declared group is no longer priced (`set_group` takes no commitment).

What a halted block does against a point target it cannot all reach (a lone turret: a rank of six at reach has about
four units in it) is the engine's, under the orders it has; widening the files at contact, a facing toward the enemy,
and fan and kite inside a group are the next questions, asked of the bench below and the user, not built here.

## 2. Deleted, with the reason

| What | Where | Why gone |
|---|---|---|
| The contact target and its party (`Foe`, `Contact`, `contact`, `PARTY_LINK`, `STATIC_MARGIN`) | `form.rs` | The shape that needed them is gone; the halt needs no target. |
| The ring and its arcs, pitches and second ring (`Stadium`, `contact_slots`, `PITCHES`, `REACH_MARGIN`, `major_axis`) | `form.rs`; H-MICRO-FORM-ARC | The user's ruling; the circle round a lone turret. |
| The contact stances `Slot` and `Wait`, `SLOT_MOVED`, the Move-or-Fight walk to a contact slot | `lib.rs` | No contact slots. |
| H-MICRO-FORM-PURSUIT (`PURSUIT`, `LET_GO_FRAMES`, `pursuits`, `let_go`) | `lib.rs` | It let go of a running target the ring followed; a halted block follows nothing. |
| The set-down after contact (`Group::reanchor`) and the march clock's restart in it | `group.rs` | The anchor never leaves its place at contact and the group is ticked through it. |
| Flee for members of declared groups, `Walker::fleeing`, `Cause::Fled`, the picture's `footwork` line and the `fleeing` outlier word | `lib.rs`, `group.rs`, the bot's `picture.rs` | The user's ruling; nothing else claims a member for flee. |
| A declared group's commitment (`set_group`'s fifth argument, `Declared::commitment`) | `lib.rs`, the hosts | Only flee read it. |
| `flee` in the player's `lane` tool | `strategist/mcp.rs`, `player.md` | It can no longer reach a declared group's soldier. |

Rejected-because, for the ledger: the ring's measured wins (the body-shape study, 2026-09-28: 25 Stouts against 4
Fatboys +0.462 against the attack-move's -0.515; the assault slices, 2026-10-25: a loss turned into a win against two
Pop-ups, survivors doubled against Guardians) were on the bench's fixed defences and plain duels, with one target that
never moves and the contact beginning on a unit in sight. Live, the target is chosen among ghosts and moving soldiers
and the ring re-forms; the user's ruling sets legibility over those margins, and the bench below says what it costs.

## 3. The bench

Eleven arms of 8 duels on Quicksilver, lane on, ranks of six, sequential, one binary each with
`WITHIN_REASON_KEEP_INSTALLED=1` (scratchpad `contact_slice.sh`): the assault drill (mix of 6 Blitz + 4 Stout + 2 Janus
against 2 HLT + 2 Sentries flat and on the 1.3 detour, and against a Guardian + 2 Sentries; 12 Blitzes against 2
Pop-ups; 10 Stouts against 2 Guardians, and against 2 Sentries + a Beamer), the advance (the mix against its mirror,
theirs as a group and as the engine's fight), and plain duels (25 Stouts against 4 Fatboys, 21 against 5 Bulls, a
Stout mirror against the engine's fight). Before: HEAD's binary copied aside (`cbase-*`); after: this change (`chold-*`).
Read with the scratchpad `assault_summary.py` (taken, margin, survivors, lost before the first shot, first shot, ramp).

Bars, set before the run: the detour and the flat mix arms hold (taken 8/8, margin within a standard error); the
standing drills (circuits, D21 arms) are untouched by construction (no enemy), so one circuit is the regression. The
arms the ring won (Pop-ups, Guardians, Fatboys) are allowed to lose margin: the numbers go to the user with the ruling's
price, not a reason to keep the ring.

### Results (2026-10-28)

`cbase` is HEAD 1139b10 (the ring), `chold` the halt for the first member standing, `chalf` and `cnever` the same
binary with `WITHIN_REASON_PACING=1.0,3,300,128,0.5` and `...,1.0` (`run/matches/*-duel-c{base,hold,half,never}-*`).
Margin (taken / lost / timeout of 8):

| arm | ring | halt for the first | halt at a half | never halt |
|---|---|---|---|---|
| mix v 2 HLT + 2 Sentries, flat | +0.66 (8/0/0) | +0.62 (8/0/0) | +0.63 (8/0/0) | +0.63 (8/0/0) |
| mix v the same on the 1.3 detour | +0.60 (8/0/0), ramp 1.0 / 2.9 s | +0.48 (8/0/0), ramp 12.2 / 27.8 s | +0.55 (8/0/0), ramp 4.1 / 11.0 s | +0.58 (8/0/0), ramp 3.2 / 12.2 s |
| mix v a Guardian + 2 Sentries | +0.83 | +0.80 | +0.80 | +0.77 |
| 12 Blitzes v 2 Pop-ups | +0.40 (8/0/0) | +0.13 (6/2/0) | +0.10 (5/3/0) | +0.03 (4/4/0) |
| 10 Stouts v 2 Guardians | +0.88 | +0.85 | +0.81 | +0.83 |
| 10 Stouts v 2 Sentries + a Beamer | +0.85 | +0.85 | +0.85 | +0.85 |
| mix v its mirror, theirs a formation | -0.04 (3/5/0) | +0.26 (7/1/0) | +0.15 (7/1/0) | +0.17 (7/1/0) |
| mix v its mirror, theirs the engine's fight | +0.22 (7/1/0) | -0.05 (3/3/2) | -0.01 (5/3/0) | +0.06 (5/3/0) |
| 25 Stouts v 4 Fatboys | +0.46 (8/0/0) | -0.32 (0/8/0) | -0.41 (0/8/0) | -0.38 (0/8/0) |
| 21 Stouts v 5 Bulls | +0.20 (7/1/0) | -0.29 (0/8/0) | -0.26 (0/8/0) | -0.27 (0/8/0) |
| 12 Stouts v 12, the engine's fight | +0.03 | -0.01 | +0.09 | -0.00 |

Standard errors 0.01-0.02 on the assault arms, 0.04-0.09 on the duels. Read: when the block halts is worth the detour
and little else; the bars of §3 hold for a half and for never on the detour (+0.55 ± 0.02, +0.58 ± 0.01 against
+0.60 ± 0.01) and miss by 0.03-0.04 on the flat mix (units lost before their first shot 1.2 → 1.8-2.4). What the ring
won and nothing here wins back is the area-fire and point-target fights: 4 Fatboys, 5 Bulls, 2 Pop-ups. There the
ring put every unit at reach at once, round the target's far side too, walking to its slot under a Move (the body-shape
study's finding, K-form-a-concave-walked-beats-area-fire: most of the gain against area fire is the walking); ranks
stand in the shells with the rear out of reach. Formation against formation the ranks win (+0.15 to +0.26 against
-0.04); against the engine's trickle they lose (+0.22 to -0.05..+0.06). The halt at a half is the default: never is
level with it within noise, and a half keeps the anchor from walking on while most of the block fights (never halts
it only by the drag, and a standing member 300 behind its slot is away and holds nothing).

Open, for the user: a contact shape that keeps the block legible and wins back the area-fire fights (deploying the
ranks into a wider line at contact; the members out of reach walking under a Move against area fire rather than
standing), measured on the Fatboy, Bull and Pop-up arms.

## 4. The files no longer narrow to the ground (2026-10-28, after contact-1)

The user, after contact-1's three timeouts: "I think we should disable the terrain-based reforming regardless. It seems
to be most of the reason we are loosing those timeout games. You see a lot of erratic thrashing of individual unit
orders that I assume is from the terrain-based reforming logic."

**What it did (D17, 2026-10-25).** Every tick on the march the group read the room about the anchor's cell on its
ground and laid the ranks in the files that fit (`Group::fit_files`: half the room less half a hull over the spacing,
one to six, widened only with half a spacing to spare; held while the anchor was on a carry past a bend, D18). Every
change of files cleared the deal, so every member's slot was dealt afresh.

**What contact-1 showed** (`run/matches/1793221642-contact-1`, the ledger row; scratchpad instruments read once a
second, so a change and its flip back inside a second count as none):
- The files changed 9-12 times a group-minute in all four games (groups of ten or more: 9.1-11.9), once every five or
  six seconds, each a re-deal of every member.
- Groups of 20 or more stood in one or two files for 16 / 35 / 30 / 45% of their engaged seconds: by structures the
  room shrinks, so the block filed into a column where it fought (game 00 12:15-12:30 at his LLT/HLT/Beamer nest, 6 to
  3 to 1 files; game 02 36:10-36:30, 103 members in one file against a Tzar). migration-3-1 had shown the same at 24:15
  ("files to one with 46 members", the drills design's live shapes): the narrowing was sized for twelve.
- Of the members' ordered points that jumped more than 150 within 2 s (8-15 a member-minute), 19-27% fell in a second
  with a change of files; about as many with a change of membership (15-27%: a join or a death re-deals the group), a
  turn of 30 degrees or more (15-26%) and a swing of the pace (13-22%: the lead is the pace's share to the fourth power,
  so a halt pulls every point back up to 300); about half with none of these. The narrowing is one source of the
  thrash, not most of it.

**The rule now.** The ranks are `form::FILES` (six) on any ground; nothing about the ground changes the shape or deals
the slots. What the ground still does: the anchor's way is planned over it and priced by the room six files want (a
march prefers wide ground and goes through a narrow pass only when it is the way), every slot is snapped onto it (in a
corridor the outer files' slots stand on the corridor's floor beside the inner ones: the unit test
`the_ranks_keep_their_files_through_a_corridor`, twelve through a corridor 192 wide, arrive with no re-deal), a member
the ground does not join to its slot is cut off, the route's end and a set-down anchor are placed where the ranks have
room (`Ground::stand`). Deleted: `Group::fit_files`, `Group::carrying` (read only by it), the `files` and `room` of
`Group` and `GroupState` and so of the record's `groups` and the duel's per-second line, the picture's "in a column of
N" / "in ranks of N" words, the files column of `run/group_march.py`; the viewer draws a record's files only when it has
them.

Rejected-because: D17 narrowed the files so a body went through a corridor in a column rather than piling its outer
files against the walls (the detour drill, circuit13-15); live, the room is read at one cell under an anchor that walks
among structures, so the shape changed every few seconds and collapsed to a column at the fights, where a block has to
be widest. Not measured on the bench before the change; the price, if any, is on the corridor and detour drills.

## 5. The advance: ranks held, fire on the move, halt when all are in range (2026-10-28 night)

The user, watching the hold-position bench's replays (hpon / hpoff / hpmove, the ledger rows): "I don't see any of
these matching the movement logic I was looking for. I was expecting units to stay in formation and advance until all
in range of the enemy, attacking asynchronously with movement. All of these seem to be variations of either halting
once they start attacking, or charging the enemy independently, or spreading out by a different method." Then: "Let's
take a fresh look at this. Review the full group-based movement control logic and try to implement something that
actually helps this time."

**What §1 did, read again.** A member with an armed enemy in reach stood: no new order, whatever order it had kept.
Under the Fight march at manoeuvre that order drove it at targets within its reach plus 100 and chased them (the
engine's `ExecuteFight`, K-engine-a-fight-at-manoeuvre-closes-on-and-chases): the charging. The anchor halted when more
than half stood, so the rear ranks never came into reach: the halting. Fan and kite stepped members sideways and back
one by one, and the spacing widened to 160 when an area weapon came within 1,500: the spreading. Hold position (hpon)
made each unit stop at its first target; the Move march (hpmove) fired on the move but kept the stand-without-order
and the halt at a half, and an idle member at manoeuvre chased within its reach plus 200.

**The rule now.** A declared group's shape is its march's, and its members do one thing: walk to their slots.

- **Every member's order is a Move to its slot every tick**, in reach or not, re-sent as the slot moves (the march's
  cadence, D20's lead); there is no standing stance and no Attack on a named target (an Attack's route still follows
  its target's sightings). A Move never diverts; the weapons fire at will on the move at what they can hit, and the
  engine picks the target.
- **Hold position** (H-MICRO-FORM-HOLDPOS): every soldier of a formation group is put at the engine's hold position
  once (`Command::MoveState` 0) and back to manoeuvre when it leaves the group. A unit at its slot in a halted block
  shoots what is in reach and chases nothing (`MobileCAI::ExecuteFight` / `GenerateAttackCmd` read `MOVESTATE_HOLDPOS`:
  no close-in, no chase, auto-targets only what a weapon can aim at now).
- **The halt.** A fighting group (FightTo, Attack, Hold; never a Go) halts its anchor when every member of its body
  (in a slot, not away, not cut off; everyone when nobody has a slot yet) has an armed enemy in sight within its own
  reach, or when any body member has one inside `Pacing::close_to` (a half) of its reach: the second half keeps a
  block deeper than its reach from walking over a point target to bring its rear rank in. Inside by `STAND_INSIDE`
  (20) to halt, kept while within reach plus `STAND_KEEP` (20). The march resumes as soon as the condition fails (an
  enemy dead or run, a member out of reach): the block creeps on until all are in range again.
- **The close.** A FightTo or an Attack at its route's end with an armed enemy in sight within `HORIZON` (700) of its
  anchor walks its anchor at the nearest, straight, at the pace, until the halt above stops it, and at most
  `CLOSE_LEASH` (600) past the route's end; a Hold stands at its end. The route's end is placed off structures with the
  ranks' room (`Ground::stand`), 250 from the Pop-ups on the bench while a Blitz reaches 180: without the close nobody
  fires (hpon, 0 of 8 without a shot).
- **No reflex in a declared group.** Its members are under no footwork rule: no flee (§1), no fan, no kite. A member
  under a commander's D-gun stands in its file; a member that outranges what it fights does not step back. The player's
  `lane` tool keeps only its company cap. Loose soldiers (the distilled bot's) keep the three.
- **`engaged`** in the state is contact: the nearest armed enemy some member has in reach, halted or not (the picture's
  words say "halted ... to fight" at pace 0 and "closing on" while the anchor walks).
- **The spacing** against area weapons (H-MICRO-FORM-SPACING) stands: it is the block's shape, set before contact (an
  area weapon within 1,500, in sight or remembered), not a step one by one. Open with the bench.

**Deleted, with the reason.**

| What | Where | Why gone |
|---|---|---|
| The stand (`Stance::Stand`, `Stance`, `Kind`, `kind_of`, `FormOrder`, the claim's stance and kind, `holding`'s stance word) | `lib.rs` | A member is never given nothing: its slot is its order. |
| The Fight march and the Attack on the named target | `lib.rs` `formation_tick` | A Fight at manoeuvre is the engine's charge; an Attack by id leaves the ranks for its target. |
| `Pacing::halt_share` | `group.rs`, the duel's `WITHIN_REASON_PACING` fifth field | The halt is all in range or any inside `close_to`, the fifth field now. |
| A declared group's footwork (`set_group`'s fourth argument, `Declared::footwork`, `Footwork::RULES` / `kept` / `keeping` / `words`), the `lane` tool's rules, `footwork_of`, the picture's `lane` words for a formation, `Shared::lane` | `lib.rs`, the hosts, `strategist/mcp.rs`, `player.md` | No reflex reaches a member. `Footwork` stays for loose soldiers (`set_commitments`). |

Rejected-because: fan's measured wins (matt-fan3: six D-gun deaths in five shots against ten in six) and kite's
(Rocketeers against Hammers) were for loose parties; in a block they are the spreading the user named. The halt at a
half (§3) kept the rear idle; the Move march won the plain duels only by the idle chase.

**The bench**: the eleven arms of §3 on this binary against hpoff (the ledger rows), plus the close at a third
(`WITHIN_REASON_PACING=1.0,3,300,128,0.33`). Bars: the flat and detour assaults hold (8/8); the Pop-ups are taken (the
close); formation against formation holds. The area-fire duels (Fatboys, Bulls) are the price to read: a halted block
stands in the shells with its rear ranks out of reach.

### Results (2026-10-28 night)

`hpoff` is the §1 binary (the Fight march at manoeuvre, the halt at a half; ledger row hpon-*), `adv` this change
(`run/matches/*-duel-adv-*`), `adv3` the same binary with the close at a third (`WITHIN_REASON_PACING=1.0,3,300,128,0.33`).
Margin (taken / lost / timeout of 8):

| arm | hpoff | adv (close at a half) | adv3 (a third) |
|---|---|---|---|
| mix v 2 HLT + 2 Sentries, flat | +0.63 (8/0/0) | +0.57 (8/0/0) | +0.48 (7/0/1) |
| mix v the same on the 1.3 detour | +0.59 (8/0/0) | +0.48 (8/0/0) | +0.51 (8/0/0) |
| mix v a Guardian + 2 Sentries | +0.79 | +0.70 | +0.73 |
| 12 Blitzes v 2 Pop-ups | +0.11 (5/3/0) | +0.28 (6/2/0) | +0.32 (8/0/0) |
| 10 Stouts v 2 Guardians | +0.81 | +0.89 | +0.89 |
| 10 Stouts v 2 Sentries + a Beamer | +0.85 | +0.84 | +0.84 |
| mix v its mirror, theirs a formation | +0.21 (7/0/1) | -0.06 (3/4/1) | +0.02 (4/3/1) |
| mix v its mirror, theirs the engine's fight | +0.01 (3/5/0) | -0.06 (3/5/0) | -0.30 (1/7/0) |
| 25 Stouts v 4 Fatboys | -0.34 (0/8/0) | +0.38 (7/1/0) | +0.34 (8/0/0) |
| 21 Stouts v 5 Bulls | -0.25 (0/8/0) | +0.49 (8/0/0) | +0.44 (8/0/0) |
| 12 Stouts v 12, the engine's fight | +0.03 (4/4/0) | +0.28 (8/0/0) | +0.27 (8/0/0) |

Read: the area-fire duels the ring won and §1 lost are won back and more (the ring: Fatboys +0.46, Bulls +0.20),
without a ring: the block walks in ranks until all are in reach and fires on the way, and at hold position nobody
leaves the ranks. The Stout mirror against the engine's fight 8 of 8. The assaults hold their take (8/8) at 0.06-0.11
less margin, and the Pop-ups are taken more often (the close). Lost: formation against formation (+0.21 → -0.06;
both sides on this binary halt with all in reach and nobody chases what backs off: open) and the mix against the
engine's trickle within noise (-0.06 against +0.01). The close at a third takes every Pop-up arm but loses to the
trickle (-0.30) and times out one flat assault: a half stays the default.

The fire instrument's shots, muzzled and first-shot columns are blind to this march
(K-engine-the-weapon-fired-event-comes-only-under-an-attack-command): the flat assault counted 213 shots against
2,780 while dealing the same damage (55,459 against 54,128 over 8 duels). `dealt` and the margin are the judges.

### 5.1 A ground rebuild moves nothing that stands (2026-10-28 night, after the replays)

The user, watching the adv-* replays: "on many occasions the group reforms differently after some number of the enemy
units die. The corpses are obstacles and the new formation sometimes places the formation on top of those corpses."
The per-second logs (scratchpad `replan_trace.py` on the record; the duel's `seconds` lines): in the second of an enemy
death the group's `way` left jumped from 0 to 100-300 and its heading swung 47-171 degrees. The chain: a death changes
the structures on the ground (a dead turret's footprint gone, its wreck a blocked cell), the ground is rebuilt (D18),
a new ground plans the way again from the anchor (`Group::plan`), and `plan` set `leg` to 0, so an anchor that had
arrived, or had closed past its end, had a new last leg back to the route's end: it turned round, walked back, arrived,
closed again, at every death. And the tick faced the anchor toward its next way point whether it walked or not, so a
block halted in contact turned its ranks about the anchor onto the wrecks when the re-planned way's first point lay at
another bearing (formation against formation: 40 and 106 degrees with the anchor never moving).

The rule: an anchor at the way's end stays at it through a re-plan (`plan` keeps `leg` at the end when it was there;
a new order sets `leg` to 0 before it plans, so it still walks), and the anchor faces its next way point only while
the lane is not halting it (`Group::tick`). Test `a_ground_rebuild_leaves_an_arrived_a_closing_and_a_halted_anchor_as_they_are`.
Live (advance-1) 28 of the 131 turn seconds in large groups sat within 3 s of an enemy death; the other source, the
hands' re-declarations (46 route jumps with the anchor still), is the hands', next.

### 5.2 The halt is all in reach; the front's guard; an attack on a party follows its nearest (2026-10-29)

The user, reading advance-2's 18:38 (`run/matches/1793242140-advance-2`, group_V1 against his Rocketeer/Mace/Warrior
party): "Our group half fights it, half stands aside, and waffles back and forth in that mode." From the per-second
record: the original company of 13 halted at 18:23 with 11 in reach and its anchor stood 14 s while his party walked
past it west; from 18:28 the in-reach count fell 9, 7, 6, 5 of 13, 12, 11, 9, and the halt held on the second term
(his Janus and a Stout inside half a front member's reach) while a Maverick, reach 365 against the Stouts' 350, stood
20 beyond the rear's reach and killed six. The joiners' company of 14 spent 17 s with 0 in reach walking at the route's
end, the party's centre (the mean of its unit ids), pulled by the very units the first company could not reach. The
user: "It needs to be generally robust to a wide variety of situations ... The original failure of 'leaving and
returning when part of a stationary enemy group dies' needs to remain solved, but 'not following an enemy we are
supposed to be attacking, when part of it is out of range of some of our units' is also bad." Then: "Build 1 and 2,
with both drills first."

The rules (micro `formation_tick`, `Group::guard_at`, `Group::update`):
- **The halt is all in reach, alone.** A fighting block halts its anchor when every member of its body has an armed
  enemy in sight within its own reach, with the 20 hysteresis; the "any inside `close_to` of its reach" term is gone.
- **The front's guard.** Every fighting kind's anchor comes no nearer than `close_to` (a half) of the body's shortest
  reach to the nearest armed enemy in sight within the horizon, on the way as in the close: the march's step is cut to
  the room left, and at the stop the anchor is `pinned` (pace 0; free again once the guard is `PIN_SLACK` further off;
  a pinned block does not turn its ranks to face a way point, as a halted one does not, §5.1). The point target a deep
  block cannot all reach is this guard's case now.
- **An attack on a party follows its nearest.** `GroupOrder::party` carries the party's unit ids (the hands set it for
  an Engage, the duel for a plain fight); each formation's way ends at the member of the party nearest its anchor in
  its sight, sticky (`party_target`: it stays unless another is nearer by `PARTY_SWITCH`, 150, or it is out of sight),
  and at the host's centre only while none is seen; the host's route moving re-aims such a group no more.
- Tests: `a_fighting_block_walks_on_with_a_straggler_inside_half_reach_and_pins_at_the_stop`,
  `an_attack_on_a_party_aims_each_formation_at_its_nearest_member`; the §5 and §5.1 tests unchanged.

The drills, both scenario files under `run/scenarios/`, run with `--theirs track --lane on`:
- `2026-10-29-advance-2-passby-1822.json`, cut from the record at 18:22 by `run/engagement.py ... --enemies truth
  --track --track-seconds 45` (radius 1400 about (5800, 4000)): 18 of ours (13 Stouts, 4 Blitzes, a Janus) against
  his 22 (6 Stouts, 4 Rocketeers, 2 Maces, 2 Pawns, 2 Hounds, 2 Warriors, a Janus, a Blitz, a Maverick, a Flea)
  walking their true track for 45 s. The live outcome it carries: ours 0.32 of value left at 30 s and 0.19 at 60,
  his 0.40 and 0.49.
- `2026-10-29-passby-synthetic.json`: ours in three ranks facing west on the same field, his 15 (6 Stouts, 4
  Rocketeers, 2 Maces, 2 Warriors, a Maverick) on a scripted track across our front from the north-west to the
  south-west, 433 from the front rank and 556 from the rear at the nearest, over 40 s.
The reader `passby_seconds.py` (the session scratchpad) counts per second who has something in reach and who fires.

Results (ledger rows pb0-*, pb1-*, pb2-*; 8 reps; pb0 the §5.1 binary, pb1 rules one and two, pb2 with the sticky
target and the pinned block not turning):

| measure | pb0 cut | pb1 cut | pb2 cut | pb0 syn | pb1 syn | pb2 syn |
|---|---|---|---|---|---|---|
| in-reach share while engaged (mean) | 0.49 | 0.67 | 0.75 | 0.50 | 0.65 | 0.62 |
| seconds with under half in reach | 24 | 13 | 6 | 21 | 12 | 11 |
| seconds halted / engaged | 28 / 54 | 27 / 58 | 30 / 53 | 24 / 44 | 14 / 46 | 14 / 46 |
| heading turns over 30 degrees | 4.6 | 14.1 | 8.8 | 5.8 | 12.5 | 10.1 |
| unit-seconds firing | 141 | 169 | 151 | 175 | 217 | 208 |
| margin (wins) | -0.08 (0/8) | -0.02 (0/8) | -0.28 (0/8) | +0.42 (5/8) | +0.45 (8/8) | +0.46 (8/8) |
| value left at 60 s, ours / his | 0.34 / 0.40 | 0.29 / 0.33 | 0.08 / 0.36 | 0.62 / 0.28 | 0.46 / 0.06 | 0.48 / 0.07 |

The movement is what was asked: three quarters of the block with something in reach where half had, the rear no
longer standing aside, the synthetic party no longer left standing 715 from a halted block. The price is in the cut:
the block now walks into his 22 with the Hounds and Rocketeers and stays, and ends with 0.07 of its value where
standing half aside left 0.35 (the live game's own figure at 60 s was 0.19). An order to attack an outranging party
that outweighs you is now carried out in full; that it is a losing order is the player's to see (its words say so:
"it outranges us ... the approach is paid under its fire"). The first binary's hopping end (turns 14 a rep, 1-6 of 18
formed round a pinned block) cost the block-against-block arms of the regression slice; the sticky target and the
pinned block's fixed heading brought most of that back:

| arm | adv2 | pb1 | pb2 |
|---|---|---|---|
| formation against formation (on/on) | +0.14 (6/8) | -0.11 (1/8) | +0.05 (5/8) |
| block against the engine's charge (on/off) | -0.06 (adv; hpoff +0.01) | -0.18 (0/8) | -0.17 (0/8) |
| Stout mirror | +0.28 (adv) | +0.15 (7/8) | +0.19 (7/8) |
| 21 Stouts v 5 Bulls | +0.49 (adv) | +0.39 (8/8) | +0.49 (8/8) |
| 25 Stouts v 4 Fatboys | +0.47 (8/8) | +0.42 (8/8) | +0.42 (8/8) |
| the six assaults | +0.43 to +0.90 | within 0.06 of adv2/adv | within 0.02 |

Open: the walk-against-the-charge arm (-0.06 to -0.17; at contact the block stood 287 deep by 212 wide against 205
by 276, its first shot at 27 s against 20: a pinned block no longer turns to face a mob that comes round it), and
whether a block ordered at a party it cannot reach should stand its ground rather than walk in (item 3 of the
proposal, the firing line, unbuilt).

### 5.3 Formations are placed off each other (2026-10-29)

The user, after advance-2: "Many times throughout that game I saw multiple sub-groups mingling with each other and
superposing. This caused both formations' movements to be delayed and mutually blocked. We should work to incentivise
formations to separate from each other. I don't think it should be forbidden for them to overlap, but we should find
ways to have them naturally separate where possible." Measured in advance-2: 396 company-pair-seconds with two anchors
within 190 of each other over 1,482 sampled seconds; two companies of one capped group on one attack (159 s), a
follower on its ward (58 s), an idle group where another held (47 s), two groups on one party (14 s). The pieces:
a formation's place is its route's end, placed off structures by `Ground::stand` at plan time and set raw by
`retarget` every tick for an attack, a party's nearest and a follower; a follower's end is its ward's anchor exactly;
an army's companies share one route; the ground knows nothing of other formations.

The rule (`Group::keep_off`, `Footprint`, `FormCtx::footprints`): the lane snapshots every formation's footprint each
tick (the ranks' rectangle in the heading's frame, `standing` when at its end, halted or pinned). Wherever a
formation's way ends this tick, after the planner, the party or the host set it, the end is kept off the footprints
of the formations that outrank it (a standing one over a marching one, else the larger, else the name): when it lies
in one, grown by `KEEP_OFF_MARGIN` (48), it moves to the nearest free place beside the occupant, left or right in
steps of the two half widths plus the margin, else behind it; nothing when none fits, so overlap is never forbidden.
The place is kept while the host's end stays within `ROUTE_MOVED` of what it was placed for and is still free, and
given back to the planner's end when released, so nothing flickers; an anchor that had arrived on a taken end walks to
its place. A marching formation whose way crosses another is left alone. Test
`two_formations_sent_to_one_end_stand_beside_each_other`.

The drill (`duel --drill pair`, `docs/harness/duels.md`): two groups of twelve, the second 300 across the axis, in
three arms, walk to one end, follow the army, attack a party of six Stouts and two Rocketeers on the goal; scored on
the anchors' mean distance, the seconds within one footprint (250), the second group's arrival and the distance then.

Results (ledger pair0 the §5.2 binary, pair2 this rule; 8 reps):

| arm | anchors apart at arrival | footprint-overlap s | second group's arrival s | outcome |
|---|---|---|---|---|
| walk | 0 to 434 | 31 to 35 | 39 to 49 | arrived 8/8 both |
| follow | 16 to 432 | 5.5 to 0 | 44 to 54 | arrived 8/8 both |
| attack | 6 to 100 | 90 to 40 | 62 to 59 | taken 6 of 8 to 8 of 8, margin +0.80 to +0.79 |

The two half widths and the margin of two blocks of twelve are 432: the second stands exactly beside the first.
The walk's overlap on the march stays (both converge on one end until the placement parts them at it); its second
group arrives ten seconds later, which is the walk to the place beside. The single-formation arms, where the rule is
a no-op, hold: the slice within 0.06 of pb2 on every arm (on/off -0.17 to -0.11, Bulls +0.49 to +0.44, the assaults
within 0.04), the pass-by within 0.04. A first build (ledger pair1) wrote the host's raw end back over the planner's
placed end whenever nothing blocked it, undoing `Ground::stand` on every single-formation arm; fixed before pair2.

**Amended 2026-10-31 (barb-ladder4-20-1, the user: "Let's start with that first fix for the Ward's place").** "A
follower's end is its ward's anchor exactly" above described the drill, not the hands: the hands sent `Group::body`'s
place, the member at the 60th percentile of distance from the centroid, which for a declared ward counts its joiners,
so seven followers of two to five Bulls were sent to a joiner crossing the base or a Bull standing away, 400-600 from
the last point, every second, and each re-sent end re-aimed, re-planned and re-dealt them (580 of the game's 2,830
follow-point moves over 150, 54 over 400). Now a follower is the lane's as an attack is: the host names the ward
(`GroupOrder::ward`, the ward group's lane name), the lane sets the follower's end to the ward's anchor every tick
(`Footprint::of_ward`, the largest of the ward's formations; `Group::retarget`), the host's stepped point re-aims
nothing (`Group::update`, `same_ward`, the route kept as `retarget` left it), and the ward outranks its followers in
`keep_off` whatever their sizes (a larger follower kept the end and stood on its ward). The hands' point is the ward's
anchor too (`Lane::group_state`), its body only for a ward the lane has no formation of. Found with it: `keep_off`
rewrote a kept place's "placed for" end with each tick's raw end, so an end creeping under `ROUTE_MOVED` a tick never
read as moved, and a follower stood placed beside where its ward had started (test
`a_follower_keeps_to_its_wards_anchor_and_stands_beside_it`). Drills: ledger ward0 / ward1 (the follow arm as face1,
and a follower twice its ward's size).
Open: group_X4's companies stood 10-50 apart for a minute and a half because the player's company cap was 8 and two
of 7 could not merge; under this rule they stand side by side.
