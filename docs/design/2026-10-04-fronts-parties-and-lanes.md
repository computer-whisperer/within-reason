# Fronts: bounded parties, lanes and seat roles

Status: draft 3, 2026-10-05 (draft 2: transfers kept and the share slider, at the user's word; draft 3: §3 rewritten
on its measurement). Written at the user's word after glitters-8v8-hard-1 and the human 8v8 replay; nothing
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

Measured 2026-10-05 (`run/group_shape.py`: groups of 20 soldiers or more at every second the hands logged; the body
is the members left when those the picture called "on the way to join" are taken out, the farthest from the group's
place first, an estimate since the log does not name them):

| game | group-seconds | soldiers on the way | body wider than 600 | body's median width | four fifths of the body within (median) | that over 600 | parted | two fights |
|---|---|---|---|---|---|---|---|---|
| glitters-8v8-hard-1 | 2,160 | 5% | 88% | 1,105 | 380 | 22% | 9% | 5% |
| same-opus-2 (duel) | 441 | 36% | 100% | 1,872 | 522 | 40% | 14% | 12% |
| going-2 (duel) | 1,235 | 10% | 84% | 960 | 332 | 11% | 12% | 2% |
| kept-1 (duel) | 140 | 36% | 96% | 1,303 | 349 | 18% | 10% | 0% |

"Four fifths within": the distance from the middle of the body's largest clump that holds four fifths of the body.
"Parted": a second clump of three or more and a fifth of the body, over 600 from the first. "Two fights": a fifth of
the body within 800 of each of two parties more than 1,200 apart.

What it says:

- **The stream is already said.** Draft 1 had this wrong: a group's entry carries `reinforcements` ("17 of its
  soldiers are on the way to join the body, the farthest 2,100 behind (from 17 from plant_3); they are not in its
  fights yet", H-HANDS-GROUP-BODY), and those soldiers are out of the body and its odds. Nothing to build there.
  In the duels a third of a large group's soldiers were on the way at any second; whether they die on the way is
  not measured here.
- **The body is compact and a few stragglers set its length.** Four fifths of a body stands within 330 to 520 of its
  middle, while its front-to-tail length, which is what `strung_out` reads (over 600), is over 600 in 84 to 100% of
  seconds. A flag that is on nearly always says nothing: "its tail is strung out behind its front" is in nearly
  every large group's words, and the odds of a strung-out group are priced on "the part near the party" nearly
  always.
- **A body in two real pieces is a tenth of the time, and in two fights less.** Worth saying when it happens; not
  where most of the fault is.

Proposed, in the order of what the measure supports:

1. **The body's length is read on its four fifths.** The farthest fifth from the body's place are stragglers: said
   as such ("5 stragglers up to 1,400 behind"), left out of the length and of `strung_out`, kept in the group. By
   the table `strung_out` then holds for 11 to 40% of a large group's seconds in place of 84 to 100%. A change to
   what is said and to which soldiers the odds are priced on, as the joiners already are; no move is chosen.
2. **A parted body is said as two parts**, each with its count and place, and each priced against the party it
   stands near. After 1, and only if games show the tenth of the time mattering.

Not proposed: a cap on a group's size, or the code cutting a group. The groups are the player's.

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
- **Feeding**: each seat's share levels for metal and energy and who receives the excess ("every lane seat shares
  metal above 30% of its store to t6"). §6.

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

Ruled 2026-10-04 (the user): **transfers stay, and are the way the back seats feed the front.** A large game becomes
four or five lanes, each a forward seat's; the seats behind develop economy or tech and give their units to the
forward seat of a lane. So a lane's army is one seat's, and it grows by gifts. (Draft 1 proposed each seat keeping
its own army and no transfers of bodies; rejected because that is not how the game is played at this size.)

What follows for the build: the forward seat's hands hold a lane's whole army, and the gates of
glitters-8v8-hard-1 grew with the square of what a seat held after its twelve transfers. The lane view (a seat's
hands see its lane's parties, not the map's) and the bounded parties are what must hold that size; the measure in
"Cost" below is done on a seat after transfers, before step 3 is built. A given unit joins the group the forward
seat's player names for arrivals, as a plant's new units do, so that a gift is not a new actor.

### Cost

The hands' context of a lane seat stops growing with the side: its parties and places are its lane's. Unmeasured;
the offline measure is the 8v8's gates rebuilt with the parties and places of other lanes cut, counted in tokens,
and the usual replay for whether the answers move.

## 5. Duels

No lanes: one seat, the whole map, and nothing of §4 is shown. §2 and §3 apply as they are; the duel's measured gain
is the steadier names (1.0% of units renamed a second against 3.5%).

## 6. Feeding

The players' mechanic is the share slider: a level on each of a team's two stores, and whatever the store holds above
that level goes to allies with room. What the engine has (Recoil 2026.07.04, read in source, not yet tried):

- Each team has a share level for metal and for energy, a fraction of its storage; every slow update the excess above
  it is spread over the allies whose stores are under 99% (`CTeam::SlowUpdate`). The default is 0.99 for metal and
  0.95 for energy, so a seat of ours already overflows to its allies when it is full.
- A person's slider sets it by a net message (`Spring.SetShareLevel`, unsynced Lua). A gadget can set any team's
  (`Spring.SetTeamShareLevel`, synced Lua). **The skirmish-AI interface has no command for it**: it can read the
  level (`Economy_getShare`) and send an amount (`COMMAND_SEND_RESOURCES`, which our `transfer` uses), nothing more.
- The game's `tax_resource_sharing_amount` and `easytax` options tax what is shared, by either road.

So for a seat of ours the slider is rebuilt on our side: a level for metal and for energy, set by the commander in
`lanes` (or by a seat's player for its own seat), and the bot sends what its store holds above the level
to the seats named, or to every ally with room when none is named. A level is the same kind of thing as a person's
slider: a setting the agent chose, carried out mechanically. Naming who receives is more than the engine's slider
gives a person and is what lets a side feed its tech seat.

A send shows as a popup to everyone watching and in the replay (the user, 2026-10-04), so the bot does not send each
second: the excess is gathered and sent as one gift when it passes an amount or an interval, both to be set with
the first trial.

To check in the engine before building: that the default overflow reaches allied seats of ours as the source says
(the records' `m` and `e` rows of glitters-8v8-hard-1 would show it), and what a send to a full ally does.

## 7. Steps

Each is played before the next is started.

1. **Parties kept and fights weighed by neighbourhood** (§2.1, §2.2), as layers that can be switched (`kept`,
   `beside`; built 2026-10-04). The old rule's parties need no log of their own: `run/party_rules.py` forms them
   from the game's record. Test: the standing duel, then a 3-seat game; the
   forbidden and step-back answers at a front, read by a person.
2. **The body's length read on its four fifths, stragglers said** (§3.1; measured 2026-10-05; built the same day
   as the layer `stragglers`: of the farthest fifth, those over 600 from the body's place), then the parted body
   if games ask for it.
3. **`lanes` with lanes and roles; the lane line in the commander's and players' reports; the hands' picture by
   lane.** Test: glitters 8v8 against the same opponent, beside glitters-8v8-hard-1: transfers of units, gate sizes
   by seat, extractors by seat at 10:00 and 20:00, Jev's bill.
4. **Feeding and the air and tech roles' prompts.** Test: the time of the first tier-2 plant on the side.

## 8. Open

- **The tool's name**: `lanes` beside the player's `lane` (footwork) invites a mistake; renaming the footwork tool
  (`footwork`) is the cleaner way.
- **Share levels carried out by the bot** (§6): proposed as a setting, like a person's slider; the user's to confirm under the ruling on code decisions.
- **Cells or drawn lines** for a lane's ground: cells are coarse on a small map and on a map whose lanes bend.
- **Who declares in a game without a commander** (one player for three seats): the player, with the same tool, or
  nobody.
- **A lane seat's hands and a fight just over the lane's edge**: drafted as "near any actor of its own is always
  shown"; whether its neighbour lane should be shown party by party is for the first game.
- **What a people's team does here**: in a game beside people the lanes are theirs to call in chat; the commander
  would declare what they said.
