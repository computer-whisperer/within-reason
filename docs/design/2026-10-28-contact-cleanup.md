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
