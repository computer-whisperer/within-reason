<!--meta
id: F-eco-intro
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Units/Unit.cpp, upstream/RecoilEngine/rts/Sim/Misc/Team.cpp, upstream/Beyond-All-Reason/gamedata/modrules.lua], game: 54199a0d, engine: 2026.07.04, dates: [2026-10-22]}
status: live
-->
## The economy: how metal, energy and build power work

What follows is read from the engine and the game's rules, not from games. Every number is per second unless said;
the engine applies income and upkeep twice a second and build steps every frame, thirty a second. Unit costs are
the unit tables'.

<!--meta
id: F-eco-store
layer: fact
sources: {code: [upstream/Beyond-All-Reason/luarules/gadgets/game_team_resources.lua:38-99, upstream/Beyond-All-Reason/modoptions.lua (startmetal, startenergy, startmetalstorage, startenergystorage), upstream/RecoilEngine/rts/Sim/Misc/Team.cpp:274-355 (SlowUpdate), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:437 (SetStorage)], game: 54199a0d}
status: live
-->
### The store

A side starts with 1,000 metal and 1,000 energy and a storage of 1,000 each (the lobby's defaults; the commander's
own 500 and 500 are inside that). Every finished building and unit adds its storage to the side's: an extractor 50
metal, an advanced extractor 600, a lab or plant 100 of each, a solar 50 energy, a fusion 2,500 energy, an advanced
fusion 9,000, a metal storage 3,000 metal for 330, an energy storage 6,000 energy for 170. What a unit stores leaves
with it.

Once a second the engine clamps each resource to its storage and the excess is gone. Before clamping it gives away
what stands above 99 percent of the metal storage and 95 percent of the energy storage to allies who have room, in
proportion to their room, filling each to 99 percent at most. So a full store is not wealth: a team-mate with room
gets the overflow, and when nobody has room it is lost. A deliberate transfer (the share order) moves resources or
units to an ally at once; the receiver cannot be given more than fills it to its share level (99 percent of its
metal storage, 95 of its energy).

<!--meta
id: F-eco-extractors
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Units/UnitTypes/ExtractorBuilding.cpp:117-148, upstream/RecoilEngine/rts/Map/MetalMap.cpp:76-125, upstream/RecoilEngine/rts/Map/ReadMap.cpp:167, upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1080-1091 (SlowUpdate), upstream/Beyond-All-Reason/units/ArmBuildings/LandEconomy/armmex.lua, armmoho.lua, upstream/Beyond-All-Reason/luarules/gadgets/unit_mex_upgrade_reclaimer.lua], game: 54199a0d, claims: [K-mech-an-extractor-draws-only-the-squares-within-its-radius, K-mech-tier-2-extractor-stands-on-ours, K-mech-upkeep-has-no-priority]}
status: live
-->
### Metal: extractors

The map is a grid of 16-elmo squares, each worth its byte of the map's metal map, 0 to 255, times the map's metal
scale (the map sheet's figure; Comet Catcher's is 0.75, so a full square is worth 191). An extractor draws every square whose centre lies within the map's extractor radius of it,
at its depth: 0.001 of the square's value a second for a tier-1 extractor, 0.004 for an advanced one, so an advanced
extractor on the same squares yields four times the tier-1 one. A Comet Catcher spot is twelve full squares: 2.3
metal a second for the extractor, 9.2 for the advanced one. The spot's value comes from the squares, not the
building, and an extractor placed off the patch's centre loses the squares that fall outside its radius.

A square is drawn by one extractor at a time, to the deepest depth asked of it. An advanced extractor built on top
of a tier-1 one draws the difference while both stand; when it finishes, the game removes the old extractor and
refunds its 50 metal, and the new one then draws the whole. If the old extractor belonged to an ally, the new one is
the ally's from the moment its frame is placed, and the refund is theirs too: an advanced constructor lent across a
team builds the ally's income, not its owner's.

Extractors cost energy to run: 3 a second for the tier-1 extractor, 20 for the advanced one. The engine takes each
unit's upkeep every half second, all of it or none: an extractor whose upkeep cannot be paid that half second yields
no metal that half second. With the energy store at zero the extractors' income stops; only the commander's own 2
metal a second goes on, and whatever energy trickles in between the half-second checks pays the extractors it
reaches, oldest first.

<!--meta
id: F-eco-energy
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1076-1098 (SlowUpdate), upstream/RecoilEngine/rts/Sim/Misc/Wind.cpp:93-143, upstream/RecoilEngine/rts/Sim/Misc/Wind.h:43, upstream/Beyond-All-Reason/units/ArmBuildings/LandEconomy/armsolar.lua, armadvsol.lua, armwin.lua, upstream/Beyond-All-Reason/units/ArmBuildings/SeaEconomy/armtide.lua, upstream/Beyond-All-Reason/scripts/Units/armsolar.bos (Close, Open, HitByWeapon), upstream/Beyond-All-Reason/units/armcom.lua, upstream/Beyond-All-Reason/units/CorBuildings/LandEconomy/corfus.lua], game: 54199a0d, note: the wind average is a simulation of Wind.cpp's walk (0.76 of the top for 0-25, 0.83 for 0-30), not the engine}
status: live
-->
### Energy: generators

A solar collector makes 20 a second while open. It closes when hit and reopens about ten seconds after the last
hit (eight of waiting, then the dishes' turn), making nothing meanwhile and taking half damage while closed; a solar
field under fire is dark. An
advanced solar makes 80, costs 350 metal and 5,000 energy to build, and does not close. A wind turbine makes the
current wind strength, capped at 25: the wind wanders between the map's minimum and maximum, picking a new target
every fifteen seconds and blending toward it, and the walk favours the top of the range, so a turbine averages about
three quarters of the range's top (19 on a map whose wind runs 0 to 25, 3 on Comet Catcher's 1 to 4) and is almost
never still. A tidal generator makes the map's tidal strength, steady. A geothermal plant makes 300 on a vent, a
fusion 750 (Cortex's 850), an advanced fusion 3,000.

The commander makes 30 energy and 2 metal a second by itself. A construction vehicle makes 10 energy, a construction
bot 7, an advanced constructor 14 to 20. These run whether the unit works or not. Numbers here are Armada's; a Cortex
twin differs by a few percent where the unit tables say so.

<!--meta
id: F-eco-build
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1991-2030 (AddBuildPower), upstream/RecoilEngine/rts/Sim/Units/UnitTypes/Builder.cpp:111-113, 349-354, upstream/RecoilEngine/rts/Sim/Units/UnitTypes/Factory.cpp:86, 251, upstream/RecoilEngine/rts/Sim/Units/CommandAI/BuilderCAI.cpp:867-930 (ExecuteGuard), upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:345-369 (maxRepairSpeed defaults to 1e20 and no BAR unit sets it), upstream/RecoilEngine/rts/Sim/Misc/Team.cpp:148-157, upstream/Beyond-All-Reason/units/ArmBots/armrectr.lua (canassist false)], game: 54199a0d, claims: [K-mech-stall-order-is-creation-order, K-mech-build-power-ratio, K-mech-turrets-assist-in-radius, K-energy-plant-draw-outruns-solars]}
status: live
-->
### Build power, build time and the draw

Every builder has a build power (the unit tables' column; the commander 300, a plant 150, an advanced plant 600, a
construction vehicle 90, a construction bot 80, an advanced constructor 210 to 310, a construction turret 200, a
resurrection bot 200; Cortex's constructors 85 and 95) and every unit a build time. The seconds a build takes are its build time divided by the build
power on it, and the power on it is the sum of every builder working on it: a plant with the commander assisting
builds at 450. Each builder, every frame, advances the frame by its power over the build time and pays that fraction
of the frame's metal and energy. So a build draws metal at its metal cost times the power on it over its build time,
and energy likewise. The table gives the common cases; the rule gives any other.

| build | by | seconds | metal a second | energy a second |
|---|---|---|---|---|
| extractor (50 m, 500 e, 1,800) | commander 300 | 6 | 8.3 | 83 |
| extractor | construction vehicle 90 | 20 | 2.5 | 25 |
| solar (155 m, 0 e, 2,600) | commander | 8.7 | 17.9 | 0 |
| solar | construction vehicle | 29 | 5.4 | 0 |
| bot lab (500 m, 950 e, 5,000) | commander | 16.7 | 30 | 57 |
| vehicle plant (590 m, 1,550 e, 5,700) | commander | 19 | 31 | 82 |
| construction turret (230 m, 3,200 e, 5,300) | construction vehicle 90 (not in the commander's build list) | 58.9 | 3.9 | 54 |
| construction vehicle (135 m, 1,950 e, 4,050) | plant alone 150 | 27 | 5.0 | 72 |
| construction vehicle | plant + commander 450 | 9 | 15 | 217 |
| Blitz (110 m, 900 e, 2,000) | plant alone | 13.3 | 8.3 | 68 |
| Blitz | plant + commander | 4.4 | 25 | 203 |
| Stout (225 m, 2,100 e, 3,400) | plant alone | 22.7 | 9.9 | 93 |
| Stout | plant + commander | 7.6 | 30 | 278 |
| advanced extractor (620 m, 7,700 e, 14,900) | advanced constructor 210 | 71 | 8.7 | 109 |
| advanced extractor | that plus two construction turrets 610 | 24 | 25 | 315 |
| advanced vehicle plant (2,600 m, 14,000 e, 27,000) | one construction vehicle | 300 | 8.7 | 47 |
| advanced vehicle plant | commander + four construction vehicles 660 | 41 | 64 | 342 |
| fusion (3,350 m, 18,000 e, 54,000) | advanced constructor 210 | 257 | 13 | 70 |
| fusion | 1,000 power | 54 | 62 | 333 |

Energy per metal differs by what is built: a lab 1.9, a plant 2.6, a Blitz 8.2, a Stout 9.3, an extractor 10, an
advanced extractor 12.4, a constructor 14.4, a construction turret 13.9, an advanced plant or a fusion 5.4, a solar 0.
What a plant draws therefore depends on what it is making and on who assists it, not on the plant.

A builder's step is all or nothing. Each frame it takes its whole step if the side holds the whole step's metal and
energy, and otherwise takes nothing and records the demand; the engine tries builders in the order they were
created, so in a stall the oldest builder (the commander, then the first plant, then the constructors in order of
birth) is served first, and a cheap step slips through where a dear one does not. Nothing is slowed in proportion;
a build either moves at full speed that frame or stands. The game adds a low-priority flag per builder (a command
people's interface sets on labs and turrets): a low builder gets its step only when the stock minus a margin minus
the high builders' demand stays positive, so low builders yield in a stall and change nothing outside one.

Assisting: a builder told to guard another builder or a plant repairs whatever that one is building, adding its
power while within its build reach (a construction turret's reach is 400, a construction vehicle's 130, the
commander's 145). A plant's own 150 is always on its unit. A resurrection bot cannot assist a build; it reclaims
and resurrects. Repair is free in both resources and runs at the repairer's power, any number of repairers adding
up.

<!--meta
id: F-eco-decay
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1034-1050 (SlowUpdate), upstream/Beyond-All-Reason/gamedata/modrules.lua:15-19], game: 54199a0d}
status: live
-->
### Frames left alone decay

A frame nobody is building on for nine seconds starts to decay: every half second it loses one part in 0.03 times its
build time of its progress, and the metal in the lost part comes back to the store. A lab frame (build time 5,000)
loses a 150th a half second and is gone in 75 seconds; a Blitz frame in 30; an advanced extractor's in about four
minutes; an advanced plant's in about seven. A builder standing on a frame, even with nothing to pay it with, stops
the decay.

<!--meta
id: F-eco-reclaim
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Features/Feature.cpp (AddBuildPower), upstream/RecoilEngine/rts/Sim/Features/FeatureDefHandler.cpp:118, upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:2054-2120 (AddBuildPower, reclaim), upstream/RecoilEngine/rts/Sim/Units/UnitTypes/Builder.cpp:395-440 (resurrect), upstream/RecoilEngine/rts/Sim/Misc/ModInfo.cpp:74-86, upstream/Beyond-All-Reason/gamedata/modrules.lua:21-44, upstream/Beyond-All-Reason/gamedata/alldefs_post.lua:553-575, upstream/Beyond-All-Reason/units/armcom.lua (featuredefs: dead has no resurrectable flag, heap has resurrectable 0), upstream/RecoilEngine/rts/Sim/Features/FeatureDefHandler.cpp:107 (default -1), upstream/Beyond-All-Reason/luarules/gadgets/unit_reclaim_fix.lua:40-61 (the fixed reclaim bonus), upstream/RecoilEngine/rts/Sim/Units/Scripts/CobInstance.cpp:243 and upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:529, 682 (severity), upstream/Beyond-All-Reason/scripts/Units/armflash.bos (Killed), upstream/Beyond-All-Reason/luarules/gadgets/unit_resurrected.lua:81 (5 percent), upstream/Beyond-All-Reason/luarules/gadgets/game_team_com_ends.lua:75-92 and modoptions.lua deathmode (default com)], game: 54199a0d, claims: [K-mech-reclaim-values, K-mech-lazarus-raises-commanders (corrected: only while an allied commander lives)]}
status: live
-->
### Reclaim, resurrection, repair

What a dying unit leaves depends on how hard the last blows hit. The engine scores the death by the damage taken in
about the last second against the unit's full health (older damage fades at ten percent a frame): up to a quarter of
its health leaves a wreck worth 60 percent of its metal cost (a Blitz 66, a Stout 135, a Mace 78) and no energy; up
to half leaves only the heap, worth 25 percent; more leaves nothing, and a self-destruct counts as twice the unit's
health, so it leaves nothing either. A unit worn down leaves a wreck; one deleted by a burst (the D-gun, a bomb, a
volley far beyond its health) leaves heap or dust. A wreck destroyed again leaves its heap. The commander is the
exception to the ratios: its wreck holds 1,250, its heap 500.

Reclaiming a wreck costs no energy and pays out as it goes. The engine's rate is the reclaimer's power over six
times the wreck's metal, and the game adds a fixed bonus to every reclaimer, so the time is six times the wreck's
metal over (1.05 times the power plus 135): a construction vehicle eats a Blitz wreck in 1.7 seconds and collects
about 38 metal of wreck a second, a resurrection bot 58, the commander 75; several can work one wreck at once, and the
fixed bonus makes many small reclaimers worth far more than their build power says. Trees and rocks are features
with the map's own values and reclaim the same way.

Reclaiming a live unit or building of your own (or an ally's, or an enemy's you can reach) returns its full metal
cost times its build progress, all of it at the end, with no energy cost and no wreck; the time is its health share
times its build time over the reclaimer's power, so the commander eats a whole bot lab in 17 seconds for 500 metal and
a construction vehicle a whole Stout in 38 for 225. Any unfinished frame can be taken apart, an enemy's too;
commanders cannot. An enemy unit keeps shooting while it is taken apart, from inside the reclaimer's build distance
(130-145; every raider's gun reaches farther), so health decides the trade: a construction bot takes a full Pawn apart
in 21 seconds while the Pawn kills it in 8, but one at a quarter health in 5 seconds, for its 54 metal; a damaged
raider that wanders among constructors, an enemy's fresh frame and a lone hurt building are what an unarmed builder can
kill. The reclaim drains health at the same time as weapons do, so the commander's laser and its reclaim add, and a
unit the reclaim finishes pays its whole cost instead of a 60 percent wreck. Stopped before the end, a reclaim pays
nothing, but the health it took stays lost.

Resurrecting a whole wreck (one not partly reclaimed) costs 50 percent of the unit's energy and no metal and takes
its build time over the resurrector's power: a resurrection bot (200) raises a Blitz in 10 seconds for 450 energy, a
Stout in 17 for 1,050, a commander in 375 seconds for 13,000. The raised unit starts at 5 percent health. Wrecks can
be resurrected; heaps cannot. A resurrected commander is a living commander and counts as one for the game's end,
but the game's default end rule (the mode our lobbies use) kills every team of a side the moment the side's last
commander dies, so a raised commander only helps a side that still has another commander alive somewhere; a seat
whose own commander died plays on while an ally's lives.

<!--meta
id: F-eco-conversion
layer: fact
covers: [docs/briefs/experience/late-game.md:31-35/1]
sources: {code: [upstream/Beyond-All-Reason/luarules/gadgets/game_energy_conversion.lua:20-29, 263-330, upstream/Beyond-All-Reason/units/ArmBuildings/LandEconomy/armmakr.lua, armmmkr.lua], game: 54199a0d, claims: [K-mech-converter-threshold, K-mech-converter-rates]}
status: live
-->
### Energy conversion

A converter turns energy into metal only out of the energy that stands above 75 percent of the side's energy
storage (people move the level with a slider; ours stays at the default), every half second, up to its capacity.
The tier-1 converter takes 70 energy a second for 1 metal and costs 1 metal and 1,150 energy; the advanced one takes
600 a second for 10.3 metal (58 energy per metal) and costs 380 metal and 21,000 energy. A converter therefore never
empties the store by itself, and a store that sits near 75 percent with converters standing is being converted, not
short.

<!--meta
id: F-eco-multipliers
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Misc/TeamBase.cpp:59-70 (handicap, advantage), upstream/RecoilEngine/rts/Sim/Misc/TeamBase.h:73-76, upstream/RecoilEngine/rts/Sim/Misc/Team.cpp:130-146 (AddResources), upstream/RecoilEngine/rts/Sim/Units/Unit.h:122 (the default), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1046 (decay), 2083 (reclaim), upstream/RecoilEngine/rts/Lua/LuaSyncedCtrl.cpp (AddTeamResource, SetUnitResourcing), upstream/RecoilEngine/rts/Net/NetCommands.cpp:1041-1075 (the share order), upstream/Beyond-All-Reason/luarules/gadgets/game_energy_conversion.lua:95-103 (SetUnitResourcing umm/uue), upstream/Beyond-All-Reason/luarules/gadgets/unit_mex_upgrade_reclaimer.lua (AddTeamResource), upstream/Beyond-All-Reason/modoptions.lua (multiplier_*, ai_incomemultiplier, tax_resource_sharing_amount)], game: 54199a0d, people: [the user, 2026-10-22: the bonus counts twice through converters]}
status: live
-->
### Handicaps and multipliers

A lobby can give a side an income bonus: the room's handicap, 50 for the "+50 percent" seats of the lobby games, is
an income multiplier of 1.5. The engine applies it to every resource the engine or the game's scripts add as income:
extractors, generators, the commander's own make, the metal a converter makes, and the 50-metal refund when an
advanced extractor replaces a tier-1 one. It does not touch costs, reclaimed metal, the refund of a decaying frame,
or what allies send or overflow. So the bonus counts twice through a converter: the energy reaching the converter is
already 1.5 times the generators' make, and the metal it makes is multiplied again, 2.25 times an unbonused seat's
metal from the same generators (a bonus seat gets 1.5 metal per 70 energy, and has 1.5 times the energy). The room's
other multipliers (resource income, metal extraction, energy production, build power, an AI income multiplier, a tax
on sharing) all default to one or zero and are off in our games unless the room says so.

<!--meta
id: F-eco-consequences
layer: fact
premises: [F-eco-extractors, F-eco-energy, F-eco-build, F-eco-reclaim, F-eco-conversion]
sources: {people: [the user, 2026-10-22: direct consequences belong in the economy section], note: arithmetic on the facts above and the unit tables; the spot's value is the map's and Comet Catcher's 2.3 is the worked case}
status: live
-->
### What follows directly

Arithmetic on the facts above, with Comet Catcher's 2.3 metal a second per spot as the worked case; other maps put
their own value in.

- An extractor repays its 50 metal in 22 seconds of its own output on such a spot (50 seconds on a spot worth 1.0),
  and its 500 energy is 25 seconds of one solar. Delay costs the thing's output over the delay: an extractor placed a
  minute late forgoes 138 metal, nearly three times its price.
- An advanced extractor adds three times the spot, 6.9 a second here, for 620 metal and 20 energy a second of upkeep
  (one solar's make): the gain repays it in 90 seconds, in 3.4 minutes on a spot worth 1.0.
- Metal per metal a second of income: an extractor 22, an advanced extractor 90, a tier-1 converter with the 3.5
  solars that feed it about 540, an advanced converter with the advanced solars that feed it about 290. While a spot
  stands free, taking it is twenty-five times cheaper than converting.
- Energy by metal: a solar is 7.75 metal per energy a second, an advanced solar 4.4 (and 5,000 energy to build, 62
  seconds of its own output), a wind turbine 40 over the map's average wind, cheaper than a solar wherever the
  average exceeds 5.2 but never steady, a fusion 4.5, an advanced fusion 3.2.
- In an energy stall only what costs no energy is built at full speed (a step is all or nothing): solars (0 energy)
  go up as fast as their builders work, an advanced solar's 5,000 waits for energy the stall does not have. The way out
  of a stall is plain solars first, advanced ones after.
- What production draws in energy: a plant alone making Blitzes 68 a second (three and a half solars), with the
  commander assisting 203 (ten); making constructors 72 and 217; Stouts 93 and 278; a constructor building
  extractors 25, building solars nothing. The energy a side needs follows from what its plants make and who assists
  them, not from how many plants stand.
- Build power by metal: a construction turret 1.15 metal a point, a construction bot 1.4, a construction vehicle 1.5,
  an advanced constructor 1.9 to 2.0; the commander's 300 is already paid for. The turret does not move and reaches
  400.
- A frame locks its metal for its build time over the power on it: an advanced plant on one construction vehicle
  holds 2,600 metal for five minutes, on the commander and four constructors for 41 seconds; the metal returns only
  by reclaim or decay.
- A fight that destroys 2,000 metal of units leaves up to 1,200 in wrecks for whoever holds the ground afterwards,
  less for every unit the last blows overkilled; a construction vehicle collects about 38 metal of wreck a second, a
  resurrection bot 58, the commander 75, so three constructors clear a 1,200-metal field in about ten seconds. Eating
  a bot lab returns 500 metal in 17 seconds of the commander's time, a vehicle plant 590 in 19.
