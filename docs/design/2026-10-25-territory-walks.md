# The territory bot's plan and its walks: prices measured in the game (2026-10-25)

The user, 2026-10-25, after the regions design's steps 2 and 4 were measured and not kept: "Take a fresh look at the
problem and let's see if we can figure out what is going wrong." Then, on the three ways on offered below: "Let's
try both -- go ahead".

Measured the same day ("Measured" below): change 1 level with the bench and the promise unmoved, kept; change 2 not
kept. The walk prices were not the cause of the promise's failure, which stays open; understanding it comes before
the next change.

## What is going wrong

Read from the standing build's bench (terr157-162) and the matching hands' (terr169-174), every start:

- **The bot floats metal from about 4:00-5:00.** Spending runs at 60-85 % of income in most minutes after 5:00, the
  bank climbs, and every start ends at its metal cap; energy sits mostly near its cap too. Growth is held by build
  power turned into buildings, not by the resources. The plan keeps its bank at zero.
- **Each plan's promise comes about half true.** Each re-plan's predicted income growth over the next two minutes
  against what the game then did, realized / planned, the range over the six starts' medians: 0.57-0.90 in minutes
  1-5, 0.49-0.77 in 5-11, 0.25-0.58 in 11-18 under the cascade; 0.41-0.64, 0.32-0.67 and 0.21-0.58 under the matching
  hands. Re-planning hides it: each plan starts from the real state, sees a bigger bank and schedules it spent within
  a minute or two (Comet from his start, game 00, 15:00: the turrets 41 to 105 within the minute; 70 stood).
- **About half the build power goes into buildings.** The build time of the frames begun over the mobile builders'
  and turrets' build power: mostly 0.5-0.6 (0.4-0.8 by window). Constructors walk 28-65 % of their time, about 45 %
  typically (straight lines from where each order found them, each walk cut at the unit's next order: a floor).
- **The plan prices those walks far cheaper.** Basic constructors helping upgrade frames walk a median 15-18 s and
  then help 8-10 s (two-thirds of the order walking; 199-873 such orders in four games), priced at 40 s of walk per
  whole upgrade's help (165 s of a constructor's work): about eight times too cheap. Turrets: a median 9-35 s of walk,
  priced 3 s. Walks home (the cascade's): not priced. Extractors: priced 15 s rising to 40 s at 10:00, which fits
  Comet's median (27 s at 5:00-10:00) but not its tail (p75 50 s) or Great Divide's 74 s.
- **Where the walking goes.** By where the builder stood at an order and at its next (the base within 1,500 of home;
  the farm's origin lies 800-1,200 out): field to field is the largest leg to 15:00 (Comet from his start, 18 and 50
  constructor-minutes a game in 5:00-10:00 and 10:00-15:00); out to the field, back to the base and the walks home
  together are a third of the walking from 10:00 and about 60 % at 15:00-20:00 (terr157, 160, 162). A first reading
  that took the order line's "from home" for the site's (it is the builder's) had the legs between base and field
  small; it was wrong.

The matching hands moved the shortfall from the turrets (the cascade's) to the opening's extractors; their lag read
the columns "on plan" because it counts orders given, not work done. The regions priced the extractors' walks,
already about right on Comet. The opening is apart: walking before 5:00 is near its price, and 5:00 reaches 0.8 of
the plan on Comet and 0.65 on Great Divide (the plan's fractional factory, among others), a question for later.

## Change 1: walks priced from the game

The bot measures, as it plays, each mobile builder type's walking against its work, per unit type it works on, and
the next plan prices each pair from it.

- **The measure.** At every hands tick, every constructor holding an order, credited to its builder type and the
  unit type of the order (a build: the type it builds; help on a frame: the frame's type): a build walks until its
  frame stands and works after; help walks until the helper is within its build distance of the frame's edge. (The
  first form, terr175-186, counted moving faster than a fifth of top speed as walking and read the extractor's walk
  at 0.81-1.58 seconds a second of work where the order-to-frame time read 1.5-2.3: turning, slowing and standing
  short of the site are not work either.) A walk home (the cascade's) is walking for no unit: its seconds are
  spread over the builder type's pairs in proportion to their work. Guards on a factory are not counted (the program
  prices an assist with no walk, and the guards were 0-11 % of the samples). The sums decay with a time constant of
  WALK_MEMORY (120 s), so the prices follow the game (the extractors' walk grows as the near spots go).
- **The price.** The program's cost of a unit to a builder becomes its build time times one plus the ratio `r` of
  walking to working, the measured seconds blended with the constant price as a prior worth WALK_PRIOR (30 s) of
  work: `r = (walked + r0 * prior) / (worked + prior)`, `r0 = worker_time * walk / build_time` from today's constant
  for the kind. With no measurement the price is today's; the commander and the turrets keep theirs (the commander's
  is derived from its spots, the turrets do not walk). The Frames columns take the same ratio on their work left.
- **Deleted:** the bot's extractor walk ramp, `walk_late` and `walk_late_from`: the measurement is that ramp, from the
  game. The constants `walk_extractor`, `walk_upgrade`, `walk_base` and `walk_factory` stay as the priors for the
  first minutes.
- **Logged:** `walks:` every minute, per builder type and unit type the ratio measured, its seconds of work, and the
  price the program pays.
- **Judged by** the bench (terr175-180): the promise kept, which should rise toward 0.8 or more, the float, and the
  marks by "What counts". The risk is the one recorded at every honest price before (the walk at 65 s, terr73-75;
  `farm_big`, terr58; the regions' prices, terr145-156): an honest plan is a lower plan, and the game gains only if
  the plan chooses better at the honest prices.

## Change 2: the hands' walks cut

On top of change 1, the walks the hands make that the plan cannot buy back:

- **No walk home.** A constructor with nothing to do stands where it is (or clears a yard it stands in) instead of
  walking to the farm's core. Deleted: `What::Walk`, WALK_STEP, the walk-home arm.
- **No basic constructor on the upgrade frames.** The program's mobile help on the upgrades (`mobile_help`, 0.35)
  set to 0: the help that walked two seconds for every one it worked.
- **Judged by** the bench against change 1 alone (terr181-186), the same measures.

## Measured

- **Change 1, first form** (terr175-180, the measure by speed): the promise kept did not rise (0.59-0.79, 0.50-0.67
  and 0.24-0.58 in minutes 1-5, 5-11 and 11-18, against the standing 0.59-0.90, 0.54-0.77 and 0.27-0.56); the marks
  inside the pool, but Quicksilver's 10:00 and 20:00 by one unit. The plan paid the measure (help on the upgrades 2.38
  against 2.42 measured), but the measure read the extractor's walk about 40 % low: the plan paid about 41 s an
  extractor where the game took 51-66 (30-46 s from the order to the frame, then the frame's 20 s). Not a fair test of
  the change; the second form measures by the order's state.
- **Change 2** (terr181-186, on the first form): not kept, held on branch `territory-walks-cut` (f426d08) and
  reverted (39e3afa). The constructors' walking after 10:00 halved (0.16-0.35 of their time against 0.29-0.52) and
  became standing, 29 % of the constructors idle from 11:00 on Comet from his start and 11 % on Quicksilver; income
  did not rise, Comet from his start fell under the pool at 20:00 (929 against 946-1,137) and Quicksilver at 10:00
  (83.9 against 89.9-102). The walks home carried constructors to work: late in the game the constructors are not
  what holds income.
- **Ruled out on the way, from the logs:** latency (the promise kept stays level or falls with the horizon, where
  a pipeline's delay would make it rise: Comet from his start at 3:00-6:00, 0.97 over one minute, 0.86 over two, 0.71
  over three, 0.65 over five); idle constructors under the cascade (none: every one holds an order); work off the
  plan (81-100 % of the constructors' order time to 10:00 is on the program's flows; the count rule and the walk home
  take 30-60 % after 15:00).
- **Change 1, second form** (terr187-192, the measure by the order's state): the measure now tracks the game (an
  extractor's walk at 5:00-10:00 read 1.04-1.87 seconds a second of work against the order-to-frame time's
  1.19-2.22, Quicksilver apart at 4.02 against 1.92), and the plan paid 10-25 % under it (the prior, and the age of
  the measure a plan was solved on): a turret at about a second of walk a second of work where the constant priced
  0.05, a basic constructor's help on the upgrades at 1.3-2.1. The promise kept stayed where it was (0.58-0.81,
  0.47-0.68 and 0.34-0.60); the marks inside the pool but Quicksilver's 10:00 (87.4 against 89.9-102, as in both
  benches before). **Walk prices are not what makes the promise fail**; the diagnosis above was wrong on that point.
  The walking is real, and late in the game it is a symptom (change 2). Kept, level on the bench and deleting the
  Comet-fitted walk ramp (the user, 2026-10-25: "We can keep T-WALKS, but the original point stands of understanding the failure first").

## What the promise instrument shows

The user, 2026-10-25: "We can keep T-WALKS, but the original point stands of understanding the failure first." One
bench with T-PROMISE (terr193-198, no decision changed): each plan's count of every unit type at 0, 1, 2 and 5
minutes beside what stood, every plan's own count at its snapshot matching the game's to a unit or two.

- **How much.** The work delivered over each plan's five minutes, weighted by build time, is 0.36-0.69 of what it
  promised (0.49-0.66 in minutes 1-5, 0.41-0.69 in 5-11, 0.36-0.58 in 11-18); finished units alone a little less, so
  the plan counting work on frames as it is paid is not the gap.
- **Where.** The shortfall by type, in build time: before 5:00 the energy (solars 40-45 % of it on Comet, wind
  generators 25-58 % on the windy maps), extractors and turrets; at 5:00-11:00 upgrades (21-47 %, Great Divide's 3 %
  apart), advanced fusions on Comet (37-46 %), wind generators on Great Divide, Mithril and Quicksilver (50-71 %),
  turrets (13-24 %); at 11:00-18:00 advanced fusions
  (56-68 % on Comet and Glitters) and advanced converters (21-27 % on Comet), wind generators on the other windy maps
  (44-91 %). Built past the plan: constructors (1.1-1.5 of it, Great Divide's 5.3) and, on Comet, extractors and
  solars in the middle game; basic converters late on Comet from his start (1.45).
- **Turrets: dispatch.** Asked, ordered, came over the four games of a start: before 5:00 7-18 / 0 / 0 on every
  start; at 5:00-11:00 the hands order 13-29 % of the turrets asked, after 11:00 18-33 %; about nine in ten of those
  ordered stand. Who takes what at 4:00-11:00: the constructors out past BESIDE (600 from home; a third to two-thirds
  of the orders) take extractors, wind generators, upgrades and help, and 0-8 turrets, since T-CHAIN keeps them off
  the turrets; the commander takes energy and guards, never a turret; the constructors at the base take the larger
  columns first, since the flows go largest in power-seconds first (Comet from his start: 102 guards on the
  advanced plant, 49 solars, 47 upgrades, 31 turrets). terr102's reading of the code, now from the game.
- **Cheap farm buildings: out of the turrets' reach.** From 11:00, 92-100 % of the wind generators' frames, 39-100 %
  of the basic converters' and 55-100 % of the solars' went down with no turret within reach; the advanced fusions,
  advanced converters and fusions had 11-29 turrets in reach. T-FARM puts the cheap rest on the slots nearest home
  outside the core, and the program lets turrets help any building up to `help` (40) times the placer's power with no
  reach (`farm_big` off: the program-side fix, turrets on big buildings only, lost every mark in terr58). On the
  windy maps the program's energy is a wind farm the turrets finish; the game builds every wind generator at a
  constructor's power, and late the hands order 17-55 % of those asked on Great Divide, Mithril and Quicksilver.
- **The program counts on it.** Offline, Great Divide's first plan under the bot's settings (`buildorder lp --hello`
  on terr195 game 00's dump, `--line vehicle --help-factor 40 --mobile-help 0.35 --max-nanos 300`; 50 / 127 / 375 /
  861 at 5, 10, 15 and 20:00 against the bot's own first plan's 45 / 140-144 / 329-350 / 827-905): 2,056 wind
  generators by 20:00, 90 % of the build power on them the turrets' help (1,912 of the units), and 455 of its 472
  solars and 240 of its 293 basic converters the same way; on Comet from his start 635 of its 679 solars. With
  `--farm-big 20000` (turrets on buildings of that much build time only) the plan builds 803 wind generators with its
  constructors (forty, the cap) and puts the turrets on fusions and advanced fusions, and its forecast barely moves:
  Great Divide 50 / 126 / 398 / 792, Comet from his start 57 / 186 / 789 / 1,483 against 57 / 186 / 745 / 1,459.
- **The farm's big buildings: short with the turrets.** In reach of 11-29 turrets, they come at 0.25-0.65 of the
  promise as the turrets do; on Comet they carry most of the late shortfall.
- **Upgrades and extractors: execution.** Ordered at about the rate asked; 0.6-0.85 of the orders stand within the
  window (the walks, priced since T-WALKS).
- **Quicksilver's floating turrets** (`nanotcplat`) are planned and never ordered: 0 of 657-1,632 units a window.

- **`farm_big` on the bench** (terr199-201; the user, 2026-10-25: "Let's try farm_big on both comet catcher and great
  divide."): at 20,000 the marks are level with the pool on the three starts, none under it and noise-sized gains
  (Comet from his start 46.8 / 177 / 520 / 1,117, the lobby's boxes 37.5 / 164 / 417 / 936, Great Divide 32.7 / 79.8 /
  204 / 357, its 5:00 at the pool's top). On Great Divide the plan became one the hands can follow: the work delivered
  over each plan's five minutes 0.63 at 5:00-11:00 and 0.57 at 11:00-18:00 against 0.41 and 0.36, the turrets 0.97
  of the planned at 5:00-11:00 against 0.41; the wind generators, now the constructors' alone, still the largest late
  shortfall (42 %, 0.14 of the planned). On Comet the plan swaps solars for fusions and the rest is as before.

So the plan's growth runs through a farm of construction turrets: turrets as build power, their help on every farm
building. The hands build that farm late and in part: the turrets wait for every larger column, the far constructors
and the commander never take them, and the cheap farm buildings stand where no turret reaches. The matching hands
(step 4) ordered the turrets as the plan asked, and their late game on Comet from his start was the strongest
measured (592-717 gained from 15:00 to 20:00 against the cascade's 533); their opening fell for another reason (an
extractor order's cost committed whole when given).

## The instrument

`run/territory_bench.py` prints per start and tag the promise kept (realized over planned income growth over the next
two minutes, the median of each game's plans, by window) and the constructors' walking share, with the marks.
