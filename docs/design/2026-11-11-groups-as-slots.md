# A group is slots at a place; units are dealt to it (2026-11-11)

**Status (2026-11-11).** Written on the user's word ("Go ahead with the target design") as the arc's ground truth,
before any code. Nothing is built. §9 lists what still needs his word; D1 is ruled (nothing holds a unit back from a
fight). The same day, on his reading: §5 as first
written made an incursion a question of its own to Jev; he ruled that out (§0, the last ruling), and §5 is rewritten
so that an answer is moves in the world pick. Rejected because a pick a threat is the threat-centred pass of
2026-09-26 that the one pass replaced, and it settles one party without the rest of the world in view.

## 0. The rulings (the user, 2026-11-10 and 11, after the lobby 10v8 on Erebos Lakes)

- "If something spans half the map, it really shouldn't be a single group node, but what should be breaking it up?"
- "groups should never grow beyond a certain size -- perhaps just 4 companies"; "Units too far away should not be
  considered part of the group for certain, but we still need to allocate for joiners from various sources"; "We
  should be searching for the simplest formulation that fits our needs without adding complications that will bite us."
- "we should have almost no hidden state to our unit control. What I mean by this is that the state we are trying to
  manipulate is completely in the game. Groups mean nothing to BAR, it's just a bookkeeping method to allow us to
  control fewer distinct units, filtered by position and action. The player benefits from coherent group namings, so
  some level of assignment coherence is useful. A unit that came out of a factory and is headed to a group has no
  special state to it versus a unit that happens to be in that same mid position. Same with a group of units that was
  off hunting -- the original group it came from has no relevance on it."
- On a group as "the units under one order", rejected: "The point of reducing the hidden state is to reduce the
  number of things we need to carefully control, not for just removing the need to carry a few bytes of state. We
  need to carefully control unit movement where the constraint is that units have limited speed and maneuverability.
  Two units with the same position, velocity, etc that are not fully interchangable is a gauge orbit we can take
  advantage of removing."
- Defence: "One major thing that needs to work in the frame we choose is defence. The player and jev should be able
  to station units in key spots to intercept runbys or defend locations, and whenever an incurson happens, the ideal
  units should be able to assigned. Those units may be from a standing defence, or they may be fresh out of the
  factory heading to the army, or they may be part of a nearby army group. Right now in operation I see most of our
  play gravitates towards a single army group, and players frequently regress to banning detachments entirely.
  Prompting of both jev and the player can be tuned to avoid this, of course, but the unit control system needs to
  align to the tasks we face regularly."
- What a group is for: "the primary value of groups is giving the player something to point at for 'group_X moves to
  pos_y' that can still be coherent after up to 30s or so of gameplay simulation governed by jev and the micro
  engine. Jev doesn't care about group names on its own, but it does need to know what cluster of units the player
  meant by 'group_x'. Combine that with the point of unit fungability, then the actual units the player meant don't
  matter, but the notion of that set of unit slots in that location does."
- Rove: "It was created as a scripted mode because the abstractions we chose for army control had no hope of
  performing the precise micro necessary. If we still keep it in its current implementation, then we will want a
  fluid way to get units in and out of rove."
- The world pick, on this design's first §5: "I don't think we want to invert how the gate and world picks work.
  Right now the world selection is authoritative and describes the whole action combination we are selecting, and
  the gate is an optimization for pruning worlds we can independently rule out. A jev ask for 'who should answer
  incursion_X' is a pattern we had before that I explicitly rejected for the sake of this new system. I could see us
  bringing something like that back as part of the gate if that is determined to be the efficient way to rule in or
  out worlds, but the authoritative thing still needs to be the world pick."
- Reinforcing a fight, on D1 as first proposed (comers gather short of the enemy and go in as a company): "we should
  not be holding units back from an active engagement. If jev decides to push the fight, we should push it with
  everything available. If we are simply feeding units to the enemy, then it is jev's decision to retreat rather
  than the micro engine's behavior."

Earlier rulings this design keeps: one menu an actor, always from our units out to the enemy ("Why do we go from our
units out to the enemy in one case, and from the enemy back to our units in the other case?", 2026-10-01); a roving
group's menu carries no moves at a party (2026-10-08); the cap is there so that small groups move independently and no group inherits
one movement domain (2026-10-25); "keeping the units in formation trivially modellable and the units out of
formation as the exceptions" (2026-10-24); "nothing on restricted ground sends a unit to align" (2026-11-10); "We
usually need the freedom for jev to detach units and repel an enemy party" (2026-10-17); a raid is always answered,
whatever the raider's weight or speed (2026-11-03); Jev must "decide what set of units to send after it (splitting
off of groups as needed)" (2026-09-26); no code decides a live flagship game except on his word (`docs/code-decisions.md`).

## 1. Why

**The string.** In the 10v8 (`run/matches/1794293029-human/00/review.md`) `group_A_t14` never had over 62 % of its
members within 1,200 of their centroid from 12:00, and usually a third; the lane held it as 10 to 23 companies from
the plants to the front, and the report placed it at its newest company. 76 of 377 Lances and 64 of 260 Bulls ever
dealt damage.

**The orders outran the body.** That group's task changed 63 times between 11:00 and 29:20, with 30 walk orders: one
every 37 s. At 12:00 to 18:00 the last units walking to a head were 70 to 200 s from it (a scratch model, straight
line at each unit's own speed). A body that needs minutes to gather under orders that change every half minute never
gathers. So the size of a group has a physical measure: what can be whole inside one decision interval, in seconds.

**The bookkeeping carries no decision.** Seat t14 declared 197 groups; from 16:00 the hands joined a unit to the body
by a pick every three to five seconds. Names made by re-clustering fare no better: on t14's record, re-clustering
every 5 s founded 66 groups and ended 62 in nineteen minutes.

**Defence is priced in new actors.** A detachment "leaves as a group of their own" (the send ladder,
`crates/bot/src/brain/pianist/menu.rs`), with its odds said "against them alone", so every answer is a new name that
needs later picks to come back (barb-ladder3-20-1: 45 of 59 detachments asked again after their hunt, 33 joined
eventually), and several answers to one party are several actors, each weighed alone. In chase-player-2 at 7:58 five
small Blitz groups each withdrew from 13 Pawns as outweighed while the base burned. The player wrote "it sends no detachments" in 9 of 11 games of the detachments study.
Production goes to the group its plant's order names, and nothing restores any other group's strength.

**The march is a position loop through on/off actuators.** No speed command reaches a unit from an AI (the engine's
command 56 is unused, the game's gadget is off, BARb's own call is an empty body; `docs/studies/barb-comparison.md`
item 4). A faster unit is paced by walking and stopping: the mixed company of 24 costs 49 orders a unit-minute since
H-MICRO-NEAR (153 before it) against 13 to 28 for a body of one type. A hundred Stouts in five companies had their
last unit in at 71 to 95 s with 29 left behind, where a plain order with a wall's door in the way had all hundred
there by 37 s, and the end asks 150 to 209 moves a second (K-group-five-companies-told-to-one-goal-do-not-settle).

## 2. Where groups are today

| layer | what it holds | what this design does with it |
|---|---|---|
| the player's tools (`strategist/mcp.rs`) | `produce`: `group` (a name, `new`, `by_type`; by default one group a factory) and `join_in`; `lane`: `company` (the cap, 24) and `heal` | the group and muster clauses go; a group's slots are declared (§3.1) |
| the hands' group (`pianist/groups.rs`) | a member list, `parent`, `joining`, one task, the body as a sort along the goal axis, the newcomer's group by its factory (`group_for_newcomer`), `admit` | the list, `parent`, `joining` and the newcomer rules go; members are read from the deal each second |
| the hands' orders (`execute.rs`, `menu.rs`) | `join` / `take_in` append one group's members to another; the send ladder and `rove_1` each make a group of their own; `start_hunt` moves the name when half go | the send rungs stay on the group's menu as lends that make no group (§5); `join`, `take_in` and `rove_1` go (§9 D2) |
| the gate and the pick (`compose.rs`) | a party's `answer` noul and an actor's `change` open moves; each move is rated; the best go to the worlds; a world gives every actor one move and its line says the combination; one Choice a component, the world drawn | unchanged; a world's line prices an answer whole (§5) |
| the lane's army (`crates/micro/src/army.rs`) | companies by history: a newcomer joins a marching company within 800, musters at a plant, or founds one where it stands; merge under the cap; `compose` takes the largest marching company's place | deleted; a group is one body laid afresh each time it forms (§7) |
| the report (`pianist/picture.rs`) | a group's place is the lane's anchor of the composed company; "each marching on its own way" | the place is the slots' place, with how many are in it (§4) |
| rove (`crates/micro/src/rove.rs`) | a flat list of rovers each tick, a memory a rover made and dropped by that list, the pack by a 400 radius | unchanged |

## 3. The frame

### 3.1 A group

A group is a **name**, its **slots**, a **task** and a **place**.

- **Slots** are its wanted strength: how many soldiers, of which types (any type when unsaid). At most four companies
  (96 at the lane's company of 24).
- **Task** is what it has today: hold, move, fight to, attack a party, follow, rove.
- **Place** is where its slots are. A holding group's place is the point it holds. A moving group's place is where
  its body stands formed, or is forming up next (§7). A roving group has a region, or none, and no place.
- **Only the player makes, resizes or ends a group.** Nothing merges, splits or renames one. An empty group keeps its
  name, place and slots. A seat that has declared none has one, `group_A`, of the cap and any type, holding beside
  its first plant.

A player's order names the slots: "group_X moves to pos_y" lands on them wherever they are when it arrives, whoever
fills them.

### 3.2 With and coming

A soldier dealt to a group is **with** it when it can reach the group's place within the gate, `GATE_SECONDS` (15)
at its own speed: a Lance (46.5) within 700, a Bull (62) within 930, a Blitz (101) within 1,500. Otherwise it is
**coming**. Travel time is the hands' price for the walker's class where a field is held, and the straight line
over the unit's speed where none is. Only the units with a group are its body: they are what the lane forms, what
the odds are priced on and what an order moves at once.

### 3.3 The deal

Once a second every soldier of the seat is dealt, from its position and type alone:

1. **Answers first.** Each live answer (§5) takes, from each of its lenders, the soldiers of that lender that reach
   its party soonest, to the count the world gave.
2. **In place stays.** A soldier inside the gate of a group with an open slot of its type is with that group (the
   nearest, when gates overlap).
3. **Rove slots.** A roving group short of strength takes the nearest soldiers of its types that no step above
   placed; they rove from where they stand.
4. **The rest by time.** Every other soldier goes to the open slot it reaches soonest, pairs taken in order of time.
   It is coming, and holds that slot.
5. **No open slot.** It stands where it is, as surplus: unnamed and shown by place. It is on no actor's menu, so it
   answers nothing until the player gives it slots.

A group resized down releases the members that reach an open slot elsewhere soonest. A slot whose unit is lent to an
answer is open: when the answer ends, each of its units is dealt like any other, usually back.

**Damping.** A soldier already walking to a slot is dealt elsewhere only when that saves `DEAL_MARGIN` (5 s to
begin with). Nothing else is remembered about a unit: its progress toward its slot is the memory.

**Where it lives.** A module of `crates/micro` over (soldiers, groups, answers), so that the drills run it with no
model. The flagship's hands call it; the bots may, and are not made to.

### 3.4 Moving strength

The player moves strength by changing slots: `station_c5` up by six and `group_A` down by six sends the six best
placed. No unit is ever named.

## 4. The report

- A group: its place and task, then "31 of 48 in place, 12 coming (half in 0:20, the last in 1:50), 5 lent to the
  answer to party_12".
- A group in contact, in the player's report and in Jev's `actors` entry alike, as facts and no verdict: how many
  are in the fight now, what arrives when, what it lost in the last 30 s and what it killed. Whether a fight fed one
  soldier at a time is kept up or left is the pick's (§0, the ruling on D1).
- An answer: its strength, its sources and how far out it is.
- Surplus by place: "14 unassigned at B5".
- Where the last minute's production went, by group.

Later, and separable: for each cluster of our extractors and plants, the seconds to the nearest soldiers. A faster
raider is only met by units already ahead of it, so this is what shows where a station is missing.

## 5. An incursion: moves in the world, a loan

The gate and the pick are not changed (§0, the last ruling). A party's `answer` noul opens the moves aimed at it,
each move is rated, the best go to the worlds, a world gives every actor one move and its line says the whole
combination, and the world is drawn from one Choice a component. Lenders at one party are one component already.
What changes is what a send is and what a world says of it.

- **A lend is a move on the lender's menu.** These are today's send rungs, on the menu of every group with soldiers
  near the party, with two changes. The soldiers are the lender's soonest to the party by the deal, those coming to
  it included ("4 of group_A answer party_12: 3 Blitzes of its body and 1 coming from plant_3, the first there in
  8 s, the last in 14 s"). And they do not leave as a group of their own. The forbidden mark stays on the rung.
- **The answer is one actor a party.** Every lend at one party goes to one body: a hunt of the lane's (H-MICRO-HUNT)
  under the party's name, with a menu of its own as a detachment has today (keep on, withdraw), opened when its
  chase meets something new (H-HANDS-CHASE). It is made and ended by the worlds, never by the player. It remembers
  how many each lender gave and nothing of units; its units are dealt each second like any others.
- **A world's line prices the answer whole.** A world in which two lenders send says the answer once: its strength
  by lender, when its first and last soldier arrive, the odds of all of it against the party, whether it catches
  it, and what each lender gives up.
- **It ends** as a hunt ends today (dead, lost, the leash), or by its own withdraw in a world. Its units are then
  dealt back by §3.3.
- A whole group's attack on a party stays on that group's menu, as today. A roving group's menu carries no moves at
  a party, as today, so rovers lend nothing.
- **A pooled question may come back as a gate layer,** and only there: "how much does party_12 want: 2, 6, 12?"
  asked once a party, ruling rungs in or out in place of a noul for every lender's rung. It would be a layer with a
  switch and an audit like the others, taken up only if stage 5 shows it prunes for less. Not in the first build.

A station is a holding group of a few slots. It refills from whatever is nearest, often soldiers walking past from
the plants, and it keeps its name when every unit of it is out on an answer.

## 6. Rove

Rove is a task on a group; the engine is unchanged. Whether a group roves is a move on its menu in the world, as
today (`rove`, `rove_<region>`), and the way back is any other move. A soldier dealt to a roving group is on the
lane's list of rovers from that tick, and off it when dealt elsewhere. Units go in and out by the group's slots
(§3.4), and its losses refill from the plants. A rover dealt elsewhere loses its working memory (way, stand, blocked
targets), a few seconds of plan.

`rove_1` (one soldier leaves as a group of its own and roves) cannot stay as it is, since only the player makes a
group. Proposed: a scout is a group of one slot the player declares, and its `rove` is the world's. That moves "split
off a scout" from the world pick to the player's slots (§9 D2).

## 7. Movement (`crates/micro`)

- **One body a group.** The lane is handed the units with the group and lays them as one body of up to four company
  blocks about one anchor, every slot dealt in one least-travel deal each time the body forms. A company is a block
  of the layout, not a set of units. The army of companies (partition, merge, muster, `compose`) is deleted; so are
  `GroupOrder`'s `cap`, `musters` and `release`.
- **Coming units walk on one plain order** to the group's place, sent again when the place has moved. Nothing holds
  one back: no muster at the plant, no gathering short of a fight. A soldier that comes within the gate of a body in
  contact takes a slot of it at once, and the body does not form again for it.
- **A stretch with nothing hostile near it is a pass** (proposed, §9 D3). H-MICRO-RELAX passes tight ground relaxed:
  the anchor is set on the forming-up ground beyond, every member is sent there once on the engine's own path, and
  the ranks form on arrival. This design applies the same pass to any stretch with nothing hostile within
  `RELAX_CLEAR` (1,000) of it. The body forms before the first stretch that is not clear, and from there the ranks
  march as they do today.
- **Not changed:** the march inside the enemy's reach (the anchor's pace, the learned leads), contact, the fight.

## 8. What goes

| where | what |
|---|---|
| `crates/micro/src/army.rs` | the file: `Army`, `Company`, `Role`, `partition`, `merge`, `compose`, `muster_due` |
| `crates/micro/src/group.rs`, `lib.rs` | `GroupOrder::cap`, `musters`, `release`; the per-company tick; `CompanyState` in `GroupState` |
| `pianist/groups.rs` | `Group::members` as a stored list, `parent`, `joining`, `admit`, `group_for_newcomer`, `Body`'s front and tail, `start_hunt`'s naming |
| `pianist/execute.rs`, `menu.rs` | `Order::Join`, `TakeIn`, `RoveOne`, `RoveOneWithin` (D2). `Order::Send` stays and changes: a count in place of unit ids, the words of §5, no group made |
| `brain/micro.rs`, `pianist` | `company_of`, `pianist.musters` |
| `strategist/mcp.rs` | `produce`'s `group` and `join_in`; `lane`'s `company` |
| `bots/distilled/src/lane.rs` | `.with_cap(COMPANY)` on its strike body and column, changed in the same commit as the lane |
| `docs/heuristics.md` | retired or rewritten: H-HANDS-GROUPS, H-HANDS-JOINERS, H-HANDS-JOIN-SYMMETRIC, H-HANDS-GROUP-BODY, H-HANDS-ARRIVAL, H-HANDS-DETACH, H-HANDS-DETACHMENTS-FOR-SMALL-PARTIES; H-MICRO-GROUP and H-MICRO-RELAX changed |

The deal, the slots and the answer are entered in `docs/code-decisions.md` with their constants in the commit that
builds each.

## 9. Open, the user's to rule

- **D1. Reinforcing a group in contact: ruled 2026-11-11, nothing is held back** (§0). Proposed and rejected:
  soldiers coming to a group in contact gathering on the last clear ground and going in as a company. Rejected
  because pushing a fight or leaving it is Jev's, and code that withholds units decides it instead. What the design
  owes for it is §4's lines on a group in contact. It also settles the muster at the plant (`join_in`,
  `MUSTER_PATIENCE`), which goes with the army of companies: a new soldier walks when it is made.
- **D2. What leaves the world pick for the player.** A `groups` tool for slots replaces `produce`'s group clause and
  `lane`'s `company`. Because only the player makes or ends a group, three moves leave the groups' menus: `join` and
  `take_in` (the world merging two groups) and `rove_1` (the world splitting off a scout). Each is a decision the
  world pick has today and the player's slots would have instead. The send rungs stay (§5). The brief changes
  between games only.
- **D3. Clear ground as a pass.** He ruled the relaxed pass for restricted ground. Extending it to every clear
  stretch is this design's proposal, judged in stage 1 before anything of the march is removed.
- **D4. Strengths are the player's.** Assumed, not objected to: Jev owns tasks, timing and answers, and moves no slots.

## 10. How it is judged

Each stage has rows that could show it did nothing or did harm. Every batch gets its line in `docs/experiments.md`.

0. **Offline, no engine.** The deal replayed over recorded seats (the 10v8's t14 and t16, a 1v1): groups' fill over
   time, re-deals a unit-minute with the damping on and off, the gate at 10, 15 and 20 s; for each recorded send,
   the soonest pooled set against what was sent. A recording is not steered by the deal, so this sets constants and
   can only refute: over one re-deal a unit-minute under damping sends the rule back.
1. **The lane, on the drill bench.** One body a group, the army deleted, the pass on clear ground. Rows to beat:
   - a hundred in one group against the five companies' 28-37 / 41-49 / 71-95 s with 29 left behind, and the plain
     order's 25 / 30 / 37; the end's 150-209 moves a second gone;
   - the mixed company of 24 on the 2,000 walk: arrival 43-47 s, formed share 0.87-0.90, 49 orders a unit-minute;
   - the yard, the wall's door and the lap of Quicksilver level with H-MICRO-RELAX's rows;
   - one WRDistilled game clean. The bots' benches are run only if he asks.
2. **The deal and the slots, drilled.** A lab feeding a group that holds 2,000 off, then attacks a post (`--drill
   gather` and `into`, `--feed`, `--lab`, `--post`): with and coming over time, and that the fight's lines of §4 say
   what the drill's truth says (in the fight, arriving, lost, killed). A lab feeding a station and an army: who
   fills when.
3. **Answers.** The raid scenario with the picket alone (today's row), then with a stream walking past the strip and
   a body 1,500 off: extractors lost, seconds to the first kill, seconds until the lenders' slots are full again,
   re-deals an answer.
4. **Rove.** The rove-2 drills with a rover dealt in and out mid-run: the kill no later than its row allows.
5. **The hands, offline.** Recorded raid gates replayed (`run/jev_ab.py`'s way) with the lends' words and the
   answer priced whole on the world's line: does the pick take a rung where it took a send, and does a world of two
   lenders win where each alone lost. The same for recorded fights fed one soldier at a time, with §4's lines on
   the group's entry: does the pick leave where it kept on. Beside it, the pooled gate question of §5 against the nouls a rung: what it
   costs and how often the two agree, which decides whether it earns a layer.
6. **A flagship game** on his word, then a lobby game read with `run/demo_net.py`.

## 11. Not in this design

The march inside the enemy's reach; static defence; the way's price for narrow ground
(K-group-the-ways-price-walks-round-a-pass); Thor corners; the bots taking up the deal; the response-time lines of §4.
