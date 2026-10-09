# Tier 2: when, how, and what it buys

Seeded 2026-09-19. "Local source" = `upstream/Beyond-All-Reason`; unit paths relative to it. Scope: 1v1 land, bot labs.

### K-t2-entry-cost
**Claim.** Entering tier 2 through bots costs ~3030-3070 metal before any benefit: advanced bot lab 2600 M / 15-16k E
(buildtime 25000-26000) plus one advanced constructor bot 430 M (Armada) / 470 M (Cortex), 6900 E. Each advanced extractor
is a further 620 M / 7700 E (Armada) or 640 M / 8100 E (Cortex). Web guides quote "roughly three thousand metal" (and one
says ~4000 including the first extractors), which matches.
**Status.** supported (2026-09-19) for costs (local source); web figure reported
**Evidence.** `units/ArmBuildings/LandFactories/armalab.lua`, `CorBuildings/LandFactories/coralab.lua`,
`ArmBots/T2/armack.lua`, `CorBots/T2/corack.lua`, `ArmBuildings/LandEconomy/armmoho.lua`, `cormoho.lua`.
https://www.crdhq.com/articles/bar-t2-timing-rules-and-juno-missile-guide; https://www.beyondallreason.info/guide/in-depth-look-at-economy.
**Would be wrong if.** (numbers) the build menu showed other costs in our game version.
**Used by.** (candidate: H-T2-GATE)

### K-t2-who-builds-what
**Claim.** The advanced bot lab is built by a T1 constructor (it is in the constructor bot's build list, not the
commander's). Advanced extractors, fusion and advanced converters are only in the advanced constructor's list. So the
chain is: T1 constructor → advanced lab → advanced constructor → advanced extractors.
**Status.** supported (2026-09-19) — local source
**Evidence.** `units/ArmBots/armck.lua:48` (`armalab` in buildoptions); `units/ArmBots/T2/armack.lua:40,46` (`armfus`,
`armmoho`); `units/armcom.lua` has neither.
**Would be wrong if.** The AI interface refused `armalab` for a constructor bot or offered it to the commander.
**Used by.** (candidate: H-T2-GATE assigns the lab to constructors and has the commander assist)

### K-t2-gate-thresholds
**Claim.** Community rule of thumb for starting the advanced lab in 1v1: income of at least +30 M/s and +500 E/s; or +20 M/s
and +500 E/s with 1000-1500 metal banked; otherwise stay tier 1, because a transition the economy cannot carry idles the
factories and loses to a tier-1 push. Deciding question: "would 3000 more metal of T1 units kill the opponent sooner?"
**Status.** reported (2026-09-19)
**Evidence.** https://www.crdhq.com/articles/bar-t2-timing-rules-and-juno-missile-guide (2026, undated; aggregator of
community advice); the official economy guide gives the softer "+20 metal and +500 energy, 1000-2000 metal stored":
https://www.beyondallreason.info/guide/in-depth-look-at-economy.
**Would be wrong if.** A brain that started the advanced lab at +20 M/s without a bank did as well against BARb medium as
one that waited for +30 M/s / +500 E/s.
**Used by.** (candidate: H-T2-GATE — our v5 brain reaches ~+38 M/s at minute 17, so by this rule it should have gone T2
around minute 10-12 when it crossed +20-30; its energy income at that time is unknown and must be logged)

### K-t2-needs-build-power
**Claim.** The advanced lab is a build-power problem as much as a metal problem: buildtime 25000 means 312 s for one
constructor bot (80), 83 s for the commander alone (300), ~54 s for commander + 2 constructors (460), at which point it
drains ~48 M/s and ~280 E/s. Without banked metal or assisting builders it either takes minutes or stalls everything else.
**Status.** supported (2026-09-19) — arithmetic from local unit files; not observed
**Evidence.** `armalab.lua` (buildtime 25000, metalcost 2600, energycost 15000); workertime of `armcom.lua` 300, `armck.lua` 80.
**Would be wrong if.** A measured advanced-lab build with known builders finished >20% off buildtime/Σworkertime while not stalled.
**Used by.** (candidate: H-T2-GATE — commander plus ≥2 constructors (or 2 construction turrets in range) assigned; require
~1000 M banked or accept a 2-minute build)

### K-t2-moho-first
**Claim.** The first thing tier 2 buys is advanced extractors on the safest existing spots: 4x yield (extractsmetal 0.004 vs
0.001) for 620-640 M, i.e. +3 extractor-equivalents per upgrade, and 2800-3900 health vs 270. With a T1 extractor at about
+2 M/s on Quicksilver (inferred from v5-probe: +38 M/s with 18 extractors) one upgrade adds ~+6 M/s and repays its metal in
~100 s; five upgrades roughly double our whole T1 income. An advanced constructor (210-220 build power) needs ~70 s each.
**Status.** supported (2026-09-19) for costs/yield ratio (local source); per-spot income and payback are inference
**Evidence.** `armmoho.lua` (extractsmetal 0.004, metalcost 620, energyupkeep 20, health 2800), `cormoho.lua` (640 M, 3900 hp),
`armmex.lua` (0.001, 270 hp). Upgrade-first advice: https://www.beyondallreason.info/guide/in-depth-look-at-economy;
upgrade command: https://www.beyondallreason.info/commands/upgrade-t1-mex. See also K-rules-mex-upgrade-in-place.
**Would be wrong if.** Metal income did not rise by ~3x the old spot's yield when an upgrade finished, or upgrades were
lost to raids faster than they repaid (~100 s).
**Used by.** (candidate: H-T2-MOHO — advanced constructors upgrade spots nearest home first, 2 advanced constructors)

### K-t2-moho-energy
**Claim.** An advanced extractor draws 20 E/s (T1: 3 E/s) and costs 7700-8100 E to build; ten upgrades need +200 E/s upkeep
plus ~110 E/s while building each. The +500 E/s gate in K-t2-gate-thresholds is what pays for this.
**Status.** supported (2026-09-19) — local source
**Evidence.** `armmoho.lua` / `cormoho.lua` (energyupkeep 20, energycost 7700/8100, buildtime 14900/14100).
**Would be wrong if.** Energy usage did not step up by ~20 per finished upgrade.
**Used by.** (candidate: H-T2-MOHO checks stored energy > 50% before each upgrade)

### K-t2-fusion-numbers
**Claim.** Fusion: Armada 3350 M / 18000 E → +750 E/s; Cortex 3600 M / 22000 E → +850 E/s; buildtime 54000-59000 (257-268 s
for one advanced constructor, so it needs assistance). Metal per E/s: fusion ~4.2-4.5, advanced solar 4.4-4.6
(350-370 M → 80 E/s), solar 7.5-7.75, wind 40/avg-wind (3.1 on Quicksilver). Fusion is therefore not cheaper energy per
metal than wind on a windy map or than advanced solar; its value is density, steadiness and builder time. It follows the
extractor upgrades rather than preceding them.
**Status.** supported (2026-09-19) for numbers (local source); the ordering advice is reported
**Evidence.** `armfus.lua`, `corfus.lua`, `armadvsol.lua` (energymake 80; the official guide says 75 — local wins),
`coradvsol.lua`. Ordering: https://www.beyondallreason.info/guide/in-depth-look-at-economy.
**Would be wrong if.** (numbers) unit files change. (ordering) a fusion-before-upgrades brain had higher metal income at minute 15.
**Used by.** (candidate: no fusion before ≥6 advanced extractors; on Quicksilver keep adding wind instead)

### K-t2-advanced-converter
**Claim.** The advanced converter turns 600 E/s into ~10.3 M/s (58 E per metal) for 370-380 M / 21000 E; the T1 converter
turns 70 E/s into 1.0 M/s (70 E per metal) for 1 M / 1150-1250 E. T2 conversion is only ~17% more energy-efficient; it is
a space/cost-per-capacity gain, not a reason to go T2.
**Status.** supported (2026-09-19) — local source
**Evidence.** `armmmkr.lua` (energyconv_capacity 600, efficiency 0.01724), `armmakr.lua` (70, 0.01429), Cortex equivalents.
**Would be wrong if.** Measured metal from converters differed from capacity × efficiency when energy was above the threshold.
**Used by.** (none)

### K-t2-unlock-army
**Claim.** Besides economy, guides give one military reason to go T2 even with a borderline economy: the opponent is dug in
behind static defence that T1 cannot break. For bots the relevant unlocks (ranges from local files) are Cortex Sheldon (850)
and Arbiter (1210), Armada Hound (650) and Sharpshooter (900), all outranging the heavy laser tower (620); only the
plasma batteries Gauntlet/Agitator (1220/1245) out-range them.
**Status.** reported (2026-09-19) for the advice; ranges supported from local source
**Evidence.** https://www.crdhq.com/articles/bar-t2-timing-rules-and-juno-missile-guide;
https://www.crdhq.com/articles/when-to-transition-to-t2-and-when-to-resign. `units/CorBots/T2/cormort.lua`, `corhrk.lua`,
`ArmBots/T2/armfido.lua`, `armsnipe.lua`, `ArmBuildings/LandDefenceOffence/armhlt.lua`, `armguard.lua`, `corpun.lua`.
**Would be wrong if.** BARb medium's base fell to T1 waves once our economy matched its own (then T2 army is unnecessary).
**Used by.** (candidate: after upgrades, advanced lab batch = Sheldon/Hound-class skirmishers behind a T1 screen)

### K-t2-barb-goes-tier-2
**Claim.** BARb medium starts a tier-2 factory in about two games in three, never before minute 15, median just before
minute 20; a game of ours that is not over by then is tier 1 against tier 2.
**Status.** measured (2026-09-20), ground truth from 96 heuristic games (v23-v26), Quicksilver only.
**Evidence.** 60 of 96 games; first advanced factory at 15.5 / 19.7 / 37.4 minutes (earliest / median / latest), mostly
advanced bot labs. Of games lasting 20+ minutes (67) it had one by minute 20 in 27; of 25+ (52), 39; of 30+ (44), 37.
NW losses 24 of 38 (median loss minute 22); SE timeouts 16 of 18; SE wins 12 of 29. Commander game 8c's attack met
corsumo and corcan at minute 32.
**Would be wrong if.** Other maps or profiles showed a different clock.
**Used by.** H-T2-GATE (why at all), the commander's brief.

### K-t2-upgrades-starve-without-help
**Claim.** With nothing banked, every consumer gets a share of the metal income, and an extractor upgrade built by one
advanced constructor beside two working labs takes minutes instead of its nominal 70 s.
**Status.** observed once (2026-09-20), t2-first-look match 00.
**Evidence.** Lab ordered minute 10, finished 13 (commander and constructors helping); first armmoho created minute 16,
finished minute 22, with metal at 0 banked, +35-40 income, fully spent, two labs and nine constructors drawing on it.
**Would be wrong if.** With two helpers and the advanced lab idle the first upgrades still took over two minutes.
**Used by.** H-T2-ASSIST (upgrade helpers), H-T2-PRODUCTION (no soldiers from the advanced lab before four upgrades).

**Simulated check (2026-09-20).** Partly contradicted; see K-t2-mix-cortex, K-t2-mix-armada and K-t2-tower-line-needs-mass.
Out-ranging the tower is not enough on its own: Sheldon and Arbiter armies are the two worst Cortex mixes in the sweep
(-0.58 and -0.79 over all scenarios) because they have no health, and only the Mammoth — which barely out-ranges the
tower at all — is reliably positive against a tower line. The Sharpshooter could not be judged (K-t2-sim-cannot-judge).

### K-t2-barb-composition
**Claim.** BARb medium's army at minutes 20 and 25 is a tier-1 army. Mean per match over 96 recorded games (v23-v26),
metal excluding its commander: as Armada 4960 M at minute 20 (armrock 8.3, armham 7.5, armpw 9.1, armwar 4.7, armstump 1.5,
armjanus 1.3) and 3770 M at minute 25, of which the only tier-2 unit worth counting is armfido (0.6 then 1.6); as Cortex
3940 M at minute 20 (corstorm 13.9, corthud 9.1, corak 5.8, corraid 2.1) with no tier-2 at all, and 4160 M at minute 25
with about 500 M of it tier-2 (coramph, cortermite, cormort, corban, corhrk, corpyro, all under one unit a game).
Its **mobile army does not grow** across minutes 15-30 (Armada 4190 -> 4960 -> 3770 -> 4480, Cortex 3640 -> 3940 -> 4160 ->
6270) while its **static line more than triples** (Armada 2170 -> 6190, Cortex 1820 -> 7820 including its factories).
Light towers 12-14 at minute 20-25; heavy towers only 1.3-2.1, plus 2 Beamers or Twin Guards; the 1220+ range plasma
batteries (armguard, corpun, armamb, cortoast) are a minute-30 thing, 0.2-0.6 a game.
**Status.** supported (2026-09-20) — counted from match logs, not arena-tested against a changed brain
**Evidence.** `docs/studies/data/tier2-barb-army.csv`, from `truth-*.jsonl` in `run/matches/*v2[3456]-*/NN`
(96 matches, 48 a faction; 31/36 reach minute 20, 23/29 minute 25). `crates/combatsim/tools/tier2_study.py army`.
Faction from `script.txt`; BARb builds a small minority of the other faction's units and those are counted as they are.
**Would be wrong if.** A later batch on another map or against a stronger brain showed its mobile army growing with its
income, or more than a third of its minute-25 metal in tier-2 units.
**Used by.** K-t2-mix-cortex, K-t2-mix-armada, K-t2-tower-line-needs-mass (all use this as the opponent).

### K-t2-lab-build-power
**Claim.** The advanced bot lab is 600 build power against the tier-1 lab's 150, and that — not the units — is the
first thing it buys. A tier-1 bot lab turns its 150 into about 10 metal a second of army (Thug 140 M / 2100 buildtime,
Mace 130 M / 2200). Sustaining +30 metal a second of the tier-1 line therefore needs 3.3 tier-1 labs' worth of build
power and +45 needs 5.0; the same spend with half the metal in tier-2 units needs 1.6 tier-1 labs plus 0.5-0.8 of the
advanced lab (+30) or 2.4 plus 0.8-1.1 (+45). A nanotower adds 200.
**Status.** supported (2026-09-20) — arithmetic from local source, not observed
**Evidence.** `armlab.lua`/`corlab.lua` workertime 150, `armalab.lua`/`coralab.lua` workertime 600; buildtimes
armzeus 8000, armfido 6500, corcan 12000, corsumo 65000, corthud 2100, corstorm 1950, armham 2200, armrock 2010.
`crates/combatsim/tools/tier2_study.py sustain`.
**Would be wrong if.** A recorded game showed a single tier-1 lab sustaining much more than ~10 M/s of bots without
assistance, or the advanced lab finishing units far off buildtime/600.
**Used by.** (candidate: H-T2-GATE and the production heuristic — at +38 M/s our brain cannot spend through two tier-1 labs)

### K-t2-mix-cortex
**Claim.** Out of a `coralab`, build **Sumos (`corcan`)** for about half the army's metal and keep Thug and Aggravator
for the other half (50 % corcan / 27.5 % corthud / 22.5 % corstorm). At equal metal against BARb's measured minute-20
and minute-25 armies this scores +0.35 (dense formation) to +0.68 (loose) of margin, where the tier-1 line alone scores
-0.07 to +0.18. It holds at 3000, 6000 and 10 000 metal, against both of BARb's factions, and against a pure tier-1
BARb army. The Sumo is 560 M for 6000 health and 344 damage a second at 275 range — 10.7 health a metal against the
Thug's 7.9 — and its beam is the one weapon type the simulator was fitted on. The tier-1 half is kept for reasons
outside the fights (K-t2-lab-build-power and K-t2-army-energy); head to head a pure Sumo army is within 0.11 of the
mix in the field and ahead of it in 9 of 12 base-assault cells at the loose formation spacing.
**Status.** simulated (2026-09-20); never run in the arena
**Evidence.** `docs/studies/tier2-army.md` section 4, `docs/studies/data/tier2-fights.csv` (3780 cells, 24 seeds each,
median seed sd 0.023, typical standard error 0.005). Termite is a close second (+0.46/+0.58) and `can+hrk` scores higher
still (+0.52/+0.74) but leans on a weapon the simulator models badly (K-t2-sim-cannot-judge).
**Would be wrong if.** An engine duel of `corthud:12,corstorm:13,corcan:5` against the same metal of pure
corthud/corstorm, both against a scaled BARb army, failed to favour the Sumo mix by at least a fifth of the simulated
margin; or an arena batch with the mix did no better than one without.
**Used by.** H-T2-PRODUCTION's Cortex line. Note for that rule: its placeholder pairs corcan with **cormort**
(Sheldon), and cormort is one of the two worst Cortex options in the sweep — `t1+mort` -0.21/+0.14 and `mort` alone
-0.58 over all scenarios, against `t1+can` +0.35/+0.68. Pair the Sumo with the tier-1 line instead, and with a
Mammoth when a tower line has to be broken.

### K-t2-mix-armada
**Claim.** Out of an `armalab`, build **Welders (`armzeus`) and Hounds (`armfido`)** for about a third of the army's
metal each and keep the tier-1 line for the last third. +0.32 to +0.49 of margin against BARb's minute-20 and
minute-25 armies, where the tier-1 line alone scores +0.06 to +0.19. The Welder is the part that does not depend on a
weapon the simulator models badly (280-range lightning, area 8, 3500 health for 350 M) but it cannot go alone: a pure
Welder army scores -0.60 at 10 000 metal because 280 range and 48 speed is out-shot on the way in. Gunslinger,
Sprinter and Platypus never beat the tier-1 line in any of the 54 field cells (closest: the Platypus at 3000 metal and
loose spacing, -0.01 behind). The tier-1 third is kept for the reasons in K-t2-lab-build-power and K-t2-army-energy,
not because the screen wins fights: `t1+zeus+fido` is within 0.06 of `zeus+fido` in the field and behind it in 21 of
the 24 base-assault cells.
**Status.** simulated (2026-09-20); never run in the arena
**Evidence.** `docs/studies/tier2-army.md` section 4, `docs/studies/data/tier2-fights.csv`. The Hound's half of this
rests on area damage (area 72), which the simulator over-rates in a dense formation, so the mix is reported with the
spacing-160 column beside the spacing-56 one; the ranking is stable between them.
**Would be wrong if.** An engine duel put a Welder-and-Hound mix at or below the tier-1 line at equal metal; or the
measured fighting spacing of a real army turned out to be tighter than 56, which would mean the Hound number is worse
than the study's own upper bound.
**Used by.** H-T2-PRODUCTION's Armada line, whose armzeus/armfido placeholder this supports at roughly one Welder to
one Hound by metal (1.7 Welders to 2.1 Hounds a minute at +30 M/s), behind a tier-1 third.

### K-t2-tower-line-needs-mass
**Claim.** At equal army metal, an assault on BARb's minute-25 base — 12 light towers, 2 heavy, 2 medium, its mobile
army behind them, all of it extra metal on top of ours — loses for almost every mix we could build. The one reliably
positive answer either faction has is the Cortex **Mammoth (`corsumo`)** behind a tier-1 screen: +0.41 (dense) / +0.25
(loose) against a garrison that comes out, where the tier-1 line is -0.38 / -0.26 and the Sumo mix -0.16 / +0.07.
Armada has no simulated answer that does not rest on the Fatboy's area-300 shell. Margins improve steeply with budget:
the same mixes are strongly negative at 3000 metal and positive at 10 000, so **the tower line is a mass problem
before it is a unit-choice problem**.
**Status.** simulated (2026-09-20); never run in the arena
**Evidence.** `docs/studies/data/tier2-fights.csv`, scenarios `base6-*` and `base12-*`. Tower line 2280 M (Armada) /
2430 M (Cortex) on top of the budget; defender energy 7000 stored and 350 a second, from BARb's own measured wind,
solar and fusion at minute 20-25 (median 314-406 a second, 7000 stored).
**Would be wrong if.** The garrison turns out to stand still rather than sally (the two brackets differ by 0.3-0.8 for
anything that out-ranges the towers), or repair and rebuilding during the assault change the answer, or BARb's
commander — 2700 M and 3700 health, left out of every one of these fights — is what actually holds the line.
**Used by.** (candidate: a wave gate that refuses a base assault below some multiple of the seen tower metal)

### K-t2-army-energy
**Claim.** A tier-2 bot army is an energy purchase as much as a metal one. Every beam and lightning weapon charges
energy a shot: Welder 59 a second at full rate, Sumo 56, Termite 75, Mammoth 125, Sharpshooter 50. Building the
recommended mixes costs about **390-500 energy a second at +30 metal a second and 580-740 at +45**, on top of 20 a
second for each advanced extractor. In the fights, performance falls off a cliff below roughly 150 energy a second of
spare income (a pure Sumo army swings 0.77 of margin between 50 and 150) and is flat above 350. A half tier-1 mix
barely moves, because the tier-1 half fires for free.
**Status.** simulated (2026-09-20) for the cliff; supported (local source) for the per-shot and build costs
**Evidence.** `docs/studies/data/tier2-energy.csv` (4800 fights, our income swept 50-2000 with 4000 stored);
`crates/combatsim/tools/tier2_study.py sustain`; `energypershot` in the unit files. This is the same mechanism as
K-units-laser-towers-need-energy and it supports the +500 E/s of K-t2-gate-thresholds from a second direction.
**Would be wrong if.** A recorded game showed a tier-2 army's damage output not dropping while energy was stalled.
Note the sweep gives 4000 stored, so a short fight is paid out of the store and this understates a long siege.
**Used by.** (candidate: H-T2-GATE's energy threshold; a production rule that falls back to tier-1 during an energy stall)

### K-t2-sim-cannot-judge
**Claim.** Four kinds of tier-2 unit the combat simulator cannot be trusted about, so their numbers in
`tier2-army.md` are not evidence. (1) **Area-damage units** — Fatboy (area 300), Recluse (90), Arbiter (70), Hound
(72), Termite (42): the simulator's armies fight in the formation they were spawned in, so a shell takes four
neighbours where the engine's takes one; their margins are upper bounds. (2) **Vertical-launch and high-trajectory
weapons** — Arbiter, Sheldon: flight time is computed as a straight line and there is no arc over terrain. (3) **The
Sharpshooter** — cloaked in the game, not in the simulator, and 580 health for 680 metal once it can be shot at; its
-0.90 is an artefact, and Armada's real answer to a tower line may well be Sharpshooters. (4) **The Fiend's
flamethrower** — the engine's flame bounces, lingers and grows; the simulator resolves each of its 16 projectiles as a
point explosion, error of unknown sign. Three crawling bombs and the EMP spider are excluded from the unit table
outright (`_excluded` in `crates/combatsim/data/units.json`).
**Status.** supported (2026-09-20) — property of the model, from `docs/studies/combat-sim.md` and the unit files
**Evidence.** `combat-sim.md` *Where it is wrong* (artillery is the worst miss in the tight duel table, and loosening
the formation to 160-240 recovers the engine's answer exactly); `crates/combatsim/tools/extract_units.py` for what is
excluded and why; unit files for cloak, `weapontype` and `areaofeffect`.
**Would be wrong if.** An engine duel reproduced a simulated area-damage margin within a fifth, or a measurement of
the spacing a real army fights at came out near 56.
**Used by.** K-t2-mix-armada, K-t2-tower-line-needs-mass (both report the caveat rather than the number alone).

### K-t2-barb-labs-at-minute-21-and-we-notice-three-minutes-late
**Claim.** BARb medium reaches a finished advanced lab in 211 of 430 Quicksilver games, median at minute 21 (10th
percentile 18, 90th 27); in this dataset it never has one before minute 15. We see the first tier-2 thing of theirs a
median of 3 minutes after the lab finished (quartiles +2 and +4 minutes), and in 17 further games we saw tier-2 units
of theirs when they never built a lab at all — their resurrection bots raising our own advanced units
(K-army-dead-waves-are-resurrected). So a tier-2 sighting is late evidence and not proof; the clock is the better part
of the estimate. A logistic on the evidence, offset by the per-minute base rate, scores a Brier of 0.054 against 0.083
for the base rate alone (AUC 0.87 within minutes 20-40).
**Status.** measured (2026-09-20) over every Quicksilver game with a ground-truth file.
**Evidence.** `docs/studies/tempo-model.md`, section `y_t2lab`; `run/tempo_model.py` (an advanced lab is a `factory`
costing >= 1500 metal, tier-1 labs being 470-570 and advanced 2600). The tier-2 unit list used as evidence was derived,
not assumed: over the 144 games of v31/v32/v34 these are the `army` names with zero sightings before the opponent's
first finished lab and at least a hundred after (`T2_ARMY` in the script).
**Would be wrong if.** Against another profile or on another map the lab came materially earlier, or a scouting rule
cut the three-minute lag.
**Used by.** (none yet) — candidate: the commander's briefing states the probability rather than waiting for a sighting.

### K-t2-mobile-build-power-by-cost
**Claim.** By metal per point of build power (`workertime`), the standard roster's builders rank: construction
turret `armnanotc`/`cornanotc` 230 metal for 200 (1.15; reach 400, static, 560 health); bot constructor `armck` 1.38
/ `corck` 1.41; vehicle constructor `armcv` 1.50 / `corcv` 1.53; the tier-2 fast assist units Butler `armfark` 1.50
(210 metal, 140 power, speed 75, 335 health, from `armalab`; it builds only solars, wind, extractors, converters and
radar), Consul `armconsul` 1.67 (250 / 150 / 78 / 1,080, from `armavp`, with a tier-1 constructor's build list) and
Twitcher `corfast` 1.68 (210 / 125 / 90 / 830, from `coralab`, a tier-1 constructor's list); advanced vehicle
constructor `armacv` 1.90 (550 / 290 / 52 / 2,050) / `coracv` 1.87; advanced bot constructor `armack` 2.05 (430 /
210 / 33 / 930) / `corack` 2.14; construction aircraft `armca` 2.20 (110 / 50 / 136 / 156) / `corca` 2.09; decoy
commander `armdecom` 2.57 (770 / 300 / 38 / 3,700) / `cordecom` 2.50; advanced construction aircraft `armaca` 2.83
(340 / 120 / 192 / 200) / `coraca` 2.77. On water: Voyager `armmls` 0.95 (190 / 200 / 66 / 1,460, advanced
shipyard) and Pathfinder `cormls` 1.00 (200 / 200 / 58 / 1,600) are the cheapest mobile build power in the game;
construction seaplane `armcsa` 2.00 (150 / 75 / 124 / 405) / `corcsa` 1.87 (140 / 75 / 140 / 435), from the
seaplane platform; sea constructor `armcs`/`corcs` 1.60 (200 / 125); construction hovercraft `armch` 1.82 / `corch`
1.65; Beaver `armbeaver` 1.88 / Muskrat `cormuskrat` 2.00 (amphibious, from the vehicle plant). The commander's 300
is already paid for. Resurrection bots (`armrectr`, 200 power for 130) cannot assist (`canassist = false`). The
experienced players (2026-10-09, through the user): the minelayer ships assist and are the naval assist units; the
land minelayers Groundhog `armmlv` and Trapper `cormlv` (57 / 40 and 65 / 40) do not, though neither file carries a
`canassist` flag and the engine default would say both do. The tier-2 construction turret (`armnanotct2`, 840 for
600, reach 500) is the Extra Units Pack and not in normal play (K-rules-extra-units-pack-is-off-in-normal-play); the
tier-3 constructors reach no standard build list.
**Status.** arithmetic from the unit files (2026-10-09); the assist exceptions reported (the players); the names from
`language/en/units.json` (the file's own `name` fields are empty).
**Evidence.** `units/ArmBots/T2/armfark.lua`, `ArmVehicles/T2/armconsul.lua`, `CorBots/T2/corfast.lua`,
`ArmShips/T2/armmls.lua`, `ArmSeaplanes/armcsa.lua`, `ArmBuildings/LandUtil/armnanotc.lua`, `ArmBots/armrectr.lua`
and the rest named, read 2026-10-09 (`workertime`, `metalcost`, `speed`, `health`, `builddistance`, `buildoptions`);
`language/en/units.json`.
**Would be wrong if.** An engine test showed a Groundhog assisting a frame or a Voyager failing to; or a unit's
`workertime` were not its assist rate.
**Used by.** The brief's builders table (`docs/briefs/experience/units.md`); `docs/design/2026-10-09-late-economy.md`
§3 and §4.10.

### K-t2-build-power-for-a-project-follows-the-spend
**Claim.** A frame builds at the sum of the build power on it, and draws a second that power times its cost over its
build time; once the store is empty every builder on it slows to the share the income allows
(K-mech-stall-slows-everything). So the build power a project can use is `build time x spend / metal` for the metal a
second you can give it, and power beyond that only stalls. Armada's advanced fusion (9,700 metal, 69,000 energy,
build time 312,500): at 100 metal a second about 3,200 power and 97 s, drawing about 710 energy a second; at 50,
about 1,600 power and 194 s. The fusion (3,350 / 18,000 / 54,000): at 50 metal a second 810 power and 67 s. The
advanced vehicle plant (2,600 / 14,000 / 27,000): at 30 metal a second 310 power (an advanced constructor and a
turret) and 87 s. The silo (8,100 / 90,000 / 178,500): at 100 metal a second 2,200 power, 81 s and 1,110 energy a
second. The players' "stop the other factories during the transition" is this rule read from the other side: the
spend is what the whole economy can give the project.
**Status.** arithmetic from the unit files and the engine's stall rule (2026-10-09); not measured on a frame.
**Evidence.** `armafus.lua`, `armfus.lua`, `armavp.lua`, `armsilo.lua` (`metalcost`, `energycost`, `buildtime`);
K-mech-stall-slows-everything, K-mech-build-power-ratio.
**Would be wrong if.** A frame with power beyond the spend finished sooner than the metal flow allows (the engine
refuses what the store lacks), or the build rate were not `workertime` summed over the builders on the frame.
**Used by.** The brief's build-power paragraph; `docs/design/2026-10-09-late-economy.md` §3; the simulator's
frontier (`crates/buildorder/src/sim.rs`, its stall model).

### K-t2-the-transition-under-pressure-loses-at-either-gate
**Claim.** A tier-1 vehicle bot that holds BARb hard_aggressive to about one win in five on Comet Catcher
(`bots/distilled`, no tier 2: 3 of 16 over two rows) loses nearly every game when it starts the advanced plant at the
community's gates: at metal income 22 one win in sixteen (8 games with the tier-1 plants paused through the
transition, 8 without), at income 30 none in eight. The plant stands at 9:00 to 13:30 in every game and the chain
runs on schedule (constructors within a minute, a fusion by 14:00 to 18:30 in most games, tier-2 extractors to 30 by
the end), while the economy and the army go: extractors 16 / 19 / 14 / 8 at 10 / 15 / 20 / 25 minutes against the
no-tier-2 arm's 19 / 33 / 37 / 39, the army 12 / 8 / 17 / 21 soldiers against 11 / 32 / 68 / 63. The pause is not
the cost (0 of 8 without it); the plant's 2,600 metal and the builders' time out of the tier-1 army in the minutes
his pressure peaks are, and the income the chain buys (150 to 245 a second at 25 to 30 minutes in the games that
lasted) arrives after the army is gone. The same shape as the player games that teched (player-30, 31, 32). **On a few-spot map the same holds against
medium**: on Great Divide (22 spots, 11 a side) the bot saturates at 10 extractors and an income of 23 by 10:00, the
gate fires there, and with tier 2 it won 2 of 16 (1-6-1, 1-7-0) where without it, on tier 1 with converters at
saturation, it won 7 of 8 at 12 to 26 minutes.
**Status.** measured (2026-10-09): `distilled-t2b-never-hard`, `distilled-t2-never-hard`, `distilled-t2b-22-hard`,
`distilled-t2b-22-nopause-hard`, `distilled-t2b-30-hard`, `distilled-t2c-divide-converters-medium`,
`distilled-t2c-divide-noconverters-medium`, `distilled-t2c-divide-never-medium` in `docs/experiments.md` (8 games
each, seeds 1-8, sides alternating, `--play-out`).
**Would be wrong if.** A later gate (an income of 60 or more, or past minute 20, behind a standing army) or a bot
whose army holds his pressure won more with tier 2 than without. Against medium on Comet the arms with tier 2 won
6 of 8 (the control row) where v2 without it won 7 of 8, a cost of ten minutes a game and not of games; the Great
Divide rows say the small map turns that cost into losses.
**Used by.** `docs/design/2026-10-09-late-economy.md` §4.4 and §4.5; the brief's argument that an early lead is
spent before his tier 2 matters, which now has a bot-measured leg beside the player games.

### K-t2-the-late-spend-ceiling-in-a-bot
**Claim.** Once tier-2 extractors lift a bot's income past about 150 a second, three things it did without thinking
stop it spending: (1) the energy rule provisioned for the soldiers' full-speed draw at the tier-2 tank's 13.7 energy
a metal and built 40 to 56 solars with the energy store at a 16,000 cap; (2) those generators crowded the yard, the
site search behind a plant found nothing within 150 and the build-power ladder asked the same plant every half
second for twenty minutes instead of moving on, so 2 plants and 1 to 8 turrets stood at income 150 to 350; (3) the
project helpers were capped at six, so an advanced fusion ran at 830 power against a budget of ten thousand and took
seven minutes. The bank sat at the storage cap with income overflowing (19,173 of 19,450 at 30:00 in
distilled-t2-smoke-4-medium; 9,000 to 16,000 at 25 to 30 minutes in distilled-t2b-30-hard). The storage cap itself
grows with the tier-2 extractors (600 metal of storage each), so "the store over half" fires later the richer the
side.
**Status.** observed (2026-10-09) in `distilled-t2-smoke-4-medium` and `distilled-t2b-30-hard`; the fixes (no energy
for the anticipated draw while the store is over 90%, a refused site holds the ladder off that plant or off plants
for two minutes and widens the turret's search to 350, generators placed 450 behind home, helpers to twelve,
builders on build power scaled with the bank) are in the build the medium rows of 2026-10-09 ran on and are judged
there.
**Would be wrong if.** The medium rows on the fixed build still banked over half the store from minute 25 with
income over 150.
**Used by.** `bots/distilled/README.md` (D-ENERGY, D-BUILD-POWER, D-PROJECT); the brief's late-game section to come
(the flagship's player shows the same bank in human-18 and player-50: the mechanism there is the player's, but the
yard-crowding and the storage-cap arithmetic are the game's).

### K-nuke-the-first-flagship-game
**Claim.** Against BARb medium on Comet Catcher the flagship (Opus 5.5 medium as the player, Jev as the hands, the
experience brief, the game's objective set to the nuke) built a silo and landed a missile on his base, and the game
was still undecided at the 40-minute cap: the silo stands and fires as the engine check said, but a missile does not
end a game whose army does not finish it. The clock: advanced plant begun 8:38 (its first frame, three constructors
and about eight extractors lost to a fourteen-Ak raid at 8:23), stood 10:41; fusion 14:26; the silo begun 14:29 by
one advanced constructor with the commander and four constructors helping (`queue` with one id for all), stood
17:31 after 182 s against 174 by buildtime over the 1,024 power on it; the first missile ready at 20:00 (149 s: energy
2,291 in against 1,799 out), fired at 21:07 at his lab, and 40 s later everything of his within 640 of the point was
gone (his lab, advanced lab, an advanced constructor, two constructors, five turrets, 17 solars, four extractors,
three converters, two storages, a construction turret) except his commander. Eight missiles by 37:37, one every
120 to 125 s as the engine rate says; two landed on points where nothing stood (a lab the picture still listed four
minutes after the third missile had killed it out of our sight, and the first base's place again), one on our own
body of 81 soldiers (the player's note "MISTAKE"; it walked clear in 22 s and lost a Janus), two aimed at his
commander where it stood at the order (it lived both times: a commander walks out of a 640 blast in the 22 s from
order to impact, or stands past the 450 at which the falling damage drops under its health). The frontier's
standard-tax silo on Comet stood at 14:11 and its first missile at 16:25; this game ran 3 to 4 minutes behind that
ceiling after the raid. BARb medium started an advanced lab at 19:24 (killed by the first missile), built no anti-nuke
by 40:00, and at the end held a commander, two extractors, two turrets and five raiders scattered over the map
against our 239 soldiers (70,000 metal), whose body reached his base at 38:31 and had not found the commander at
40:00. The economy under the silo: extractors 23 at 16:00 fell to 13 at 20:00 and 10 at 24:00 (his raiders on the
outer spots while the army of nine held one post and the silo's 8,100 was paid), income 82 to 66; then 28
extractors and 218 a second by 36:00 with the bank at the storage cap from 32:00 (11,000 to 17,800) while an
advanced fusion took 332 s under three advanced constructors and a second silo 208 s: the bank of §1 again, with
the sink chosen too late.
**Status.** observed (2026-10-09), one game: `nuke-1-medium` (`run/matches/1791573192-nuke-1-medium`;
`run/nuke_read.py` reads it).
**Evidence.** The record's `created`/`finished`, `round` and `fire_at` rows, the truth file for what stood within 640 of
each point and was gone 40 s later, the player's `fire` calls and notes in `strategist-0.jsonl`.
**Would be wrong if.** A second game with the same objective had the missile decide it, or a commander standing at
the point at impact survived.
**Used by.** The brief's late-game part (the nuke-1-medium case); `docs/design/2026-10-09-late-economy.md` §10 (e):
what the next game changes (the army finishes while the missiles fall; a missile at a remembered building is a
missile at nothing; his commander is not a missile's target while it can walk).

### K-nuke-the-second-flagship-game
**Claim.** Against BARb hard_aggressive on Comet Catcher with the same seat and objective as K-nuke-the-first-flagship-game
the economy won the nuke race and the player lost it: the silo stood at 16:32, inside the frontier's standard-tax window,
and held two missiles from 20:38, a minute before his anti-nuke was begun, and the player fired nothing until 23:39,
nine seconds after that anti-nuke's first interceptor could be ready; every missile sent at his base afterwards was
taken and every one sent at his army in the field landed; the game was lost at 32:22 to his tier-2 army and two
Sokolovs. The clock: advanced solar 6:27, advanced plant begun 8:25 and stood 10:01, fusion 14:29, the silo begun
13:19 by one constructor and joined by more (986 mean power, the store empty in 26 % of its samples), stood 16:32 after
193 s against 181 by the power; the first round 18:38 (126 s), the second 20:38, the third 22:38. His advanced vehicle
plant stood 13:52; his anti-nuke was begun 21:40 and stood 22:08 (28 s), so its first interceptor was ready by 23:38;
a silo of his was begun 29:50 and an advanced aircraft plant 32:02, neither standing at the end. The player's one `fire`
call before 23:39 was at 18:18, twenty seconds before the first round, and was refused; its turn reports from 18:28 to
23:39 carried no line about the silo's rounds (the picture's `stockpilers` entry is in the picture tool, which it did not
call), its notes read "silo still 0 ready (energy-limited)" at 20:53 and "silo had 3 missiles ready (unseen in
reports)" at 23:39, and its plan from 20:08 was to push the army north to find his base "so the missile has a target",
the base whose vehicle plant it had named at 18:18 by its coordinates; no building of his base (plant, anti-nuke,
gantry) was ever seen. Seven missiles: three at his base, all intercepted (23:47 at his vehicle plant, 304 from the
anti-nuke, the point holding the plant, a fusion, a gantry, a Shiva, three mohos, 16 advanced solars, 15 construction
turrets and his commander; 23:47 at his advanced plant, 558 away; 25:46 at his mohos, 682 away; nothing within 640 of
any was gone 40 s later), and four at his forward forces, all landed (24:43 at D3-E3: a Banisher, two Tzars, two
Erasers, two turrets, two extractors, a construction aircraft; 30:00 two at once from the two silos at points 380
apart on our own line at B5-C5: four to eight forward constructors, a Tzar, a Fink, a turret, two extractors, and of
ours two Stumpies out of three Bulls and two Stumpies within 640; 31:33 at mid-map: three extractors, five turrets, a
Muskrat, two radars). The player pulled the body 1,300 west before the 24:43 shot and told it of the 30:00 ones. The
second silo, begun 23:59 under 1,877 power with the store never empty, stood in 96 s at 25:34; the advanced fusion
20:54 to 25:58 (304 s); a Mercury at 30:00. The economy: extractors 9, 17, 27, 23, 23, 28, 12, 8 at the four-minute
marks from 4:00 to 32:00; income 18, 36, 58, 106, 210, 186, 97, 147; the bank 6,600 at 20:00 and 12,180 at 24:00, the
cap; the army 7 to 58 Bulls and Stumpies. His: extractors 5, 11, 17, 31, 36, 42, 57, 56; movers on our half 5 at 22:00,
12 at 26:00 (Reapers, a Karganeth, a Shiva, a Punisher), 20 at 32:00 (Vipers, Toasters, Sokolovs, jammers): twenty of
our twenty-eight extractors fell between 24:00 and 32:00. The loss: the commander, listed at 31:18 to rebuild five
extractors at C8-D8 under the player's own rule (never north of spot_69; home while a heavy is within 900), walked
east, was shot from 32:04 by two Sokolovs it could not see (corsok, assault hovercraft, 950 metal, first seen 30:52),
100 to 19 % in five seconds; the player ordered it home at 32:05 ("being shot by units it cannot see"), the hands
turned it at 32:10, it died at 32:13. Jev $0.92 (22.0M input tokens: gate 1,156 calls, pick 741, decode 513); the
player $32.84 over five sessions of 170 turns; 36.5 minutes of wall clock.
**Status.** observed (2026-10-09), one game: `nuke-2-hard` (`run/matches/1791577775-nuke-2-hard`; `run/nuke_read.py`
reads it, his anti-nuke and the shots' distances from it included).
**Evidence.** The record's `created`/`finished`, `round` and `fire_at` rows; the truth file's sixth field for his
anti-nuke's begun and stood frames, and what stood within 640 of each point and was gone 40 s later; the turn prompts,
`fire` calls and notes in `strategist-0.jsonl` (no prompt from 18:28 to 23:39 contains "missile", "silo", "ready" or
"stockpil"); the own-unit samples for the commander's last minute.
**Would be wrong if.** A game whose reports name the rounds as they come ready had the player hold them as long; or
hard_aggressive's anti-nuke in a repeat stood later than 21 to 25 or took longer than a minute to stand and arm.
**Used by.** The brief's late-game part (the nuke-2-hard case and the nuke paragraph: a missile needs no sight, the
base is the target only before his anti-nuke); `docs/design/2026-10-09-late-economy.md` §10 (e), the second game:
the report says nothing about rounds (proposal), the fire answer says nothing about interception (proposal), the
batch count fixed.
