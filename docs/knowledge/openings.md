# Openings and build orders

Seeded 2026-09-19 from public sources plus the local game source. "Local source" means `upstream/Beyond-All-Reason`
(the checkout we run; changelog head "September"); unit-file paths are relative to it. Web claims are `reported` and
unchecked in the arena. Scope unless stated: 1v1, land, Armada/Cortex, default mod options.

### K-open-start-resources
**Claim.** A team starts with 1000 metal and 1000 energy (storage 1000 each); the commander adds +2 metal/s, +30 energy/s
and 300 build power. Nothing else produces until the first building finishes.
**Status.** supported (2026-09-19) — read in local source, not measured in a match
**Evidence.** `modoptions.lua:2405-2449` (startmetal/startenergy/storage defaults 1000); `units/armcom.lua`, `units/corcom.lua`
(metalmake 2, energymake 30, workertime 300).
**Would be wrong if.** The bot's first status line showed a different starting stock with default options.
**Used by.** (candidate: budget the opening against 1000 M / 1000 E instead of a fixed list)

### K-open-energy-binds-first
**Claim.** In the first minute energy, not metal, is the binding resource: an extractor costs 50 M but 500 E, a bot lab
500 M + 950 E (Cortex 470 M + 1050 E), a wind turbine 40 M + 175 E, a solar 155 M + 0 E. Two extractors plus a lab already
cost 1950 E against 1000 E stock and +30 E/s. Solar's zero energy cost is why it is the classic first generator even where
wind is better later.
**Status.** supported (2026-09-19) for the costs; the conclusion is arithmetic, not observed
**Evidence.** `units/ArmBuildings/LandEconomy/armmex.lua`, `armsolar.lua`, `armwin.lua`, `LandFactories/armlab.lua`, Cortex
equivalents. Official economy guide makes the same point qualitatively ("solars … when you have more metal than energy, or
as an emergency source if you are stalling"): https://www.beyondallreason.info/guide/in-depth-look-at-economy (undated).
**Would be wrong if.** bot.log showed energy never below ~200 during the first 90 s with a wind-only opening.
**Used by.** (candidate: opening = 2 mex, then energy chosen by wind rule, watching stored energy before placing the lab)

### K-open-two-mex-first
**Claim.** The standard opening is two extractors before anything else, then energy, then the first factory, then radar
and one or two light laser towers. Priority order in every written guide found: extractors → energy → (constructors) →
factory.
**Status.** reported (2026-09-19)
**Evidence.** https://www.beyondallreason.info/guide/how-to-start-manage-your-economy ("typically build 2 extractors
before building anything else", "one or two Light Laser Towers once you have metal, energy, production and intel"; undated
official guide). https://www.crdhq.com/articles/bar-build-order-metal-solar (2026, low-reliability aggregator).
**Would be wrong if.** A 3-mex or factory-first opening reached a higher army value at minute 5 over 24+ matches.
**Used by.** H-ECO-OPENING already does this (2 mex, 2 solar, lab); this entry is its first outside support.

### K-open-wind-threshold
**Claim.** Choose wind over solar when the map's average wind is above a threshold; sources disagree on the threshold:
official economy guide ">7", official starter guide ">10", a third-party guide "wind above 8, solar below 5, mix between".
From local costs the pure metal break-even is ~5.2 (wind 40 M per [wind speed] E/s vs solar 155 M per 20 E/s = 7.75 M per
E/s); the higher published thresholds price in wind's variance and its 175 E build cost.
**Status.** reported (2026-09-19); break-even arithmetic from local unit files
**Evidence.** https://www.beyondallreason.info/guide/in-depth-look-at-economy (>7);
https://www.beyondallreason.info/guide/how-to-start-manage-your-economy (>10);
https://www.crdhq.com/articles/bar-build-order-metal-solar (8/5). `armwin.lua` (metalcost 40, windgenerator 25 cap),
`armsolar.lua` (metalcost 155, energyupkeep -20).
**Would be wrong if.** On a map with average wind ≥ 10 a solar-only brain matched a wind brain's energy income per metal
spent at minute 8.
**Used by.** (candidate: H-ECO-ENERGY-CHOICE — read map min/max wind at start; wind if (min+max)/2 ≥ 8, else solar)

### K-open-quicksilver-is-a-wind-map
**Claim.** Quicksilver Remake 1.24 has wind 3–17 (site lists average 12.7), so wind turbines give ~2.5x the energy per
metal of solars there (12.7 E per 40 M vs 20 E per 155 M). Our opening and generator rule build solars / advanced solars only.
**Status.** supported (2026-09-19) for min/max wind (local map archive); average and the conclusion are reported/arithmetic
**Evidence.** `run/data/maps/quicksilver_remake_1.24.sd7` → `mapinfo.lua:118-119` (minWind 3.0, maxWind 17.0).
https://www.beyondallreason.info/map/quicksilver (wind 3-17, average 12.7, tidal 10; page updated 2026-06-26).
**Would be wrong if.** A wind-first brain on Quicksilver showed lower energy income at minute 5 and 10 than the solar brain
at equal metal spent on generators, or stalled more often in the low-wind troughs.
**Used by.** (candidate: replace solars in H-ECO-OPENING and H-ECO-ENERGY-BY-STORAGE with wind on this map; keep 1-2 solars
or an energy storage as a floor for wind troughs)

### K-open-build-times
**Claim.** Build time in seconds = `buildtime / total workertime` while resources last. Commander alone (300): extractor 6 s,
wind 5.3 s, solar 8.7 s, bot lab 16.7 s, radar 3.8 s, LLT 8 s. A T1 constructor bot (80-85) takes 3.6x as long. So the
2 mex + 2 solar + lab opening is ~46 s of pure build time plus walking.
**Status.** supported (2026-09-19) for the inputs; the formula is standard Spring/Recoil behaviour, not measured by us
**Evidence.** buildtime/workertime in `units/armcom.lua`, `ArmBuildings/LandEconomy/*.lua`, `ArmBots/armck.lua` (80),
`CorBots/corck.lua` (85).
**Would be wrong if.** Logged start/finish frames of commander builds disagreed by more than ~10% when not stalled.
**Used by.** (candidate: detect a stalled opening — lab not finished by ~1:30 means energy or pathing trouble)

### K-open-factory-drain
**Claim.** A bot lab (150 build power) working alone drains roughly 5 M/s on raiders and constructors and 9-10 M/s on
line units: Pawn 11 s (4.9 M/s, 82 E/s), Grunt 8.3 s (5.2 M/s, 98 E/s), Rocketeer 13.4 s (9 M/s, 75 E/s), Thug 14 s
(10 M/s, 82 E/s), Centurion 28 s (9.6 M/s, 111 E/s), constructor bot 23 s (4.8 M/s, 70 E/s). An opening economy of
2 extractors + commander cannot feed continuous line-unit production; ~100 E/s is needed per busy lab.
**Status.** supported (2026-09-19) — arithmetic on local unit files
**Evidence.** metalcost/energycost/buildtime in `units/ArmBots/*.lua`, `units/CorBots/*.lua`; lab workertime 150 in
`armlab.lua`/`corlab.lua`.
**Would be wrong if.** Measured metal usage with one un-assisted lab on repeat differed from these rates by >20%.
**Used by.** (candidate: size lab count and nano count from income: labs ≈ metal income / 9)

### K-open-constructors-early
**Claim.** Guides put constructors ahead of or right behind the first fighters (priority "extractors, energy,
constructors, factory"; "secure three additional mexes within the first five minutes"), while our arena evidence says
fighters first against BARb's early raids. The two reconcile as: one or two raiders/scouts, then a constructor, never
several constructors in a row.
**Status.** reported (2026-09-19)
**Evidence.** https://www.crdhq.com/articles/bar-new-player-guide-getting-started,
https://www.crdhq.com/articles/bar-beginner-guide-getting-started (2026, undated articles). Our K-army-fighters-before-constructors.
**Would be wrong if.** Moving the first constructor from slot 3 to slot 1 or 2 in the first batch changed neither extractor
count at minute 5 nor early losses over 24+ matches.
**Used by.** H-PROD-BATCH (already raider, raider, constructor …)

### K-open-one-factory-plus-assist
**Claim.** Early on, one factory with build assistance beats two factories: guides say "one factory with nano turrets beats
two without" and 2-3 construction turrets double or triple output. A quantitative community analysis narrows it: ~1-3
assisting builders on a T1 lab is a clear win, beyond that a second factory is more efficient.
**Status.** reported (2026-09-19)
**Evidence.** https://www.crdhq.com/articles/bar-economy-guide (2026);
https://roguel1kegaming.com/beyond-all-reason-patch-construction-turrets/ (post-turret-nerf patch analysis, undated; quotes
turret cost 250 M — local `armnanotc.lua` says 230 M, 3200 E, 200 build power, range 400).
**Simulated 2026-09-19.** K-open-sim-turrets-before-second-lab agrees (no arena test yet).
**Would be wrong if.** At equal metal spent, 2 labs out-produced 1 lab + 2 turrets in army value by minute 10.
**Used by.** (candidate: H-ECO-MORE-LABS — add up to 2 construction turrets beside the first lab before a second lab)

## From the build-order study (simulator, no enemy)

Entries below come from `../studies/build-order.md` (tool: `crates/buildorder`). The simulator reproduces our recorded
openings to about 10 % while nothing is lost and knows no enemy, terrain or losses; its optimum is a ceiling. Nothing here
has been played in the arena, hence `conjectured` throughout. Scope: Quicksilver Remake 1.24, tier 1, game commit 54199a0d.

### K-open-sim-expansion-gap
**Claim.** The largest gap between our opening and a good one is expansion pace in minutes 1-5: a no-enemy ceiling holds
7 extractors at minute 3 and 16 at minute 5 (6 and 13 if every walk is 40 % longer) where we hold 2 and 3-4 and BARb
medium 4 and 5; metal income at minute 5 is 31.6 against our 8.7. The first minute (lab at 0:40 against our 0:59-1:15)
is a small part of it.
**Status.** conjectured (2026-09-19); our and BARb's numbers are measured (12 records, census of 24 and 42 games)
**Evidence.** `../studies/build-order.md` table "The curve"; `../studies/data/optima.md`, `census-medium-*.csv`, `calibration.md`.
**Would be wrong if.** A brain that reached 6+ extractors by minute 3 and 10+ by minute 5 in the arena was no richer at
minute 8 than today's (raids eating the difference), or did not win more.
**Used by.** (candidates: H-COM-LEASH, H-PROD-BATCH constructor share, H-ECO-BASE-TURRETS timing)

### K-open-sim-constructors-first
**Claim.** For army value at minute 8 or later, the lab's first 5-6 units (up to 9-10 when allowed) should be
constructors, with the first fighter shortly before minute 4; for army value at minute 5 or earlier, none. The
break-even horizon of a constructor lies between 5 and 8 minutes.
**Status.** supported for two constructors first (2026-09-19, v21-early-expand-ab10); more than two untested. The
simulator has no raids, so its 5-6 is an upper bound, not a recommendation.
**Evidence.** `../studies/data/optima.md`: `army` at 3 and 5 minutes build 0 constructors, `army` at 8 and 10 and every
`mix` from 5 minutes on build 6 (the cap); side run with 12 allowed used 10, +5 % (inside search noise).
Arena, 10-minute A/B against medium, 4 games per arm and corner, means ours/theirs. With the fighter-first batch
and constructors building generators before extractors, v20 sat on 2.0 extractors until minute 4 with one constructor,
all metal spent as it came (north-west 0-12). Two constructors first, constructors to spots before anything else, and a
1500 commander leash for five minutes: north-west extractors at minute 4 4.0/4.5 against 2.8/5.0, army value at
minute 10 2992/2696 against 1798/3361; south-east extractors at minute 4 5.2/2.5 against 2.5/2.8. Army value at minute 4
was level between the arms: the early fighters bought nothing.
**Would be wrong if.** In the arena, batches opening constructor-heavy (say ck x4 before the second fighter) had no more
extractors at minute 5, or lost them all to raids before minute 8.
**Used by.** H-PROD-BUILDERS-FIRST, H-ECO-EARLY-EXPAND.

### K-open-sim-turrets-before-second-lab
**Claim.** One lab plus 3-4 construction turrets builds as much army by minute 10 as any plan containing a second lab,
and about 12 % more than two bare labs; one bare lab is 20-35 % behind. The first turret belongs at about minute 4-4.5,
when metal income passes 25-30/s; roughly one more per further 10 metal/s.
**Status.** conjectured (2026-09-19); agrees with the reported K-open-one-factory-plus-assist and with BARb medium's
one lab + one turret (K-barb-medium-observed-build). Turrets are uncalibrated in the simulator (no recorded game has one).
**Evidence.** `../studies/data/optima.md`, `mix` / `army` at 10 min: 1 lab 15381 / 8085; 2 labs 16164 / 10874; 1 lab +
turrets 17810 / 12196; both 17456 / 12178.
**Would be wrong if.** At equal metal income, a brain with lab + 2 turrets had not built more army by minute 10 than the
current 2-3 lab brain over 24+ matches.
**Used by.** (candidate: H-ECO-MORE-LABS)

### K-open-sim-energy-ratio
**Claim.** A well-run tier-1 economy needs about 11-12 energy/s per metal/s of income through minute 10 (181/15, 332/32,
528/44, 566/46 at minutes 3/5/8/10). We run at 16-22 and our four-turbine opening overflows about 2100 energy in the
first three minutes while holding two extractors. Converters do not belong in the first 10 minutes while spots are free.
**Status.** conjectured (2026-09-19); our ratios are measured (12 records)
**Evidence.** `../studies/build-order.md`, finding 4; `calibration.md` median table; `optima.md` (0-2 converters in every
`army`/`mix` optimum, 7-11 only in `income` at 8-10 minutes after all 22 spots are taken).
**Would be wrong if.** A brain holding energy income near 12x metal income stalled on energy (stored energy near zero
for more than ~10 % of the first 10 minutes) in the arena.
**Used by.** (candidates: H-ECO-WIND opening count, H-ECO-ENERGY-BY-STORAGE, H-ECO-CONVERT-SURPLUS)

### K-open-sim-solar-is-the-robust-generator
**Claim.** On Quicksilver (wind 3-17) an opening tuned to mean wind, mostly turbines, loses 34 % of its minute-10 army
if the game's wind averages 8 and 58 % at 5; an opening built on solars loses 6 % against it at mean wind and nothing
in poor wind. Game-average wind over the first 10 minutes ranged 8.2-15.5 in 12 recorded games (mean 12.8).
**Status.** conjectured (2026-09-19); qualifies K-open-quicksilver-is-a-wind-map, whose per-metal arithmetic stands. The
simulator's constant wind hides in-game troughs, so the effect is more likely understated than overstated.
**Evidence.** `../studies/data/optima.md`, last table, and `../studies/build-order.md` finding 5.
**Would be wrong if.** Arena batches with a solar opening built measurably less by minute 10 than the wind opening in
games whose recorded wind average was at or above the mean.
**Used by.** (candidate: H-ECO-WIND)

### K-open-energy-stall-minute-5
**Claim.** Today's opening runs out of energy at 4:30-5:00 with 600-740 metal unspent, and the extractors stop with it:
metal income falls from 20-22 to 14-16 for 20-40 s. The spending that does it is the first construction turret, light
turrets and Hammers started together (230-270 energy/s asked of 130-190 coming in, wind falling).
**Status.** measured (2026-09-20): 8 of 8 games, Quicksilver and Mithril Mountain, both factions, BARb easy.
**Evidence.** `open-cal2-quicksilver`, `open-cal2-mithril`: stored energy reaches 0 between 4:30 and 5:00 in every
record; `buildorder calibrate` shows the income dip the simulator lacked until it modelled it.
**Would be wrong if.** A batch against BARb medium (more early losses, so less to spend on) showed no stall.
**Used by.** (none yet) — the opening search's energy floor (`docs/design/2026-09-20-opening-search.md`).

### K-open-seconds-lost-between-builds
**Claim.** Our mobile builders lose 3.5-4.0 s between one build's last frame and the next one's first when the next site
is already in reach (53 trips), and 4-5 s before the game's first build. At some thirty builds in the first five
minutes that is about two minutes of builder time. The orders are given one at a time when the builder reports idle;
nothing is queued.
**Status.** measured (2026-09-20), two maps, 8 games; the cause (idle detection on a half-second tick, the site query,
the engine's own start-up of a build) is not split.
**Evidence.** `buildorder walks` on `open-cal2-*` (recorded against predicted, by length of walk).
**Would be wrong if.** Queued orders (the plan executor's) left the gap where it is: then it is the engine's and not ours.
**Used by.** `Scenario::mobile_overhead` (3.5 s) in `crates/buildorder`.

### K-open-sim-snapshot-replays
**Claim.** The economy simulator started from a played game's own state at 3:00 (standing buildings, each builder's
job with its nanoframe's health as its progress, the stock) and given what the builders started after that, matches
the record to minute 6 as well as a replay from the empty start does, or better: extractors within 0-27 % (from the
start: 27-33 %), metal income within 6-19 % (19-26 %), army metal built from there within 8 % at minute 6.
**Status.** measured (2026-09-21), the four open-cal2 records with terrain (two a map), `buildorder calibrate --from 180`.
**Evidence.** `buildorder calibrate <records> --minutes 6 --from 180`; `docs/design/2026-09-21-rolling-planner.md`.
**Would be wrong if.** A snapshot mid-raid (builders dead, jobs abandoned) drifted at once: the check has no losses
in it (0-8 units lost by minute 6). What the snapshot leaves out: builders helping another's nanoframe (their power
is not counted), tier-2 constructors and resurrectors (not the plan's), and the army (nothing of it is modelled).
**Used by.** H-OPEN-SEARCH, H-OPEN-PLAN (the rolling planner re-plans from such a snapshot).

### K-open-five-minute-horizon-undervalues-extractors
**Claim.** With the opening search's horizon at five minutes and 90 s of terminal income, a start whose plan
predicts six extractors at 5:00 for one more Pawn party (4448, 1544 on Quicksilver's north box: 6 extractors, 1,960
army metal, score 10,141) outscores one with eleven (4000, 2152: 11 extractors, 1,428 army metal, 9,993); at ten
minutes the ranking flips (25,934 for the latter, 23,252 for the former; 16 against 12 extractors predicted at 5:00,
24 against 22 at 10:00). Commander games 6 and 7 started at the former and had the worst economies of the series
(five and six extractors at 6:00 against nine); game 5 started at the latter and won.
**Status.** measured (2026-09-21): the arena's `--place` candidate scores at both horizons on the same record.
**Evidence.** `docs/briefs/commander.md` cmd-opus-low-7; `planner-smoke-1`'s arena output (candidate lines).
**Would be wrong if.** A batch from the ten-minute choice held fewer extractors at 5:00 than one from the five-minute
choice; or the Pawn-heavy opening from the former start won more (the opening series found it standing outmatched).
**Used by.** H-OPEN-SEARCH (horizon 600 s), the arena's placement (`crates/arena/src/place.rs`).

### K-plan-ten-minute-horizon-trades-early-army
**Claim.** The rolling planner with the `Tempo` objective ten minutes ahead (metal made + 90 s of terminal income
+ army metal x 1 + the 2:30 contact term) spends the first five minutes on the economy: 5 constructors and 8
soldiers alive at 5:00 against the five-minute one-shot opening's 2 and 15, army metal built 720 against 1,327.
It holds 10 extractors at 5:00 and 9 at 10:00 against 8 and 5, income 20.5 and 19.3 against 17.9 and 13.3, and
loses more games (2-19-3 against 5-15-4): the raids of minutes 5-10 take the larger economy from the smaller army.
The simulator prices every extractor at full worth to the horizon and knows no enemy, so the plan's economy is
overbuilt for what stands against it.
**Status.** measured (2026-09-21), 24 games an arm, Quicksilver north box, BARb medium, mirrored.
**Evidence.** `planner-base`, `planner-1` (ledger); `docs/design/2026-09-21-rolling-planner.md` step 1.
**Would be wrong if.** The army share were the objective's and not the horizon's: an army weight or a contact term
that keeps 15 soldiers by 5:00 at the ten-minute horizon and holds the extractors too would show it. The
commander's requirements (step 3) are the intended lever for the share.
**Used by.** H-OPEN-SEARCH.

### K-plan-three-minute-expectations
**Claim.** The rolling planner three minutes ahead with `Objective::Expect` (what stands at the horizon paid against
the clock's expectations: extractors 2 / 4 / 6 / 8 / 10 at minutes 1-5, constructors 1-5 from 1:30 to 10:00, army
metal 200 / 650 / 1,400 / 3,200 at 2 / 3 / 5 / 10 min; a constructor beyond the expectation costs its metal) holds
the economy the ten-minute form gained (9 / 9.5 / 9 extractors at 5 / 10 / 15 min against the old opening's
8 / 5 / 5, income 18 / 18 / 21 against 18 / 13 / 13) and builds the army from it (army metal 4,014 / 7,297 at
10 / 15 min against 3,346 / 5,958; 25.5 / 39 soldiers against 23.5 / 31), for 9-13-2 against 5-15-4 (the ten-minute
form: 2-19-3). The first five minutes are still army-poor: 870 army metal and 9.5 soldiers at 5:00 against the
expectation's 1,400 and the old opening's 15.
**Status.** measured (2026-09-21), 24 games an arm on the same setup as planner-base; 24 games is within noise of
the base's wins (docs/README: 12 games ±14 points), the economy and army curves are not.
**Evidence.** `planner-2` against `planner-base` and `planner-1`; `docs/design/2026-09-21-rolling-planner.md`.
**Would be wrong if.** A 48-game rerun lost the win margin, or the early-army shortfall came from the executor
(labs idle) rather than the objective (the plan's factory queues). Open: the expectation curves are mine, by eye
from the players' and the searched openings; the commander's requirements (step 3) are meant to set them.
**Used by.** H-OPEN-SEARCH.

### K-plan-single-goal-strips-the-rest
**Claim.** A `target` search with one goal leaves every other soldier out of its order: the objective pays for the
goal's units, its makers and 20 s of income, and a raider is metal spent on none of those. Given a second goal at its
metal alone the search still drops it when it competes with an expensive chain (four Flashes by 3:30 against Bulls:
one Flash at 5:00); a shortfall penalty of three times the unit's metal per unit missing at the deadline holds it
(four Flashes by 3:00, three Bulls by 10:00 against four without them).
**Status.** observed (2026-09-22), the hands-2 record's header under `buildorder optimize --palette roster`, four
restarts of 20,000 iterations each; the player's own searches in plan-1, plan-2, hands-1 and hands-2 were all
`target armbull` alone, and the player added the raiders to the returned order by hand each time.
**Evidence.** `run/matches/1790117628-hands-2-bulldogs/00/record-0.jsonl`; the four games' `strategist-0.jsonl`
`search` calls; `target_goals_count_up_to_their_count_and_seed_every_chain` in `crates/buildorder/tests/arithmetic.rs`.
**Would be wrong if.** The annealer kept a cheap deadline goal against an expensive chain at metal alone given more
iterations, or the penalty of three made the search over-serve a raider goal at the expensive goal's cost in a game.
**Used by.** H-PLAYER-PLAN-SEARCH (goals after commas; `TARGET_SHORTFALL`).

### K-open-search-beats-rules
**Claim.** An opening found by half a second of search over a simulator of this game's economy beats our hand-ordered
opening on every opening measure at once, on the same seeds: extractors 5.2 / 8.2 / 11.4 at minutes 3 / 4 / 5 against
4.6 / 6.8 / 9.1, metal income 20.2 against 16.0 at minute 5, army metal built by minute 5 1116 against 922, six
constructors against four, and 1.6 s against 15 s with stored energy at zero. What it does differently: four
constructors before the first soldier, generators interleaved with the commander's extractors so the stall at 4:30
never comes, and solars where the rules took wind.
**Status.** measured (2026-09-20), Quicksilver, 8 games an arm, against the same opening played through the same executor.
The simulator's prediction holds to minute 3 (extractors 5.2 predicted, 5.2 played; income 12.5 against 11.5) and is
optimistic at minute 5 (14 extractors and 30 metal/s predicted, 11.4 and 20 played): it knows no enemy and no losses.
**Evidence.** `open-search-1-ab`; `run/opening_ab.py <batch dir>`.
**Would be wrong if.** Full games lost the gain. **They did** (open-search-2-ab, 2026-09-20: 3-7-2 against 7-5-0 on
Quicksilver, 1-7 against 2-5-1 on Mithril): with no turret to minute 6 the extractors it takes are raided away from
minute 6 on. The claim stands for the opening measures only; an opening objective has to price exposure
(`Objective::Tempo`'s `exposed`, open-search-3-ab).
**Used by.** H-OPEN-SEARCH.


### K-open-early-pawn-pressure-is-standard
**Claim.** Early raider pressure (a lab by about 0:50, then a handful of Pawns or Grunts before anything else, arriving
at the opponent's base before 2:30) is standard 1v1 play on maps like Quicksilver, in basically every game; 1v1 games
rarely reach 20 minutes. Against BARb medium the handful is enough to win outright: it has two LLTs and no soldier at
2:30 and offers the kill. Long macro games, the kind our bot is meant to be capable of, come from 2v2 or bigger on
larger maps, where the game stabilises long enough.
**Status.** stated by the user 2026-09-20 from watching an experienced player's replay and his own knowledge of the
game; the replay's record agrees (below). Not measured over many games.
**Evidence.** `run/matches/*-replay-player2-vs-medium` (run/replay_match.py on the player's demo): lab by 0:50, 5
Pawns at minute 2 and 12 at minute 3 with metal income held at 6, BARb's first extractor dead at 2:28, its lab at
3:30, its commander at 4:45; the player lost 15 Pawns; BARb built no soldier. Pawns finished per game minute 1-4: 5 / 7 / 6 / 8 (26 by minute 5) beside 2 constructors and 7 extractors; neither player lost an extractor. The second replay (Ben vs medium, `*-replay-ben-vs-medium`) is the
same shape with expansion beside it: 3 / 7 / 10 / 15 Pawns at minutes 2-5 while extractors go 2 to 9 by minute 6
(income 6 to 16); BARb, raided from 2:25, never left 2 extractors and income 6, had 2-5 Pawns and 3 LLTs, and its
commander died at 6:08; Ben lost 22 Pawns and one resurrection bot. Neither player's commander left its base.
**Update 2026-09-20 night (matt-plan, matt-search).** Matt's order transcribed and played by the bot gives his Pawn
count (21 by 5:00) and the same 3-9 as the searched opening: BARb in our arena games has 1-4 soldiers and 1-3 towers at
3:00 and 5-9 soldiers with 3 towers at 4:00 in ten games of twelve (in Matt's replay it had no soldier and two towers,
the variant two of our twelve met), and our pressure rule waits where Matt went in with twelve and paid seven for the
lab and every tower. "The handful is enough to win outright" holds for the soft variant; against the usual one the
question is the party's commitment, not its size.
**Would be wrong if.** Experienced players opened with constructors and a turret against each other on this map and
raided only from minute 4; or the same rush lost to BARb medium in half of a batch.
**Used by.** (candidates) the opening search's army term: raiders early, weighted by a first-contact time near 2:30
rather than metal alone (the design's leftover); H-ARMY-CONTACT: the first parties to answer are 5-12 Pawns at 2:30
with two turrets and no army of ours, which is a turret and commander question; the commander-unit rules of the
opening design, step 4 (BARb's commander does not fight either).

### K-open-matt-order-timings
**Claim.** Matt's opening on Quicksilver, by the clock: extractors 0:03 and 0:10 on the two spots beside the start,
wind 0:16, 0:22, 0:28, the lab 0:33 (finished 0:50) with energy at 490 and rising, then wind 0:53, solar 0:59, wind
1:08, 1:13, 1:19, 1:24, 1:30 (ten generators by 1:30), the commander assisting the lab from then; the lab a
constructor at 0:50 then Pawns from 1:12 every ten seconds (eleven by 2:45) and a second constructor at 1:45; far
extractors from 1:45 with a turret beside them at 2:32; a converter at 2:55. Energy dipped to 15 once (1:01 to 1:07,
the first constructor) and was full by 1:35; metal was spent as it came (8 in the bank at 2:00).
**Status.** transcribed (2026-09-22, from `run/matches/1789930356-replay-player2-vs-medium/record-0.jsonl`; the
user: copy it)
**Evidence.** The record's created events and resource samples for the first three minutes, listed above.
**Would be wrong if.** The replay's record misattributed the builder (it names none; the order is the commander's
own until the first constructor stands at 0:50, and the lab's units are the lab's).
**Update (human-5 review).** The commander's commands in the record: `guard` on the lab at 0:50 as it finished; seven
wind and a solar ordered 0:53 to 1:04 as one batch, built by 1:30; `guard` again at 1:36 and, after a reclaim, at
2:02, and nothing else until 3:04. Pawn intervals 11-12 s while the lab worked alone (1:12, 1:23, 1:35), then 4-14 s
with the commander's 300 build power on the lab's 150: metal-bound (income 6, a Pawn 54) with the bank spent to 8
by 2:00. The second constructor came at 1:45 after six Pawns.
**Used by.** the player's brief (the opening to copy, written as a packet); the hands' default instructions
(`default.md`: two extractors, three wind, the lab, then generators; the lab one constructor then raiders).

### K-open-comet-solars-starve-the-opening
**Claim.** On Comet Catcher (wind 1 to 4) a vehicle opening that buys its energy with solars starves its metal: six
solars are 930 metal at 155 each, eight Blitzes' worth, and with two extractors giving 6 metal a second the store sat
at zero from minute two to minute thirteen while the plant, which with the commander helping can spend 25 a second
on Blitzes, made 28 Blitzes in seventeen minutes (comet-3). Wind on Quicksilver costs 40 a generator, which is why
Matt's eight generators were cheap there. The bound on this map is metal: extractors on the strip's spots before
generators beyond the second, and a solar only when energy stalls.
**Status.** observed (2026-09-23), one game, the corrected order untried.
**Evidence.** `run/matches/1790052453-comet-3-medium`: the list (two extractors, three solars, plant, three solars,
assist), `analyze_match` curves (bank 0 and income 6 to 8 through minute 11).
**Would be wrong if.** The same list with two solars and four extractors were no faster to the tenth Blitz.
**Used by.** the player's brief (Comet Catcher section).

### K-open-comet-vehicle-list
**Claim.** On Comet Catcher from the west strip, the list "extractor spot_45, extractor spot_50, solar, solar,
vehicle_plant, extractor spot_36, extractor spot_43, solar, assist" for the commander, `armcv:2` then Blitzes from
the plant, and a list of the strip's spots with a turret after each pair for each constructor, gives the plant at
0:41, two constructors by 1:30, the first Blitz at 2:08 and thirteen by 4:07, six extractors at 2:30, nineteen and
income 41 by minute 10 (comet-4); against BARb medium that is ahead on the board all game (its army value under
ours until minute 11; extractors 19 to 7 at minute 10). What it does not do is spend the bank: 2,100 metal banked
from minute 9 on one plant.
**Status.** measured (2026-09-23), one game; the same list against hard is untried.
**Evidence.** `run/matches/1790052919-comet-4-medium`: the turn-one calls, `analyze_match` curves.
**Would be wrong if.** The list on the east strip, or against hard, were a minute slower.
**Used by.** the player's brief (the Comet Catcher section).

### K-open-comet-pro-order
**Claim.** Experienced players open Comet Catcher with a vehicle plant before 1:00 on solars, and the order by the
clock is: extractors 0:03, 0:10 (and 0:24 when a third spot is in reach), solars 0:31, 0:40, 0:49, the plant 0:58
(standing 1:16), a solar at 1:29; the plant two Rascals (1:17, 1:20), a constructor (1:24), a second (1:41), two
Rascals, then Flashes from 2:12 one every ten to fifteen seconds (thirteen by 5:00, twenty-four by 6:50) with a
constructor at 3:09, 4:30 and 5:46 and a Rascal now and then (seven in all). Metal is spent as it comes (the store
under 30 through minutes 2-4), then 300-500 sits banked from 5:00 while income goes 23, 32, 43 at 4:00, 5:00, 6:52 (9,
13, 19 extractors); solars 5 by 2:20, 7 by 4:00, 17 by 6:46 for one plant, the energy store dipping to 22 while the
plant went up and to 114 in minute 5, never zero. From 5:00 the plant gets nano turrets (three by 6:37) and an energy
storage (6:18): one plant made faster, no second factory in seven minutes. The other player's opening was the same
shape a half-minute faster (two solars, the plant at 0:34 standing 0:53, a solar, then extractors) and his energy sat
at 0-30 through minutes 4-6 on 18 solars because his plant ran without pause.
**Status.** transcribed (2026-09-22) from one people's 1v1 (both rank 5, skills 52 and 43, Armada against Armada,
2026-09-09, game test-31251).
**Evidence.** `run/matches/1790122851-replay-vak-vs-artur-comet` (run/replay_match.py on the user's demo): record-0 is the west player ([MADO]VAK, the winner, from
(954, 2520): fifty elmos from where the arena places us), record-1 the east player; created events, plant production by
`by`, resource samples.
**Would be wrong if.** Other high-level games on this map opened with a bot lab, or with wind, or banked nothing for
a second plant by minute 5.
**Used by.** the player's brief (the Comet Catcher section: the opening to copy on this map).

### K-open-comet-commander-expands
**Claim.** On Comet Catcher the commander is the frontier builder, not the plant's helper: from 2:00 it walks toward
the middle and builds an extractor every twenty to sixty seconds at the middle spots nearest home (our table's
spot_43 at 2:33, spot_38 and spot_39 by 3:36, spot_26 and spot_24 by 4:46, spot_33 and spot_31 by 6:15, at x 1,900 to
3,700), a light turret beside each pair (2:45, 3:42, 4:53, 6:22), a solar or two and a radar at each outpost (4:09,
5:40), and stands 2,600 elmos east of its start at 6:30 at full health; the other commander did the same from its
side and the two never met. The four or five constructors take the strip's spots behind it (spot_45, spot_50,
spot_52, spot_61, spot_63, spot_64, spot_59, spot_54, a turret at the far ones) and from 5:00 build the nano turrets on
the plant. Neither commander fought except against the first scout cars at home.
**Status.** observed (2026-09-22), one game, both players.
**Evidence.** `run/matches/1790122851-replay-vak-vs-artur-comet`: the commander's position every 30 s and `by` on the created events (the commander built 12 of the
west player's 21 extractors, all eight light turrets, three radars and 12 of 17 solars).
**Would be wrong if.** Other high-level games on this map kept the commander at the plant past minute 3, or lost it
in the middle to the other side's raiders.
**Used by.** the player's brief (the Comet Catcher section). Our games to date keep the commander home helping the
plant (comet-4's list ends `assist`); this is the largest difference between the players' game and ours.

### K-open-comet-rascals-lose-to-the-commander
**Claim.** The plant's first units on both sides are Rascals (scout cars, 31 metal), for sight; a Rascal swarm loses.
The east player made sixteen in six minutes and sent them at the west base in twos and threes from 1:30; the
commander, one light turret at the plant (built 2:03, standing 2:16) and the first Flashes killed fifteen of them (six
in minute 2 alone, at the base) for one Rascal and one Flash, and he had no army at 4:00 and five Flashes at 6:00
against fourteen. The winner made seven Rascals and spent them as scouts deep in the other half (lost 4:21, 5:23,
5:31 at x 5,300-6,800).
**Status.** observed (2026-09-22), one game.
**Evidence.** `run/matches/1790122851-replay-vak-vs-artur-comet`: plant production by `by`, destroyed events with places (record-1: fifteen armfav lost by 5:35,
six of them 2:04-2:34 within 900 of the west plant).
**Would be wrong if.** A Rascal opening in another high-level game on this map paid for itself in constructors or
extractors killed.
**Used by.** the player's brief (two Rascals for sight, then Flashes; a light turret at the plant by 2:00 for the
other side's scout cars).

### K-open-comet-flash-raids-take-the-resign
**Claim.** The winner's Flashes were never one ball: from 4:00 they spread 2,400-4,800 elmos across the map in twos
and threes at the other side's extractors, constructors and solars (a start extractor at 4:44, four solars beside the
plant 5:46-5:51, the north end's two extractors, a turret and two constructors 6:08-6:29), losing seven Flashes, three
of them to turrets at the plant; the other player resigned at 6:45 with fifteen extractors to nineteen, income 36 to
43 and five Flashes to fourteen. The resign came on the army count and the unanswered raids, not on the economy.
Against this opponent the same raids go at extractors and constructors, never at the commander, whose D-gun takes a
small group (comet-1).
**Status.** observed (2026-09-22), one game; against BARb untried in this shape (our Flash groups have gone as one
ball).
**Evidence.** `run/matches/1790122851-replay-vak-vs-artur-comet`: army centroid spread per 30 s (r 2,466-4,863 from 4:00), enemy_destroyed events with places,
record-1's four self-destruct commands at 6:45 (CMD_SELFD, the resign).
**Would be wrong if.** The raids in other high-level games here were one group, or a person held on at equal extractors
and won on a later army.
**Used by.** the player's brief (the Comet Catcher section).

### K-plan-target-order-is-a-chain
**Claim.** A `target` search answers with the chain to the thing asked and what pays for it inside the horizon, and
nothing else: no soldiers of other types, no economy past the horizon, no defence. Followed to the letter it gets the
unit early and leaves the game thin around it: in hands-2-bulldogs seven constructors by 9:30, one plant, no soldier
between the sixth Flash (about 4:30) and the first Bull (9:36), metal full from minute eight. The user's own annealer
of the same shape did the same ("they work at getting the unit you asked for quickly, but can fail at setting you up
for a stable economy afterwards"). The economy and the army around the chain are the requester's to add, as goals
(units with counts and times, `income:N by M:SS`) or by deviating from the answer; experience is meant to layer over
the search, not the other way round.
**Status.** observed (2026-09-22), four player games (plan-1, plan-2, hands-1, hands-2) and the user's experience.
**Evidence.** the four games' `strategist-0.jsonl` `search` calls (`target armbull` alone each time) and their
`queue` calls; hands-2's curves (`run/floor.py`: mfull 50 %).
**Would be wrong if.** A target search with income and raider goals, followed to the letter, held the economy and
the front through minute fifteen in a Bulls game. Close to it already: yard-1-bulldogs asked six Flashes, the plant,
three Bulls and income 25 in one search, won at 17:10 with 13 Bulls and metal full 0.7 % of the game; but the player
skipped the answer's advanced solar and stalled energy 8:45-10:00 (its own note), so the answer was still not
followed to the letter.
**Used by.** H-PLAYER-PLAN-SEARCH (income goals; the guide: "a target order is a chain to the thing asked").

## Comet Catcher Remake 1.8: the replay survey (2026-09-23)

Claims from the game cards of 29 public 1v1 duels (both players OS 25 and above, games of 2026-09-20 to 2026-09-23 on BAR test-31357 to 31383), filed by the synthesis agent against `docs/knowledge/questions.md` and checked with `run/replays/check.py`; the map file `docs/knowledge/maps/comet_catcher_remake_1.8.md` and the index `docs/knowledge/replays.md` point here. Spot numbers are the cards' (the game's own 80-spot list), not the bot's (the engine's 75); they map to the bot's by position (nearest within 130), and every spot carries its grid cell (8x8 over 8192x6144). Unit names: armfav Rover, corfav Rascal, armflash Blitz, corgator Incisor, corak Grunt, armpw Pawn, armflea Tick, armstump Stout, corraid Brute, armllt/corllt the light laser turret. "W"/"L" are the winners' and the losers' sides. Remaining card limits: `fights` is the eight costliest cell-minutes per game; the cards carry no unit positions over time (held back or chasing cannot be read); the killer of the first extractor is "?" on 10 sides; one manifest start (bcaeb16a, Chronopolize) disagrees with the card's B5 and the card is used. Fixed after the survey (cards re-cut 2026-09-23 evening): the faction field, the spots' cells, the first-enemy-seen field (removed: a replay sees everything), the factories' order, the last 15 s of kills counted apart, cells on every building, the first tier-2 unit.

### K-open-start-geometry-mirror-pairs
**Claim.** On Comet Catcher the starts come in half-turn mirror pairs: A4 (about 900, 2540) with H5 (7290, 3600), B5
(1420, 3560) with G4 (6770, 2580); H1 (7290, 760) has no west mirror in these games. The spot cells mirror to within a
cell (A4's spot_28/spot_30 and H5's spot_48/spot_51; B4's spot_36 and G5's spot_42; the B3-B1 strip and the G6-G8 strip).
Each start has three spots within the first minute's walk (two before the factory, the third at 0:44-1:25), and the
first raid on an extractor comes at 3:22 from G4 (8 sides) but 5:10 from its mirror B5 (12), so the geometry alone does
not set the raid clock. Pairings: A4-H5 11 games (A4 won 7), B5-H5 8 (B5 won 5), A4-G4 5 (G4 won 3), B5-G4 3 (G4 won 2),
H1 2 (H1 lost both); west starts won 17 of 29 with the higher-OS player in the west in 14.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 (card start cells; manifest start positions).
**Would be wrong if.** A spot list with coordinates showed the spots off the half-turn by more than a spot's spacing. What
the question asks (spots in the first minute's walk, the distance to the middle, the passages) needs walking times and
spot coordinates per start, which the cards do not carry: a map-facts block with the start's spot distances would.
**Used by.** (candidate: the arena's start placement; a fair test uses mirrored starts)

### K-open-comet-radar-nano-solars-habits
**Claim.** Standing habits across the 58 sides: a radar by 3:04 (winners' median; losers 2:58), in 57 of 58 sides by
8:00; the first nano turret at 5:42 (winners and losers alike), in 50 of 58 by 8:00, winners a median 2 by 8:00 against
1; no wind generator in any of 58 sides (solars only: two before the factory, 16 by 8:00 for winners against 13); constructors
at 2:00 a median 2 for winners against 1 for losers;
dragon's teeth (cordrag) in the opening in 2 Cortex sides.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29.
**Would be wrong if.** Later versions changed the wind range; wind on this map is low (K-open-comet-solars-starve-the-opening
says 1 to 4). **Refines** K-open-comet-solars-starve-the-opening: the people buy solars, two before the plant and a
median 13 to 16 by 8:00, and it does not starve them because they keep adding extractors (18 by 8:00 for winners); the
claim's point (extractors before generators beyond the second) holds only for the first two minutes.
**Used by.** (candidate: the brief's standing instructions: radar by 3:00, a nano turret by 6:00)

### K-open-comet-constructors-take-the-flanks
**Claim.** The extractors past the home three and the next pair are the constructors' work: the commander built a median
5 of the winners' extractors to 8:00 (25%) and the constructors the rest; the commander built half or more in 7 of 29
winners and 10 of 29 losers, and built 5 or more after 2:00 in 9 winners and 10 losers. Winners had more constructors at
10:00 (median 7 against 5) and at 12:00 (9.5 against 6).
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 (`build_order[].by`).
**Would be wrong if.** The card's `by` attributed an assisting constructor's work to the commander or the reverse.
**Refines** K-open-comet-commander-expands (one game): commander-built extractors in the middle columns C-F before 8:00
were 3 or more in 10 winners and 11 losers, so it is a style, not the winners' pattern.
**Used by.** (candidate: the brief's constructor count, 7 by 10:00)

