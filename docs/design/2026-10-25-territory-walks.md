# The territory bot's plan and its walks: prices measured in the game (2026-10-25)

The user, 2026-10-25, after the regions design's steps 2 and 4 were measured and not kept: "Take a fresh look at the
problem and let's see if we can figure out what is going wrong." Then, on the three ways on offered below: "Let's
try both -- go ahead".

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
  Comet's median (27 s at 5:00-10:00) but not its tail (p75 50 s) or Great Divide's 74 s. Walks between the base
  and the field are small; the walking is among field jobs and round the base.

The matching hands moved the shortfall from the turrets (the cascade's) to the opening's extractors; their lag read
the columns "on plan" because it counts orders given, not work done. The regions priced the extractors' walks,
already about right on Comet. The opening is apart: walking before 5:00 is near its price, and 5:00 reaches 0.8 of
the plan on Comet and 0.65 on Great Divide (the plan's fractional factory, among others), a question for later.

## Change 1: walks priced from the game

The bot measures, as it plays, each mobile builder type's walking against its work, per unit type it works on, and
the next plan prices each pair from it.

- **The measure.** At every hands tick, every constructor holding an order: moving (faster than a fifth of its top
  speed) is walking, else working, credited to its builder type and the unit type of the order (a build: the type it
  builds; help on a frame: the frame's type). A walk home (the cascade's) is walking for no unit: its seconds are
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

## The instrument

`run/territory_bench.py` prints per start and tag the promise kept (realized over planned income growth over the next
two minutes, the median of each game's plans, by window) and the constructors' walking share, with the marks.
