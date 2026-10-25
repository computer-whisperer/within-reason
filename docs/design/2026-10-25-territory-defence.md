# The territory bot's defence: territory as a race of force (proposal, 2026-10-25)

Accepted 2026-10-25. The user's model, in his words: "the basic model I had in mind is a projection of defence force.
If you start from a basic model of valuing combat units by their metal (with a multiplier for static defences), then
we can model our claimed territory by how much of our material can reach (movement plus attack range) a given point
on the map before an enemy force of a certain size can do the same from fog of war. With that basic model we can
figure out how many units we need to build as we expand and how to keep reasonably efficient defences up for our
perimeter. Whether to have defence or construction lead our territory claims is an open question." On the
refinements and the order below: "Agreed, lets write this up and try it."

The economy stage is closed (`docs/design/2026-10-25-territory-walks.md`; the standing build benched as terr202-207
against NullAI). This is stage 2 of the regions design (`docs/design/2026-10-25-territory-regions.md`, part 3), which
already says how defence enters the planner: a region's income less its expected loss, the loss reduced by cover.
It never said what cover is. The race says it.

## Where the bot starts from

Nothing. The territory bot builds no soldiers and no turret for defence (the factory palette is constructors only
unless the `soldiers` arm is on, and the program values nothing but metal made). It knows his start and his armed
units in sight, which make the ground within 1,000 of them dead to its builders (T-SCENARIO). Its commander walks out to the
spots near home in the opening. Its deaths are logged (`died:` lines, with the attacker), and every match's
`truth-N.jsonl` has his units (id, name, position, health, frame or not) every 2 s.

## The model

**Currency.** A combat unit is worth its metal. A static defence is worth its metal times a multiplier: it cannot
move, but inside its reach it answers at once, and it is cheap for its gun. The multipliers are measured, not
guessed: the fight simulator (`crates/combatsim`) says what one light laser turret is worth in Pawns or Flashes,
once per pair of classes, not per decision.

**The race.** For a place on the map, two clocks:

- *his arrival*: the seconds his force needs to bring its weapons onto what stands there, starting from where we
  cannot see;
- *our response*: the seconds until enough of our armed metal is in reach of the place to meet that force.

What a place risks is what stands there times the share his force kills before our response arrives. Not who arrives
first: a Flash (101 elmos a second, reach 180) kills a 50-metal extractor in a few seconds, so help that comes 15 s
late has lost it. A turret's response is zero inside its reach (430 for the Armada's light laser turret), which is
most of the static multiplier falling out of the model by itself. Our territory is where the risk is small; the
contested ground is where the clocks are close; his is where he answers first.

**His force starts at the edge of our sight.** Not anywhere in the fog: our buildings see 270-490 elmos around them,
3 to 5 s of a Flash's run; a radar (60 metal) sees 2,100, some 20 s. So scouting and radar are defence in this model,
bought for the warning they add, which is how players use them. The edge is bounded by where his army could have got
since we last saw it; without that bound all fog counts and everything is contested. (The built form bounds it by
connection: the uncovered ground joined to his start's, or with his start in sight the largest piece of uncovered
ground. Without it the holes our radar leaves inside its own coverage counted, and the warning stayed at 1-6 s with
94-98 % of the map covered, terr238-243.)

**The threat is two things.**

- *Raids*: small, fast, early, at the edges. Turrets beat them on cost.
- *Pushes*: large, slow, aimed at the base. They need the mobile army.

Their sizes over time, S(t), start as a measured curve per opponent: the size of his armed groups that enter our
ground, read from the truth files.

**Mobile metal counts once; turrets count everywhere.** An army answers one raid at a time, and two raids on opposite
edges is how players beat a defence that is all army. So the army is a few stations, each answering for the places
within its response time. The perimeter's length sets how many stations; each region added at the edge adds its share
of one. That is the user's "how many units we need to build as we expand".

**Who leads.** Priced, not ruled. Cheap things can lead into contested ground: a basic extractor is 50 metal and pays
back quickly, so losing a few is cheap. Dear things go only where the ground is held: an advanced extractor (620),
the turret farm, a constructor walking out (135). Defence leads where the stakes are high, construction where they
are low. In the planner that is the loss term: a region's income is worth its income less its expected loss, and
soldiers and turrets are worth the loss they remove.

## What the game gives the model

- **Our sight**: the engine's own coverage, cut by the ground, which the host sends once a second (`Snapshot::sensors`:
  sight in 64-elmo squares, radar in 32). Radar counts: it shows his units as dots, which is warning enough.
- **The walks**: `terrain::Field` per movement class over the 16-elmo grid. One field from the edge of our sight for
  his approach; one per group of our armed mobile units for our response. Turrets need no field.
- **His speed and ground**: the first form walks his approach on the commanders' ground (a bot's, more permissive
  than his vehicles') at a fixed raid speed of 100 and reach of 180 (a Flash; an Incisor is alike). His seen units
  replace that when the instrument shows it matters.
- **Armed**: a unit with a weapon (`weapon_count` above zero, `reach` above zero) that is not a constructor; the
  commander counts, at its metal.
- **The regions**: the map's spots in regions of about 20 s of a constructor's walk (T-REGIONS). What stands of ours
  belongs to the region of its nearest spot, so home and the farm fall in home's region.

## The bench

NullAI cannot measure defence. The defence bench is the economy bench's six starts and four seeds against BARb
medium (`run/territory_bench.sh FIRST TAG --opponent BARb:stable --profile medium`), to 20:00, played out. BARb medium
raids, and our distilled bot beats it 8-0 (the `distilled-v0` rows), so a bot that survives it exists. What it reads:

- the outcome and when (our commander's death ends the game);
- income at 5, 10, 15 and 20 minutes while alive, as on the economy bench;
- metal lost to his weapons, by type and minute, and by region;
- his army's metal over time and the size of his groups in our ground (the truth files);
- the promise kept (T-PROMISE), now with losses in it.

The economy bench against NullAI stays: a defence change must not cost the economy where nobody attacks.

## Steps

Each step its bench rows and its register rows; what it replaces, deleted.

1. **The baseline.** Today's bot against BARb medium on the six starts (terr208-213). No change to the bot. Done:
   18 of 24 lost, 6 to the cap (below, "Measured").
2. **The race as an instrument.** Built (T-RACE, 6878eb7) and benched (terr214-219); the kill test as written does
   not separate held from open, and cannot yet (below). A `race:` line every 30 s: per region, the metal standing,
   his arrival (warning) in seconds, and our armed metal in reach by then and 15, 30 and 60 s later. A reader (`run/territory_race.py`)
   sets it beside the deaths and his groups from the truth files. Deciding nothing. The kill test: do the losses
   land where the race says the ground is open, and do they spare the regions it says are held? If losses are no
   likelier in the open regions than in the held ones, the model is wrong and nothing gets built on it. With no army
   the commander is the only response, so step 2 tests the threat side most (warning times, his group sizes, where
   he goes); the response side is tested again at step 3.
3. **Radar and turrets, first form** (below, "Step 3: radar and turrets"), then **the guard, first form** (below,
   "Step 3b: the guard"): soldiers to the size of his army shown, at the base, answering the raids they match.
   Stations, the guard pushing, and the planner seeing it as a draw on metal and factory time come after it.
4. **Region status from the race.** Ours, contested, his. Construction respects it: cheap work may go into contested
   regions, dear work only into held ones. This is step 5 of the regions design, with the race as its source.
5. **Cover in the planner.** The loss term: the program trades soldiers and turrets against extractors itself.

## Step 3: radar and turrets

The user, 2026-10-25, on the choice between a guard with radar and turrets alone: "Let's start with radar and turrets
alone as the simpler variant. For this part of the project, I think we can consider it a win if we end up with double
the enemy metal income at some point past the opening."

- **The win.** Our metal income twice his at some point from 15:00, each the mean over 30 s, read from the truth file
  (its `income` field, both sides' income through the engine with cheat access; the bot never sees his); a game lost
  before 15:00 is not a win. The reader gives each game's best ratio from 15:00 and when, and the ratio at 5, 10, 15
  and 20 minutes. Amended by the user, 2026-10-25: "Let's make the win line start at 15:00". It started at 5:00, and
  the bot without defence met that in 20 of 24 games (terr226-231): BARb medium makes 10-17 metal a second at 5:00,
  so our income passes twice his before his raids take our economy apart.
- **His raid, S.** What the bot has seen: the largest metal of his armed mobile units in sight within 800 of a
  building of ours, over the last five minutes; his commander at 1,800 and a moving radar dot at 110 (the distilled
  bot's D-WEIGHT), at most 1,000 (a raid; the push of 3,000-4,000 at 10:00 is the guard's). Nothing seen, nothing
  wanted: against NullAI no turret is built.
- **Turrets (T-COVER).** A region holding something of ours wants turrets while the turrets' weight there (their
  metal times 3, D-WEIGHT's multiplier until combatsim measures one) is under S, and while its turrets' metal stays
  under the metal of what they guard: never more turret than what it protects. The next turret goes to the region's
  building with the least turret weight in reach (the one nearest his start on a tie), 120 toward his start. The type
  is the one the builder makes with the most ground fighting strength per metal, √(damage a second × health) / metal
  from combatsim's table (no anti-air turret: it cannot hit the ground): Armada's light laser turret (3.71), Cortex's
  double light laser (3.76), Legion's light turret (4.52).
- **Radar (T-RADAR).** A region holding something of ours whose warning (the race's, T-RACE) is under 8 s, whose
  forward building (the one nearest his start) is out of our radar's coverage, and with no radar standing, framed or
  ordered within 1,000 of that building, gets the cheapest radar the builder makes, 150 behind it toward home.
- **Who builds.** From 3:00, a constructor with no order (the commander too, within its leash) takes the nearest
  wanted site within 1,200 before the plan's flows (after T-COMMIT's factories); one order at a site at a time; not
  within his start's dead radius or with an armed unit of his within 450 (the builders' dead ground, 1,000 of any armed
  unit of his, forbade every site while raided, terr222). The energy budget and the stock's cover hold for these
  orders as for any.
- **The planner** does not see turrets (its palette has none, and their frames are not its columns): a constructor on
  a turret is one it counts free. The promise (T-PROMISE) says what that costs.
- **Measured** on the defence bench, and on the economy bench against NullAI, where the radars alone are built.

## Step 3b: the guard

The user, 2026-10-25, after step 3's results: "Let's make the win line start at 15:00 and start adding mobile guard
units." The games radar and turrets still lose are lost to his army, 3,000-4,000 metal at 10:00, which the turrets'
cover (at most 1,000 a region) is not sized for. The first form is one guard at the base that answers the raids it
can beat, sized to what he has shown.

- **His army shown, A.** Every armed mobile ground unit of his seen in sight (a weapon, a reach, no build menu, a
  movement class: no aircraft, which the guard cannot hit; not his commander) counts its metal for five minutes after
  it was last seen, sight or radar, and not at all once seen destroyed. A radar dot never identified counts nothing.
- **The guard wanted, G**: A, from 3:00 (DEFENCE_FROM, as the turrets). Against NullAI nothing is shown and nothing
  is made.
- **Its type.** Among the products of our standing factories, the mobile ground unit that builds nothing with the most
  ground fighting strength per metal, √(damage a second × health) / metal from combatsim's table, the turrets' measure:
  Armada's Blitz (2.33; the Stout 1.70, the Rover 1.96, the advanced plant's best 1.32), Cortex's Incisor (2.31). Only
  the factories that make it make soldiers, so the advanced plant keeps to the plan's constructors. The Blitz is also
  fast (101), which a guard answering raids across our ground wants.
- **Production.** A factory that makes the type makes one before the plan's products while the guard's metal (its
  soldiers standing, the frames at full cost, and the orders not yet begun) is under G, under the affordability and
  energy tests every order passes. The planner does not see soldiers; the factory time and metal they take show in
  the next plan's snapshot.
- **The base point.** The guard stands 400 from home toward his start, moved on by 200 while it lies in a factory's
  yard or exit lane.
- **The answer.** His units near our buildings as T-COVER sees them (armed mobile units within 800 of a building of
  ours, his commander at 1,800, a moving radar dot at 110), in groups of units within 400 of one another, aircraft
  apart. The guard fights to the group with the most of our building metal within 800 of it, among the groups it
  matches: the guard's metal, plus three times the metal of our turrets with the group's centre in their reach, at
  least the group's weight. With a raid near and none matched, the guard stays at the base point, where the turrets
  and the commander stand with it, and answers nothing for 20 s if it had been answering (a retreat). With no raid
  for 20 s it goes back to the base point.
- **Orders.** A fight order (the engine's attack-move) to the place, to every soldier, given again when the place moves
  by 200 or a soldier stands idle 5 s away from it.
- **The `guard:` line** every 30 s: A and how many units it holds, G, the guard's metal standing and making, the
  soldiers made, and where the guard is and why.
- **The arm** `guard=on|off`, on by default.
- **Measured** on the defence bench against terr256-261 (radar and turrets alone) and on the economy bench against
  NullAI, where nothing should change.
- **Left out of the first form.** Stations away from the base (the race line's response columns now count the guard,
  and say which regions it leaves late); the guard pushing; scouting, so A is what came to us; the planner pricing
  the guard.

## Step 3c: stations

The user, 2026-10-25: "Look into the army-shown overcount first, then stations." The model's "mobile metal counts
once": one guard answers one raid at a time, and the first form sent all of it across the map to a raid it matched
whatever was at stake there (terr268 game 00: a raid of 480 on 50 metal at 13:30). Stations split the guard over the
ground it has to answer for, and a raid takes the soldiers nearest it.

- **The places.** Every 30 s, after the race line: for every region holding something of ours that his raid can
  reach (a finite warning), its forward spot, the region's spot nearest its forward building (the building nearest
  his start); and home.
- **Cover.** A place covers a region when the guard's type walks from it to the region's forward building, less its
  reach, at its speed, within the region's warning plus 15 s (the race's second column: help that ends a raid
  soon, not help that saves the first building). The walk is the planner's ground, the commanders' (its fields
  stand at the spots and at home, made once): vehicles climb less than bots, so it is short of their walk on steep
  maps.
- **How many.** The place covering the most standing metal not yet covered is a station, then the next, while metal
  stays uncovered and the stations number fewer than the guard's metal over the raid seen (T-COVER's raid, at most
  1,000): a station must match a raid alone. At least one.
- **Shares.** The soldiers split over the stations in proportion to the metal each covers first, each at least one
  while soldiers last; a soldier goes to the nearest station short of its share. Home's station stands at the base
  point, another 200 from its spot toward home.
- **The answer.** Every half second, the raids by stake (our building metal within 800): each takes the free soldiers
  nearest it until their metal, with three times our turrets' metal reaching the raid, matches its weight; if all the
  free soldiers cannot, none go. A raid near home's region (within 800 of its buildings) that all of them cannot
  match calls every free soldier to the base point, where the turrets and the commander stand. A soldier that
  answered a raid and is not taken again holds its last place 20 s; one taken off a raid while a raid is within 900
  of it goes back to its station and is not free for 20 s (the retreat).
- **The `stations:` line** every 30 s: each station's place, the regions and metal it covers, its share; the regions
  holding something that no station covers. The `guard:` line's place becomes the soldiers' counts by state.
- **Measured** against terr268-273 (the guard at the base), the overcount left as it is in both. Not kept: held on the
  branch `territory-stations` (below, "Measured").

## What could go wrong

- **The edge of sight is too pessimistic.** With no bound on where he could be, every building at the edge of our
  sight reads 3-5 s from danger, all game. If the losses do not follow the warning, the bound (his last seen army
  and his speed) comes in before step 3.
- **Metal is a poor currency for some pairs.** Light laser turrets against Pawns, Ticks against everything. The
  multipliers per pair of classes are combatsim's to measure; if one number per class misleads, the race uses the
  simulator's odds for the pairs that matter.
- **The fields cost time.** A field over Glitters' 246,000 cells every 30 s, one per group, is fine for an
  instrument; a decider that wants one per region per tick needs a coarser grid or a worker thread.
- **BARb medium may not be the right pressure.** If it kills the baseline before 5:00 on every start, the bench
  measures only the opening; if it never raids, it measures nothing. The baseline says which.

## Measured

**The baseline (terr208-213).** The standing build, no soldiers, against BARb medium: 18 of 24 games lost (Great
Divide all four at 9-10 minutes), 6 to the cap. Income at 5:00 mostly held (Comet from his start 34-41 against
NullAI's 41-53); 10:00 fell to a third or less (46-73 against 172-230). Losses before 5:00 are small (936-5,540 metal
over a start's four games); from 5:00 to 15:00 they are 10,000-50,000. His armed metal, his commander apart: 650-1,150
at 5:00, 2,800-4,000 at 10:00, 6,500-9,500 at 15:00 (`run/territory_race.py`, from the truth files; terr208-219
pooled). Great Divide's at 5:00 was sixteen Grunts, at 7:30 twenty Grunts and thirteen Thuds. The raids that kill are
small: his metal within 800 of each loss, median 92-159 before 5:00 (one or two raiders), 258-412 from 5:00 to 10:00,
438-1,210 from 10:00 to 15:00, the largest 480-758, 1,250-3,708 and 3,126-6,790.

**The race (terr214-219).** The same build with the `race:` line: 22 lost, 2 to the cap, the spread of one build.

- *The warning is 0-1 s at every region, every start, all game.* The bot builds no radar, and a building sees about a
  second's run past a raider's reach (an extractor's sight 273, a Flash's reach 180). With sight alone a mobile
  response cannot save the first building a raid reaches; it can only end the raid. A radar (2,100) would make it
  about 20 s.
- *The kill test, as written, does not separate held from open.* Held regions lost a building in 34-67 % of the
  windows with his metal near, open ones in 31-65 %; held lost one as often or more on five starts. But held meant
  home: the commander is the only armed unit, and held windows are 42-79 of each start's at home against 3-53 away.
  Home is where most stands (a median 900-6,200 metal against 86-150 for an open region away) and where his pushes
  come, so a building lost there is likelier whatever holds it.
- *As a share of what stands, held loses less on every start*: 3.5-18.6 % of the metal standing a window at home,
  against 19.4-58.1 % in open regions away; held regions away (15-53 windows on four starts) 5.6-9.0 % on three and
  35.3 % on Comet's boxes. That fits the model, and is confounded as well: a raid kills a few of a large cluster.
- *The response side is not tested.* "Held" counts our armed metal that could reach a region within 15 s of his
  arrival; nothing in the bot sends it. The commander fires at what comes into its reach while it builds and chases
  nothing. The test means something only once a guard answers raids, which is step 3.

What the numbers say for step 3: with a second's warning, defence that works is defence already standing where the
raid arrives (a turret, or a guard stationed there) or warning bought (radar), and the raids it must meet are 100-400
metal to 10:00 and about 1,000 after; his whole army, the push, is 3,000-4,000 at 10:00.

**Step 3, radar and turrets (terr220-267).** Four builds against BARb medium (terr220-225 the first; terr232-237 with
the defence sites' own clearance; terr244-249 with the race's edge on the ground joined to his start; terr256-261 with
the largest uncovered piece when his start is in sight and no radar where our radar already covers), against the same
build with `defence=off` (terr226-231), which logs both sides' income too.

| | lost of 24 | twice his income at 5:00 | 10:00 | 15:00 | 20:00 | best ratio from 5:00 at 2 or more | from 15:00 (the win line since) |
|---|---|---|---|---|---|---|---|
| without defence (terr226-231) | 22 | 9 | 10 | 2 | 0 | 20 | 2 |
| terr220-225 | 13 | 9 | 12 | 9 | 5 | 18 | 9 |
| terr232-237 | 15 | 11 | 10 | 9 | 8 | 20 | 10 |
| terr244-249 | 14 | 6 | 8 | 5 | 3 | 16 | 7 |
| terr256-261 | 13 | 6 | 10 | 6 | 4 | 17 | 8 |

(The two earlier benches without defence, terr208-213 and 214-219, lost 18 and 22; their truth files have no income.)

- *The win line as the user put it does not separate the builds.* BARb medium makes 10-17 metal a second at 5:00
  and the bot 9-45, so our income is twice his at some point past 5:00 in 20 of 24 games without any defence,
  before his raids take the economy apart. Read at a fixed mark it does: twice his at 15:00 in 5-9 games with radar
  and turrets against 2 without, at 20:00 in 3-8 against none.
- *Radar and turrets keep games alive.* 13-15 lost of 24 against 18-22. Mithril and Glitters lose zero to two games a
  start against two to four, the Comet starts one to three against three to four; Great Divide is lost in every build
  (sixteen Grunts at 5:00 against 0-6 turrets at 10:00), Quicksilver three or four in every build.
- *The economy pays little where nobody attacks.* Against NullAI no turret is built, the radars are 2-17 a game, and
  every start and mark is inside the pooled range (terr262-267).
- *Radar works only with the edge bounded.* On the uncovered ground joined to his start the warning per region rose
  from 1 s to a median 6-16 s from 5:00 on Comet, Glitters and Mithril (terr244-261). Quicksilver's stays at 1-2 s in
  every build, not looked into.
- *What it does not do.* The games still lost are lost to his army, 3,000-4,000 at 10:00, which the turrets' cover
  (at most 1,000 a region) is not sized for: the guard's step. Our income at 20:00 in the games that live, 40-500 and
  most 100-300, is a fifth to a half of the NullAI bench's.

**Step 3b, the guard (terr268-279).** The first form over radar and turrets, against terr256-261 (radar and turrets
alone) and terr226-231 (no defence):

| | lost of 24 | the win line (twice his from 15:00) | twice his at 15:00 | 20:00 |
|---|---|---|---|---|
| without defence (terr226-231) | 22 | 2 | 2 | 0 |
| radar and turrets (terr256-261) | 13 | 8 | 6 | 4 |
| with the guard (terr268-273) | 4 | 20 | 15 | 18 |

- *Every start gains.* The win line per start: Comet from his start 4 of 4 (2), Comet's boxes 4 (3), Great Divide 2
  (0), Glitters 4 (2), Mithril 3 (0), Quicksilver 3 (1). Great Divide keeps two games to the cap for the first time
  on this bench, one of them at 2 metal a second at 20:00; the four games lost end at 15.8-17.2 minutes.
- *The losses move to the soldiers.* Lost to his weapons, the guard apart, 205-340 units a start (four games) against
  263-559; the guard loses 95-199 soldiers a start.
- *Our income at 20:00* in the games that live is 2-570, half of them 210-450, against his 22-112; the NullAI bench's
  medians are 350-1,000 by start.
- *Where the guard was*, at the 30-second lines: answering a raid in 54-90 of a start's lines, holding the last raid's
  place 13-26, back at the base after a raid it no longer matched 10-25, at the base 5-15. It crosses the map for a
  raid it matches whatever is at stake there ("raid 480 with 50 at stake", terr268 game 00 at 13:30).
- *The army shown is not his army.* At 10:00 it is under what he has on the map in 17 of 24 games (240-2,591 against
  930-3,910): the guard sees what comes to us. At 15:00 it is over in 12 and under in 11, once twice his army (terr268
  game 01: 4,600-6,000 from 12:30 against 2,000-2,900 on the map). The overcount is his units counted again after
  their death (terr280, a probe): the engine sends the death event and keeps the dying unit in its active list while
  its death script runs, the host's enemy list does not leave it out, and the next sighting counts it for five more
  minutes. His bots, whose death animation lasts, are counted again nearly every time (Grunt 31 of 37, Hammer 28 of
  31); his vehicles almost never (Incisor 0 of 21). At 15:00 of one game 3,836 of the 4,064 shown was dead. Not
  fixed: the guard's results above are with it.
- *Nothing changes where nobody attacks.* Against NullAI no soldier is made in 24 games and every start and mark is
  inside the pooled range (terr274-279).

**Step 3c, stations (terr281-292).** Two builds against the guard at the base (terr268-273), the overcount in all:

| | lost of 24 | the win line | twice his at 15:00 | 20:00 | soldiers lost | other units lost |
|---|---|---|---|---|---|---|
| the guard at the base (terr268-273) | 4 | 20 | 15 | 18 | 927 | 1,628 |
| stations (terr281-286) | 3 | 17 | 15 | 15 | 1,142 | 1,503 |
| stations, raids answered at twice their weight (terr287-292) | 4 | 15 | 13 | 13 | 1,119 | 1,465 |

- *Stations do not move the bench.* The win line is 17 and 15 against 20, inside what one build's repeats spread over
  in step 3 (7-10 among builds that differed less), so not shown worse either; no start gains.
- *They split the guard little.* One station at 10:00 in 46 of the 47 games with soldiers then (the guard's metal is
  under twice the raid seen), one to four at 15:00. What changed for most of the game is where the one station
  stands (the place covering the most metal, not always the base) and that each raid takes the soldiers nearest it.
- *They spare buildings and spend soldiers*: other units lost 1,503 and 1,465 against 1,628, soldiers 1,142 and 1,119
  against 927.
- *Not even fights.* Answering at twice the raid's weight, or with every free soldier, left the soldiers' losses where
  they were (1,119), so the conjecture that nearest-first sends just enough soldiers to lose half is wrong.
- Held on `territory-stations`; `territory` keeps the guard at the base.
