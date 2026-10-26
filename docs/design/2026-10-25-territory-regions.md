# The territory bot off Comet: regions, territory as an input, builders matched to located work (proposal, 2026-10-25)

Accepted 2026-10-25 ("Decided" at the end), built step by step (status under "Steps"). The user, 2026-10-25: "Let's work
on making this more general -- the point was to start with building a bot that can ramp economy effectively, then teach
it to defend territory, then attack. How canned is the current economy growth build? We will need to find a way to let
it adapt to different terrain and allowed territory expansion, so it may need a deep redesign." Then "Run the baseline
with those maps", "Start with the placement and enemy-location fixes" (built, 983ab1b), and "let's write that redesign
document".

The planner stays a linear program over the simulator's accounting (`docs/design/2026-10-20-territory-lp.md`). What
changes is how space enters it, how territory enters it, and how the hands turn its plan into orders. Three parts:

1. **Regions.** The map's metal spots grouped at the start of the game into regions of a few spots each. Each region
   carries walking times measured on the map's own ground. The planner prices extractors per region instead of with
   one walk constant for the whole map.
2. **Territory as an input.** Each region has a status: ours to take, his, dead (in his commander's reach), or allowed
   from some minute. Today's filter on spots becomes that status. Defending (stage 2) becomes cover the planner buys
   per region, and attacking (stage 3) becomes value on his regions.
3. **Hands that match builders to located work.** Every dispatch, an idle builder takes the piece of the plan's work
   that is furthest behind the plan, less the walk to it. This replaces the twelve-rule cascade.

## Why: what the baseline showed

The bot is not a scripted build order: no rule names a unit, and the roster, spots and terrain come from the game.
It is fitted. Every batch from terr1 to terr102 ran on Comet Catcher against an opponent that builds nothing, and
from terr85 almost always from thebluegecko's start. The cross-map baseline (terr103-107) and the fixes that followed
(terr108-132) measured what that fitting costs. Metal income a second at 5:00 and 20:00, four seeds a start, and
the planner's own ceiling from each start as a yardstick (`buildorder lp --hello`, every spot, its default settings):

| start | 5:00 baseline | 5:00 now | 20:00 baseline | 20:00 now | share of ceiling at 20:00 now |
|---|---|---|---|---|---|
| Comet, thebluegecko's start | 47.9-54.8 | 47.9-52.5 | 979-1,066 | 1,056-1,107 | 0.58-0.61 |
| Comet, the map's own starts | 34.1-41.0 | 36.4-41.0 | 895-925 | 935-986 | 0.50-0.54 |
| Great Divide | 30.4-30.5 | 26.0-32.6 | 267-307 | 277-371 | 0.24-0.34 |
| All That Glitters | 11.2-31.4 | 25.9-27.7 | 11-310 | 870-995 | 0.44-0.51 |
| Mithril Mountain | 34.0-42.0 | 32.5-40.0 | 147-579 | 512-582 | 0.37-0.43 |
| Quicksilver | 30.0-35.9 | 32.0-33.9 | 433-499 | 382-426 | 0.29-0.32 |

"Now" is terr127-132, after the placement and enemy-location fixes (`bots/territory/README.md` T-SITES,
T-SCENARIO). Those fixes removed the hard failures: a plant facing a rock, an advanced plant on a plateau no vehicle
could climb, his start taken as his box's centre, the commanders' duel, the bot feeding constructors to his
commander. What remains is the fitting itself, in three places.

**The planner cannot see the map.** All free spots are one pool, with a count and a mean metal. Every extractor costs
a constructor a fixed 15 s of walking, 40 s after 10:00. Both numbers were measured from our own Comet games. So the
planner prices thebluegecko's start and Comet's lobby start identically, to the unit: 686,584 metal made from either.
Yet the lobby start has 22 spots within 3,000 against 28, and from it 5:00 is 34-41 against 48-55. A map whose spots
lie twice as far apart would be priced as if extractors cost half their real walk. An earlier attempt at distance
rings (terr31-33) priced each extractor as a trip from home. The hands chain spot to spot, so the realized walk was
a fraction of that. That note's conclusion stands: "a ring model that pays would price the step between neighbouring
spots, not the trip from home." Regions are that model.

**Territory is a yes/no filter on spots.** A spot is in play or not: every spot, or our half by straight-line
mirror, less those within 1,000 of his start or of an armed unit of his in sight. Nothing says when ground becomes
reachable, how contested it is, or what it is worth holding. There is nowhere to put defence or attack.

**The hands are a cascade fitted on Comet's failures.** An idle builder walks a fixed sequence of about twelve rules.
They are the opening, a factory commitment, the factory's class pick, help on a plant frame, the plan's flows largest
first, the far constructor's chain, an energy class, a count fallback, the dearest frame, a free energy building, a
walk home and a yard clear. Each rule was added to fix a failure seen on Comet, and the cascade carries about fifty
tuned constants. Its last change (terr102, the far fallbacks) measured level with the one before. It also starves
whatever the plan wants least by power-seconds: the turrets at 10:00-12:00 (terr102), and the windy maps' late game,
where the metal sat at its cap for eleven minutes while the hands ordered 51-72 wind generators a minute (terr120
game 00; terr106 game 00 the same, twelve minutes at 45-51 a minute).

### The settings fitted to Comet, and what each becomes

| setting | value | where it came from | under this design |
|---|---|---|---|
| `walk_extractor`, `walk_late`, `walk_late_from` | 15 s, 40 s from 10:00 | the hands' realized chain on Comet (terr73, terr83) | derived per region: the step between its spots, plus the trip to it shared over its spots |
| `walk_upgrade`, `advanced_walk_extractor` | 40 s; mean free-spot distance | Comet's plant at home (terr97) | derived per region from the plant |
| `rings`, `ring_overhead` | 1, 5 s | the rings' failure (terr31-33) | deleted; the overhead measured from the `begun:` lines |
| `com_leash`, `opening_spots`, `commander_walk_*` | 700, 600, 5 / 9 s | his start's spots at 166, 251 and 537 | the commander's columns on home's region and its neighbours, priced at its own walk |
| `plant_helpers_from`, `FACTORY_PLACER`, `BESIDE` | 4,000, 1,000, 600 | Comet's distances | walk times, or region membership ("out" means not in the base region) |
| `line` | vehicle | thebluegecko's line on Comet | later: a choice per map, since regions carry walks per movement class and bots climb what vehicles cannot |
| `max_nanos`, `help_factor` | 300, 40 | his 311 turrets | later: the farm's capacity counted from the base's ground |
| `DEAD_SPOT` | 1,000 | the user's ruling against NullAI | stays, as a region status |
| `mobile_help` | 0.35 | his builders' share on the moho frames | stays: a ratio of play, not of the map |

## 1. Regions

**What a region is.** A set of metal spots a constructor works as one trip: walk there, then step from spot to spot.
Each region carries:

- its spots, their count and their metal;
- a centre (the medoid spot) and the walking time to it from home, per movement class (vehicle, bot, the commander's);
- its chain step: the walking time from one of its spots to the next, at the class's speed;
- its walking time to each neighbouring region;
- its status (part 2).

The walking times come from the walking fields the bot already builds from the terrain (`terrain::Field`, one per
movement class; `buildorder::game::Walked`). Straight lines are wrong on Quicksilver and Great Divide.

**Not natural clusters.** On maps whose spots are spread evenly there are none. Linking spots within 700 elmos of each
other already chains 30 of Comet's spots and 113 of Glitters' into one group, and within 1,000 all of Comet becomes
one. So regions are a partition with a bounded size. Take the unassigned spot nearest home, gather the unassigned
spots within a walking radius of it, and repeat. Measured offline on the five maps with straight distances and a
radius of 1,200, that gives 5-30 regions of a median 2-5 spots and at most 10, home's own region holding 5-9. A
radius of 800 gives 9-58 regions of a median 2-3 spots and at most 5. The radius should be a walking time, not a
distance: about 20 s of a basic vehicle constructor's walk, some 1,100 elmos at its 54 a second, near the 1,200
tried here. The regions are computed once, at the start; his start and the dead radius mark them, they do not
reshape them.

**The base region.** Home and the farm are a region too. Its ground carries the building slots the farm already
counts (T-FARM). That is what later replaces `max_nanos`: the base's ground made visible to the planner.

## 2. The planner over regions

**Extractor columns per region.** Today an extractor column is one ring holding every spot, priced at one walk. It
becomes one column per region and extractor kind (basic, upgrade). Each column carries the region's spot count as
its cap, its metal, and a price of

    walk[b, r] = step[b, r] + trip[b, r] / spots[r]

seconds a builder of type `b` loses per extractor in region `r`. The trip from where builders of that type are made
(home, or the advanced plant for the upgrades) is shared over the region's spots, as a constructor that walks out
clears the region. This is the step-plus-trip shape the rings lacked. The near regions are cheap, the far ones dear,
and two starts on one map now price differently. The cost is size. With 30 regions, two extractor kinds and three
builder types, the program grows by about 5,000 variables over 29 buckets. It has about 5,000 today, and the bot's
re-plans take a median 94-124 ms (terr127) against a budget of 400. If the larger program does not fit, the regions
beyond what the hands can reach before the horizon merge back into one column at their mean price.

**The commander.** Its extractor columns are home's region and the regions within its reach, priced at its own walk
and speed. That replaces `com_leash`, `opening_spots` and the commander's walk settings. His opening, three spots
then the solars then the plant, should fall out of home's region being cheap and the plant's commitment (T-COMMIT).
If it does not, the opening rule (T-OPENING) stays until it does. That is decided by the 5:00 marks, which the user
has ruled the ones that matter most.

**What the hands get from it.** A flow on a region's column is an extractor in that region. The hands' spot choice
(T-SITES) takes the region's next free spot, nearest the builder, as it takes the ring's today.

## 3. Territory as an input

Each region carries a status the planner reads as data:

- **ours**: its spots are in play from now;
- **from minute m**: in play from the bucket containing `m` (an expansion schedule, or a teammate's slice);
- **dead**: the spots within 1,000 of his start or of an armed unit of his in sight, per spot as today (the user's
  ruling); a region whose spots are all dead is dead;
- **his**: not in play.

The stage-1 policy says which regions are ours. It can be every region not dead, as today against NullAI. It can be
our half by walking distance: the regions nearer our start than his on foot, not a mirror line. Or it can be a
schedule. The bench runs one policy; the arm picks it.

**Defence (stage 2)** builds on the formulation already written in the planner's design: a region's income becomes
its income less its expected loss. The loss is a variable bounded below by the income less the cover, and cover is a
concave, piecewise-linear function of the turrets and soldiers stationed in the region. Regions are what makes that
term tractable. Cover stations at a region, not at eighty spots, and his reach is read per region from what we have
seen of him. Turrets and soldiers then enter the plan as units whose only worth is the income they protect.

**Attack (stage 3)** is value on his regions: income denied, read the same way.

## 4. The hands: builders matched to located work

**Work items.** Every dispatch, the plan's flows to `now + lead` become work items with a place:

- an extractor in region `r`, at its next free spot;
- a frame to help, where the frame stands;
- a building on the farm, at its slot;
- a guard on a factory, at the factory.

Each item carries its column's lag. The lag is how far the hands' commitment on the column runs behind the plan, in
seconds of the plan's own rate on it:

    lag[w] = (planned[w] to now + lead  -  committed[w]) / planned rate[w]

A column the plan does not run over the lead has no items.

**The rule.** An idle builder takes the item it can build with the largest `lag[w] - travel[builder, w]`: the work
furthest behind its plan, net of the walk to it. Two properties follow.

- Every column is served in proportion to how far behind it is, not by its size. Today the plan's largest flows by
  power-seconds go first, so the plant's guards and the moho help outrank eleven turrets a minute until the spots run
  out (terr102, read from the code and the per-minute job lines). Under the lag rule eleven turrets a minute behind
  is as urgent as a guard a minute behind.
- The walk is counted from where the builder stands. The far constructor's chain (T-CHAIN), the count fallback's walk
  limit and the walk home (T-DISPATCH, terr101-102) all become the travel term. A constructor out among the spots
  finds the next spot of its region nearest; one with nothing near finds the base's work worth the walk.

**What of the cascade stays.** Only the rules that encode the relaxation's integrality or a mechanical need:

- the factory commitment (T-COMMIT): a factory is started whole, since the plan holds fractions of one;
- the class pick for a factory's products;
- the energy budget (T-ENERGY);
- the plant frame's helpers, as an item with its own cap;
- clearing the yard (T-YARD).

The opening rule stays until part 2 makes it redundant, and the energy class (a stall's solar collectors ahead of
everything) until the lag rule has been measured through a stall.

**What goes:** the flows' ranking, the count fallback, the dearest frame as a fallback, `chain_on`, the walk home,
and the free energy building. These are deleted first and the rule built in their place, not morphed into it.
Before the rule is built, one bench run logs each item's lag at every dispatch. That is the instrument the turret
diagnosis lacked: it read the ranking from the code, not from the game.

## The bench

Six starts, four seeds each, 24 games, about six minutes:

- Comet from thebluegecko's start (`--starts 941,2534:6899,684`);
- Comet from the map's own starts;
- Great Divide;
- All That Glitters;
- Mithril Mountain;
- Quicksilver.

`run/territory_bench.sh FIRST TAG` runs them, one batch a start labelled `terrN-TAG-START`, and
`run/territory_bench.py TAG...` prints per start: the marks at 5, 10, 15 and 20, the 20:00 mark as a share of the
planner's forecast, the deaths to his weapons, the plant and advanced plant times, the factory sites, the commander's
farthest from home, and the planner's forecast from the start under the bot's own settings (the first plan's
`targets:` line). Given several tags it reads the last against the pooled games of the others.

- **Targets.** On Comet from his start, his curve (62 / 213 / 651 / 1,706) stays the target. Elsewhere there is no
  record of a strong player against InactiveAI, so the planner's own forecast from the start is the yardstick. With
  regions it becomes a fair one, since it then sees each start's ground.
- **Noise.** Measured at step 0 (terr133-138: the standing build again, on terr127-132's seeds). The repeat's median
  of four left the first run's range of four at 9 of the 24 start-marks, and moved by up to 1.8 at 5:00, 9 at 10:00,
  47 at 15:00 and 65 at 20:00. One game of the same seed moved by up to 134 at 20:00 (Comet from his start, seed 4:
  1,080 and 946). One run's four games are not the spread; the pooled games of every run of the standing build are.
- **What counts.** A change counts when it moves the bench and no start's median falls below the pooled range of
  the standing build's games, not when it moves Comet alone. At four seeds a start's median has to move more than
  the run-to-run gaps above to be read as moved at all; the 5:00 and 10:00 marks are the sharp ones.

## Steps

Each step one bench run (two where a lever is new), its ledger rows, its register rows; what it deletes, deleted.

0. **The bench.** Done (terr133-138): the script and the reader, and one repeat of the standing build, which
   measured the spread above. No change to the bot.
1. **Regions computed and logged.** Done (terr139-144, T-REGIONS): a `regions:` line at the start, each region's
   spots, metal, centre, tour and the seconds an extractor there costs each builder type; offline,
   `buildorder regions --hello DUMP`. The walks between regions and the status wait for the steps that read them (4
   and 5). No change to the bot's decisions, and the bench inside the pooled range.
2. **The planner over regions.** Extractor columns per region, prices derived, the commander's columns on its
   regions. Offline first, the two Comet starts priced apart and the program's 5:00 on each, before the bench.
   Measured in two forms, not kept as it stands (terr145-156; the code on branch `territory-regions-step2`,
   2eab7f8). Offline the regions do not price the two Comet starts apart (51 and 50 at 5:00 in the first plan),
   and the program is two to three times the size (93-345 ms a plan against 100-177). The first form, the
   commander given the home region to 1,250 and trips from home, fell under the pooled range on both Comet
   starts at 5:00-15:00: the commander left the plant's guard at 2:00, and the far regions priced at the trip
   from home cost about twice their realized walk. The second, the commander's extractors within 700 and trips
   from the nearest region held, holds 5:00 at every start; Comet from his start at 15:00, Mithril at 15:00-20:00
   and Quicksilver at 10:00-15:00 fall under it, the first plans alike in shape and lower from 8:00. The
   realized walk per region (`run/territory_region_walks.py`) against the price: 0.79-0.93 on Comet and
   Quicksilver, 1.14-1.30 on Mithril and Great Divide, where the tour, walked on the commander's ground, misses
   the vehicles' detours. The decision on how to go on is the user's. Deleted: `walk_extractor`, `walk_late`, `walk_late_from`, `walk_upgrade`, `advanced_walk_extractor`,
   `rings`, `ring_overhead`, `com_leash` and `commander_walk_*` as arms. The hands take the region's spot.
3. **The lag instrument.** Lags logged at every dispatch, one bench run, under the current cascade. Done
   (e13bcd2, T-LAGS, terr157-162): the turrets the column most behind on every start, at 5:00 and again late (14-73
   units on Comet from his start at 15:00-19:00, while the cascade built metal makers and walked constructors home);
   the cascade's order the rule's pick in 2-50 % of orders, under 30 % in most start-minutes.
4. **The hands rebuilt as matching.** The cascade's ranking and fallbacks deleted, the lag rule in their place.
   `BESIDE`, `FACTORY_PLACER` and `plant_helpers_from` become walk times. Measured in two forms, not kept
   (terr163-174; the code was branch `territory-matching-step4`, f97c321, dropped 2026-10-25 on the user's word). Both under the standing build at 5:00 on
   every start and at 10:00 on five, inside at 20:00 on five (Comet from his start at its top); from 10:00 the hands
   keep every column within seconds of the plan, where the cascade let the turrets fall tens of units behind. The
   opening's loss is in the accounting: an extractor order charges its whole cost, the walk included, when given, so
   the extractor column reads on plan or ahead while the solars it shares with the commander read behind, and the
   constructors go to solars and turrets (Glitters game 00: 7 extractors from 4:00 to 5:00 with 8 constructors and
   the metal spent). The second form put the cascade's opening window back (to 3:00, constructors on extractor items
   only); it did not bring the openings back. The decision on how to go on is the user's.
5. **Territory as an input.** Region status, the stage-1 policies, the dead radius as a status.
6. **The opening.** T-OPENING deleted, if the plan over home's region reproduces it at 5:00 on the bench.
7. **Later.** The factory line per map; the farm's capacity from the base's ground (`max_nanos`); then stage 2.

## What could go wrong

- **Size.** Thirty regions may push a re-plan past its 400 ms budget. The answer is merging the regions the hands
  cannot reach before the horizon; measured at step 1 offline before the bot carries it.
- **The trip shared over a region's spots is a guess** about how constructors split regions. Two constructors in one
  region pay the trip twice. The realized walk per region, from the `begun:` lines, is the check, as the ring's was.
- **The lag rule could break the opening.** The cascade's first rules encode his opening, which took ten batches
  to fit (terr86-95). Step 4 keeps T-OPENING and T-COMMIT untouched, and the 5:00 marks on all six starts gate it.
- **The bench is noisy where the late game floats.** Glitters' 310-995 spread needs either repeats or the float
  fixed. The float is the hands' late economy, which step 4 is meant to address, so the spread should narrow after it.
- **Regions could be wrong in shape.** A radius by walking time can still cut across a chokepoint. The `regions:` line
  and the viewer let us look before the planner relies on them.

## Decided

The user, 2026-10-25, on the questions this section asked: "Your leans sound reasonable, so lets begin."

- **Order.** The planner over regions first (steps 1-2), then the lag instrument and the hands (steps 3-4).
  Reopened after step 2's two forms (the user, 2026-10-25: "Agreed with hands first, regions kept -- proceed"):
  the hands next (steps 3-4), the regions kept (step 1 computes them for the hands' located work), the program
  over regions held on branch `territory-regions-step2` until the hands follow a plan well enough that an honest
  one should pay. Chosen because the lag instrument measures where the middle game loses under either program, and
  the honest program's lower plans were followed into lower games (terr151-156); planner-first rejected for now
  because two forms of it fell under the bench at 10:00-20:00 on three starts.
- **The stage-1 territory policy on the bench.** Every region not dead, as today against NullAI. The proposal stated
  no lean on this one; this one keeps the bench comparable with every batch before it, and our half by walking
  distance enters as an arm at step 5.
- **Targets off Comet.** The planner's own forecast from the start under the bot's settings (the first plan's
  `targets:` line). On Comet from his start his curve stays the target.
- **Region size.** By walking time: about 20 s of a basic constructor's walk.
- **Weight.** All six starts by the rule under "What counts": a change counts when it moves the bench and no start
  falls below its spread. Comet from his start is not weighted above the others.
