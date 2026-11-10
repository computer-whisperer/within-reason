# Restricted ground: the formation relaxes, the units push through (2026-11-10)

The user, after the Apex v0.5.07 game (`1794285488-apex-v0507-comet-sym0-opus-1`): "One thing to make sure we have a
good handling for is when the group is moving through an area where the formation cannot hold. The base is one of
those, since the tight channels look more like an ant farm now. There are also times where our units pass close to
terrain and many slots map to [un]reachable (or worse, reachable but via a long detour) terrain."

I proposed a column laid on the way for those stretches. The user: "I would lean more towards a relaxed posture than a
rigid column here. The formation drives value in consistent unit control when out on the field and in a fight. When
dealing with restricted area, usually the best thing the units can do is push through it as fast as they can. The
column also can't really handle a curving channel all that well. One failure mode to reiterate from an earlier cycle:
when we had an army of a hundred units that tried to pass through some rocks. The column was ordered as ~5 wide and
~20 deep, causing many units to travel a long distance to simply align with their slot." On the design below:
"agreed, proceed."

## 0. What the records show and what the lane did (read 2026-11-10, HEAD 90cd29fc)

`run/group_ground.py <match seat dir>`: for every second the largest marching company of eight or more of a declared
group was on the march, the six-file ranks laid at the recorded anchor on the recorded heading (the record keeps the
travel's heading, not the ranks' facing, so the slots are approximate) against the lane's ground (the members'
strictest class over the terrain, less every structure by its footprint), and the pace as a share of full.

| Where | Slots off the ground | v0.5.07 game (hull pitch) | v0.5.04 game (64 x 96) |
|---|---|---|---|
| Within 1,500 of a plant of ours | none | 0.81 (57 s) | 0.93 (51 s) |
| | up to a quarter | 0.64 (39 s) | 0.84 (20 s) |
| | over a quarter | 0.38 (32 s) | 0.60 (37 s) |
| Further out | none | 0.93 (188 s) | 0.93 (784 s) |
| | up to a quarter | 0.75 (123 s) | 0.92 (198 s) |

A slot with no ground within the snap's reach is rare (0.1 % of slots or less in three games); the lane's own cut-off
exception is 0.9-1.5 % of member-seconds. My report of the v0.5.07 game put the crawl by the plant on fresh units
joining; with every slot on the ground the pace there was 0.81 against 0.93 further out, so the joining is the
smaller part.

What the lane did with such ground:
- The shape never changes: six files on any ground (the contact cleanup §4, 2026-10-28, removed the narrowing by the
  room at the anchor's cell: the file count changed every five or six seconds, each change a re-deal of every member,
  and a block filed into one column where it fought).
- The block is rigid: the rear ranks hang straight behind the anchor on the held facing, so on a way that winds
  through a base the rear ranks sweep across the buildings the way went round.
- A slot on a structure or a cliff is moved to the nearest standable cell within six cells (`Ground::snap`), by
  straight distance, with no test that the cell joins the body's ground: behind a wall, on a cliff's other level.
- A member whose walk to its slot is over twice the straight line is cut off (D17): it holds the anchor by nothing,
  and is still sent to that slot.

## 1. The rule

**Nothing on restricted ground sends a unit to align.** No unit walks away from where the group is going to take a
place in a shape (the hundred through the rocks; the column I proposed was the same fault at the cap's size).

A formation has two postures on the march:

- **Ranks**, as before, wherever the ranks stand.
- **Relaxed**, across a stretch of the way where they do not: every member is sent once, by a Move, to its place on
  the forming-up ground beyond the stretch, and the engine paths it. No slot is kept on the way, nobody waits for
  anybody, no member is away, behind or stuck against a slot it was never to hold. The anchor stands on the
  forming-up ground; the group walks on in ranks when its body is there.

### 1.1 Which stretches (decided when the way is planned)

After `Group::plan`, the way is sampled every `RELAX_STEP` (32). A sample is **tight** when one rank laid across the
way there does not fit (`Ground::rank_fits`): closed up to the ground (`Ground::fit_rank`: each slot on its centre's
side of every wall, the half rank moved in toward the centre a hull apart), two slots stand nearer than a hull, or
the centre is off the ground. Every rank has to come through the place the front rank does not fit, so the rank is
the measure, not the block. Six Stouts abreast want about 280 of ground: a corridor of 192 is tight, an obstacle beside the way that leaves
the half rank room to close up is not. Tight samples nearer than the ranks' depth plus 96 (192 at the least) are one
stretch; a wall's door is one sample and a stretch. The stretches are the plan's, found again only with the plan or
when the members' count or the pitch changes: nothing reads the ground under the anchor tick by tick (October's
narrowing), and a stretch is passed once (`Group::passed`).

(What I told the user before building, "the records suggest starting at a quarter of the slots off the ground", did
not survive the first build: a quarter of the block's slots off the ground was reached only with the front rank a
rank's depth into the lanes, nine seconds of crawl on the yard drill (rg1). Whether a rank can stand a hull apart
between the walls is the question itself. A least length for a stretch, which I also named, is gone with it: a door in a thin wall is a place no rank passes however short it is, and the brush of
one building no longer reads as tight.)

The forming-up ground of a stretch is where the rear rank stands a rank and a footprint's margin past it (the mouth
stays open behind the block), or the first sample within 192 on from there where every slot of the block stands
joined. A stretch that reaches the way's end forms up at the end.

### 1.2 Entering, and what holds the ranks

The posture relaxes when the anchor comes within a step of a stretch (or is planned inside one: a group standing in
the base), the group is on the march (not halted, pinned or at its end), and:
- a walk (`Go`) always relaxes;
- a fighting kind (`FightTo`, `Attack`, `Hold`) relaxes only when nothing hostile lies within `RELAX_CLEAR` (1,000)
  of the way from the anchor to the forming-up ground: no armed enemy in sight, no building of theirs seen or
  remembered. His base, a turret among wrecks, the end of an attack are restricted ground where the ranks hold as
  before: the formation's value is in the fight, and October's narrowing failed exactly there.

Once relaxed the pass is finished whatever appears: the body cannot form where it is, and a Move fires at will on
the way.

### 1.3 The pass

- The anchor is set on the forming-up ground, facing along the way there; pace 0.
- The slots are the ranks there, dealt by arrival (`Group::deal_by_arrival`): the member with the least time to go
  takes the front rank, so the first out drives furthest and the next forms behind it. Nobody is dealt a slot by
  least travel across the stretch (that deal gives the nearest unit the rear rank and the last the front).
- Each member's order is its slot, once. A member the engine has stopped short (K-engine-a-blocked-move-ends-short)
  is sent again after `Pacing::idle_after` while it stands further than the formed band from its slot
  (`Lane::take_slot`): in ranks a halted anchor's slots go out once and that stays.
- No drag, no away, no cut-off, no stuck against a slot. A member not through that has not moved `WAIT_MOVED` for
  `Pacing::stuck_after` is given up (away: the state says it, the host decides).
- A member is through (`Group::through`) within the formed band of its slot, or within the away distance of it with
  no wall between them (a door leaves every member within the away distance of its slot before it has moved).
- The pass is waited for by the body as the ranks counted it when the pass began (`Group::pass_body`; everyone near
  the anchor when a group is planned inside a stretch and has no books yet). A joiner still on its way to the group,
  or added since, is sent to its place beyond the stretch like everyone, is said to be joining, and holds nothing,
  as in ranks (D22). The first build waited for every member: a lap of Quicksilver with a feed from the lab cost a
  company of Bulls 4-8 s and its joiners eleven (rg6fc-bull12).
- The pass ends when every member of that body is through or given up, and somebody is through. The slots are then
  dealt by least travel where the members stand (each takes the slot nearest it) and the march goes on in ranks from
  the forming-up ground.
- A new order ends the pass like any march, and the anchor is re-aimed onto the densest cluster of the body whoever
  stands by the forming-up ground (`pass_broken`).
- The forming-up ground is kept off other formations (`Group::place_forming`): when the block as it will stand
  reaches into the footprint of a formation that stands, or of one forming up that outranks this one (the larger,
  else the name: an army's companies come to one door within a second of each other), the anchor goes on along the
  way to the first place where the block stands whole and off every such footprint. An army's companies so form up
  one ahead of the other on the way they are all going. A formation forming up is priced in the others' ways as a
  marching one (`Footprint::forming`), not a standing one.

Two wrong turns on the bench, kept for the next reader:
- The forming-up ground beside the formation that held it, as a way's end is kept off (`keep_off`): three steps of a
  block's width along a wall put a company 1,300 from the door it came through (rg4).
- A formation forming up priced as a standing one: the block behind a wall's door closed the door in the price, and
  the companies behind it planned round the wall's end, 1,400 off (rg3).

### 1.4 What the state says

`GroupState::passing`: the members through while the posture is relaxed, `None` in ranks. A member of the pass's
body on its way through is reported `Behind` (walking in), one given up `Stuck` or `Away`, a joiner `Joining`. The hands' picture says "passing tight
ground, N of M through"; the record's `groups` carries `passing`.

## 2. Ground that is only partly restricted

Where a rank still fits (a cliff along one side that leaves the half rank room) the ranks hold, and one fault is
repaired:

- **A cut-off member is sent along the way behind the body** (the way's point its rank's depth behind the anchor),
  not to its slot. Sent to the slot the ground cut it off from, it set off on the detour (the user's "or worse,
  reachable but via a long detour").

**Not built: the slots moved onto the body's side of the walls.** I told the user a slot off the ground would slide
inward along its own rank, so it never lands across a wall or on a cliff's other level. Two forms of it were built
and measured against the rule in force (the nearest standable cell within six, by straight distance) with the
posture off, so the ranks walked the tight ground, and both were slower among a base's buildings:

| Ranks through the yard (first / median / last arrival of the 2,000 walk) | nearest cell | every rank closed up to its centre | a slot off the ground slid in |
|---|---|---|---|
| Tier-2 company of 24 | 42-51 / 44-53 / 46-55 | 62-71 / 66-73 / 68-75 | 39-47 / 42-50 / 44-52 |
| 24 Stouts | 29-36 / 32-37 / 33-38 | 43-54 / 45-56 / 47-58 | 35-41 / 38-42 / 40-44, one of four unfinished |
| 12 Bulls | 32-46 / 34-47 / 35-49 | 51-54 / 52-56 / 55-58 | 34-39 / 36-42 / 39-46 |
| 24 Stouts through the wall's door | 26-30 / 28-32 / 30-33 | 25-27 / 27-29 / 29-31 | 25-28 / 28-30 / 30-32 |
| Bulls' lap of Quicksilver (posture on) | 277-280 | 278-287 | 276-282 |

The reason is in the yard's shape: the outer files' own places are ground, in the lanes beside the one the centre
walks, and the nearest cell leaves a body several lanes abreast; moved to the centre's side of the first wall, every
file is in one lane (K-group-a-rank-closed-up-to-its-centre-makes-a-yard-one-lane). The rank closed up is kept as
the measure of whether a rank passes (`Ground::rank_fits`, §1.1) and places no slot. A slot that lands across a wall
is then the cut-off member's case, above.

No drill of the bench cuts a member off (two fences beside the way gave none), so the cut-off member's order stands
on its unit test and on the laps, which are level with it.

## 3. The bench

No drill had tight ground on the way. Three arms of the walk drill, on Full Metal Plate (flat, so the structures are
the only ground):

- `--yard NAME@PITCH,REACH`: our structures on a lattice of `PITCH` covering `REACH` elmos round the army's start,
  the army's own standing ground left clear: the walk out of a base. Lanes of `PITCH` less the structure's side.
- `--wall NAME@ALONG,GAP,HALF`: a wall of our structures across the axis `ALONG` from the start, `HALF` to either
  side, with a gap of `GAP` in the middle: the pass. With `--cap 24` and a hundred units, the user's rocks.
- `--fence NAME@A1,C1,A2,C2` (any number): a straight line of them; two beside the way are a corridor.

Read by `run/drill_cross.py ALONG <batch dir>` (the second the first / median / last unit was `ALONG` past the
start: the pass itself) and by the arrivals, against the lane off (a plain engine order, the floor) and the ranks of
before. The open-ground rows (Pawns, Stouts, Bulls, Marauders, Thors, the mix; the Quicksilver laps) are the check
that nothing else moved. `WITHIN_REASON_RELAX=0` keeps the ranks on any ground.

## 4. Chose / rejected

- Chose the relaxed posture (the user). Rejected-because, the column on the way: it paces the slowest type through
  the place where speed is the point, only its centre file follows a curve, and filing in is the alignment walk the
  rule forbids.
- Rejected-because, one company through at a time: orderly, and the opposite of pushing through.
- Rejected-because, relaxing wherever the ground is tight: his base is tight ground, and units would enter it in
  order of speed.
- Rejected-because, the slots moved onto the body's side of the walls (§2): measured slower among a base's buildings
  in both forms; the nearest standable cell stays, and the member the ground cuts off from its slot follows the way.

## 5. Results

Full Metal Plate 1.7, four duels a row, the committed build's `duel` (ledger rg0-* the lane of before and the lane
off, rgZ-* the build; rg1-rg6, rgF-rgJ the builds on the way). Seconds from the first order, first / median / last
unit, as the range over the duels.

**Out of the yard** (solars on a lattice of 160, lanes of 80, 640 round the start; out is 900 on, 260 past the
yard), and the whole walk of 2,000:

| | plain engine order | ranks, before | now |
|---|---|---|---|
| 24 Stouts, out | 14 / 17 / 19-20 | 17-24 / 20-27 / 23-38 | 14 / 17 / 19 |
| 24 Stouts, arrived | 25 / 28-29 / 31 | 29-43 / 31-45 / 33-47 | 26 / 27-28 / 28-29 |
| tier-2 company of 24, out | 12 / 15-16 / 26-27 | 24-28 / 30-32 / 34-38 | 12 / 16 / 26-28 |
| tier-2 company, arrived | 21-22 / 26-28 / 42-43 | 43-47 / 45-49 / 47-51 | 34-36 / 36-39 / 39-41 |
| 12 Bulls, out | 16 / 18-20 / 19-29 | 20-26 / 23-29 / 24-31 | 16 / 18 / 19 |
| 12 Bulls, arrived | 30 / 32-34 / 34-43 | 34-41 / 37-43 / 38-45 | 30-31 / 32 / 34 |
| tier-2 company, lanes of 64 (wind turbines at 112), out | 11-12 / 15-17 / 26-29 | 20-25 / 25-31 / 29-35 | 13-14 / 21-23 / 26-28 |
| tier-2 company, lanes of 128 (solars at 208), out | 12 / 14-15 / 26 | 26-30 / 30-35 / 33-39 | 13 / 17 / 26-28 |

The units are out of the yard when the engine alone would have them out. The mixed company then arrives together at
39-41, where the slowest type alone needs 41 and the ranks of before brought everyone at 47-51.

**Through a wall's door of 160** (dragon's teeth across the axis):

| | plain engine order | ranks, before | now |
|---|---|---|---|
| 24 Stouts, 200 past the wall | 13 / 16-17 / 19 | 14 / 18-21 / 22-26 | 14 / 17-18 / 20 |
| 100 Stouts in five companies, the wall 900 on, 200 past it | 16 / 21 / 28 | 24-26 / 33-35 / 41-43 | 18-24 / 28-30 / 36-38 |
| 100 of tier 2 in five companies, the same | 14-15 / 18-19 / 33 | 24-32 / 41-44 / 50-54 | 25-32 / 34-37 / 45-51 |
| 100 Stouts, the wall 2,000 on (the companies formed and marching when they come to it): 200 before it | 25-26 / 29-30 / 34-35 | 36-40 / 43-46 / 48-50 | 32-40 / 42-45 / 46-49 |
| the same, 200 past it | 30-31 / 35-36 / 41 | 41-49 / 54-58 / 62-69 | 38-47 / 49-52 / 57-59 |
| the same, 2,000 past it | 54 / 59-60 / 65-67 | 66-87 / 91-95 / 99-104 | 62-80 / 78-86 / 83-95 |

The door costs the last of a hundred 9-12 s (from 200 before it to 200 past it) where it cost 14-21 and costs the
engine's own order 6-7. What is left between the hundred and the engine's order is not the door: the five companies
are 12-14 s behind before they reach it, and at the goal they do not settle (below).

**What the bench did not move.** The open ground: Pawns 21 / 22 / 23, Stouts 25 / 27 / 27-28, Bulls 30 / 31-32 /
32-33, Starlights 46 / 48-49 / 50, Marauders 20 / 20-21 / 21, Thors 35-36 / 37 / 38, the tier-3 mix 41-47 / 44-51 /
48-54, each as on 2026-11-09 to the second (rg6f-*; no stretch on that ground, so no pass). The laps of Quicksilver
with a feed, four duels a row, the median unit's lap in seconds. From the gantry, the ranks (the posture off) and the
posture on the one build, after the route's count was fixed (below): twelve Marauders 174-182 and 174-193, the
tier-3 mix of sixteen 431-455 and 435-469, the mix with a Thor 458-486 and 443-469, eight Thors 440-466 and 406-443
(a rank of six Thors does not fit Quicksilver's narrow places: six passes a lap, 1-15 s each; no rank of
Bulls or smaller is tight anywhere on the loop). From their own plants: Pawns 188-193 (180-194 on 2026-11-09),
Stouts 226-238 (219-225), Bulls 275-289 (277-280), which is what four laps tell and no more: the same code gave
Stouts 220-227 and Bulls 277-280 an hour earlier (rgHc-fit0).

**A fault the laps turned up, fixed** (K-group-a-second-re-plan-began-the-route-again): on a route of several points
a second re-plan with no point passed since the first began the route again. The flagship and the bots declare
one-point routes; the circuit drill's has five, and every gantry lap since 2026-11-09 was walked on a route that
restarted when the first fed unit appeared (the Thor laps of that day, 390-468 s, among them; on the fixed build
the ranks lap 440-466). `Group::way_skips`.

**Open, found on the way** (K-group-five-companies-told-to-one-goal-do-not-settle): a hundred in five companies told
to one goal time out on open ground before and after (four duels each: 28-37 / 41-49 / 71-95 with 29 members left
behind, and 30-37 / 42-46 / 72-250 with 25), and form their five sets of ranks out of one block before they march,
which is the 8-10 s they are behind the engine's order at the first 900. The posture does not touch either.

**A canyon** (a wall with a door of 288 and two fences on from its edges: a tube 288 wide and 1,200 long, 250 on
from the start; arrivals of the 2,000 walk):

| | plain engine order | ranks, before | now |
|---|---|---|---|
| the wall too wide to go round (3,200 either side): tier-2 company of 24 | 21 / 27 / 42 | 37-48 / 40-52 / 43-54 | 23-24 / 28-32 / 39-40 |
| the same, 24 Stouts (a rank of six nearly fits 288, and is not tight) | 25 / 28 / 31 | 26-31 / 29-33 / 30-35 | 28-30 / 30-34 / 32-36 |
| the wall 1,400 either side, a way round of about 3,700: tier-2 company | 21 / 27 / 41-42 | 80-83 / 84-87 / 86-89 | 75-90 / 81-95 / 83-97 |
| the same, 24 Stouts | 25 / 28 / 31 | 55-56 / 58-59 / 59-60 | 55-57 / 58-60 / 59-62 |

Where the canyon is the way the company passes it relaxed and arrives when the engine's order would. **Where there
is a way round, the planner takes it, before and after:** the way's price is the ranks' (a cell with less room than
the ranks want costs up to six times a flat one, `terrain::clearance_costs`), so 1,200 of canyon cost more than
3,700 of open ground, and the posture never sees the canyon. With the posture a tight stretch costs a forming-up and
not a crawl, so that price is now too high; what it should be decides which way armies go and is left for the user
(K-group-the-ways-price-walks-round-a-pass). Two fences alone beside the way (no wall) are walked round the same
way, 32-33 / 34-36 / 36-37 before and after.
