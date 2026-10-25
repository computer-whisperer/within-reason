# The territory bot off Comet: regions, territory as an input, builders matched to located work (proposal, 2026-10-25)

A proposal, not built. The user, 2026-10-25: "Let's work on making this more general -- the point was to start with
building a bot that can ramp economy effectively, then teach it to defend territory, then attack. How canned is the
current economy growth build? We will need to find a way to let it adapt to different terrain and allowed territory
expansion, so it may need a deep redesign." Then "Run the baseline with those maps", "Start with the placement and
enemy-location fixes" (built, 983ab1b), and "let's write that redesign document".

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

A script runs them and a reader prints per start (both to be written at step 0; this session ran them from
throwaway scripts): the marks at 5, 10, 15 and 20, the deaths to his weapons, the plant and advanced plant times,
the factory sites, the commander's range, and the planner's forecast from the start under the bot's own settings.

- **Targets.** On Comet from his start, his curve (62 / 213 / 651 / 1,706) stays the target. Elsewhere there is no
  record of a strong player against InactiveAI, so the planner's own forecast from the start is the yardstick. With
  regions it becomes a fair one, since it then sees each start's ground.
- **Noise.** Measured so far: on Comet ±4-5 at 5:00 and ±60 at 20:00 per game. On Glitters the same build read 310
  and 961 at 20:00 in one seed (terr117, terr120), the late float. Step 0 measures every start's spread with one
  repeat.
- **What counts.** A change counts when it moves the bench without moving any start outside its spread, not when it
  moves Comet alone.

## Steps

Each step one bench run (two where a lever is new), its ledger rows, its register rows; what it deletes, deleted.

0. **The bench.** The script and the reader, and one repeat of the current state to measure each start's spread. No
   change to the bot.
1. **Regions computed and logged.** A `regions:` line at the start: each region's spots, metal, centre, walking times
   and status. Offline, `buildorder lp --hello DUMP --regions`, to show the two Comet starts now price apart and the
   program's 5:00 on each. No change to the bot's decisions.
2. **The planner over regions.** Extractor columns per region, prices derived, the commander's columns on its
   regions. Deleted: `walk_extractor`, `walk_late`, `walk_late_from`, `walk_upgrade`, `advanced_walk_extractor`,
   `rings`, `ring_overhead`, `com_leash` and `commander_walk_*` as arms. The hands take the region's spot.
3. **The lag instrument.** Lags logged at every dispatch, one bench run, under the current cascade.
4. **The hands rebuilt as matching.** The cascade's ranking and fallbacks deleted, the lag rule in their place.
   `BESIDE`, `FACTORY_PLACER` and `plant_helpers_from` become walk times.
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

## Open for the user

- **Order.** I recommend the planner over regions first (steps 1-2): it is the root, since the planner cannot see the
  map, and the hands' matching reads its regions. The other order, hands first (steps 3-4), would go after the
  measured losses sooner: the turrets on Comet, and the float on the windy maps.
- **The stage-1 territory policy on the bench.** Every region not dead (today's ceiling against NullAI), or our half
  by walking distance (closer to a real 1v1).
- **Targets off Comet.** The planner's own forecast as the yardstick, or a survey of public replays for strong
  players' economy curves on these maps (the replay survey tools exist, though those games are against people, not
  InactiveAI).
- **Region size.** By walking time (about 20 s of a constructor), or by spot count.
- **Weight.** Comet from his start as the primary target with the other starts as guards, or all six equal.
