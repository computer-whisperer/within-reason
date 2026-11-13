# A group is slots at a place; units are dealt to it (2026-11-11)

**Status (2026-11-11).** Written on the user's word ("Go ahead with the target design") as the arc's ground truth,
before any code. Nothing is built. §9 lists what still needs his word: D1 is ruled (nothing holds a unit back from a
fight), D2 agreed, D3 withdrawn with the one body a group (the march and the companies stay as they are), D4 agreed. He then said
"Let's proceed": the build runs by §10's stages, the flagship game of stage 6 still on his word. **Built the same day:
stages 0 and 1, and the hands' steps 1 to 5 of §12 (the deal, the lane without the muster, the hands on the deal,
the `groups` tool, the texts, the registers). Not built, not run: §12 says which. Played that night on his word
("The pianist-only smoke then flagship"): four smokes and one flagship game against Apex v0.5.07, won at 17:01
(18:30 on the old hands); four faults and an order flood found in it and mended (§11, stage 6).** The same day, on his reading: §5 as first
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
- A reinforcement at a threat its army has passed (2026-11-12, after the second game, §11; put to him as: walk on
  as the army did, take the army's way, or stay held and say so): "Agreed with walking on and reporting it for now. We may decide to do something more complicated later, but let's see the simple approach in practice first".
- The march, on D3 as first proposed (every clear stretch walked relaxed): "why are we extending the relaxed pass?
  The formation movement still has value in controlling the layout of our units if we encounter the enemy, and so
  far does a good job of allowing units to maneuver without bumping into each other. What part of that do you think
  needs to change for this proposal?" The answer is §7: none of it.

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

## 2. Where groups are today

| layer | what it holds | what this design does with it |
|---|---|---|
| the player's tools (`strategist/mcp.rs`) | `produce`: `group` (a name, `new`, `by_type`; by default one group a factory) and `join_in`; `lane`: `company` (the cap, 24) and `heal` | the group and muster clauses go; a group's slots are declared (§3.1) |
| the hands' group (`pianist/groups.rs`) | a member list, `parent`, `joining`, one task, the body as a sort along the goal axis, the newcomer's group by its factory (`group_for_newcomer`), `admit` | the list, `parent`, `joining` and the newcomer rules go; members are read from the deal each second |
| the hands' orders (`execute.rs`, `menu.rs`) | `join` / `take_in` append one group's members to another; the send ladder and `rove_1` each make a group of their own; `start_hunt` moves the name when half go | the send rungs stay on the group's menu as lends that make no group (§5); `join`, `take_in` and `rove_1` go (§9 D2) |
| the gate and the pick (`compose.rs`) | a party's `answer` noul and an actor's `change` open moves; each move is rated; the best go to the worlds; a world gives every actor one move and its line says the combination; one Choice a component, the world drawn | unchanged; a world's line prices an answer whole (§5) |
| the lane's army (`crates/micro/src/army.rs`) | companies by history: a newcomer joins a marching company within 800, musters at a plant, or founds one where it stands; merge under the cap; `compose` takes the largest marching company's place | kept, and handed only the soldiers with the group; the muster and `compose`'s place go (§7) |
| the report (`pianist/picture.rs`) | a group's place is the lane's anchor of the composed company; "each marching on its own way" | the place is the slots' place, with how many are in it (§4) |
| rove (`crates/micro/src/rove.rs`) | a flat list of rovers each tick, a memory a rover made and dropped by that list, the pack by a 400 radius | unchanged |

## 3. The frame

### 3.1 A group

A group is a **name**, its **slots**, a **task** and a **place**.

- **Slots** are its wanted strength: how many soldiers, of which types (any type when unsaid). At most four companies
  (96 at the lane's company of 24).
- **Task** is what it has today: hold, move, fight to, attack a party, follow, rove.
- **Place** is where its slots are. A holding group's place is the point it holds. A moving group's place is the
  centre of the soldiers with it, so it follows its body; with nobody with it, it stays where it was. A roving group has a region, or none, and no place.
- **Only the player makes, resizes or ends a group.** Nothing merges, splits or renames one. An empty group keeps its
  name, place and slots. A seat that has declared none has one, `group_A`, of the cap and any type, holding beside
  its first plant.

A player's order names the slots: "group_X moves to pos_y" lands on them wherever they are when it arrives, whoever
fills them.

### 3.2 With and coming

A soldier dealt to a group is **with** it when it can reach the group's place within the gate, `GATE_SECONDS` (15)
at its own speed: a Lance (46.5) within 700, a Bull (62) within 930, a Blitz (101) within 1,500. Otherwise it is
**coming**. Travel time is the hands' price for the walker's class where a field is held, and the straight line
over the unit's speed where none is. A soldier with a group stays with it out to `GATE_LEAVE` (30 s), so that a body four companies across does not shed
its edges; both gates are set in stage 0. Only the units with a group are its body: they are what the lane forms, what
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
- **The soldiers coming, as amended 2026-11-12** (§0, §7): "62 more soldiers are coming to it (62 Stouts), each
  alone, fighting what it meets and turning back for nothing: 26 at E2, 20 at D3, 12 at B5 and 4 elsewhere; by the
  straight line half of them are 50 s of walking away and the last 110 s; 9 of them under fire now (7 at E2, 2 at
  F3); 5 lost on the way in the last minute (4 at E2, 1 at F3): 4 to a Sentry, party_7; they are not in its fights
  yet". Where they are and what is happening to them are observations; the seconds are named as the straight
  line's. A soldier that dies coming is no loss of the body's: the body's `health` and `losses` lines count the
  soldiers with it, and the wake for losses counts both and says how many died on their way.

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

### 5.1 The answer priced whole, as built (2026-11-11 night; the user: "Build the joint pricing and replay tonight's moments offline")

What the first game showed (§11, stage 6). World 1 said of a party with an answer on its way "Not met: party_5
(...): group_answer_party_5 is told to attack it, but nothing of it has the party in reach (the nearest of its 2
soldiers is some way off, about 10 s of walking away, on its way)", with no odds; a world lending four more said
"with the 2 already answering it, we outweigh it heavily ... Sent at: party_5 by group_A". Read side by side, a
party not met against a party sent at and outweighed: 55 of 91 lends went at a party lent at within the ten seconds
before. An answer in reach read "Met: party_23 met by group_answer_party_23", again with no odds, beside a
withdrawal whose words carried them; and one world both lent four to an answer and withdrew it.

The rule: **every world says a party's answer once and whole, world 1 among them.**

- **The clause.** World 1: "group_answer_party_5, the answer to party_5 (1 armpw, at spot_50, killing our Sentry
  (armllt)): 2 soldiers (2 from group_guard), none has the party in reach yet (the nearest is some way off, about
  10 s of walking away, on its way): we outweigh it heavily; it is faster than the party." A world that lends:
  "group_answer_party_5, the answer to party_5: 6 soldiers (2 from group_guard, 4 more from group_A), none has the
  party in reach yet (...); the ones lent now there in 20 to 33 s, no sooner than those on their way: we outweigh
  it heavily, as in w1; they can catch it." Its soldiers by lender; where the standing ones are and when the ones
  lent in this world arrive, beside the arrival of those already on their way; the odds of all of it against the
  party; whether it catches the party. The verdict is set beside world 1's: "as in w1", or "(in w1, with its 2: an
  even fight)". A new answer: "The answer to party_5: 2 soldiers (2 from group_guard), the first there in 8 s and
  the last in 9 s: we outweigh it; they can catch it." A world that withdraws it says "No answer to party_5 then".
- **A lend's own words on a world's line shrink** to who lends what at which party and what the lender keeps ("4
  armflash of group_A (its soonest to it) join the answer to party_5 (1 armpw, at spot_50, killing our Sentry
  (armllt), heading south-west: our next extractor on its way is at spot_73); group_A keeps its course with its
  other 5"). The party is in the rung's own words, heading and all: with its name alone a first lend lost a tenth
  of its picks. The gate's question about the rung keeps the rung's full words: the gate is not changed.
- **The answer's actor is out of `Met`, `Sent at` and `Not met`:** those clauses stay for a whole group's attack.
- **No world both withdraws an answer and lends to it** (a clash, like two builders on one spot).
- **Pure.** The worlds are composed off the brain's thread (`compose::follow_up`), so the pooled odds are computed
  there from a table made with the menus: `combat::Table`, the lookups the power formula reads for the types of the
  hunters and the party, under the one formula the brain's own odds use (`combat::Lookup`). Each lend carries its
  awake hunters as a force, the answer's menu its soldiers, and each party with either its force and what is true
  of it whatever comes at it (unarmed; cannot hit the ground).

Not changed: the gate, the nouls, the draw, the ladder's sizes, the answer's two moves. A pick of the whole
combination is still one Choice a component.

**Asked again offline before it was built in this wording** (`docs/studies/2026-11-11-answer-pricing.md`): 269
recorded picks of the first game and a smoke, each as recorded and in the new lines. A lend at a party already
answered loses a third of the pick's mass (0.46 to 0.31) and is the top world in 12 % of those picks for 41 %; where
no lend on offer changes the verdict or arrives sooner, 0.39 to 0.22; a lend at an answer that is even or
outweighed keeps its mass (0.37 to 0.32 on the lends that change the verdict); a withdrawal 0.55 to 0.47; a first
lend 0.46 to 0.49. Not ended: a fifth of the mass still goes on a lend that changes nothing, at every second the
answer walks, and the pick is a draw. In the one game with a player on it since (§11, the second game) a lend within
ten seconds of one at the same party was 48 % of the lends for 60 %.

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

**The march is not changed** (§0, the ruling on D3). Ranks, the anchor's pace, the learned leads, the relaxed pass on
restricted ground, contact and the fight stay as they are. So do the companies: a group's body is up to four
companies of the cap, each a formation on its own way, which is what keeps one group from being one movement domain
(the ruling of 2026-10-25).

What changes is what the lane is handed:

- **Only the soldiers with the group.** No company is founded far from the body, because a far soldier is not in the
  body. Today the partition founds a marching company where a far newcomer stands; that rule made the string. Within
  the body the partition and the merge are today's: a soldier joins the nearest company under the cap, else founds
  one beside it. Its company and its slot stay the lane's working state, as they are now.
- **Coming units walk on one plain order** to the group's place, sent again when the place has moved. Nothing holds
  one back: no muster at the plant, no gathering short of a fight. A soldier that comes within the gate of a body in
  contact joins a company of it at once. **Nor the lane's flee, from 2026-11-12** (§0): as built until then a comer
  was a loose soldier to the lane, and H-MICRO-FLEE stepped it back from every threat on its way and stood it at
  the edge while its goal lay beyond (§11, the second game). The hands now tell the lane who is coming, and it
  turns none of them back; they keep the fan and the kite. Soldiers with no slot keep the flee.
- **The muster goes** (D1): the mustering role, the muster points and the release count.
- **A company founded on a route of several points begins where its nearest neighbour is** (found by stage 1: a
  soldier that came up with the body half way round a circuit founded a company that began the lap again).
- **The group's place is §3.1's.** `compose` no longer takes it from the largest marching company.

## 8. What goes

| where | what |
|---|---|
| `crates/micro/src/army.rs` | `Role::Mustering`, the muster rule of `partition`, `muster_due`, `MUSTER_PATIENCE`, `MUSTER_MIN`; `compose`'s place |
| `crates/micro/src/group.rs`, `lib.rs` | `GroupOrder::musters`, `release`; the muster's release in the tick |
| `pianist/groups.rs` | `Group::members` as a stored list, `parent`, `joining`, `admit`, `group_for_newcomer`, `Body`'s front and tail, `start_hunt`'s naming |
| `pianist/execute.rs`, `menu.rs` | `Order::Join`, `TakeIn`, `RoveOne`, `RoveOneWithin` (D2). `Order::Send` stays and changes: a count in place of unit ids, the words of §5, no group made |
| `brain/micro.rs`, `pianist` | `pianist.musters` |
| `strategist/mcp.rs` | `produce`'s `group` and `join_in` |
| `docs/heuristics.md` | retired or rewritten: H-HANDS-GROUPS, H-HANDS-JOINERS, H-HANDS-JOIN-SYMMETRIC, H-HANDS-GROUP-BODY, H-HANDS-DETACH, H-HANDS-DETACHMENTS-FOR-SMALL-PARTIES; the muster's words in H-HANDS-LANE |

Kept as they are: the lane's `company` setting, `GroupOrder::cap`, H-HANDS-ARRIVAL, and the bots' calls into the lane
(neither bot gives the lane a muster point).

The deal, the slots and the answer are entered in `docs/code-decisions.md` with their constants in the commit that
builds each.

## 9. The four points put to the user, and his rulings

- **D1. Reinforcing a group in contact: ruled 2026-11-11, nothing is held back** (§0). Proposed and rejected:
  soldiers coming to a group in contact gathering on the last clear ground and going in as a company. Rejected
  because pushing a fight or leaving it is Jev's, and code that withholds units decides it instead. What the design
  owes for it is §4's lines on a group in contact. It also settles the muster at the plant (`join_in`,
  `MUSTER_PATIENCE`), which goes: a new soldier walks when it is made. That extension is this design's reading of
  the ruling, told to him, not his words.
- **D2. What leaves the world pick for the player: agreed 2026-11-11** ("that sounds fine, agreed"; a bare assent,
  so cheap to reopen). A `groups` tool for slots replaces `produce`'s group clause. Because only the player makes
  or ends a group, three moves leave the groups' menus: `join` and `take_in` (the world merging two groups) and
  `rove_1` (the world splitting off a scout). The send rungs stay (§5). Changed since he agreed: `lane`'s `company`
  stays, because the companies do (D3). The brief changes between games only.
- **D3. Clear ground as a pass: withdrawn 2026-11-11** on his question (§0). Proposed and rejected: every stretch
  with nothing hostile near walked relaxed, and with it a group laid as one body of four company blocks about one
  anchor with the army of companies deleted. Rejected because the frame needs neither. On open ground the ranks cost
  a company of one type about two seconds (Stouts 25 / 27 / 27-28 s in ranks on 2026-11-10; twelve on a plain order
  24 / 25 / 25 in October's table, not the same body), while a plain order funnels a line into a file two wide and
  strings a mixed body out at 47 elmos a second (K-engine-one-goal-point-funnels-a-line-into-a-file), which is how
  an enemy nobody saw would meet it. And four blocks about one anchor is the slaving of companies-1, which the cap was made
  to end. What I had cited for it was the five companies' end and the order rate; neither is this design's (§11).
- **D4. Strengths are the player's: agreed 2026-11-11** ("D4: agreed"; a bare assent). Jev owns tasks, timing and
  answers, and moves no slots.

## 10. How it is judged

Each stage has rows that could show it did nothing or did harm. Every batch gets its line in `docs/experiments.md`.

0. **Offline, no engine.** The deal replayed over recorded seats (the 10v8's t14 and t16, a 1v1): groups' fill over
   time, re-deals a unit-minute with the damping on and off, the gate at 10, 15 and 20 s; for each recorded send,
   the soonest pooled set against what was sent. A recording is not steered by the deal, so this sets constants and
   can only refute: over one re-deal a unit-minute under damping sends the rule back.
1. **The lane, on the drill bench.** The lane handed the body alone, comers on plain orders, the muster gone. It
   must move nothing of the march:
   - the mixed company of 24 on the 2,000 walk (arrival 43-47 s, formed share 0.87-0.90), the yard, the wall's door
     and the lap of Quicksilver level with their rows;
   - new: a lab feeding a group on a walk (`--drill circuit`, `--feed`, `--lab`): the body's companies, the share of
     the group's soldiers inside the gate (a third in the game's string), and when each comer joins;
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

## 11. What the stages said

**Stage 0 (2026-11-11): nothing refuted, the constants stand at 15 / 30 / 5.** `crates/micro/src/deal.rs` replayed
by `cargo run --release -p micro --example deal_replay -- <record>` over the 10v8's two seats from 3:00 (248,000 and
173,000 soldier-seconds) and the Apex v0.5.07 1v1 (37,000), a stand-in for the player declaring a group of 96 where
24 soldiers no group has room for stand inside one gate, each group's place following the soldiers with it.

| seat | gate / leave / margin | re-deals a unit-minute | with to coming, same group | with / coming / surplus |
|---|---|---|---|---|
| t14 | 10 / 20 / 0, 5 | 0.43, 0.22 | 0.04, 0.05 | 48 / 42 / 10 % |
| t14 | 15 / 30 / 0, 5 | 0.18, 0.13 | 0.02 | 69 / 24 / 7 % |
| t14 | 20 / 40 / 0, 5 | 0.14, 0.09 | 0.01 | 77 / 18 / 5 % |
| t14 | 15 / 15 / 5 (no leaving gate) | 0.25 | 0.21 | 58 / 35 / 7 % |
| t16 | 15 / 30 / 0, 5 | 0.11, 0.07 | 0.03, 0.04 | 81 / 12 / 6 % |
| 1v1 | 15 / 30 / 0, 5 | 0.00 | 0.03 | 41 / 41 / 18 % |

The bar was one re-deal a unit-minute under the margin; the worst row is 0.22. The leaving gate is what keeps a
body's edge (a tenth of the falls from with to coming), and the margin cuts the re-deals by a third at 15 s and by
half at 10. Four groups carry each seat of the 10v8 to the end, where the hands declared 197 and 155. The 1v1's
surplus is the stand-in's (it founds nothing under 24; the design's default group stands from the first second).
Read with the recording's limit: nobody in it came when the deal said coming. Not done here, moved to stage 5: the
recorded sends against the soonest pooled set.

**Stage 1 (2026-11-11): the march did not move, and comers join at least as well** (ledger `slots1-*`,
`slots2-late-*`, `slots3-circ-*`, with `slots0-*` on the binary of the day before). The lane without the muster,
handed the body alone by the duel director's deal; first / median / last arrival, before and now:

| row | before | now |
|---|---|---|
| the mixed company of 24, 2,000 | 43 / 46 / 50, formed 0.91 | 44 / 47 / 49, formed 0.91 |
| 24 Stouts out of a yard | 26 / 27 / 29 | 26 / 28 / 29 |
| a hundred Stouts through a wall's door | 35 / 46 / 92, 13 left behind | 35 / 44 / 84, 12 left behind |
| twelve Marauders' lap with a feed of six | median 177, formed 0.80 | median 181, formed 0.85 |
| new: 24 Stouts, twelve more 1,500 behind | done 55-57 s, one at the time limit, formed 0.64-0.69 | done 53-54 s, formed 0.77-0.79 |
| new: 24 Pawns' lap, a feed of 36, cap 24 | 409-437 s, formed 0.92-0.94, 0-1 left behind | three at 415-418 s, formed 0.93-0.94, none left behind; one at the time limit |

The lap at the time limit had every soldier formed from 420 s and one of the first 24 never inside the lap's radius
of the lab: three companies' ends beside each other, the end this design leaves alone (§12). In the game this is a
one-point route. Mended on the way: a company founded half way round a route of several points began it again.

The stage's bot game was not run with it. It ran the same night after the first flagship game, on the user's word
(ledger `slots-distilled-smoke`): WRDistilled against BARb easy on the line of the smoke of 2026-11-10, won 11:37
for 15:43, no fault, the strike body's formed share 0.90. One game; the bot uses no hunt and no deal.

**Stage 6, the first games (2026-11-11 night; the user: "Let's proceed with testing this. The pianist-only smoke
then flagship"): the rebuilt hands play, the game was won, and it showed four faults and a flood of orders, all
mended the same night.** Two smokes of the pianist alone on a fixed packet (ledger `slots-smoke-1`, `-2`): no
runtime fault; against BARb medium 40 lends into 16 answers, every soldier with its group, lent or on its way back
at each look. Then one flagship game on the line and seed of apex-v0507-comet-sym0-opus-1 (the old hands, WON
18:30), on the `claude` account at his word: **slots-apex-v0507-comet-opus-1, WON 17:01**, called by the referee.

| | the old hands, 2026-11-09 | the slots, 2026-11-11 |
|---|---|---|
| result | won 18:30 | won 17:01 |
| extractors at 5 / 10 / 15 | 10 / 36 / 45 | 9 / 31 / 52 |
| army metal at 5 / 10 / 15 | 942 / 5,281 / 12,270 | 1,162 / 5,510 / 14,521 |
| extractors lost to raids | 26 | 10 |
| groups | dozens, named by the hands to `group_P1` | five, all the player's, in thirteen `groups` calls |
| lends (sends) and the parties they went at | 22 at 12 | 91 at 31 |
| a lend within 10 s of one at the same party | 7 | 55 |
| `join` moves; withdrawals | 38; none | none; 16, five of them lent at again within 10 s |
| Jev | $0.92, 21.85M tokens in, the actors' section 6.04M | $0.52, 12.46M, the actors' section 2.76M |
| orders to soldiers a unit-minute | 52.1 | 75.7 (59 % of them the hunt's attack, sent again at every tick) |

What the game showed of the design:

- **The player used it.** `group_A` (the army), `group_guard` (three to six at home), `group_scout` (Rovers
  roving), `group_hunt` (three Rovers at a spot in the middle) and `group_mid` (eight Stouts at D3), resized as the
  game went, and no detachment was forbidden in its packet. Lends came from every group, and several groups lent to
  one answer (sixteen soldiers of three groups on party_51).
- **The report said what the design meant it to.** The player at 16:01: "group_A is down to 20 soldiers worth about
  4,000 metal ... The 27 Stouts behind it arrive over the next minute and a half, the first half in about 30
  seconds."
- **Coming is a fifth of the soldier-seconds** (20.7 %; 9.5 % lent): with the army in his base at 16:00, 28 of 64
  soldiers were on the road to it. Four of the 85 soldiers lost died coming (64 with `group_A`, 14 in answers).
- **Churn:** a soldier's named group changed 0.21 times a unit-minute; 0.97 with lends and returns counted, against
  the bar of one.
- **Faults, each mended** (K-deal-a-slot-of-any-type-took-what-a-named-slot-asked-for,
  K-micro-a-hunters-attack-went-out-at-every-tick):
  1. A slot of any type took what a named slot asked for: the army took the scouts' first Rover (three groups
     declared at home, the tie in seconds broken by the list's order), and when the army was given 96 slots of any
     type it took both roving scouts, which were inside its gate. The player gave the army slots by type both
     times, which left a Stout and later two Bulls with no slot. Now a slot naming a type takes a soldier of that
     type first, among the gates it is inside and for a roving group's open slot, and a rover stays its group's
     wherever it roves.
  2. A roving group with no rover had no entry, so the report listed it as an actor "gone (dead, merged or split)".
  3. The `groups` tool refused a new group's place by its name (`spot_44`, `spot_33`) and a value sent as a string
     of JSON.
  4. A hunter's attack order went out at every tick (ten a second; since the hunt was built, and through a guard of
     2026-11-07 that never took effect): 22,095 of the game's 37,582 orders to soldiers, the worst minute 118 a
     second against the seat's budget of 100.
- **For his word, not built:**
  - Jev tops an answer up a few soldiers at a time and takes one back to lend at the same party again (the two
    rows above). The lender's rung and the answer's `stay` and `withdraw` are judged apart; a world prices neither
    against the other (§12, the joint world's line). **Built the same night on his word, §5.1:** every world says
    the answer once and whole. Asked again offline, a lend at an answered party loses a third of the pick's mass
    and is not ended; in the second game, below, 48 % of the lends for 60 %.
  - The deal fills the nearest group with room first, so a group far from the plants fills only once every nearer
    one is full or names other types. It did not bite in this game (the small groups named their types and stood
    nearer), and a station of slots of any type beyond an army gathering at home would stand empty.

**Stage 6, the second game (2026-11-11 night; the user: "We should also start a new game to test all changes before
continuing past this"): won, the mends held, and reinforcements did not reach an army standing beyond his turrets.**
slots-apex-v0507-comet-opus-2 on fbd2157d, the same line and seed: **WON 20:02**, called by the referee. The three
games by one script (the counts are its, so a row here can differ from the table above by a few):

| | the old hands | the slots, first game | the slots, second game |
|---|---|---|---|
| result | won 18:30 | won 17:01 | won 20:02 |
| groups | 60 names | 5 | 7 |
| lends, and the parties they went at | 22 at 12 | 91 at 31 | 88 at 33 |
| a lend within 10 s of one at the same party | 7 (32 %) | 55 (60 %) | 42 (48 %) |
| withdrawals; lent at again within 10 s | none | 16; 5 | 16; 5 |
| world 1's share of the pick: two worlds, three, four or more | 0.54, 0.38, 0.56 | 0.48, 0.35, 0.51 | 0.48, 0.38, 0.46 |
| the gate: a party with its answer on its way needs answering | | 67 % of 163 | 51 % of 175 |
| orders to soldiers a unit-minute; the worst minute a second | 52.1; 67 | 75.7; 118 | 30.5; 99 |
| the hunt's attack orders a hunter-second | 6.79 | 7.38 | 0.13 |
| soldier-seconds with a group / lent / coming or no slot | | 69 / 10 / 21 % | 63 / 3 / 33 % |
| soldiers lost with a group / in an answer / coming or no slot | 72 | 67 / 14 / 4 | 83 / 3 / 5 |
| extractors lost to raids | 26 | 10 | 14 |
| Jev, by `run/batch_read.py` | $0.74 | $0.43 | $0.68 |

- **The mends held.** No group reported gone, no rover taken by another group, `groups` took a spot's name, `home`
  and values sent as strings of JSON; the hunt's attack goes out once (0.13 a hunter-second) and the orders to
  soldiers are under the old hands'.
- **Reinforcements do not reach an army standing beyond his turrets**
  (K-deal-a-comer-alone-is-held-at-the-edge-of-a-threat-its-body-passed). From 17:50 to 19:40 `group_A` held at H1
  with 21 to 25 soldiers while every report read 57 to 71 more coming, "half in about 50 seconds". At 19:00, 68
  soldiers were with no group, 46 of them between D2 and F4 and 26 at E2 and E3, before his turrets at E2, F2 and
  F3. A comer walks alone on a fight order to the group's place; it is in no body, so every threat on its way is
  one "its group was not priced against", and H-MICRO-FLEE steps it back and stands it at the edge "while its
  order's goal is inside such a threat" (330, 106 and 419 fires in the game's last three minutes, under 12 a minute
  to 16:00). The body had passed as a body, on the pick's word. Three comers died there; a third of the game's
  soldier-seconds were coming or with no slot. The player at 19:06: "Its 62 reinforcing Stouts have been 'about 50
  seconds away' for the last few reports, so they may be held up on the walk", and at 19:34 it declared a second
  army at D4 for it. **This is code holding units back, which the ruling on D1 forbids, and nobody chose it.** The
  first game had it smaller (45 and 79 fires in its last two minutes, 28 of 64 on the road at 16:00). His word,
  2026-11-12 (§0): they walk on, and the report says it (built the same day, §4 and §7).
- **The arrival times in the report are the straight line's** (§12): "half in about 50 seconds" stood for two
  minutes.
- **Soldiers of a type no open slot names stand until the player sees them.** At 8:04 the player made the army
  Stouts only, to move its four Blitzes to three posts; five of its fourteen `groups` calls after that homed
  soldiers standing unassigned (a Blitz three times, a Shellshocker, four Whistlers), each within a turn or two.
- **The answer priced whole, in a game:** a lend within ten seconds of one at the same party fell from 60 % of the
  lends to 48 %, and the withdrawals did not move. The gate's wording and the draw each second are unchanged
  (§5.1's "not ended").
- **World 1 without its label and its sentence:** no difference that one game against another can show (the
  offline arms said three to five points on two-world picks).
- The game was three minutes slower than the first by its middle, not by the slots: ten Stouts into his F2 turrets
  at 9:10 (eight lost), the energy store empty and up to 2,900 of metal banked from 9:00 to 13:00.

## 12. The build's state (for whoever picks it up)

**Done and committed (2026-11-11).**

- The deal with its replay (8bce1fbc, stage 0).
- The lane without the muster and the duel director on the deal (a214aee4, stage 1).
- The hands on the deal (43f129bb): `Group` carries its slots, its place, the soldiers with it and those coming;
  `keep_groups` makes the player's declared groups (none: `group_A`), deals every soldier, sends the comers; a
  `send` is a lend to the one answer to its party, an actor with `stay` and `withdraw`; `join`, `take_in`, `rove_1`
  and everything of a soldier's origin are gone.
- The player's side and the registers (the commit after it): the `groups` tool; `produce` without `group` and
  `join_in`; the report's `slots`, `reinforcements`, `answer_to` and `unassigned` lines; the player's prompt, the
  example packet, one of the hands' rules and the brief in the new words; H-MICRO-DEAL, H-HANDS-SLOTS,
  H-HANDS-ANSWER and the retired rows in `docs/heuristics.md`; three rows in `docs/code-decisions.md`; two claims
  in `docs/knowledge/formations.md`; the duel harness notes.

**Not built.**

- (Built 2026-11-11 night, §5.1: the answer priced whole on every world's line.)
- §4's line on where the last minute's production went, and "what it killed" among a fighting group's facts (its
  losses, its comers with their seconds and the enemies near it are there).
- A group with nobody in place has no menu (§3.1 as written). So when an army dies where it held, its comers walk
  to that place until the first of them is inside the gate and the group is asked again, or the player moves it.
  Before, reinforcements stopped at the plant because the group was gone. The user has not been asked.
- The travel time is the straight line over the soldier's speed, not the hands' fields: a place across a cliff
  reads nearer than it is. In the second game it read "half in about 50 seconds" for two minutes of comers held
  before his turrets (§11). From 2026-11-12 the report names it as the straight line's and says where the comers
  stand; the deal still fills slots by it.
- (Built 2026-11-12, §0, §4, §7: a comer is under no flee and the report says where the comers are, who is shot at
  and who died on the way. The user: "We may decide to do something more complicated later": a comer taking its
  body's way, or gathering, is not built.)
- The Distilled and Territory bots do not use the deal.

**Not run.** Stages 2 to 5 of §10: the drills of a station beside an army, the raid scenario with lends (its
director still pickets by its own rule), rove with a rover dealt in and out, and the hands' new words replayed to
Jev over recorded gates. Run since (stage 6, §11): six smokes of the pianist alone, one WRDistilled game and two
flagship games, both won; no lobby game, no team game, nothing with aircraft. Of stage 5, the answer's lines were asked again over the first
game's picks (§5.1); the build with them has two smokes and the second flagship game behind it.

## 13. Not in this design

The march, in or out of the enemy's reach, and its order rate; the end of several companies told to one goal
(K-group-five-companies-told-to-one-goal-do-not-settle), which a group of four companies still has; static defence; the way's price for narrow ground
(K-group-the-ways-price-walks-round-a-pass); Thor corners; the bots taking up the deal; the response-time lines of §4.
