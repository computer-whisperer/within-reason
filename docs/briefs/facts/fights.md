<!--meta
id: F-fight-intro
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Units/Unit.cpp, upstream/RecoilEngine/rts/Sim/Weapons/, upstream/Beyond-All-Reason/gamedata/armordefs.lua, upstream/Beyond-All-Reason/gamedata/modrules.lua], game: 54199a0d, engine: 2026.07.04, dates: [2026-10-22]}
status: live
-->
## Fights: how damage, range, sight and ground work

Read from the engine and the game's rules. Unit numbers (health, range, damage a second, speed) are the unit tables';
this says what the engine does with them.

<!--meta
id: F-fight-damage
layer: fact
sources: {code: [upstream/Beyond-All-Reason/gamedata/armordefs.lua, upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1320-1330 (DoDamage), upstream/Beyond-All-Reason/units/ArmBuildings/LandDefenceOffence/armllt.lua, upstream/Beyond-All-Reason/units/ArmVehicles/armflash.lua, armstump.lua, upstream/Beyond-All-Reason/units/CorVehicles/corlevlr.lua, run/unit_stats.py (weapon_dps)], game: 54199a0d, claims: [K-mech-air-damage-per-armour-class]}
status: live
-->
### Damage by armour class

Every unit belongs to one armour class and every weapon carries a damage per class; a class the weapon does not
list takes its default. The classes that matter: the default (almost every land unit and building), commanders,
aircraft, submarines, shields, construction turrets, light and heavy ships, mines. The unit tables' "dps" is the
weapon's default damage times bullets a burst over the reload, so it is the figure against ordinary ground units,
and "dps against aircraft" is the aircraft row. The rows differ a lot: the light laser tower does 75 a shot to a
tank, 112 to a commander and 5 to an aircraft or a submarine; a Blitz bullet does 9 to a tank and 2 to an aircraft;
a Stout shell 97 and 18; a Pounder shell 190, with rows of 27 for aircraft and 70 for submarines that it never
uses, since its mount fires at surface targets only. Each mount also says what it may target at all: the light laser
tower and the commander's laser never fire at a submarine, the Pounder never at an aircraft. Most ground weapons may
fire at aircraft only when nothing on the ground is in range, so a tank body under bombers answers with a fifth of
its damage at best; the commander's laser, having no aircraft row, hits them for its full 187 a second, still only
when nothing on the ground is in reach; anti-air mounts do their full damage upward and almost none downward.

<!--meta
id: F-fight-health
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1100-1104 (SlowUpdate), 694, 1341 (restTime), upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:329-331, upstream/Beyond-All-Reason/gamedata/alldefs_post.lua:618-624, upstream/Beyond-All-Reason/units/armcom.lua:4 (autoheal 5), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1361-1368, 1441-1466 (experience), upstream/Beyond-All-Reason/gamedata/modrules.lua:6-10, 123-127, upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:659 (power), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:307-310, 683, 1183-1200 (flanking), upstream/Beyond-All-Reason/gamedata/modrules.lua:46-50 (defaultMin 1, defaultMax 2 override the engine's 0.9 and 1.9), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1268-1300 (paralysis and experience)], game: 54199a0d, claims: [K-engine-a-unit-left-alone-heals, K-mech-experience]}
status: live
-->
### Health, healing, experience, flanks

A unit that has taken no damage for a minute heals 5 health a second until hit again (the game's default for every
unit); the commander heals 5 a second at all times besides. Repair by a builder is free and adds its power's worth of
the unit's build time a second.

Units learn. Each hit on an enemy gives the attacker experience equal to three hundredths of the fraction of the
victim's health it took, scaled by the victim's worth over the attacker's (worth is metal plus energy over sixty).
Experience raises health by up to 250 percent and fire rate by up to 125 percent along a curve that saturates:
killing one unit of its own worth gives a unit about 7 percent more health; ten such kills give 58 percent more health
and 29 percent faster firing. Paralysing damage teaches a tenth of that, and nothing once the target is stunned;
ordinary damage to a stunned unit teaches in full.

Where a hit lands matters. Every unit carries an armour direction that turns toward whoever hits it: the first hit
sets it, and later hits move it by an amount that regrows while the unit is left alone, three tenths a second. A hit
from the direction the armour faces does its damage once; a hit from the opposite side does it twice. A body under
fire from one side has turned its armour that way within a shot or two, and a second attacker from the other side
then hits it for double what the first does, until the armour swings back.

<!--meta
id: F-fight-range
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Weapons/Weapon.cpp:1100-1118 (range sphere), 1118-1162 (HaveFreeLineOfFire, TestCone), upstream/RecoilEngine/rts/Sim/Weapons/WeaponDef.cpp:49-53 (avoid flags; avoidfeature is false in nearly every BAR weapon), 107 (heightmod default 0.2), 392-397 (heightMod 0.8 cannons, 1.0 beam lasers), upstream/RecoilEngine/rts/Sim/Weapons/Cannon.cpp:40-57, 248-275 (ballistic range), upstream/RecoilEngine/rts/Sim/Weapons/BeamLaser.cpp:289, 419-424 (beam intensity), upstream/RecoilEngine/rts/Sim/Weapons/Weapon.cpp:479-490 (the shot's cost), upstream/Beyond-All-Reason/units (minintensity on a BeamLaser only in coresupp.lua 0.6 and corsiegebreaker.lua 0.5; the Sharpshooter's is on a LaserCannon where it does nothing; cylindertargeting in armflash.lua, armpw.lua; corlevlr.lua heightmod 1 heightboostfactor 0; corhlt.lua energypershot 50; corcom.lua range 262), note: the Stout's uphill and downhill reaches were computed by the refuter from Cannon.cpp with mygravity 0.1445 (alldefs_post.lua:1055-1066), not measured], game: 54199a0d, claims: [K-engine-a-shot-is-refused-across-a-friend, K-mech-height-and-lasers, K-mech-stall-slows-everything]}
status: live
-->
### Range, height and the line of fire

A weapon's range is tested as a sphere, not a circle, with the height difference scaled by the weapon's kind. For
beam lasers it counts in full, so a 430-reach laser tower 300 above or below its target reaches only 308 across the
ground. For missiles and laser cannons it counts at a fifth, so they nearly ignore height. Cannons are ballistic:
their shells are flown by the engine's formula with the height difference at 0.8, and the effect is large, not
small: a Stout (350 on the flat) firing at ground 100 below it reaches about 420 across, 100 above about 250, and
cannot reach 300 above at all (computed from the engine's formula, not measured; the Pounder is set to count height
in full and gain nothing downhill). The light guns of the Blitz and the Pawn test a cylinder instead and reach their
full 180 up or down a slope.

Beam lasers (the light laser tower, the Beamer, the heavy laser tower, the Overwatch and the Warden) lose damage with
distance: a beam does its full damage at the muzzle and a little over half at the end of its range, in a straight line
between. Two Cortex sea units fall less; no beam laser in the game is exempt.

A gun fires only when the line to its target is clear. Before each shot the engine traces a cone from the muzzle to
the aim point and refuses the shot if a friendly unit or the ground stands in it; wrecks and trees do not stop the
check for nearly any weapon. A body packed closer than its guns' spread therefore muzzles itself: the units behind
cannot fire past the units in front, and a shot that is fired can still hit a friend who steps into it.
High-trajectory cannons (the Shellshocker, the Gauntlet's lob) arc over what blocks a flat shot.

About a tenth of the units pay energy to fire, the lasers above all. The light laser tower draws 20 a shot (43 a
second firing), the Beamer 6 a shot but ten shots a second, the Overwatch 75 and the Warden 50, the commander's D-gun
500; the silo and the anti-nuke pay at stockpiling, not at firing. A gun whose shot the side cannot pay does not fire:
in an energy stall a laser line fires only as energy trickles in, in competition with the builders.

<!--meta
id: F-fight-accuracy
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Weapons/WeaponDef.cpp:71-73 (areaOfEffect is a diameter, halved), 83, 132-134 (accuracy, sprayAngle, movingAccuracy, targetMoveError), upstream/RecoilEngine/rts/Game/GameHelper.cpp:107-150 (DoExplosionDamage, the falloff), upstream/Beyond-All-Reason/units: armguard.lua, armart.lua, armstump.lua, corlevlr.lua, armjanus.lua, armflash.lua], game: 54199a0d}
status: live
-->
### Hits, misses and splash

Beams hit what they are aimed at. Shells and rockets scatter by the weapon's accuracy and spray, more against a
moving target: the Gauntlet's and the Shellshocker's shots land in a patch, the Stout's nearly on the mark. Missiles
that track (the Janus's) follow their target. Splash: the game writes a weapon's area of effect as a diameter, and
damage reaches every unit whose body is within half of it of the burst, full at the centre and falling to nothing at
the rim; the weapon's edge factor flattens the fall, so the Pounder (radius 72, factor 0.9) does nearly full damage
to everything inside while the Stout (radius 24, 0.15), the Mace (18) and the Janus (64, 0.15) do half at half their
radius. A Blitz bullet's splash is 4. Nearly every weapon spares the unit that fired it; the nuclear missile, the
Juno, mines and crawling bombs do not, and hurt their own side's units in the blast.

<!--meta
id: F-fight-sight
layer: fact
sources: {code: [upstream/Beyond-All-Reason/units/ArmBuildings/LandUtil/armrad.lua, armarad.lua, CorBuildings/LandUtil/corrad.lua, upstream/Beyond-All-Reason/units/armcom.lua, corcom.lua (radardistance, sightdistance), upstream/RecoilEngine/rts/Sim/Misc/LosHandler.cpp:83-162, upstream/RecoilEngine/rts/Sim/Misc/LosMap.cpp:16, 545, 654, upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:480-491, upstream/RecoilEngine/rts/Sim/Misc/LosHandler.h:273-283, 324-325 (radar error 96, factor 2), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:131, 1494-1504, 2314, 2331 (targfac), upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:492 (isTargetingUpgrade), upstream/Beyond-All-Reason/units/ArmBuildings/LandUtil/armtarg.lua, CorBuildings/LandUtil/cortarg.lua, Legion/Utilities/legtarg.lua (the Pinpointers; the user pointed to them 2026-10-22 after a grep for the engine's internal name missed the Lua key)], game: 54199a0d, claims: [K-mech-under-the-water-is-sonar-and-torpedoes]}
status: live
-->
### Sight, radar, sonar

Sight is a ray cast over the terrain from the unit's eye (its middle plus its sight height, with five elmos of grace),
out to its sight distance: a ridge hides what is behind it, and a unit on a rise sees over what a unit below cannot.
Aircraft are seen at one and a half times a unit's sight distance. Radar is cast the same way from the radar's own
height, so radar too is blocked by terrain and a radar on a hill covers more. A unit seen by radar alone is a contact
whose position is blurred by a random offset of up to 96 elmos. Each powered Pinpointer a side owns (tier 2, 810
metal and 7,200 energy, 100 energy a second to run, anywhere on the map) halves that offset for the whole side: one
makes it 48, two 24. Jammers hide what is within their circle from radar; stealth units never show on radar; under water a unit is
seen by sonar alone, in a plain circle, and is hit only by torpedoes and by weapons made for water, the commander's
sea laser among them. The distances: the radar tower (60 metal) sees 2,100 by radar and 680 by sight, the advanced
radar tower (400) 3,500 and 1,000, the commander 700 by radar and 450 by sight; the unit tables carry every unit's
sight (a scout car 635, a Blitz and a Stout 350, the light laser tower 494, the Gauntlet 455, whose gun reaches 1,220:
it fires on what radar shows it).

<!--meta
id: F-fight-ground
layer: fact
sources: {code: [upstream/Beyond-All-Reason/gamedata/movedefs.lua:1-60 (SLOPE, DEPTH, CRUSH), 70-400 (the classes), upstream/RecoilEngine/rts/Sim/MoveTypes/MoveDefHandler.cpp:86-96 (the written angle times 1.5, then 1 minus cosine), upstream/Beyond-All-Reason/gamedata/movedefs.lua:545-556 (setMaxSlope divides the written angle by 1.5 first, so the net limit is the SLOPE table's 27, 33 and 54), run/matches/1792636232-snapshot-smoke/00/record-0.jsonl (the hello's move_classes max_slope: 54.0, 27.0, 33.0 degrees), upstream/RecoilEngine/rts/Sim/Objects/SolidObject.h:341 (crushable false for units), upstream/RecoilEngine/rts/Sim/Features/Feature.cpp:85, upstream/RecoilEngine/rts/Sim/MoveTypes/MoveMath/MoveMath.cpp:209-217, upstream/Beyond-All-Reason/gamedata/modrules.lua:71 (allowUnitCollisionDamage), upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:442 (speed)], game: 54199a0d, claims: [K-engine-a-pushed-tank-crushes-a-light-unit (its mechanism corrected: collisions, not crushing), K-mech-a-low-muzzle-does-not-fire-from-shallow-water]}
status: live
-->
### Ground: slopes, water, crushing

Speeds in the unit tables are elmos a second on the flat; slopes slow a unit by its class's factor and stop it at its
limit: tanks climb to 27 degrees, hovercraft to 33, bots and the commander to 54, so ground a bot crosses can wall a
tank. (The game writes each class's angle divided by one and a half and the engine multiplies it back; the engine
reports these limits in every game's hello, and the record of 2026-10-22 shows 54.0, 27.0 and 33.0.) Tanks and bots wade water to 20 deep (the Tick to 5); the commander, the amphibious bots
and the amphibious tanks walk the sea floor; hovercraft cross water at speed; ships need 8 of depth and submarines 15.
A unit standing in water with its muzzle under the surface cannot fire. Units push each other and are never crushed
(only wrecks and trees are crushable), but skidding collisions do damage, which is the likely death of the light units
jostled among tanks in our records.

<!--meta
id: F-fight-death
layer: fact
sources: {code: [upstream/Beyond-All-Reason/weapons/Unit_Explosions.lua:193-205 (windboom), 209-220 (metalmaker), 238-250 (advmetalmaker), 269-283 (energystorage), 675-689 (fusionExplosion), 753 (customfusionexplo, the advanced fusion's, same numbers as 737-752), 841-856 (commanderexplosion), 991-1000 (smallBuildingExplosionGeneric, the solar's), upstream/Beyond-All-Reason/units/ArmBuildings/LandDefenceOffence/armsilo.lua (nuclear_missile), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:525-531], game: 54199a0d, claims: [K-mech-commander-blast, K-mech-buildings-explode, K-mech-missile-and-blast-deaths-unattributed]}
status: live
-->
### Death blasts

What a death leaves to reclaim is in the economy's section on wrecks: worn down, a wreck; killed by a burst, a heap
or nothing. Some deaths also explode. A commander's death does 5,000 damage at its centre falling to nothing at 350
elmos, to friend and foe alike; an energy storage 980 to 210; a fusion 2,650 to 240; an advanced fusion 10,600 to
640; a nuclear missile 12,500 to 640 (2,500 to a commander). A converter bursts for 660 to 105, an advanced converter
1,220 to 160, a solar or a wind turbine 89 to 90. Blasts chain where such buildings stand together, and our records
often carry no attacker for a unit a blast or a missile kills.

<!--meta
id: F-fight-big-weapons
layer: fact
sources: {code: [upstream/Beyond-All-Reason/units/armcom.lua (disintegrator), upstream/Beyond-All-Reason/units/ArmBuildings/LandDefenceOffence/armsilo.lua, armamd.lua, upstream/Beyond-All-Reason/units/ArmBuildings/LandDefenceOffence/armgate.lua, upstream/RecoilEngine/rts/Sim/Weapons/WeaponDef.cpp:547, upstream/Beyond-All-Reason/gamedata/alldefs_post.lua:1098-1103], game: 54199a0d, claims: [K-mech-dgun, K-mech-shields-stop-shots-by-type]}
status: live
-->
### The D-gun, the nuke, the anti-nuke, the shield

The commander's D-gun reaches 250 (Cortex's 262), costs 500 energy a shot, fires every 0.9 seconds and deletes anything it hits except
another commander, which it does not scratch; a side at zero energy has no D-gun.

A silo costs 8,100 metal and 90,000 energy and takes 178,500 of build time (five minutes at 600 power). Each missile
costs 1,000 metal and 125,000 energy and takes 120 seconds to stockpile; the silo fills itself to ten without being
told and fires only by hand, anywhere on the map, for 12,500 at the centre falling to nothing at 640. An anti-nuke
costs 1,500 metal, 38,000 energy and 60,000 of build time; each interceptor costs 150 metal and 7,500 energy and takes
90 seconds, to a stock of twenty, and covers everything within 2,000 of the anti-nuke. A missile aimed inside that
circle is shot down while an interceptor stands ready.

A shield (the Keeper, 3,000 metal and 54,000 energy) holds a bubble of radius 550 with 6,175 strength, starting at
2,090 and regenerating 130 a second at 562 energy a second; it stops cannon shells and nothing else: lasers, missiles,
rockets and bombs pass. It stops only shots from outside, so units inside it fire out freely.

<!--meta
id: F-fight-consequences
layer: fact
premises: [F-fight-damage, F-fight-health, F-fight-range, F-fight-accuracy]
sources: {note: arithmetic on the facts above and the unit tables (crates/bot/data/units.json); one-on-one figures assume both in range, firing at the rate of the tables, no misses}
status: live
-->
### What follows directly

- One on one, in range and no misses: a Blitz kills a Pawn in 4.1 seconds and a Pawn a Blitz in 8.1; a Stout kills a
  Blitz in 9.0 and a Blitz a Stout in 20; a Pounder kills a Blitz in 6.9; a Stout and a Pounder kill each other in
  17; a light laser tower, its beam at four fifths at the Blitz's and Pawn's 180, kills a Pawn in 2.9 and a Blitz in
  5.7, and a Stout standing at its own 350 in 19; a Bull kills a Stout in 7.3 and a Stout a Bull in 58.
- Range buys free fire. A light laser tower (430) against a Blitz (reach 180, speed 101) fires for 2.5 seconds before
  the Blitz can answer, about 260 of its 730 health once the beam's falloff over the approach is counted; a Bull (460)
  on a Stout (350, 75) fires 1.5 seconds first, 360 of 1,800. The free damage is the reach difference over the closing
  unit's speed, times the longer gun's damage a second.
- When every unit is in range and firing, a side's weight is its count squared times one unit's damage a second times
  its health: twice the units is four times the weight. Per metal, damage a second times health over metal squared
  ranks tier-1 land units: Pawn and Grunt 11, Blitz and Incisor 5.4, Centurion 4.0, Rover 3.8, Mace 3.6, Thug 3.4,
  Pounder 3.1, Brute and Stout 2.9, Aggravator 2.5, Rocketeer 2.1, Janus 1.6, Shellshocker 1.4, Wolverine 1.1,
  Whistler 0.9, Lasher 0.9. The ranking ignores range, speed, splash, scatter and overkill, which is what the rest of
  this page and the derivations are about: the cheap swarm wins only when it reaches the fight whole.
- Splash against a packed body: a Pounder shell does nine tenths of its 190 to every unit within about 40 of the
  burst, so five Blitzes packed that close take about 850 a shot and all die on the fifth shot, 7.2 seconds after the
  first, where five spread out die one at a time over 34; a Stout's shell, radius 24 and falling fast, touches two at
  most.
- A beam at the end of its range does half: a light laser tower puts 161 a second on a Blitz at its muzzle and 80 at
  430; a Blitz that closes takes the full rate only at the end. A laser tower 300 above its target reaches 308 across
  the ground, so a tower on a rise covers less ground than its range says and a unit below it is in reach later.
- Two sides beat one: the second attacker on a unit's far side does twice the damage the first does, until the
  armour swings.
- Ten kills of its own worth make a unit 58 percent tougher and 29 percent faster firing; a veteran body is worth
  more than its metal and is worth repairing.
- Laser lines eat energy: ten light laser towers firing draw 430 a second, a Beamer alone 60; a side at zero energy
  defends with cannons and missiles only, and has no D-gun.
