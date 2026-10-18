# Scouting and information

Seeded 2026-09-19. Numbers from local unit files (`upstream/Beyond-All-Reason/units/…`) are `supported`; advice is `reported`.
Scope: 1v1 land. Our bot currently has no scouting and targets "the mirrored home position" (H-ARMY-TARGET).

### K-scout-radar-tower-is-cheap
**Claim.** A radar tower costs 60 M / 630 E, builds in 3.8 s by the commander, and covers radius 2100 (sight 680); the
commander itself has radar 700. On a 7168-wide map, one tower at the base and one at each forward outpost covers most of
our half. Official opening advice lists radar as step 4 of 4 (metal, energy, factory, intel). Terrain blocks radar, so
towers go on high ground or edges.
**Status.** supported (2026-09-19) for numbers (local source); placement advice reported
**Evidence.** `ArmBuildings/LandUtil/armrad.lua`, `CorBuildings/LandUtil/corrad.lua` (radardistance 2100, metalcost 60),
`units/armcom.lua` (radardistance 700). https://www.beyondallreason.info/guide/how-to-start-manage-your-economy;
https://www.beyondallreason.info/guide/radar (official, undated).
**Would be wrong if.** The AI interface did not report radar contacts to the bot at all (then radar only helps unit
auto-targeting, not decisions).
**Used by.** (candidate: H-INTEL-RADAR — radar right after the first lab, one more with each outpost turret)

### K-scout-radar-blips-are-inaccurate
**Claim.** Units firing at a radar-only contact aim at a wobbling position and miss most shots; line of sight on the target
is required for accurate fire. Identified buildings stay as ghosts once seen. Targeting facilities (T2) halve the wobble
each (50% / 75% / 87.5%).
**Status.** reported (2026-09-19)
**Evidence.** https://www.beyondallreason.info/guide/radar;
https://www.beyondallreason.info/guide/important-knowledge-on-advanced-mechanics (official, undated).
**Would be wrong if.** Rocket bots hit a tower at max range equally often with and without a spotter in sight range.
**Used by.** (candidate: H-ARMY-SIEGE — long-range groups need a cheap spotter within its sight radius of the target:
rocket bots see 380 but shoot 475)

### K-scout-sight-shorter-than-range
**Claim.** Several units shoot farther than they see, so alone they fire at blips: Rocketeer/Aggravator sight 380 vs
range 475; Shellshocker 364 vs 710; Wolverine 330 vs 710; Sheldon 380 vs 850; Arbiter 380 vs 1210. Cheap eyes: Tick sight
600 (21 M), Grunt 520 (43 M), Pawn 429, Rover `armfav` 635 (31 M), Rascal `corfav` 600 (26 M), LLT 494, radar tower 680,
Beholder camera `armeyes` 560 (32 M, cloaked).
**Status.** supported (2026-09-19) — local source
**Evidence.** `sightdistance` / `range` in `ArmBots/armrock.lua`, `CorBots/corstorm.lua`, `ArmVehicles/armart.lua`,
`CorVehicles/corwolv.lua`, `CorBots/T2/cormort.lua`, `corhrk.lua`, `ArmBots/armflea.lua`, `CorBots/corak.lua`,
`ArmVehicles/armfav.lua`, `CorVehicles/corfav.lua`, `ArmBuildings/LandUtil/armeyes.lua`.
**Would be wrong if.** UnitDefs through the AI interface show different sight values (alldefs_post may scale them).
**Used by.** (candidate: every attack group includes ≥1 raider ordered to stay ~100 ahead of the skirmishers)

### K-scout-what-to-look-for
**Claim.** One early scout pays for itself by revealing the opponent's factory type and first units (raiders vs line
units vs air), which decides anti-air, tower placement and army composition. Things a bot should extract from a scouting
pass: factory types and count; advanced lab under construction; defence positions and types on the approach;
extractor count; commander position.
**Status.** reported (2026-09-19)
**Evidence.** https://www.crdhq.com/articles/bar-beginner-guide-getting-started (2026);
https://masterbel2.wordpress.com/beyond-all-reason-units-guide-uses-tactics-and-counters/ (scouts give composition, radar
gives area information).
**Would be wrong if.** Against BARb (whose build is near-deterministic per profile) a scouting brain made no different
decisions than a blind one — then hard-code what opponents-barb.md says instead.
**Used by.** (candidate: H-INTEL-SCOUT — first factory unit for Armada is a Tick sent to the mirrored start; Cortex sends
the first Grunt; repeat every ~3 minutes)

### K-scout-air-scout
**Claim.** The T1 air scouts (Blink `armpeep` 52 M, sight 865, radar 1140, speed 375; Finch `corfink` 51 M, sight 835,
radar 1120, speed 360) cross Quicksilver in ~20 s and see far more than ground scouts, but need an aircraft plant
(Armada 650 M / Cortex 630 M).
**Status.** supported (2026-09-19) for numbers — local source; not evaluated as a plan
**Evidence.** `ArmAircraft/armpeep.lua`, `CorAircraft/corfink.lua`, `ArmBuildings/LandFactories/armap.lua`, `corap.lua`.
**Would be wrong if.** (numbers only)
**Used by.** (none)

### K-scout-target-the-economy
**Claim.** When the enemy army is stronger, the advice is to avoid it and hit several economic targets faster than they can
be defended; that needs knowledge of where the enemy's extractors are. On a symmetric 1v1 map the enemy's likely spots are
the mirror images of ours, and any extractor once seen persists as a ghost.
**Status.** reported (2026-09-19)
**Evidence.** https://masterbel2.wordpress.com/beyond-all-reason-units-guide-uses-tactics-and-counters/ ("raid multiple
enemy bases faster than they can defend them"); ghosts: https://www.beyondallreason.info/guide/radar.
**Would be wrong if.** Raider groups sent to mirrored spots on Quicksilver found them empty or towered most of the time.
**Used by.** H-ARMY-SWEEP (candidate: run the sweep with raiders from minute ~4, not only when the main target is empty)

### K-scout-we-never-see-their-extractors
**Claim.** Our bot has essentially no sight of the opponent's economy. Averaged over 430 games on Quicksilver we have
identified and not watched die 0.0 of their extractors in minutes 1-8, 0.5 in minutes 11-15 and 1.3 of their 14.6 in
minutes 20-40; in 39 % of late-game rows not one enemy extractor has ever been seen. Static defence is barely better:
467 metal of turrets in hand against 3805 standing. Their army is the one thing we do see — 3435 metal of the 4853
they have by minute 20 — because it comes to us.
**Status.** measured (2026-09-20) over every match that has a ground-truth file.
**Evidence.** `docs/studies/tempo-model.md` table "What the bot actually has in hand", from `run/tempo_model.py`
(`f_live_mex_n`, `f_live_turret_m`, `f_live_army_m` against `truth-*.jsonl`).
**Would be wrong if.** With a scouting rule that visits their half, the number of enemy extractors ever identified per
game rose materially. That is the test of any scouting change, and it now has a baseline to beat.
**Used by.** (none yet) — it is the reason an estimate of their economy has to be inferred rather than observed
(K-barb-tempo-is-half-clock-half-sighting), and the strongest argument yet for H-ARMY-SCOUT / radar coverage.

### K-scout-enemy-base-is-a-spot-cluster
**Claim.** An opponent's extractors in the opening are all in the vicinity of its start, which is a metal-spot cluster
(the reasonable start spot; D7 on Quicksilver from the south); the first contact with anything of theirs is enough to
say which cluster. The other spots in its box (C7/B7) and the middle (F6) are low priority to look at, but get looked
at eventually. Scouting is one Pawn or Tick at a time, occasionally, tracking when each spot was last seen.
**Status.** stated by the user 2026-09-20 from watching rush-13-harass3-noplace 00. Not measured.
**Evidence.** In that game the first contact was at 1:52; BARb's start (3371, 5556) is D7 and its extractors by minute
3 lie within 700 of it (its records in `run/matches/*-rush-1[0-4]-*`).
**Used by.** H-SCOUT-SPOTS, H-SCOUT-ROUTE, H-MAP-ENEMY-CLUSTER; H-ARMY-PRESSURE's unknown targets.

### K-map-forward-extractor-pulls-the-guess
**Claim.** Refining the base guess from the first building of theirs seen within 2,500 takes a forward extractor in
the contested middle for the base on medium, where BARb expands to the middle spots by minute 6; the guess then
sits on an empty mid-map cluster, and nothing moved it back when our army stood there and saw nothing.
**Status.** demonstrated (2026-09-21)
**Evidence.** pianist-player-5 bot.log at f=11745 (6:31): "enemy base guessed at (4168, 4264) from a building at
(4112, 4240): a cluster of 2 spots (was (3656, 5816))"; the truth file has the enemy commander within 350 of
(3656, 5816) all game; the ball held at the E5 guess from 8:52 with `enemy.base` reading "not found; presumed at
E5" to the end.
**Would be wrong if.** The building at (4112, 4240) had been a factory (it was an extractor at spot_21).
**Used by.** H-MAP-ENEMY-CLUSTER, H-MAP-ENEMY-GUESS-EMPTY.

### K-scout-terrain-cuts-sight-and-radar
**Claim.** The engine computes sight and radar coverage by raycasting over the terrain (`LosHandler.cpp:92`: `LOS_TYPE_LOS`
and `LOS_TYPE_RADAR` use `LOS_ALGO_RAYCAST`; jammers, sonar, air sight and seismic are plain circles). The emitter stands
at the unit's middle plus its `sightEmitHeight` (default 20) or `radarEmitHeight` (default the sight height). A hill
between a radar and a base hides the base whatever the radar's range. On Altored Divide the ridge at z ~2,330 (450 to
500 elmos) hides the whole north-east from an advanced radar at spot_41 (G4, ground 135): 19 of 1,984 64-elmo squares of
E1 to H2 within 3,500 are in its line at any emitter height up to 300 (human-8v8-1, `scratchpad ray.py` over
`terrain-4.bin`). The AI interface exposes the resulting grids (`Map_getLosMap`, `Map_getRadarMap`, `Map_getAirLosMap`;
BAR mip levels 3, 2, 4: 64, 32 and 128 elmo squares), the ally team's, from 2026-10-18 in `Snapshot::sensors`.
**Status.** supported (2026-10-18): engine source, the heightmap raycast, the interface header.
**Evidence.** `upstream/RecoilEngine/rts/Sim/Misc/LosHandler.cpp` 92, 159-162; `rts/Sim/Units/UnitDef.cpp` 477-478;
`SSkirmishAICallback.h` 1576-1612; `docs/design/2026-10-18-sight-jammers-cloak-shields.md`.
**Would be wrong if.** A map's heightmap were flat (then the circle and the grid agree).
**Used by.** H-SCOUT-LOS-GRID, H-REPORT-RADAR-WORDS, H-ENEMY-GONE-NEEDS-SIGHT; the brief's sight passage.

### K-scout-jammers-hide-from-radar-only
**Claim.** A jammer tower hides every unit of its own side within its radius from enemy radar and nothing from sight
(`LosHandler.cpp` `InRadar`: in the radar grid and not in a jammer of the unit's side; BAR `separateJammers = true`, so
each side's jammers hide only its own units and the engine's jammer map handed to an AI is its own side's). Radii:
Sneaky Pete `armjamt`, Castro `corjamt`, Nyx `legjam` 390; Veil `armveil`, Shroud `corshroud`, Erebus `legajam` 760
(125 to 130 metal, 125 energy a second); Rampart `legrampart` 500 besides its anti-nuke and 2,100 radar. None of them
cloaks anything: this BAR has no area cloaker (`cloakcost` is on commanders, spies, mines, Beholders, the Pitbull and
Ambusher pop-ups, snipers, the Gremlin, the Phantom and the Shockwave only; `legacluster` and `legapopupdef` do not cloak).
**Status.** supported (2026-10-18): engine source and unit files.
**Evidence.** `LosHandler.cpp` 942-995; `gamedata/modrules.lua` 53; `units/*/armveil.lua`, `corshroud.lua`,
`Legion/Utilities/legajam.lua` (radardistancejam 760), `legjam.lua` (390), `Legion/Economy/legrampart.lua` (500);
`grep -l cloakcost units`.
**Would be wrong if.** A game version added area cloak (BAR's `unit_cloak.lua` gadget has only a decloak-when-damaged option, off).
**Used by.** H-REPORT-JAMMERS; the brief's sight passage.

### K-scout-juno-kills-jammers-and-radar
**Claim.** The Juno (`armjuno`, `corjuno`, `legjuno`: 660 metal, 16,000 energy, made by tier-1 constructors) stockpiles a
pulse (75 s a round, map-wide range 32,000, `commandfire`) that destroys every unit marked `juno_kill` within 1,400 of
where it lands: radar towers and radar bots, every jammer tower and jammer bot (Veil, Shroud, Erebus, Sneaky Pete,
Castro, Nyx), Beholder cameras, mines, scout cars; not the Rampart, not buildings or soldiers. Our own radars in the
area die too. It is the counter to a jammed base: one pulse, then radar shows what the jammers hid; our `fire` tool
already fires any stockpiler by handle.
**Status.** supported (2026-10-18): unit files and the gadget.
**Evidence.** `units/CorBuildings/LandDefenceOffence/corjuno.lua` (stockpile, range 32000, `juno_pulse` areaofeffect
1400); `luarules/gadgets/unit_juno_damage.lua` 20-35 (`juno_kill`, radius 450 denial ring); `grep -l "juno_kill = true" units`.
**Would be wrong if.** The gadget's kill list changed (it reads `customparams.juno_kill` at load).
**Used by.** the brief's sight passage; the player's `queue` and `fire` tools (nothing in code).

### K-scout-cameras-are-the-cheapest-eye
**Claim.** The Beholder `coreyes`, Argus `legeyes` and Armada's `armeyes` are 32-metal cloaked camera towers with 540
sight (560 in older notes), built by any tier-1 constructor or the commander anywhere a building stands, invisible to
radar and to sight until something of his is within their decloak distance; one beside a base gives standing sight
over 540 of it that no jammer removes (a Juno pulse kills them). Radar bots (Augur `corvoyr`, `legaradk`: 925 sight,
2,200 radar) and scout planes see further but are jammed or shot.
**Status.** supported for numbers (units.json, unit files); the use is reported (the user, 2026-10-18).
**Evidence.** `crates/bot/data/units.json` entries; `units/*/coreyes.lua`, `legeyes.lua`.
**Would be wrong if.** The camera's cloak cost or sight changed.
**Used by.** the brief's sight passage.
