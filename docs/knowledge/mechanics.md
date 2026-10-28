# Mechanics a bot must respect

Seeded 2026-09-19. Paths relative to `upstream/Beyond-All-Reason/` (the game source we run). Entries here are mechanics
that change what a rule should do; pure engine-interface facts belong in `game-rules.md` / `harness/`.

### K-mech-converter-threshold
**Claim.** Energy converters only run while stored energy is above a team-wide fraction of energy storage, default 75%
(`mmLevel` 0.75, set for every team including AIs; players move it with a slider). The gadget converts only the energy above
that level, every 15 frames. Consequences: converters never cause an energy stall by themselves; stored energy hovering at
~75% with converters built means "surplus is being converted", not "short"; and a rule that waits for >80% stored energy
before adding converters is looking at a level the existing converters actively pull down to 75%.
**Status.** supported (2026-09-19) — local source, not observed in a match
**Evidence.** `luarules/gadgets/game_energy_conversion.lua:263,273,295-297` (`teamMMLevels[tID] = 0.75`;
`convertAmount = eCur - eStor * mmLevel`).
**Would be wrong if.** bot.log showed converters producing metal with stored energy below 70% of storage.
**Used by.** H-ECO-CONVERT-SURPLUS (its 80% trigger interacts with this: after the first converters, stored energy will sit
near 75% and the rule may never fire again — check in logs), H-ECO-ENERGY-BY-STORAGE (40% floor is safely below 75%)

### K-mech-converter-rates
**Claim.** T1 converter: 70 E/s → 1.0 M/s, costs 1 M + 1150-1250 E, 167 hp. Feeding one takes 3.5 solars (542 M) or, at
Quicksilver's average wind, 5.5 turbines (220 M): +1 M/s from conversion costs ~220-540 M versus 50 M for a free extractor
spot at roughly +2 M/s and ~100 M per +1 M/s for an advanced-extractor upgrade. Converters are the last resort for metal, and
the right sink for surplus energy.
**Status.** supported (2026-09-19) for rates/costs (local source); ranking is arithmetic using an inferred ~2 M/s per spot
**Evidence.** `units/ArmBuildings/LandEconomy/armmakr.lua` (energyconv_capacity 70, energyconv_efficiency 0.01429),
`cormakr.lua`; generator costs as in openings.md. Agrees with official guide (converters "used primarily if expansion becomes
too difficult"): https://www.beyondallreason.info/guide/in-depth-look-at-economy. A third-party rule says build converters
"when energy income exceeds 2x metal income" (https://www.crdhq.com/articles/bar-economy-guide) — that ratio is always true
in BAR (E income is ~10-20x M income), so the rule is useless as written.
**Would be wrong if.** Measured converter output differed from 1 M/s per 70 E/s consumed.
**Used by.** H-ECO-CONVERT-SURPLUS, K-eco-expansion-before-conversion (supports it)

### K-mech-stall-slows-everything
**Claim.** When a resource runs out, construction drawing on it slows, energy-per-shot weapons stop firing, and units with
energy upkeep (extractors 3 E/s, advanced extractors 20 E/s, radar, cloak) can switch off. An energy stall is therefore
worse than a metal stall: it also cuts metal income and defence. Guide targets: energy bar above 50% at all times, metal
bar between 20% and 80% (above 80% means too little build power, not wealth). The guides' "every builder slows in
proportion" is not how the engine does it: see K-mech-stall-order-is-creation-order.
**Status.** reported (2026-09-19); upkeep and per-shot numbers supported from local source; the proportional slowing
refuted from the engine source 2026-10-13 (no builder ever gets a fraction of a step)
**Evidence.** https://www.crdhq.com/articles/bar-economy-guide (2026); https://www.beyondallreason.info/guide/in-depth-look-at-economy.
`units/ArmBuildings/LandEconomy/armmex.lua` (energyupkeep 3), `armmoho.lua` (20), `LandDefenceOffence/armllt.lua` (energypershot 20).
**Would be wrong if.** Metal income stayed flat through a logged period of zero stored energy.
**Used by.** H-ECO-ENERGY-BY-STORAGE; (candidate: H-ECO-BUILD-POWER — stored metal > 80% for 30 s ⇒ add a construction turret
or lab rather than wait for the 500-metal test in H-ECO-MORE-LABS)

### K-mech-build-power-ratio
**Claim.** Official rule of thumb: about 200 build power (one construction turret, or 2+ constructor bots) for every +5 M/s
and +100 E/s of income. Reference build power: commander 300, bot lab 150, advanced lab 600, constructor bot 80-85,
constructor vehicle 90, advanced constructor bot 210-220, rez bot 200, construction turret 200 (230 M, 3200 E, reach 400,
immobile). Per metal, the turret (1.15 M per build power) is cheaper than a constructor bot (1.4-1.5) and cannot wander off.
At our +38 M/s the rule asks for ~1500 build power in total.
**Status.** reported (2026-09-19) for the ratio; build-power numbers supported from local source
**Evidence.** https://www.beyondallreason.info/guide/in-depth-look-at-economy (official, undated).
`workertime` in `units/armcom.lua`, `ArmBuildings/LandFactories/armlab.lua`, `armalab.lua`, `ArmBots/armck.lua`,
`CorBots/corck.lua`, `ArmVehicles/armcv.lua`, `ArmBots/T2/armack.lua`, `ArmBots/armrectr.lua`, `ArmBuildings/LandUtil/armnanotc.lua`.
**Would be wrong if.** Our logs showed metal never pooling (stored metal < 20%) — then build power is not our bottleneck and
more of it only deepens stalls.
**Used by.** (candidate: H-ECO-BUILD-POWER — target build power = 40 × metal income; add turrets next to labs)

### K-mech-turrets-assist-in-radius
**Claim.** A construction turret helps anything within 400 elmos: it assists factories, builds/repairs, and reclaims. Placed
between two labs or beside lab + base buildings it raises production without pathing. It can be given a guard/assist order
on a factory like any builder. Low-priority ("passive") builders only draw resources left over after normal builders, which
lets assistants yield during a stall instead of slowing everything.
**Status.** supported (2026-09-19) for reach and the passive-priority gadget (local source); usage advice reported
**Evidence.** `units/ArmBuildings/LandUtil/armnanotc.lua` (builddistance 400, workertime 200);
`luarules/gadgets/unit_builder_priority.lua:2-23,59-62` (command `priority`; "low priority cons build either at their full
speed or not at all"). https://www.crdhq.com/articles/bar-build-order-metal-solar ("1-2 nano turrets next to factory").
**Would be wrong if.** A turret ordered to guard a lab 300 away did not raise that lab's output.
**Used by.** (candidate: turrets guard the lab; the passive flag is K-mech-builder-priority-is-a-builder-flag)

### K-mech-stall-order-is-creation-order
**Claim.** The engine shares nothing out in a stall. Every frame each builder or factory with a target tries one whole
step (`buildSpeed / buildTime` of the target, priced at the target's cost times that) and takes it only if the team holds
the whole price in both resources, else it does nothing that frame; the tries run in the order the units were created
(`activeUnits` is appended on creation). So the oldest builder wins a stall (our commander, then the first plant, then
the constructors in order), and a cheap step (an extractor's) slips through where a dear one (a plant's unit, a fusion's)
does not: the dear build starves first, whoever holds it. A target with a builder trying on it does not decay.
**Status.** conjectured (2026-10-13): read from the engine we run; the pro replays' one-second samples neither show
nor refute it (in 193 stall seconds where one lone high builder's target moved and another's did not, the oldest's
moved in 124, the youngest's in 114; in 97 % of such seconds every high builder's target moved)
**Evidence.** `upstream/RecoilEngine/rts/Sim/Units/Unit.cpp` `CUnit::AddBuildPower` (`HaveResources` then `UseResources`,
`lastNanoAdd` stamped before the check), `rts/Sim/Misc/Team.cpp` (`res >= amount`), `rts/Sim/Units/UnitHandler.cpp`
(`push_back`, the random insertion `#if 0`; `SlowUpdateUnits` before `UpdateUnits` in `Update`). The study
`docs/studies/2026-10-13-builder-priority.md` §1, §5 (`stalls.md`) and its proposed cheat-game test (§6).
**Would be wrong if.** In a metal stall two constructors of different ages on targets of the same cost both progressed
at half speed, or the younger one progressed and the older did not.
**Used by.** H-HANDS-PRIORITY-BIRTH (the plant and the turrets low, so the creation order among high builders is the commander's and the constructors')

### K-mech-builder-priority-is-a-builder-flag
**Claim.** BAR's priority is a low/high flag on a builder (commander, constructor, factory, construction turret), never on
a structure: `CMD_PRIORITY` 34571 with one parameter, 0 low, 1 high, default high, state in the unit rules param
`builderPriority`. A low builder runs at full speed or not at all: every 1 to 6 frames the gadget grants it its whole
expense for the interval only if the team's stock minus a margin (the larger of 20 % of income and 1 % of storage) minus
the high builders' expense would stay positive, else sets its build speed to 0. High changes nothing: among high
builders the engine's creation order (K-mech-stall-order-is-creation-order) decides. The stock widget sets every lab and
construction turret low and every constructor high at creation and at finish, the commander never; it runs only in a
person's LuaUI, so an AI's builders (ours, BARb's, Apex's) are all high. The command is consumed by `AllowCommand`
(never queued, never seen by `UnitCommand`); the demo stream is its only trace.
**Status.** supported (2026-10-13) from the game source we run and 75 pro replays (the widget's defaults in every game's
`bp` samples: labs low in 41 of 41 players, turrets low in 40, constructors low in 5; the hand toggles in the packet
stream; in hard metal stalls a low commander kept 45 % of its speed, a low constructor 78 %, a low turret 46 %, against
88 % and 102 % for high ones, and the flag cost nothing outside a hard stall)
**Evidence.** `luarules/gadgets/unit_builder_priority.lua` (`UpdatePassiveBuilders`, `AllowCommand`),
`luaui/Widgets/unit_builder_priority.lua` (`lowpriorityLabs`, `lowpriorityNanos` true, `lowpriorityCons` false),
`modules/customcommands.lua:39`; `upstream/CircuitAI/src/circuit/unit/CircuitUnit.h:58` (defined, never sent); the study
`docs/studies/2026-10-13-builder-priority.md` §2-3 and `run/replays/priority.py`.
**Would be wrong if.** A replay's `bp` showed a lab at priority 1 with no hand toggle on it, or a low constructor
progressing while the team's metal sat at zero with a high constructor stalled.
**Used by.** H-HANDS-PRIORITY-BIRTH, H-HANDS-LIST-LOW (`Command::Priority` through `COMMAND_UNIT_CUSTOM`, study §4),
D-PRIORITY (`bots/distilled`). What the pros do with it: K-econ-commander-low-while-assisting,
K-econ-expanding-constructor-high (economy.md).

### K-mech-factory-repeat
**Claim.** Factories have a repeat state: with repeat on, each finished unit's order is re-appended, so a short composition
queue runs forever without the bot re-issuing orders. Guides treat "factory always on repeat, never idle" as a basic rule.
**Status.** reported (2026-09-19)
**Evidence.** https://www.crdhq.com/articles/bar-build-order-metal-solar; https://www.crdhq.com/articles/bar-beginner-guide-getting-started.
Engine command `CMD_REPEAT` (standard Spring/Recoil; not checked in our shim).
**Would be wrong if.** A lab with repeat on and a 5-unit queue went idle after 5 units.
**Used by.** H-PROD-BATCH (candidate: use repeat for the steady-state batch and only rewrite the queue when the composition
changes; interacts with K-rules-factory-shift-means-five)

### K-mech-reclaim-values
**Claim.** A dead unit leaves a wreck worth 60% of its metal cost; a wreck that takes enough damage becomes a heap worth 25%.
Reclaiming wrecks and features costs no energy and is gradual (metal arrives as you go); several builders can work one wreck.
Reclaiming a live unit returns 100% of its metal times its build progress, all at the end, and leaves no wreck; an enemy's
the same (`allowEnemies`), any unfinished frame always, commanders and decoy commanders never (`reclaimable = false`). The
reclaimer drains the unit's health (`unitDrainHealth`) by its full health times reclaim speed over build time a second,
so it comes apart in its health share times its build time over the reclaimer's reclaim speed (BAR's files leave that at
the build power): a construction bot (80) takes a Pawn apart in 20.6 s at full health and 5.2 s at a quarter, while the
Pawn's 90 a second kills the construction bot in 8.4 s; the commander (300) in 5.5 and 1.4 s, its laser alone kills the
Pawn in 2.0 s, and the two add. The unit keeps fighting while it is taken apart (`unitMethod` 1: no nanoframe); an
interrupted reclaim pays nothing, but the health it drained stays lost. Resurrecting costs 0 metal and 50% of the unit's
energy cost. Repair is free. The commander's wreck holds 1250 M. After an even mid-map trade of 40 units the field holds on
the order of 2000-3000 M — whoever holds the ground afterwards collects it.
**Status.** supported (2026-09-19) — local source; the mid-map estimate is arithmetic. The live-unit reclaim amended
2026-10-27 from the engine's source (an enemy's, the health share); the enemy-only fight drill (`docs/data/scenarios-2026-10-27/`) is the runtime check.
**Evidence.** `gamedata/alldefs_post.lua:556-569` (wreck_metal_ratio 0.6, heap_metal_ratio 0.25); `gamedata/modrules.lua:21-39`
(reclaimMethod 0, unitMethod 1, unitEfficiency 1, featureEnergyCostFactor 0, repair energyCostFactor 0, resurrect 0.5);
`units/armcom.lua:114` (dead metal 1250); RecoilEngine `rts/Sim/Units/Unit.cpp` `CUnit::AddBuildPower` (the reclaim branch: step =
power / buildTime, health drained by maxHealth × step, the lump refund cost × buildProgress × unitEfficiency) and
`AllowedReclaim` (an unfinished unit always), `UnitDef.cpp:368` (reclaimSpeed defaults to workerTime). Web agrees: https://www.beyondallreason.info/guide/reclaim-resurrect-repair.
**Would be wrong if.** A logged area-reclaim over a battle site yielded far less than 0.6 × the metal of units lost there.
**Used by.** (candidate: H-ARMY-RECLAIM — after a fight within our half or the middle, send rez bots/constructors to
area-reclaim; BARb gets that metal otherwise. Relates to K-army-piecemeal-midmap: our mid-map trades feed whoever reclaims)

### K-mech-dual-guns-shoot-aircraft-last
**Claim.** A land gun that can hit both ground and air (`onlytargetcategory = "NOTSUB"`) with `badtargetcategory = "VTOL"`
picks an aircraft only when no other target is in its reach, and gives up an aircraft it holds at its next search when a
ground target has come into reach. BAR sets this on about 120 unit files, the commanders, the tier-1 raiders and tanks
(Pawn, Blitz, Stout, Mace, Grunt, Thug, Brute, Incisor), the light and heavy laser towers and the Banisher among them; a
gun that hits aircraft alone (`onlytargetcategory = "VTOL"`: Fury, Thistle, flak) is not affected. So a body fighting on
the ground has no anti-air but its anti-air-only units.
**Status.** supported (2026-10-27) — local source; the in-game sighting is a player's (All That Glitters, 2026-10-25:
"Banisher anti-air will not fire while any ground target is in reach")
**Evidence.** RecoilEngine `rts/Sim/Weapons/Weapon.cpp` `CWeapon::AutoTarget` (the best bad-category target is kept only
while no good one is found; targets are sorted with bad categories last) and the target-change test before it (a
non-user target in the bad category triggers a new search); `units/CorVehicles/T2/corban.lua` (one MissileLauncher,
range 800, reload 7.5, NOTSUB with badtargetcategory VTOL); `corsent.lua`, `corrl.lua`, `corflak.lua`, `armflak.lua`,
`armrl.lua` (VTOL only); `luarules/gadgets/unit_aa_targeting_priority.lua` (its air priorities, bombers first, apply only
to weapons with no air category among their bad targets, so not to these); run/matches/1792971120-human/00 (the player's
notes 37:55-38:07).
**Would be wrong if.** A drill with a Banisher under a Wasp and a Stout both in reach shows the missile going to the Wasp,
or a BAR gadget re-orders weapon targets for these units.
**Used by.** F-fight-range (the brief's facts).

### K-mech-stun
**Claim.** A stun (paralyzer) weapon adds its damage, times the target's `paralyzemultiplier` (1 for most, 0.2 for 16 unit
files, 0 for commanders, transports, spies, walls and a few tier-3 giants), to a stun store; the unit is stunned while the
store is above its maximum health: it neither moves nor shoots, and keeps its orders. The store falls by a fortieth of the
maximum health a second (BAR's `paralyzeDeclineRate` 40) and is capped at the health plus the weapon's `paralyzetime`
of that fall, so a unit wakes at most `paralyzetime` seconds after the last shot (7 for the Shuriken: 600 a shot, reach
220, reload 1.2). A stun weapon aims only at what can be stunned (BAR's `EMPABLE`: a surface unit whose multiplier is not
0); the engine's own aim puts a stunned unit behind the rest, and a queued attack skips one. The seconds a unit stays
stunned are `40 * (store / health - 1)`, readable for ours and for his in sight (`Unit_getParalyzeDamage`, `Unit_isParalyzed`).
**Status.** supported (2026-10-27) — local source and the stun drill: the shim's seconds counted down one a second, were
7.0 again at each new hit and never above 7; a Pawn went down to one shot, a Blitz to two
**Evidence.** RecoilEngine `rts/Sim/Units/Unit.cpp` 996-1013 (the fall, stunned while above the health) and 1253-1305
(the cap); `rts/Game/GameHelper.cpp` 754 (a stunned target's worse priority for a paralyzer); `rts/Sim/Units/CommandAI/CommandAI.cpp`
`SkipParalyzeTarget`; `rts/ExternalAI/SSkirmishAICallbackImpl.cpp` (stun readable for allies, and enemies in sight);
BAR `gamedata/modrules.lua` (paralyzeOnMaxHealth, paralyzeDeclineRate 40, 20 under the `emprework` option),
`gamedata/alldefs_post.lua:201` (EMPABLE), `luarules/gadgets/unit_paralyze_damage_multiplier.lua`,
`unit_paralyze_damage_limit.lua` (the same cap for mobile units), `units/CorAircraft/corbw.lua`; the drill
`run/matches/1793154711-duel-stun` (`docs/data/scenarios-2026-10-27/stun.json`).
**Would be wrong if.** A game with `emprework` on (the fall is a twentieth, the cap 10 s), or a drill showing a commander
stunned.
**Used by.** H-HANDS-STUN, H-MICRO-ROVE, H-MICRO-LANE, D-air-not-optional (the Shuriken).

### K-mech-dgun
**Claim.** The commander's D-gun: range 250, 500 energy per shot, 0.9 s reload, damage 99999 to everything except commanders
(0), small area (36); it is a manual-fire weapon (`commandfire`), so it never fires unless explicitly ordered. The commander's
automatic laser has range 300, 75 damage per 0.4 s. With 3700 hp, 5 hp/s self-repair and the D-gun, a commander with ≥500
stored energy beats any small T1 group that comes within 250 — raiders (range 180-215) must — but not rocket bots (475) that
keep their distance.
**Status.** supported (2026-09-19) — local source; tactical reading is ours, untested
**Evidence.** `units/armcom.lua:4,187-205,258-287` (autoheal 5; laser range 300; disintegrator commandfire, energypershot 500,
range 250, reloadtime 0.9, damage default 99999 / commanders 0). Web agrees on 500 E:
https://www.beyondallreason.info/commands/dgun.
**Would be wrong if.** The AI interface cannot issue the manual-fire command, or a D-gun order with <500 stored energy still fired.
**Used by.** H-COM-RETREAT (candidate: H-COM-DGUN — enemy ground unit within 240 and stored energy ≥ 500 ⇒ D-gun it, highest
metal cost first; keep 500 E reserved while enemies are near home)

### K-mech-commander-blast
**Claim.** A dying commander explodes for 5000 damage at the centre falling to 0 at the edge (edge effectiveness 0) of an
area of effect of 700. Spring weapon defs give area of effect as a diameter, so the radius is 350 and T1 units (≤1600 hp)
die within roughly 240. In a 1v1 the opposing commander is protected: a gadget caps
the blast damage it takes at 33% of its current health, so the game cannot end in a double kill. Our own units and buildings
get no such protection from our own commander's blast.
**Status.** supported (2026-09-19) — local source
**Evidence.** `weapons/Unit_Explosions.lua:841-856` (AreaOfEffect 700, edgeeffectiveness 0, default 5000; the diameter convention is
engine knowledge, not checked in Recoil source), `units/armcom.lua:24,48`; `luarules/gadgets/game_preventcombomb.lua:100-130`.
**Would be wrong if.** A logged enemy-commander death next to our army killed units more than ~350 away.
**Used by.** (candidate: H-ARMY-COM-SNIPE — when the enemy commander is below ~30% health, finish it with ranged units and
pull melee units back beyond 400; never fight next to our own low-health commander)

### K-mech-height-and-lasers
**Claim.** Ballistic weapons (plasma, unguided rockets) gain range firing downhill and lose it uphill; lasers lose damage
with distance (100% at point blank to 50% at max range); units take bonus damage (up to 2x) when hit from directions other
than the one they have been absorbing fire from (flanking bonus, engine default mode 1).
**Status.** reported (2026-09-19); flanking mode 1 default confirmed in `gamedata/modrules.lua:46-47`, magnitudes not checked
**Evidence.** https://www.beyondallreason.info/guide/important-knowledge-on-advanced-mechanics (official, undated).
**Would be wrong if.** Logged damage per Grunt shot was constant with distance.
**Used by.** (candidate: attack from two directions when the group is ≥ 16; place towers on high ground)

### K-mech-experience
**Claim.** Units gain experience from damage dealt relative to target value; experience raises max health (healthScale 2.5) and rate of fire (reloadScale 1.25) along a saturating `xp/(xp+1)` curve, not damage.
Only one unit type (Gunslinger `armmav`) gains range from experience. Veteran units are worth preserving, but the effect is
small for T1 units that die in one fight; it is not a reason to change rules now.
**Status.** supported (2026-09-19) for the settings (local source); the exact bonus formula is the engine's and was not verified
**Evidence.** `gamedata/modrules.lua:5-10,123-127` (experienceMult 0.3, powerScale 0, healthScale 2.5, reloadScale 1.25);
`luarules/gadgets/unit_xp_range_bonus.lua`; `rangexpscale` appears only in `units/ArmBots/T2/armmav.lua`.
**Would be wrong if.** (low value; no test proposed)
**Used by.** (none)

### K-mech-buildings-explode
**Claim.** Buildings explode when killed and can chain: energy storage and fusion blasts are large, converters and wind
turbines are fragile (167-220 hp). Advice: spread generators and converters rather than packing them, and keep converters
away from labs.
**Status.** reported (2026-09-19); health values supported from local source; measured 2026-10-24 on the drill bench: an
advanced fusion given as a weaponless target died once under a mixed column and its blast wiped the twelve units that
killed it (66,727 damage dealt by a side with no weapon, 8 of 8 duels). The blasts from `weapons/Unit_Explosions.lua`:
`customfusionexplo` (armafus) 10,600 damage over 1,280; `fusionExplosion` (armfus) smaller; `hugeBuildingexplosiongeneric`
(the gantry, the plasma deflector) 1,330 over 420; `largeBuildingExplosionGeneric` (the plants) 670 over 340;
`mediumBuildingExplosionGeneric` (storages, radars) less; the walls' `WallExplosionMetal` 0 over 36.
**Evidence.** https://www.beyondallreason.info/guide/important-knowledge-on-advanced-mechanics (energy storage self-destruct
1280 damage in 260 AoE); https://www.crdhq.com/articles/bar-new-player-guide-getting-started. `armmakr.lua` (health 167),
`armwin.lua` (196).
**Would be wrong if.** Raids on our generator field in logs killed only what they shot at.
**Used by.** (candidate: building placement keeps ≥ 1 footprint gap between converters/turbines)

### K-mech-a-fight-order-ignores-walls
**Claim.** A unit on a fight (attack-move) order fires at enemy units and buildings in its reach but never at a wall:
twelve Blitz, Stout and Janus fight-ordered onto a fortification wall (armfort, 8,900 health) walked to it, stood by
it and fired no shot in 240 s, where the same column onto a gantry opened fire at 19 s. Killing a wall takes an
explicit attack order on it.
**Evidence.** The drill bench, 2026-10-24 (`duel --drill into --target armfort`, one duel: no shot, timeout; `--target
armshltx` the same minute: engaged at 35 s). BARb's start-up script un-ignores walls for its own squads
(K-barb-wall-decoy), which is the other side of the same fact.
**Status.** observed (2026-10-24, one duel each way). Used by: the drill bench's choice of target (design D9).

### K-mech-builder-trip-overhead
**Claim.** A mobile builder's time from finishing one building to laying the next frame is the straight-line walk (distance
minus build reach, at the unit's `speed` in elmos/s) plus a fixed loss: median 1.2-1.5 s when the next site is already in
reach, 2.4-3.4 s when it has to walk; on walks over 1000 elmos the total is 8 % above straight-line time. Build reach is
measured to the target's edge, roughly `builddistance` + 40. With these, `buildtime / workertime` predicts our first
factory's finish within 0.7 s (mean absolute, 12 games), which also checks K-open-build-times.
**Status.** supported (2026-09-19) on Quicksilver, near home; our bot's orders (0.5 s tick) are part of the fixed loss
**Evidence.** 565 commander and constructor trips in the 12 records of `v15-terrain-medium`; `buildorder calibrate`
(`../studies/data/calibration.md`). Far trips across cliffs are barely sampled.
**Would be wrong if.** On another map or for far outposts the recorded trip times exceeded this by more than ~20 %.
**Used by.** `crates/buildorder` (`Scenario::detour`, `mobile_overhead`, `walk_overhead`, `reach_bonus`).

### K-mech-upkeep-has-no-priority
**Claim.** An extractor's energy upkeep stands in the same queue as construction: with stored energy at zero extractors
stop (income 22 to 14 with 11 extractors standing), while a build that costs no energy (a solar collector) goes on at
full speed through the stall.
**Status.** measured (2026-09-20) for the extractors (8 games); the solar half is the engine's rule as we read it
(each builder is refused only the resources its own target needs) and fits the recorded recoveries, not isolated.
**Evidence.** `open-cal2-*` records around 4:30-5:00; `crates/buildorder/src/sim.rs` step 3 models both, and the
simulator's metal-income bias at minute 5 went from +2.0 / +2.4 metal/s to +1.1 / -1.1 (Quicksilver / Mithril).
**Would be wrong if.** A cheat-spawned test (extractors, zero energy, no builders) kept paying metal.
**Used by.** `crates/buildorder` (the opening search's model).


### K-engine-damage-direction-from-unseen-attackers
**Claim.** The AI interface lists no projectiles, but its damage event carries the weapon definition and a unit vector
from the hit unit toward the attacker's (radar-error) position whether or not the attacker is in our sight; only the
attacker's id is withheld when it is not. With the weapon's range from the callback, one hit gives a bearing and a
reach, and two hits on units standing apart give a point.
**Status.** read from the engine source (2026-09-22); the bearings' accuracy is pianist-player-9's to show.
**Evidence.** `rts/ExternalAI/EngineOutHandler.cpp` `UnitDamaged`: `attackeeDir` is set whenever `attacker` is not
null, and only `visibleAttackerUnitId` depends on LOS or radar; `AISEvents.h` `SUnitDamagedEvent` has `dir_posF3`
and `weaponDefId`.
**Would be wrong if.** The engine zeroed the direction for an unseen attacker (it does not: the check is on the
pointer, not on visibility) or the radar error put the bearing far off (it is the attacker's error position).
**Used by.** H-HANDS-SHELLED.

### K-mech-chat-echoes-our-own-lines
**Claim.** The engine reports a line the AI says (`Game_sendTextMessage`) back to it as a chat event from its own
host player, like any other player's line.
**Status.** demonstrated (2026-09-22, human-1)
**Evidence.** human-1: "gl hf! Opening: ..." said at 0:18 by the player came back at 0:18 as chat from player 3,
the AI's host, and woke the player ("That was my own chat echoed back"); the banner lines came back the same way.
**Would be wrong if.** The host player's number were someone else's (the banner, which the bot itself says at
frame 75, came back under the same number).
**Used by.** `relay_chat`: lines we said and have not yet seen back are dropped when they come back.

### K-engine-orders-run-at-the-servers-frame
**Claim.** An AI's orders and chat lines go to the game's server and run at the frame it stamps them with, in the
order sent. A host running behind the server (a client stalled at the start of a lobby game, catching up after)
has every order of every AI it hosts run that many frames late, and our own line's echo times it: the frames from
saying a line to hearing it back are the lag on the orders sent beside it.
**Status.** demonstrated (2026-10-25, human 1792971120)
**Evidence.** human 1792971120 (All That Glitters Extended, 6v2, the user's client hosting our seat, two WRTerritory,
two WRDistilled and BARb): the banner said at frame 75 came back at 960, and the four heuristic bots' banners showed
at 960 too; the commander stood idle where it spawned until 960, and the vehicle plant ordered at 405 was started
at 1022. The player's turns' wall clock: game frames 330 to 1680 passed in about 25 s (the client catching up). After
it, the player's "gl hf" said at 2160 came back at 2169, and winds beside the commander ordered at 1605 and 2025 were
started 18 frames later. Every other human game: the banner back in 6 to 9 frames; an arena game at speed 50: 29
(companies-2).
**Would be wrong if.** A line could come back by a path the unit orders do not take (it would time the chat, not
the orders); the plant's start 62 frames after the echo agrees with the shared path.
**Used by.** H-HANDS-ORDER-LAG (`brain/echo.rs`): the hands' grace on an order's outcome.

### K-mech-lobby-box-beside-us
**Claim.** A lobby's start script can give an enemy ally team a start box whose centre is next to our own start,
so the box guess of the enemy base is worthless there; the mirror is the better guess.
**Status.** demonstrated (2026-09-22, human-2)
**Evidence.** human-2 (`run/matches/1790045403-human-1`): three ally teams in the script; the terrain line says
"enemy start 605 away on foot, 579 in a straight line" from our start at (4066, 2237); the picture's `enemy_base`
was at D3, "presumed at E3"; the lab yard, forward of home toward it, faced spot_10. The same in human-1 ("presumed
at E3"; the player marked its own guess at 2:25 "to stop the commander wandering toward the bad guess").
**Would be wrong if.** The boxes were right and a person really started there (nobody did: the Pawns came from the
south).
**Used by.** H-MAP-ENEMY-START (a guess nearer than a third of the map's short side is discarded for the mirror;
the boxes are logged at the start).

### K-mech-factory-exit-is-its-front
**Claim.** A factory's finished units leave through its front (the side its building facing points to: 0 south +z,
1 east, 2 north, 3 west): the engine sends each to `pos + frontdir * (radius + unit radius)` and searches empty
spots forward of the front (`Factory.cpp` `SendToEmptySpot`). A building whose centre lies within about 170 elmos of
the front seals the yard: the units finished after it stand in the yard, the engine gives up their moves, and the
factory builds nothing more while they stand there, whatever is in the bank.
**Status.** demonstrated (2026-09-22, hands-2-bulldogs), one game; the engine source read for the exit rule.
**Evidence.** `run/matches/1790117628-hands-2-bulldogs/00`: solars at the player's mark `avp_side` finished 10:54-11:54,
one 16 x, 165 z off the plant's centre (the plant faced south); Bulls three to seven finished 11:42-14:24 stood within
thirty elmos of the exit 5.0-7.5 min each; 156 move-failed events on Bulls; the plant finished nothing 14:24-19:04
with metal 2,000-5,600 of storage full; all five left at 19:10 as the solars and then the plant were destroyed. The
first two Bulls (9:36, 10:42, before the solars) left within seconds.
**Would be wrong if.** Units left through another side too, or a building at the front but off the engine's search
strip let them out; `yard_min` on the scorecard over the coming games tells (yard-1-bulldogs, with the lanes kept
clear: 0 minutes, and thirteen Bulls left within half a minute but one held by its group).
**Used by.** H-ECO-YARD-LANE, H-HANDS-STUCK-WORDS, H-PLAYER-REMOVE.

### K-mech-air-damage-per-armour-class
**Claim.** Every weapon in the game carries a damage per armour class, and aircraft are the `vtol` class: the Blitz's
gun does 9 a bullet to ground and 2 to aircraft, the Stout's 97 and 18, the Pawn's likewise a fifth; the Whistler's
second missile 160 to aircraft and 1 to ground, the Nettle's the same shape. Whether a weapon may fire at aircraft is
the mount's `onlytargetcategory` (VTOL is aircraft; SURFACE and NOTAIR exclude them; NOTSUB includes them) and
`badtargetcategory` VTOL makes them a target of last resort: a Blitz fires up only when nothing on the ground is in
range. So eight Blitzes put about 50 damage a second on a 560-health Banshee, and only once the ground is clear.
**Status.** read from the unit files (2026-09-23), `upstream/Beyond-All-Reason/units/*/armflash.lua` lines 139-148
and the like; whether a rocket or a shell that may target an aircraft hits a moving one is not measured.
**Evidence.** The unit files; human-9 (`run/matches/1790132629-human-9`): six Banshees killed eight Blitzes at 8:22 with no loss.
**Would be wrong if.** The engine ignored the class table for aircraft, or a Blitz group in a duel took a Banshee down
at its ground rate.
**Used by.** H-HANDS-ROSTER (the glossary's air words, `run/unit_stats.py`).

### K-mech-spot-centre-is-the-metal-centroid
**Claim.** The engine's metal spot positions (`Map_getResourceMapSpotsPositions`) are not where the game and BARb put
extractors: on Comet Catcher they sit 82 elmos (+40, +72) from every extractor BARb built at the north-east start and
60-200 from 32 of the 42 the experienced players built; both snap to the metal-weighted centre of the patch. An
extractor asked at the engine's point can be refused ("the site was bad": human-9, spot_5 at 0:28, the third start
extractor lost for two minutes) though most are tolerated and earn within a tenth of the players' rate.
**Status.** supported (2026-09-23 00:30): the centroid the shim now publishes matches BARb's four extractors at the
north-east start to the elmo (spots-1-easy's header against escalate-7's truth file); the players' 42 extractors sit a
median 93 from it, 36 of them 60-200 off, which is the valid area around the centre, not the centre: people build
where the walk is shortest, as `extractor_site` does for us.
**Evidence.** The yard-1 header's spot table against BARb's four extractors in escalate-7's truth file (all four
+40, +72); the players' 42 extractors in `run/matches/1790122851-replay-vak-vs-artur-comet`; human-9's refusal.
**Would be wrong if.** The centroid sat as far from BARb's extractors as the engine's point did, or another map's
engine spots matched the game's.
**Used by.** H-ECO-SPOT-CENTROID.

### K-mech-spot-held-within-extractor-radius
**Claim.** The game counts an extractor as standing on a metal spot when it is within `Game.extractorRadius` (120
elmos on Comet Catcher and Quicksilver) of the spot: `cmd_mex_denier.lua` denies a build order for a spot that has an
extractor of the same ally team within that radius (`mexExists`), and allows a position only where the radius covers
the spot's whole patch. Our own site chooser places extractors up to that radius off the centre toward the builder
(median 101, max 113 in four games), so any smaller test of "held" reads our own spots as free.
**Status.** measured (2026-09-23) in fixes-1, concurrent-1, schedule-1 and quick-1: 53 of 59 refused extractor
orders were for a spot where our own extractor already stood when the order was given (offsets 99-104 from the
centre), the picture calling the spot "free metal spot"; the orders cost 2-26 s of walking each and were never
retried (the spot was taken). H-ECO-SPOT-HELD makes the radius the one test; verified in held-1-hard-aggressive: 0 refused
extractor orders and no held spot called free in 393 sampled entries.
**Evidence.** `upstream/Beyond-All-Reason/luarules/gadgets/cmd_mex_denier.lua`,
`common/upgets/api_resource_spot_finder.lua` (`IsBuildingPositionValid`); the four matches' `bot.log` "never
started" lines against their records' `created` events.
**Would be wrong if.** Extractor orders at spots we hold kept being refused with the radius test in place, or
free spots near our extractors were read as held (a neighbouring spot within 120 of one of ours).
**Used by.** H-ECO-SPOT-HELD, H-ECO-SPOT-CENTROID.

### K-mech-tier-2-extractor-stands-on-ours
**Claim.** A tier-2 extractor (Advanced Metal Extractor, `armmoho`/`cormoho`) over a spot we hold is built exactly on
our standing extractor's position: the game upgrades it in place, and any other position on that spot is refused
(the spot's extractor is within the extractor radius: `cmd_mex_denier.lua` `mexExists`). Our extractors stand up to
113 elmos off the spot's centre (H-ECO-SPOT-CENTROID's offset placement), so a moho ordered "at spot_N" and placed at
the centre is refused.
**Status.** supported (2026-09-23): low-1-jev-hard-aggressive, three `armmoho spot_N` list steps at 24:46 refused
"site bad" at 11 elmos from the centres and 97, 104 and 25 from our extractors; the player flagged it. Fixed in
`economy.rs` `build_site_for` (the site is our extractor's position): pace-1 built eight mohos through it, every one
on our extractor's position, and the 18 refusals left were mohos over a frame or over a finished moho
(K-hands-moho-over-a-frame-was-refused).
**Evidence.** `run/matches/1790185145-low-1-jev-hard-aggressive/00/bot.log` "armmoho order ... never started" against
the record's `created` extractors.
**Would be wrong if.** A moho placed on our extractor's position were still refused (a footprint or terrain rule).
**Used by.** H-ECO-SPOT-CENTROID, H-HANDS-SCRIPT.

**Amended 2026-10-20 (terr2-nullai-comet-1, terr3-nullai-comet-1):** the position must match to the last digit, and
the AI interface reports a standing Cortex extractor 0.01 off the build grid (x.01, z.01; Armada's at .00): a tier-2
extractor ordered at that reported position was dropped by the engine every time for seat 0 as Cortex (0 of 94
orders begun over two batteries) while Armada's went up, and seat 1 as Cortex was spared because `cmd_mex_denier`
counts our own extractor only when the ally team of the AI's hosting player is ours (it reads `playerID`, player 0 in
the arena). The host now snaps every extractor order to the engine's own build grid (`ai-host` `snap_to_build_grid`,
`CGameHelper::Pos2BuildPos`): Cortex seat 0 then began 40 of 46 tier-2 orders (terr3-nullai-comet-1 game 02).

### K-mech-nano-turrets-idle-without-an-order
**Claim.** A construction turret does nothing on its own: the game gives it no order when it stands, and it helps a
factory only under a guard (or repair) order. In escalate-7 five turrets stood beside the plant from 6:15 and were
idle in all 2,388 samples of them, with no command ever sent to one; every "nanos on the plant" lesson in the brief
had bought nothing.
**Status.** stated by the user 2026-09-23, watching escalate-7 ("the construction turrets don't actually appear to be
configured to assist the lab they were placed near"); confirmed in the record; with the guard order (H-ECO-NANO-GUARD)
fixes-1's five turrets were idle in 4 samples of 2,686.
**Evidence.** `run/matches/1790132331-escalate-7-hard-aggressive`: finished events for armnanotc (6:15, 7:56, 8:18, 8:45, 9:08), the `cmd` records (none to those
ids), the samples' idle flag.
**Would be wrong if.** A turret with no order helped a factory in its reach anyway in some game version.
**Used by.** H-ECO-NANO-GUARD.

### K-mech-start-script-names-the-seats
**Claim.** The game's start script, which the AI interface hands the shim whole (`Game_getSetupScript`), names every
seat: a `[PLAYERn]` section per person with `name`, `team`, `spectator` and, in lobby games run by SPADS, `skill=[43.19]`
and `rank`; an `[AIn]` section per AI with `shortname`, `version`, `team`, `name` and an `[options]` block carrying
BARb's `profile`; `[TEAMn]` with `allyteam` and `side`. Nothing else in the interface says who plays a team. In games
the user hosts the AI's chat lines appear under the host's name, so the script is the only reliable source.
**Status.** read from the scripts (2026-09-23): the arena's (`run/matches/*/00/script.txt`), the lobby's in the demo
headers (`run/matches/1789929416-replay-ben-vs-medium/*.sdfz`, `1790122851-replay-vak-vs-artur-comet`).
**Evidence.** The scripts named; `crates/ai-shim/src/script.rs` tests parse both shapes.
**Would be wrong if.** A lobby wrote AI seats without a `team` or people without a `name`.
**Amended 2026-10-03.** The AI's chat under the host's name is verified: the engine takes an AI's text only as a
slash command and `/say` is chat from the player hosting the AI (`ai-shim/src/engine.rs`, `Command::Say`); in
human-12 our line "Hi, good luck! ..." came back as a chat event from player 0, and `[player0]` of that script is
`computer_whisperer` (the user's name, with an underscore in the script). Spectators are `[PLAYERn]` sections with
`spectator=1`.
**Used by.** H-PLAYER-SIDES; H-PLAYER-PEOPLE; the record header's `seats`.


### K-engine-a-shot-is-refused-across-a-friend
**Claim.** Sight is terrain-only: the line-of-sight map is a ray cast over the heightmap from the unit's sight-emit
height (BAR sets none, so every unit sees from 20 elmos), and units are not occluders. What blocks a ball of tanks is
the shot: every weapon keeps the engine's default `avoidFriendly` (BAR overrides it in no unit file and no gamedata
file), so before each shot the engine traces the ray, or the cannon's arc, against allied hulls and refuses to fire
when one crosses it. A unit with its target in range and its shot refused stops and points at the target; it does
not step aside (only 13 unit files set `strafeToAttack`: spiders, the Razorback, the Pyro, the Termite, none a tank).
A shell already flying and any splash still land on a friend (`collideFriendly`), so friendly fire is possible while
the refusal keeps most of it out. The AI interface reports shots (`EVENT_WEAPON_FIRED`, the firing unit's team) and
hits on enemies (`EVENT_ENEMY_DAMAGED`, sent to the attacker's own team only, and only while the enemy is in sight
or on radar), so "in reach and silent" is observable for our own units and not for a replay's spectator seat.
**Status.** read from the engine and the game (2026-09-24), after experienced players told the user the balls "cannot
see past each other's turrets"; measured once (worth-1, 24 min, a Stout army): 8% of soldier-seconds with an enemy in
reach were muzzled (no shot for 3 s or two reloads), 36% of those with a friend within a hull's width of the line to
the nearest enemy, 26% with the enemy at the reach's edge, 39% clear; the share did not rise with the ball (6 or more
friends within 120: 6%; alone: 14%); Stouts fired 0.51 a second in reach against 0.83 possible; friendly fire 5.4%
of damage dealt. A friend on the line costs about 3% of a Stout's time in reach: real and small in this game. In the duel
harness (2026-09-24, `muzzle-stout-shapes`) a ball of 24 Stouts on flat ground was muzzled under 0.5% of its
in-reach seconds and fired 0.83 a second: the arc clears the hulls there (K-units-duel-a-stout-ball-is-not-muzzled-on-flat-ground).
**Evidence.** `rts/Sim/Misc/LosMap.cpp` (raycasts against `mipHeightMap` only, `LOS_BONUS_HEIGHT` 5);
`rts/Sim/Units/UnitDef.cpp` `losHeight` default 20; `rts/Sim/Weapons/WeaponDef.cpp` `avoidFriendly` default true;
`rts/Sim/Weapons/Weapon.cpp` `HaveFreeLineOfFire` (`TraceRay`/`TestCone` with the avoid flags) and `Cannon.cpp`
(`TestTrajectoryCone`); `rts/Sim/Units/CommandAI/MobileCAI.cpp` `ExecuteAttack` (`StopMove` and `KeepPointingTo`
when `TryTargetRotate` holds; the strafe branch needs `strafeToAttack`); `grep -rl strafetoattack
upstream/Beyond-All-Reason/units` (13 files); `rts/ExternalAI/EngineOutHandler.cpp` `UnitDamaged` and `WeaponFired`.
**Would be wrong if.** BAR's gadgets set `avoidFriendly` false at load (none found in `gamedata/`), or the engine's
cannon arc cleared every hull at the ranges tanks fight at (the instrument will say).
**Used by.** The fire instrument (the record's `dealt`, `shots`, `ff`, `xo`, `xi`; `run/fire.py`); nothing in the
brain models it yet (the focus rule counts every soldier with a target in reach as a shooter; H-MICRO-FAN opens
lines only for the D-gun).

### K-engine-hold-fire-keeps-a-target
**Claim.** Setting a unit to hold fire (fire state 0) stops its weapons choosing new targets but not shooting a target
they already have: a weapon that took an enemy as its target in the frames after the unit appeared keeps firing at it.
A stop order drops every weapon's target.
**Status.** supported (2026-09-24)
**Evidence.** `rts/Sim/Weapons/Weapon.cpp` `AllowWeaponAutoTarget` (the fire-state test only gates auto-targeting);
`rts/Sim/Units/CommandAI/CommandAI.cpp` `ExecuteStop` (`DropCurrentTarget` on every weapon). In the engine (scenario
smoke runs of `2v1b-hard-1510-truth`, 2026-09-24): with hold fire alone, 14 of the enemy's Pawns given beside Stouts
held three frames after they appeared died in the next five seconds, the engine naming held Stouts as the killers;
with a stop after the hold the held units fired 5 shots in all (the Rovers that still died, 8, died to Stouts that
fired no shot: K-engine-a-pushed-tank-crushes-a-light-unit). How many of the 14 were shot rather than crushed was not
counted.
**Would be wrong if.** Held units with no target from before the hold fired at enemies that appeared later.
**Used by.** H-DUEL-SCENARIO-PREP (harness only).

### K-engine-a-unit-left-alone-heals
**Claim.** A hurt unit that takes no damage for a while heals by itself, slowly: a Stout hurt to 95% of its health was
whole again about 95 s later, one hurt to 88% had reached 97% after about 85 s.
**Status.** conjectured (2026-09-24): measured on a handful of units in one smoke run; the rate and the delay before it
starts are not read from the game's files.
**Evidence.** Scenario smoke run of `2v1b-hard-1510-truth` (2026-09-24), each unit's health when its hurter stopped and
when the fight began.
**Would be wrong if.** A unit hurt and left alone for two minutes, with nothing near it, kept its health.
**Used by.** H-DUEL-SCENARIO-PREP (the hurters keep their targets down until all are done, and go at once).

### K-engine-a-pushed-tank-crushes-a-light-unit
**Claim.** A light vehicle given beside tanks that were given close together (and push each other apart) is killed by
them without a shot: the engine names the tank as its killer.
**Mechanism corrected 2026-10-22 (the engine's code):** units are never crushed (`SolidObject.h:341` crushable is false for units and only features set it; `MoveMath.cpp:209-217`), so the Rovers died of unit-to-unit collision damage (`modrules.lua:71` allowUnitCollisionDamage true), a hypothesis from the code, not traced in the records. See docs/briefs/facts/fights.md F-fight-ground.
**Status.** conjectured (2026-09-24): the mechanism (crushing) is inferred from the killer being a tank that fired no
shot; not read from the engine.
**Evidence.** Scenario smoke runs of `2v1b-hard-1510-truth` (2026-09-24): Rovers given 100-160 from Stouts in the same
frame died within two seconds to Stouts with no `WeaponFired` event; given two seconds later, 45 clear of everybody,
none died.
**Would be wrong if.** A Rover given 20 from a standing Stout that nothing pushes is killed by it without a shot.
**Used by.** H-DUEL-SCENARIO-PREP.

### K-engine-a-short-move-order-brakes-the-unit
**Claim.** A ground unit given a move order to a point about 100 elmos away, re-issued every 32 elmos, moves at about
half its speed: the engine slows a unit approaching its goal, and a goal always within its braking distance keeps it
slow.
**Status.** measured (2026-09-25) in three player games' records, the second after an order: after a lane step (a
Move within 130 of the unit) Stouts moved 28-30 elmos/s (n = 62-550) against 48-55 after a far Move and 67-74 after a
Fight; Blitzes 25-40 against 52 and 77-100 (2v1b-hard, 2v1-hard_aggressive; bank-1's Blitzes 58 against 56, its
Stouts 38 against 40). The flee's step order now goes 2.5 times further than its cell (H-MICRO-FLEE).
**Evidence.** The records of `run/matches/1790261454-2v1b-hard`, `1790259885-2v1-hard_aggressive`,
`1790257668-bank-1` (the main checkout); the engine's `GroundMoveType` brakes toward the goal.
**Would be wrong if.** The live game with the longer step showed the same speed after a step as before.
**Amended 2026-09-25 (the chase instrument, `duel --chase`; the user's lead from standing-1).** It is the goal's
distance, not the order's frequency: a Fight at a goal 300-1,100 away re-sent every 10, 15, 30 or 60 frames leaves a
Rover at 168 of 168 elmos/s and a Blitz at 98-100 of 101 (chase-fav-*, chase-flash-*), while H-MICRO-FORM's slot
150 ahead re-issued every 64 elmos held Rovers at 126 (75%) and Blitzes at 87 (86%) over the same chase
(chase-lane-*-prefix), and standing-1's Rover trailing a Flea at 300-370 for half a minute moved at 33-44% of its
speed under those orders (two to three a second, points 64-66 apart). Over the live records (120,000 soldier-seconds
with a goal over 300 away) the speed share by orders received in the second is 0.99 / 0.91 / 0.83 / 0.63 / 0.59 for
0 / 1 / 2 / 3 / 4+, the fast types worst (Rovers 0.87 / 0.44 / 0.33): each order to a goal within the unit's braking
distance restarts the approach. The lead a slot sends a unit to should be seconds of its speed, not a fixed 150
(`form::lead_for`), and a chase's goal should lie past the enemy, not short of it.
**Amended 2026-10-24 (the mechanism, from the engine's source).** The brake is not it: `GroundMoveType::ChangeSpeed`
brakes within `v^2 / (2 x maxdec)` of the goal, 30 to 50 elmos for our tanks, and only when the order is the last
in the queue. What caps a unit short of a near goal is the end-of-path rule in the same function: while the unit's
current waypoint is its goal (a short order has one waypoint) its target speed is at most the goal distance times
pi over the frames a full turn takes, so `speed <= distance x turnrate x 0.001438` elmos a second: a Stout (turnrate
340) at a goal 100 ahead runs at 49 of 75 and is free of the cap beyond 153, a Blitz (544) at 78 of 101 and free
beyond 130, a Rover (653) free beyond 179, a Janus (271) beyond 139. The measured 28 to 30 for Stouts after a lane
step within 130 is that cap. The same rule is a pacing law: a goal placed `wanted speed / (turnrate x 0.001438)`
ahead of a unit and re-issued before it gets there holds it at that speed (the drill bench's lead-law arm,
`docs/design/2026-10-24-formation-drills.md` §2). `WAIT_FOR_PATH` is 0 in this Recoil: a unit is not held for its
path. Vehicles slow for turns only beyond `turninplaceanglelimit` (90 degrees for every unit read), to a tenth of
their speed while they turn.
**Amended 2026-10-24 night (the drill bench's kill-test, `duel --drill walk --lead L --reissue S`, 36 duels, ledger
row lead-*).** On a straight walk the end-of-path cap does not bite: a lone Blitz sent every tick to the point L ahead
of itself holds 0.95 of its speed at L = 70 and 0.98-0.99 at 100-200 where the cap predicted 0.55 and 0.78; its
condition, the unit's heading off its waypoint's direction, is not met once the unit faces its goal, and a unit on a
straight line faces it. What slows a unit near its goal there is the brake zone, `v^2 / (2 x maxdec)` (42 elmos for a
Blitz at full speed), over a goal distance shortened by the order's latency (about seven frames from the snapshot to the
engine): at L = 40 the Blitz holds a steady 72 (0.69-0.71, 67-75 second to second). Re-placing the same lead every
second instead gives a sawtooth (43-66 at L = 70, 75-88 at 100; the record's 0.63-0.59 at three and more orders a
second). The September live numbers (Stouts at 28-30 after a flee step within 130) came from steps off the unit's
heading, where the cap's condition holds; a turning unit under a near goal is untested on the bench. The pacing law
that stands is a slot moving at the pace, K-engine-a-slot-moving-at-a-pace-paces-the-unit.
**Used by.** H-MICRO-FLEE, H-MICRO-FORM (the lead and the Close point); the drill bench's pacing arms.

### K-engine-a-slot-moving-at-a-pace-paces-the-unit
**Claim.** A ground unit sent every tick to a point that moves along its way at a pace below its speed settles at that
pace, trailing the point inside its brake zone, with no speed command: the engine's own approach to a goal is the
speed controller. A formation whose slots all move at one pace translates rigidly and arrives as it set out.
**Status.** measured (2026-10-24 night, the drill bench, 24 duels, ledger row pace-*): a lone Blitz (101) behind a slot
at 54 holds 52-56 a second once caught up (share 0.57; 22 s over 1,500 against 14 unpaced) and 72-83 behind one at 75
(0.80, 16 s); at a 1 s cadence the per-second mean holds too (51-57). A Stout (75) behind a slot at 54 swings 34-77
second to second (mean 0.77), the one unsteady reading. The 4 Blitz + 4 Stout + 4 Janus column in two ranks of six at
64, every slot carried at 49 (0.9 of the Janus's 54), arrives 34 / 36 / 37 s over 2,000 with every type within 35-37,
98 deep x 320 wide at contact against the ranks' nominal 96 x 320 and an engagement ramp of 0.9 / 1.8 s, where the
same column under one order is 18 / 25 / 35, 923 deep, 7.1 / 17.1 (K-engine-a-mixed-column-under-one-order-is-one-
column-a-speed); at the Janus's full 54 the Janus (0.95) never regains the slot it lost at the start and the column
arrives 31 / 33 / 36, 194 deep.
**Evidence.** `run/matches/*-duel-pace-*` (`run/drill_table.py --types --seconds`); design
`docs/design/2026-10-24-formation-drills.md` D11 and §5.
**Would be wrong if.** A formation of other types, on a slope or through a passage, strung out under the same arm; or
the Stout's swings turned out to be the rule for the slower tanks rather than one type's.
**Used by.** The anchor (design §2): the lane's group follows slots that move at the group's pace.

### K-hands-a-turret-being-built-read-as-a-threat
**Claim.** Until 2026-09-28 nothing on our side knew whether an enemy building was finished: the shim read
`Unit_isBeingBuilt` for our own units and the truth file only, the enemy message had no such field, and the micro lane
stamped every armed building in sight at its full damage rate over its full reach, so a turret at 5% pushed units off
its approach, counted in the odds' "turrets covering it", and was remembered as a full threat until seen destroyed;
nothing preferred it as the target it is (low health, harmless, its whole metal on death).
**Status.** demonstrated from the code (2026-09-28, the user's question "how does micro combat logic handle a currently
building turret?"); the effect in games unmeasured (the records before this date carry no flag for enemies).
**Evidence.** `crates/micro/src/lib.rs` `threat_sources` and `crates/bot-protocol/src/messages.rs` `EnemyUnit` before
the change; the truth files, which had the flag all along (`[id, name, x, z, health, being built]`).
**Would be wrong if.** The engine reported an unfinished enemy building as unarmed (`weapon_count` 0), or refused
`Unit_isBeingBuilt` for enemies in sight.
**Used by.** [[H-HANDS-UNFINISHED-HARMLESS]].

### K-hands-a-queued-timed-assist-never-held
**Claim.** A `queue` list's `assist N` step, ordered when the plant before it was 60% built and so queued behind it,
held the commander at the plant for about one second once the plant stood: the queued step's words were never
recorded, the runner read the step before it, found no timer, and ordered the next step the same second; the
person's advice that the commander assist the lab for its first units was written in every opening packet and never
happened. A second fault compounded it: a builder diverted from its list by the pass (`attack party_N`) had its step
re-ordered the next second, and the two alternated.
**Status.** demonstrated (2026-09-28; thebluegecko: "the commander should be assisting the lab when it first comes up")
**Evidence.** bluegecko-3v1-comet-catcher-10: the list rows `1:04 assist 25 -> next, queued`, `1:11 extractor -> build`,
`1:20 assist 25 -> queued`, `1:23 corsolar -> build`; the commander's commands `1:11 guard` then `1:11 build`; the
plant's second unit 25 s after its first (1:13, 1:38) with the store falling 278 to 0 by 2:05. Player-11: `1:05 assist
25 -> queued`, `1:13 extractor -> build`; 3:12-3:26 `attack party_2` and `help plant build` alternating. `floor.py`'s
`assist4` (67 and 58 s) counted the later, untimed `assist` steps, not the ones at the plant's first units.
**Would be wrong if.** The engine ended a guard order by itself, or the pros' commanders left the plant for its first units.
**Used by.** [[H-HANDS-SCRIPT]] (amended 2026-09-28).

### K-mech-under-the-water-is-sonar-and-torpedoes
**Claim.** In the engine (Recoil, `rts/Sim/Misc/LosHandler.cpp`, `rts/Sim/Weapons/Weapon.cpp`, `rts/Sim/Units/Unit.cpp`,
read 2026-09-28): a unit under the surface (`IsUnderWater`) is detected by sonar only, never by line of sight or radar
(`requireSonarUnderWater`: `if (unit->IsUnderWater() && !InRadar(unit, allyTeam)) return false`, where a submerged
unit's "radar" status comes from sonar); a unit at the surface (`IsInWater`) is seen by sight and by sonar; a unit's
own sight and sonar radii do not change when it goes under. A weapon that is not a water weapon cannot fire at a
submerged target from above the water (`if (!weaponDef->waterweapon) { if (!owner->IsUnderWater() && TargetUnderWater(..))
return false; }`); a shooter under the surface hits only what is in the water; a weapon without `fireSubmersed` cannot
fire while its own muzzle is under water (a commander walking the seabed shoots nothing). Targets must be in sight or
radar status, which a sonar contact gives. So on a water map: submarines and a commander walking the seabed are
invisible without sonar and untouchable without torpedoes or depth charges, and his torpedoes reach nothing on land.
**Status.** demonstrated from the engine's source (2026-09-28); BAR's `requireSonarUnderWater` assumed on (the game's
modrules), unverified in a game.
**Evidence.** The engine files named; the AI interface's `UnitDef_getSonarRadius`, `UnitDef_isAbleToSubmerge`,
`WeaponDef_isWaterWeapon`, `WeaponDef_isSubMissile`, `Map_getTidalStrength` (`SSkirmishAICallback.h`); game 11 on
SailAway 2: `corsub` seen 8 times against 65 sightings of frigates, our seat 0 with one torpedo launcher and no sonar
tower by 13:00.
**Would be wrong if.** BAR set `requireSonarUnderWater` off (then sight would see submerged units), or its commanders
could not walk under water.
**Used by.** [[H-HANDS-UNDER-WATER]], [[H-ECO-TIDAL]].

### K-hands-a-ships-field-from-a-land-start
**Claim.** The distance field of a movement class was seeded at our start; for a ship class from a start on land the
seed is impassable, the field was never made, the class fell back to the soldiers' reach on foot, and a construction
ship was offered no metal spot at all: game 12 on SailAway 2, the ten construction ships' states over the whole game
held factories, tidal generators, torpedo launchers and reclaim, never `extractor_spot_N`, and they built three
extractors between them (from the player's lists) while the side stood at 14 extractors at 8:00.
**Status.** demonstrated (2026-09-28, `run/matches/1790532515-bluegecko-3v1-comet-catcher-12`, the Jev log's state ids
for `constructor_2328` and `constructor_8612`); fixed the same night (the seed is the nearest cell the class stands on).
**Evidence.** `crates/bot/src/brain/routes.rs` `survey` (the classes map) before the fix; the user: "The shipyard should
build constructors and expand with mexes like normal, and that didn't happen last game."
**Would be wrong if.** The ships had been offered spots and declined them; they were not offered any.
**Used by.** [[H-HANDS-WATER-START]] (amended).

### K-mech-an-extractor-draws-only-the-squares-within-its-radius
**Claim.** An extractor draws the metal squares whose centres lie within the map's extractor radius of where it
stands, and the game lets one be built a square further out than that: an extractor placed off the patch's centre
under the game's rule loses the far squares. Ours were placed toward the builder, up to the whole allowance: in
player-52 the 64 extractors stood 101 elmos off centre at the median (BARb's 39: 0) and each drew 1.9 metal a second
from a patch worth 2.3 (Comet Catcher: 12 squares of 255, `maxMetal` 0.75, `extractsMetal` 0.001), a sixth of the
metal income of every game since the offset was written; the pool's income per extractor at 8:00 is 2.3.
**Status.** demonstrated and fixed 2026-10-03 (`mex-centre-smoke`: 13 extractors 0 to 80 off centre, 2.3 a second
each at 30 s, 60 s, 180 s, 210 s, 240 s with energy in the store).
**Evidence.** `upstream/RecoilEngine/rts/Sim/Units/UnitTypes/ExtractorBuilding.cpp` (`sqrCenterDistance <
Square(extractionRange)`); the income steps of player-52's first four minutes (6.2, 8.1, 10.0, 11.9, 13.7 at 2 to 6
extractors); found while checking the map sheet's figure against the brief's "about 2.6" (which no measurement
supports).
**Would be wrong if.** The income steps were short for another reason (energy: the steps were read with energy in
the store).
**Used by.** [[H-ECO-SPOT-CENTROID]] (amended).

### K-hands-a-role-word-named-a-unit-the-plant-cannot-build
**Claim.** The role words of a `produce` list were the bot lab's units whatever the factory: `raider:5` written before
the game was `armpw:5` at an Armada vehicle plant, which builds no Pawn; the tool took it without a word and the plant
stood idle until the next list.
**Status.** demonstrated (sonnet-low-1, the list of 0:00 and the plant idle 1:58-2:11, from the model review, the
list verified in the transcript; sonnet-medium-1 wrote the same list); fixed 2026-10-03 for `raider`, `line` and
`constructor` at a vehicle plant.
**Evidence.** `run/matches/1791050868-sonnet-low-1/00/strategist-0.jsonl`, the first `produce` call.
**Would be wrong if.** The plant's idle seconds had another cause (not checked beyond the review).
**Used by.** [[H-PLAYER-ROLE-WORDS-AT-THE-PLANT]].

### K-mech-a-low-muzzle-does-not-fire-from-shallow-water
**Claim.** A unit standing in shallow water can be unable to fire, depending on where its muzzle sits: in human-16
(Cape Violet, three seats of ours against a person) our Incisors standing in water 10 to 20 deep had an enemy in
sight inside 90% of their reach for 86 unit-seconds and fired in 2% of them. They counted in the fight's odds and
did nothing in it.
**Status.** observed (the user, watching human-16, 2026-10-03; the record's `shots` against `en` and the terrain
file's heights). Which units and what depth is not mapped: the same record's other low firing shares (Wolverines 11%,
Lashers 22% on dry ground) include slow reloads and targets the weapon cannot hit, and were not separated.
**Evidence.** `run/matches/1791059154-human-16/00/record-0.jsonl`, e.g. 6:27-6:28 at (5260, 2450), ground -14.
**Would be wrong if.** The Incisors were held from firing by something else there (a hold-fire state, friends in
the line); not checked against the engine's code.
**Used by.** [[H-HANDS-CANNOT-FIRE]].

### K-hands-the-d-gun-was-fired-through-our-own-plant
**Claim.** The D-gun destroys everything on its line, and the hands' D-gun move said nothing of what of ours stood
on it: in player-55 at 3:35 the commander D-gunned a Rover beside our vehicle plant, and the plant and the
constructor on its pad were destroyed (19,824 of friendly-fire damage by the commander in that second; the plant's
`destroyed` event names the commander). Production stood until the plant was rebuilt.
**Status.** demonstrated (2026-10-03, `run/matches/1791061704-player-55-experience/00/record-0.jsonl`); the move's
words named what of ours the line to the nearest member crossed, within half a footprint and 24, from player-56.
**Amended 2026-10-24.** The line check missed both friendly kills since: the shot does not run along the line to where
the target stands when Jev picks, because the target moves until the impact (33 and 39 frames from the gate to the
first kill: Jev's answer of 12 to 18, the turret's aim, the fireball's flight at 10 a frame). In
`1792809574-human/00` seat 1 at 7:21 (Boreal Falls, the user's 2v3) a Blitz 239 away moved 84 before the shot, which
took our advanced vehicle plant and an advanced constructor that stood 17 and 2 outside the band; in
`1792547311-human/00` seat 1 at 14:52 (SD-129) a Raider 78 away circled the commander and the shot went ninety degrees
off the checked line through a Moho and an advanced solar (the words had named both 75 frames earlier and dropped
them as the Raider moved on). The engine's part: the fireball (collision radius 10, the DGun type's default) keeps
flying and exploding through everything to the weapon's range (`noexplode`, `TraveledRange`), each explosion a radius
of 18 (`areaofeffect = 36` is a diameter, `WeaponDef.cpp` scales it by 0.5) with 99,999 damage falling to 15 % at the
rim, measured to the collision volume's surface (`GameHelper.cpp` DoExplosionDamage); the shot fires at the unit's
position when the turret has aimed, and does not track after. The words now span the cone the target can swing the
line through (`ours_in_the_shot`); `run/dgun_line.py` replays every recorded shot: the cone names all four victims
and warns on two of the nine clean shots (the second shot at the same circling Raider a second later, and an
extractor beside the commander on the second Boreal shot), and Jev re-asked with its words drops the shot from 0.59
to 0.33 and 0.66 to 0.11. Not yet seen in a game.
**Would be wrong if.** The plant died to something else in the same second; the record's attacker is the commander.
**Used by.** [[H-HANDS-DGUN-LINE]].


### K-mech-shields-stop-shots-by-type
**Claim.** A shield (Keeper `armgate`, Overseer `corgate`, Soteria `legdeflector`: 3,000 to 3,200 metal, radius 550,
6,175 shield strength, regenerating) stops a shot when `shield.interceptType & weapon.interceptedByShieldType != 0`
(`WeaponDef.cpp` 547). The engine's defaults by weapon type: Cannon 1, LaserCannon and BeamLaser 2, MissileLauncher 4,
AircraftBomb 8, Flame 16, Rifle 128, Melee 256; BAR sets StarburstLauncher to 1024 and DGun to 512
(`gamedata/alldefs_post.lua` 1098-1103, under the lobby's `experimentalshields = unchanged`). The three shields
intercept 1: plasma (Thug, Karkinos, Fatboy, the Astraeus from outside) is stopped; rockets (Arbiter), missiles
(Thanatos), lasers and heat rays pass. They are `exterior`: only shots from outside the bubble are stopped, so a unit
inside it shoots freely. The AI interface gives the shield per definition (`UnitDef_getShieldDef`, radius, power,
intercept type, exterior) and each weapon's mask; the shim carries them from 2026-10-18 (`UnitDefInfo.shield`,
`intercepted_by`).
**Status.** supported (2026-10-18): engine and game source.
**Evidence.** `upstream/RecoilEngine/rts/Sim/Weapons/WeaponDef.cpp` 470-548; `upstream/Beyond-All-Reason/gamedata/alldefs_post.lua`
1095-1150; `units/Legion/Utilities/legdeflector.lua` (shield_power 6175, radius 550, intercepttype 1, exterior);
`units/CorBots/T2/corhrk.lua` (StarburstLauncher), `units/Legion/Bots/T2 Bots/leghrk.lua` (MissileLauncher).
**Would be wrong if.** The lobby set `experimentalshields` to `absorbeverything` or `bounceeverything` (then every weapon reads 1 and the shields stop everything).
**Used by.** H-REPORT-SHIELDS; the brief's sight passage.

### K-mech-lazarus-raises-commanders
**Claim.** A commander's wreck resurrected by a resurrection bot gives its side a new, living commander, so a side whose
every commander has died is not out while a Lazarus reaches a commander wreck; the kill counts only when the wreck is
reclaimed, destroyed or stood on.
**Corrected 2026-10-22 (the game's rules, `luarules/gadgets/game_team_com_ends.lua`, deathmode `com`, the mode our lobbies use):** a side (ally team) is killed the frame its last commander dies, so the raise only helps while another commander of the side lives, as it did in the game below (Apex's two allies' commanders stood); a seat whose own commander died plays on while an ally's lives. See docs/briefs/facts/economy.md F-eco-reclaim.
**Status.** observed (2026-10-19)
**Evidence.** apex-v032-altored-3v3-apex60-1 (truth file): Apex's three commanders (ids 9947, 26858, 5831) died at
21:14, 25:52 and 37:48; new commanders 6141 and 15353 first appear at 23:54 (E8) and 43:24 (G7), both alive at 60:00;
Apex fielded 51 Lazarus (`cornecro`) over the game. Our bodies killed the commanders and moved on; nobody reclaimed the
wrecks, and the game ran to the cap.
**Would be wrong if.** The new ids were commanders given by a lobby rule or built, not raised (no building makes one;
the lobby had three seats).
**Used by.** The briefs' commander-kill words (candidate: reclaim or stand on a commander wreck; H-PLAYER-OWN-TRADE).

### K-mech-missile-and-blast-deaths-unattributed
**Claim.** The engine names no attacker for a unit killed by a silo's missile or by another unit's death blast: the
`UnitDestroyed` event's attacker is -1 for both, and the kills are told apart only by the weapon of the last
`UnitDamaged` on the dead unit (a warhead such as `crblmssl`, whose range is 72,000 against a map of 8,192; an
explosion weapon such as `korgExplosion`, `fusionexplosion`, `nukeBuilding` or `commanderExplosion`, which sets no
range and carries the engine's default of 10) and by what died beside it with a death blast
(`UnitDef_getDeathExplosion`: the Juggernaut's 10,600 over 640, a commander's 5,000 over 350, a fusion's 2,650 over
240). A Juggernaut's death blast is nearly a warhead's and reads the same in the losses. The blast's victims fall
after the exploder's own `UnitDestroyed`: 9-18 frames after a fusion, 21 and 84 along a converter chain, 210-229
after an advanced fusion (its death animation runs before the explosion), 127 of a Juggernaut's 136 within 30 frames.
The engine does name one of our own units as the attacker, for exactly two causes: its fire (a Mauser's or Janus's
shell landing among ours) and its reclaim (`CUnit::AddBuildPower` kills the reclaimed unit with the builder as the
attacker and no `UnitDamaged`). It also names our own exploding structure as the attacker of its blast's victims, with the explosion as the weapon (a vehicle plant, a moho, a construction turret and a converter in barb-ladder-50-1 and -100-1), while the fusion's four victims in apex60-2 had no attacker. The game's unit weapons are named `<unit>_<weapon>` (`corstorm_cor_bot_rocket`,
`corcom_disintegrator`, `corpun_plasma`), so the last hit's weapon tells a gun from a blast and names the gun's unit
type without the attacker. The AI interface's `UnitDestroyed` event also carries the killing weapon's id
(`rts/ExternalAI/Interface/AISEvents.h` `SUnitDestroyedEvent.weaponDefID`, passed at `EngineOutHandler.cpp:351`),
negative for the engine's own causes (`SolidObject.h`: reclaimed 12, kamikaze 18, construction decay 19, turned into a
feature 20); the shim passes it as the `UnitDestroyed` event's `death` from 2026-10-19 evening (`ai-host` `engine.death`).
**Status.** observed (2026-10-19, apex60-1 and apex60-2)
**Evidence.** apex-v032-altored-3v3-apex60-1: 15 of ours died at 33:10-33:13 within 806 of (1410, 1391) with `by`
null (five mohos, winds, solars, a flak gun, a construction turret; seat 2's player: "a missile from the silo at G7
hit the west side of our home at about 33:12"); 21 at 50:36-50:40 within 583 of (6361, 1312), one second after the
anti-nuke there died; 77 at 57:17 within 2,350 of (2301, 1812) as Juggernaut 16356 died at (2614, 1762), beside 27
more the Juggernaut's own guns were credited with; every one read "lost ... to unseen", and seat 2's player wrote of
the Juggernaut that died at 34:43 at D8 "I suspect a second missile from his silo landed on the fight, but the report
doesn't confirm it". apex-v032-altored-3v3-apex60-2 (the record's `w` from this game on): seat 1's fusion died at
16:42 (frame 30084) to a Dragon and three winds and an advanced solar died at frames 30093-30102 with
`w: fusionexplosion` and `by` null; three converters at 16:02-16:06 with `w: metalmaker` at +21 and +84 frames; two
Shredders at 28:07 with `w: largebuildingexplosiongeneric`. Of 20 deaths in the three records with one of our own
units as `by`, every attacker was alive two seconds later (Mausers 6, Janus 7, Bulldog, Whistler, Blitz; constructors
4, where the player had ordered the dead frame taken apart). The delay histogram is from apex60-1's records (own
unattributed deaths within an exploder's radius + 100 of its `enemy_destroyed` frame). The weapon names are from the
game's unit files (`upstream/Beyond-All-Reason/units`: `corsilo.lua` `crblmssl` range 72000, `corkorg.lua`
`explodeas = "korgExplosion"`, `armcom.lua` `explodeas = "commanderExplosion"`), the explosions from
`weapons/Unit_Explosions.lua` (no `range` in the file), the default from `rts/Sim/Weapons/WeaponDef.cpp`
(`WEAPONTAG(float, range).defaultValue(10.0f)`); the host already passes each hit's weapon name and range (`ai-host`
`engine.weapon`).
**Would be wrong if.** The engine named the silo as the attacker when it stands in our sight, or an explosion's
weapon id reached the AI as -1 (then only the deaths-beside rule would tell a blast), or the game gave an explosion
weapon a range (then it would read as a gun from out of sight).
**Used by.** H-REPORT-BLAST-CAUSE; the briefs' anti-nuke passages (the user, 2026-10-19).


### K-mech-a-column-anneals-and-a-turn-churns
**Claim.** Units have acceleration, deceleration and a turn radius, so an army is not a point with a speed: a body of
one speed told to move far churns for a while and then settles into a working column with every unit at full speed;
the same column told to turn around loses that order and spends a fair time churning, each unit running through its
turn radius and bumping into the others before it accelerates back the way it came. A turn therefore costs time and
formation on its own, beyond the ground given up, and two courses rated alike to commit to are not alike to switch to
from each other.
**Evidence.** The user's observation from play, 2026-10-24 (`docs/design/2026-10-24-pick-rework.md` §2). The hands'
side of it in the logs: a group ordered again each second does not move (rework-series-2 game 3, group_B
15:28-15:39: eight `gather` picks in twelve seconds, the tail 625-691 behind throughout; the arrival test ended each
one, `groups.rs`).
**Status.** the user's observation, 2026-10-24; the per-unit floor computed the same evening from BAR's defs through
the engine's speed code (brake at `maxdec`, the 180 at `turnrate`, the climb back at `maxacc`; the turn itself at a
tenth of speed for a vehicle, `turninplace` with a 90-degree limit): Rover 3.4 s, Blitz 4.5, Stout 5.9, Bull 5.6,
Luger 6.7, Janus 7.4, Pawn 1.3, the commander 1.2; bots turn on a dime, the vehicle opener churns. Measured
2026-10-24 on the drill bench (`duel --drill reverse`, twelve in ranks of six at 64 turned back 10 s into a 2,000
walk, 8 duels an arm, `docs/design/2026-10-24-formation-drills.md` §5): the median unit is back at 90% of its speed
4.0 s after the turn in every arm, pure Blitz or the Blitz/Stout/Janus mix, lane off or on (a moving column's
front turns while its rear still travels, so it beats the standing floor); the turn costs 8-13% of the whole
drill's speed share, the slowest type the most (Janus 0.75 of its speed against the Blitz's 0.88, 0.67 with the lane
on); the spread after the turn is the spread before. The annealing half: a pure column is at full speed 3 s after
the order and stays there, so the churn is the turn's, not the walk's.
**Would be wrong if.** A recorded column's reversal shows its units back at full speed within a second or two of the
order, with no bunching.
**Used by.** the pick rework's state words (`docs/design/2026-10-24-pick-rework.md` §6, not built); the drill bench.

### K-engine-a-mixed-column-under-one-order-is-one-column-a-speed
**Claim.** Units given the same move or fight order each walk at their own full speed, so a body of several types
arrives as one column per type, the gap between the first and the last the distance over the speed difference: over
2,000 a Blitz (101) arrives 7 s before a Stout (75) and 17 s before a Janus (54). Nothing in the engine and nothing
in today's lane holds them together; re-issuing the order or giving a fight instead of a move changes nothing, since
the goal and so the path are unchanged.
**Evidence.** The drill bench's baselines, 2026-10-24 (`duel --drill walk`, 6 Blitz + 4 Stout + 2 Janus in ranks of
six at 64, 2,000 east, 8 duels an arm, lane off and on, `--order move`, `--reissue 2`: arrival 18 / 25 / 35 s first /
median / last in every arm, each type at 0.94-0.96 of its speed, spread 359-387 at the median arrival;
`docs/design/2026-10-24-formation-drills.md` §5). A pure column of either type arrives first to last within 1-2 s.
**Status.** measured (2026-10-24); the stringing the games show (player-32, the pick rework's games) is this. What
holds a mixed body together is the next arc's question (the group API's pacing arms, measured against this number).
**Amended 2026-10-24 night (the lane's group, H-MICRO-GROUP; ledger rows group-*, group2-*).** Under one declared
group whose slots all move with an anchor at the slowest type's speed, the same column arrives 33 / 35 / 37 with every
type within 34-37, twelve formed every second, 135 deep x 352 wide at contact and a ramp of 1.4 / 2.5 s, against 18 /
25 / 35, 932 x 183 and 7.2 / 17.2 under one order; the price is the Janus's pace for everyone.

### K-engine-one-goal-point-funnels-a-line-into-a-file
**Claim.** Units given one move or fight order to the same point each walk their own straight line to it, so a body
abreast narrows in step with the distance left and arrives as a file two wide: twelve Blitzes in ranks of six at 64
are 320 wide at the order, 167 halfway along a 2,000 walk and 105-116 at the goal, their depth holding near the two
ranks' 88 throughout. The ball a group makes at its destination is this funnel's end. A per-unit destination in
ranks (the lane's march rewrite) holds the width the whole way (335-381) at a cost in depth (64 to 160).
**Evidence.** The drill bench's baselines read with the depth and width measure, 2026-10-24 (`duel --drill walk`,
8 duels an arm, lane off and on, the per-second `shape`; `docs/design/2026-10-24-formation-drills.md` §5, D8).
**Status.** measured (2026-10-24). The hands' advance is one point for the whole group (`execute.rs`, every member
the same `to`); in the flagship's games the lane's march (H-MICRO-FORM, on unless disabled) rewrites it into ranks of
six at the destination, which is why a body arrives in ranks there and not in a ball. The funnel is what the order
alone does, to any unit the lane does not hold, and to the harness with the lane off; what holds a body's width on
the move at a chosen depth is the next arc's question.
**Amended 2026-10-24 night (H-MICRO-GROUP).** A slot per unit that moves with the group's anchor holds the width the
whole way: twelve Blitzes in ranks of six at 64 arrive 98 deep x 352 wide at contact under the paced group (320 at the
order), where one goal point gave 89 x 116 and the old march's destination ranks (`--lane ranks` now) 94 x 339. The
march rewrite this entry's status describes is gone with the group: a host that declares no group gets the funnel.

### K-engine-slow-guard-paces-a-guarder-to-its-guardee
**Claim.** The engine's one native pacing is slow guard: a unit guarding a slower moving unit, once within the game's
`guardMovingProximityGoal` of it (BAR's `modrules.lua`: 200), has its wanted maximum speed set to the guardee's speed
at that moment and is given its own position plus the guardee's velocity times `guardMovingIntervalMultiplier`
(2.13 s) as its goal, so it moves parallel at the guardee's pace and keeps its offset; out of range it intercepts the
guardee's predicted position at full speed and the cap is lifted. The cap is set once on entering range and lifted
only on leaving it, so the pace corrects by the guarder drifting out and back. Guard is not fight: the guarder
attacks only whoever hit its guardee within the last 40 frames, then returns to guarding; the guardee's death ends
every guard on it at once.
**Status.** conjectured from the engine's source (`MobileCAI::ExecuteGuard`, `StartSlowGuard`, `StopSlowGuard`;
`ModInfo.cpp` defaults and BAR's `gamedata/modrules.lua` guard block), 2026-10-24; untested in a game or the harness.
We use guard for nanos and builder assists, never for soldiers.
**Would be wrong if.** A column of Blitzes guarding a Stout on a 2,000 walk arrived strung out as a plain column does,
or converged into a ball at the pacesetter.
**Used by.** nothing yet; a pacing arm of the drill bench (`docs/design/2026-10-24-formation-drills.md` §2, §4).
