# Fronts: bounded parties, lanes and seat roles

Status: draft 1, 2026-10-04. Written at the user's word after glitters-8v8-hard-1 and the human 8v8 replay; nothing
is built. It takes up §10's "whose army is it" of `2026-10-03-commander-seat.md`.

A word on names. "Lane" here is a strip of the map a seat is responsible for, as the players of large team games use
the word. The player's existing `lane` tool and H-HANDS-LANE are the micro engine's control lane, another thing (§8).

## 1. What is measured

`run/party_rules.py` runs six rules for forming enemy parties over the same seconds of a record. The columns: `wide`,
the share of enemy units standing in a party wider than 800; `spread`, the share of a party's metal farther than 600
from its centre; `left out`, the share of the enemy metal within 600 of a party's centre that the rule put in another
party; `renamed`, the share of units a second whose party's name changed; `new`, names made a minute.

glitters-8v8-hard-1, seat 0, from minute 10 (what our side saw, 55 mobile enemies a second; every seat of ours is
shown the same enemies):

| rule | parties | biggest | widest | wide | spread | left out | renamed | new/min |
|---|---|---|---|---|---|---|---|---|
| chain 400 (today) | 4.7 | 227 | 3,634 | 64.5% | 22.2% | 2.1% | 1.8% | 27.6 |
| chain 250 | 6.4 | 227 | 3,433 | 53.9% | 15.8% | 9.0% | 3.2% | 46.9 |
| split 800 | 6.3 | 105 | 799 | 0% | 0% | 31.3% | 11.3% | 34.1 |
| cells | 6.8 | 116 | 1,434 | 41.3% | 2.6% | 28.1% | 5.6% | 32.8 |
| seeds 400 | 6.9 | 120 | 795 | 0% | 0% | 24.1% | 9.8% | 51.0 |
| kept 400 | 7.3 | 122 | 1,150 | 21.5% | 0% | 33.4% | 2.5% | 30.0 |

The human 8v8 on the same map, from minute 15, the enemies within 1,200 of a unit of one side (63 a second):

| rule | parties | biggest | widest | wide | spread | left out | renamed | new/min |
|---|---|---|---|---|---|---|---|---|
| chain 400 (today) | 5.3 | 135 | 5,106 | 82.2% | 34.4% | 3.2% | 7.0% | 35.7 |
| chain 250 | 12.3 | 103 | 3,742 | 43.3% | 5.4% | 20.4% | 10.6% | 90.4 |
| split 800 | 9.8 | 43 | 800 | 0% | 0% | 26.3% | 15.9% | 50.0 |
| cells | 8.6 | 48 | 1,731 | 57.5% | 5.7% | 24.3% | 6.0% | 16.6 |
| seeds 400 | 11.5 | 51 | 795 | 0% | 0% | 24.2% | 14.0% | 85.0 |
| kept 400 | 12.5 | 43 | 1,107 | 13.6% | 0% | 31.1% | 3.9% | 39.9 |

(All of his army at once, 654 units a second: today's rule makes one party of 1,098 units 11,670 wide.)

A duel (same-opus-2, from minute 3, 6 enemies a second): today's rule renames 3.5% of units a second and leaves 12.7%
in a wide party; `kept 400` renames 1.0% and leaves 2.3%.

What it says:

- **The blob is in our own games.** In our arena 8v8 two thirds of the enemies we saw stood in a party wider than
  800, and a fifth of a party's metal was more than 600 from the place the picture gave it.
- **A shorter chain does not mend it** (`chain 250`: still half of the units in a wide party).
- **Cutting a wide party each second mends the width and breaks the names** (`split 800`, `seeds 400`: 10 to 16%
  of units renamed a second, against 2 to 7% today). Names that move are what H-HANDS-PARTY-NAMES was built
  against, and they defeat the `same` layer.
- **A party kept from second to second holds both** (`kept 400`: no spread, names steadier than today's in the
  human game and the duel, a little less steady in the arena 8v8). With members kept within 400 of the centre in
  place of 500, 1 to 2% of units stand in a party wider than 800, for one more point of renaming.
- **No rule that bounds a party can weigh a fight by the party alone.** Under every bounded rule a quarter to a
  third of the enemy metal within 600 of a party's centre belongs to a neighbour. A front is continuous; wherever
  it is cut, the neighbour is in the fight.

Our own groups, from the hands' logs and the records (every group, every second):

| game | soldiers: median, 90th, most | groups of 20 or more wider than 600 | wider than 1,500 | median width |
|---|---|---|---|---|
| glitters-8v8-hard-1 | 5, 31, 148 | 90% | 56% | 1,802 |
| same-opus-2 (duel) | 4, 48, 82 | 100% | 97% | 4,679 |
| going-2 (duel) | 1, 23, 77 | 93% | 57% | 1,704 |

The width counts members still walking to the group from their plant, which the group's body (H-HANDS-GROUP-BODY)
leaves out of its core. So this is an upper bound on the body's own length; what it shows for certain is that a large
group of ours is a stream from the plant to the front in nearly every second, in duels as much as in team games.

From glitters-8v8-hard-1 itself (the ledger and the commander's transcript): the commander named four lanes in prose
at 0:20 ("WEST t0+t7 (columns A-B) ..."), with each seat's spots listed by hand, and dropped them at 7:20; it moved
units between seats twelve times to make one body of two; the seats that received them were the seats whose gates
grew with the square of what they held. Every seat's hands were shown every party on the map (seat t2's picture at
17:38 lists three parties in A2 while its own fight is in D3).

## 2. Parties

Three changes, for every game size.

1. **A party is kept, not re-formed** (the `kept` rule, with 400 for both radii). A party lasts from second to
   second. Each second its members still in sight and within 400 of its centre stay; an enemy without a party joins
   one whose centre is within 400; the rest seed new parties, the unit with the most others within 400 first; two
   parties whose centres come within 250 become one, under the larger's name. No party is then much wider than 800,
   and its place is where its units stand.
2. **A fight is weighed on what stands there, not on the party.** The odds of a group against a party count every
   enemy within 600 of the party's centre, as they already count the turrets that cover it (`turrets_covering`).
   The party's line says so: "party_512: 10 units worth 2,675 at D3; with party_500 and party_513 within 600,
   6,900 stands there". The party is the name and the target; the neighbourhood is the weight.
3. **Many parties are said by lane** (§4): a seat's hands are shown, one line each, the parties in its lane and
   those near any actor of its own wherever they are; the rest of the front is one line a lane.

Not in this: the count of names (30 to 45 new a minute in an 8v8, `party_517` by minute 17). The wire's letters
(H-HANDS-WIRE-CUT) could carry them if they turn out to cost Jev; unmeasured.

## 3. Our groups

The body already says front, tail and length, and the odds are already priced on the part near the party when the
body is strung out. Two things are missing, and both are words, not decisions:

- **The stream is said as a stream.** A group's entry says how many of its soldiers are with the body and how many
  are on the way, with where the nearest and farthest of those are ("31 with the body at D5, 17 on the way between
  the plant and D3"). Today the joiners are out of the core and out of the words.
- **A body in two fights is said as two.** When the core stands near two parties more than 600 apart, each part is
  priced against its own (the `within` the strung-out case already uses), not the whole against each.

Not proposed: a cap on a group's size, or the code cutting a group. The groups are the player's.

First step before either: the same measurement on the core alone (the hands' log does not carry who is joining; the
picture does), to know how much of the width is the stream and how much is the body.

## 4. Lanes

### What the commander declares

One tool, `lanes` (the name is open, §8): the whole arrangement at once, replacing the last.

- **Lanes**: a name and the grid cells it covers ("west": A1-B8). Lanes may overlap and need not cover the map.
  Cells, because the commander already speaks in them ("columns A-B, rows 1-4") and nothing has to be drawn.
- **Seats**: for each seat a role and, for a lane seat, its lane. Roles:
  - `lane`: its own army, its lane's spots, its lane's defence. One or two seats a lane.
  - `air`: fighters and bombers for the whole side, scouting, anti-air cover. Reads every lane.
  - `tech`: goes to tier 2 (and on) first on the side's metal, builds what the side shares (constructors of the
    higher tier given to the lane seats, the costly buildings). Reads every lane.
- **Feeding**: which seats send what share of their metal and energy income to which seat, and until when ("every
  lane seat sends 25% of its metal to t6 until its advanced plant stands"). §6.

The direction stays prose and stays the place for everything else. The structured part is only what code must know
to draw each seat's view.

### What a lane is to each agent

| Agent | With lanes declared |
|---|---|
| commander | Its report gains one line a lane (below) in place of reading the bodies and parties of the whole map. |
| player of a lane seat | Its report opens with its lane's line and its neighbours'; free spots, enemy parties and scouting are listed for its lane in full and for the rest by lane line. It is told its lane and role as a fact of the report, beside the direction. |
| player of an air or tech seat | Every lane's line; no lane of its own. |
| hands of a lane seat | Parties in its lane and those near its own actors, one line each; one line for each other lane. Places and spots outside its lane leave the menus unless the player's instructions name them. |

A lane's line, the same words for all of them:

> west (t0, t7): contact at A5. At the line ours 3,400 (t0 group_B 2,100, t7 group_A 1,300) against 2,900 of his
> seen and 2 turrets; behind his line 1,200 seen 2:10 ago in A6-A7; behind ours 14 extractors, 6 turrets, 9 spots
> free. In our rows: party_446 (1 Stout) at our extractor at spot_44.

"Contact" is the cell where the nearest soldiers of the two sides in the lane face each other; "at the line" is
within 1,200 of it. Both are sums of what is seen, as the report's other lines are; nothing here chooses a move.

### Whose army

Proposed ruling for §10 of the commander-seat design: **each seat's army stays its own.** Two seats in a lane are
two bodies told one place; the hands already see "with 12 allied soldiers of seat t2 beside it: together we outweigh
it" in the odds. The commander's prompt stops recommending `transfer` of a body to make one of two (the twelve
transfers of glitters-8v8-hard-1 made the largest gates of the game) and keeps it for what a role hands over: the
tech seat's constructors to a lane seat, a lane seat's leftovers to its lane partner when it changes role.

Rejected because: one seat holding a lane's whole army is one player and one pair of hands for 100 and more
soldiers, which is where both the player's turn time (human-18) and Jev's bill (the square) went wrong.

### Cost

The hands' context of a lane seat stops growing with the side: its parties and places are its lane's. Unmeasured;
the offline measure is the 8v8's gates rebuilt with the parties and places of other lanes cut, counted in tokens,
and the usual replay for whether the answers move.

## 5. Duels

No lanes: one seat, the whole map, and nothing of §4 is shown. §2 and §3 apply as they are; the duel's measured gain
is the steadier names (1.0% of units renamed a second against 3.5%).

## 6. Feeding

The roles need metal to move on a standing rule, not on a commander's turn each time. `transfer` is one gift when it
is called. Proposed: a standing share, declared in `lanes` and carried out by the harness each second (a seat sends
the stated share of its income while the condition holds). It is the commander's stated rule executed, not a choice
of the code's; whether that reading stands under the ruling on code decisions is the user's to say (§8).

## 7. Steps

Each is played before the next is started.

1. **Parties kept and fights weighed by neighbourhood** (§2.1, §2.2), as layers that can be switched, with the
   old rule's parties logged beside the new for a game. Test: the standing duel, then a 3-seat game; the
   forbidden and step-back answers at a front, read by a person.
2. **The stream and the two fights in a group's words** (§3), after its measurement.
3. **`lanes` with lanes and roles; the lane line in the commander's and players' reports; the hands' picture by
   lane.** Test: glitters 8v8 against the same opponent, beside glitters-8v8-hard-1: transfers of units, gate sizes
   by seat, extractors by seat at 10:00 and 20:00, Jev's bill.
4. **Feeding and the air and tech roles' prompts.** Test: the time of the first tier-2 plant on the side.

## 8. Open

- **The tool's name**: `lanes` beside the player's `lane` (footwork) invites a mistake; renaming the footwork tool
  (`footwork`) is the cleaner way.
- **Standing shares carried out by code** (§6): allowed as the commander's rule, or must each gift be a call?
- **Cells or drawn lines** for a lane's ground: cells are coarse on a small map and on a map whose lanes bend.
- **Who declares in a game without a commander** (one player for three seats): the player, with the same tool, or
  nobody.
- **A lane seat's hands and a fight just over the lane's edge**: drafted as "near any actor of its own is always
  shown"; whether its neighbour lane should be shown party by party is for the first game.
- **What a people's team does here**: in a game beside people the lanes are theirs to call in chat; the commander
  would declare what they said.
