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
**Claim.** When a resource runs out, every builder and factory drawing on it slows in proportion (construction gets only the
fraction of resources available), energy-per-shot weapons stop firing, and units with energy upkeep (extractors 3 E/s,
advanced extractors 20 E/s, radar, cloak) can switch off. An energy stall is therefore worse than a metal stall: it also cuts
metal income and defence. Guide targets: energy bar above 50% at all times, metal bar between 20% and 80% (above 80% means
too little build power, not wealth).
**Status.** reported (2026-09-19); upkeep and per-shot numbers supported from local source
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
**Used by.** (candidate: turrets guard the lab; consider setting turrets passive — needs the custom `priority` command id in
the shim)

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
Reclaiming your own live unit returns 100% of its metal, all at the end. Resurrecting costs 0 metal and 50% of the unit's
energy cost. Repair is free. The commander's wreck holds 1250 M. After an even mid-map trade of 40 units the field holds on
the order of 2000-3000 M — whoever holds the ground afterwards collects it.
**Status.** supported (2026-09-19) — local source; the mid-map estimate is arithmetic
**Evidence.** `gamedata/alldefs_post.lua:556-569` (wreck_metal_ratio 0.6, heap_metal_ratio 0.25); `gamedata/modrules.lua:21-39`
(reclaimMethod 0, unitMethod 1, unitEfficiency 1, featureEnergyCostFactor 0, repair energyCostFactor 0, resurrect 0.5);
`units/armcom.lua:114` (dead metal 1250). Web agrees: https://www.beyondallreason.info/guide/reclaim-resurrect-repair.
**Would be wrong if.** A logged area-reclaim over a battle site yielded far less than 0.6 × the metal of units lost there.
**Used by.** (candidate: H-ARMY-RECLAIM — after a fight within our half or the middle, send rez bots/constructors to
area-reclaim; BARb gets that metal otherwise. Relates to K-army-piecemeal-midmap: our mid-map trades feed whoever reclaims)

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
**Status.** reported (2026-09-19); health values supported from local source
**Evidence.** https://www.beyondallreason.info/guide/important-knowledge-on-advanced-mechanics (energy storage self-destruct
1280 damage in 260 AoE); https://www.crdhq.com/articles/bar-new-player-guide-getting-started. `armmakr.lua` (health 167),
`armwin.lua` (196).
**Would be wrong if.** Raids on our generator field in logs killed only what they shot at.
**Used by.** (candidate: building placement keeps ≥ 1 footprint gap between converters/turbines)

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
**Used by.** H-PLAYER-SIDES; the record header's `seats`.


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
**Used by.** H-MICRO-FLEE, H-MICRO-FORM (the lead and the Close point).

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

