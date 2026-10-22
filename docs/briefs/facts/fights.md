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
a Stout shell 97 and 18; a Pounder shell 190, 27 to aircraft and 70 to a submarine. Most ground weapons may fire at
aircraft only when nothing on the ground is in range, so a tank body under bombers answers with a fifth of its damage
at best; anti-air mounts do their full damage upward and almost none downward.

<!--meta
id: F-fight-health
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1100-1104 (SlowUpdate), 694, 1341 (restTime), upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:329-331, upstream/Beyond-All-Reason/gamedata/alldefs_post.lua:618-624, upstream/Beyond-All-Reason/units/armcom.lua:4 (autoheal 5), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:1361-1368, 1441-1466 (experience), upstream/Beyond-All-Reason/gamedata/modrules.lua:6-10, 123-127, upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:659 (power), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:307-310, 683, 1183-1200 (flanking), upstream/RecoilEngine/rts/Sim/Misc/ModInfo.cpp:112-114, upstream/Beyond-All-Reason/gamedata/modrules.lua:46-48], game: 54199a0d, claims: [K-engine-a-unit-left-alone-heals, K-mech-experience]}
status: live
-->
### Health, healing, experience, flanks

A unit that has taken no damage for a minute heals 5 health a second until hit again (the game's default for every
unit); the commander heals 5 a second at all times besides. Repair by a builder is free and adds its power's worth of
the unit's build time a second.

Units learn. Each hit on an enemy gives the attacker experience equal to a thirtieth of the fraction of the victim's
health it took, scaled by the victim's worth over the attacker's (worth is metal plus energy over sixty), and
experience raises health by up to two and a half times and fire rate by up to a quarter along a curve that saturates:
killing one unit of its own worth gives a unit about 7 percent more health; ten such kills give 58 percent more health
and 29 percent faster firing. Damage dealt to a stunned unit teaches nothing.

Where a hit lands matters. Every unit carries an armour direction that turns toward whoever hits it: the first hit
sets it, and later hits move it by an amount that regrows while the unit is left alone, three tenths a second. A hit
from the direction the armour faces does 0.9 of its damage; a hit from the opposite side does 1.9. A body under fire
from one side has turned its armour that way within a shot or two, and a second attacker from the other side then
hits it for twice what the first does, until the armour swings back.

<!--meta
id: F-fight-range
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Weapons/Weapon.cpp:1100-1118 (range sphere), 1118-1162 (HaveFreeLineOfFire, TestCone), upstream/RecoilEngine/rts/Sim/Weapons/WeaponDef.cpp:49-53 (avoid flags), 392-397 (heightMod), upstream/RecoilEngine/rts/Sim/Weapons/Cannon.cpp:40-57, 248-275 (ballistic range), upstream/RecoilEngine/rts/Sim/Weapons/BeamLaser.cpp:289, 419-424 (beam intensity), upstream/RecoilEngine/rts/Sim/Weapons/Weapon.cpp:479-490 (the shot's cost), upstream/Beyond-All-Reason/units (minintensity in 19 files; cylindertargeting in armflash.lua, armpw.lua)], game: 54199a0d, claims: [K-engine-a-shot-is-refused-across-a-friend, K-mech-height-and-lasers, K-mech-stall-slows-everything]}
status: live
-->
### Range, height and the line of fire

A weapon's range is a sphere, not a circle: a target higher or lower than the gun is nearer the edge of reach. For
lasers and missiles the height difference counts in full, so a 430-reach laser tower 300 above or below its target
reaches only 308 across the ground; for cannons it counts at 0.8, and a cannon firing downhill reaches a little
farther and uphill a little shorter besides, by the arc of its shell. The light guns of the Blitz and the Pawn test a
cylinder instead and reach their full 180 up or down a slope.

Beam lasers (the light laser tower, the Beamer, the heavy laser tower) lose damage with distance: a beam does its full
damage at the muzzle and half at the end of its range, in a straight line between. Nineteen units are exempt and keep
their damage to the end, the Sharpshooter and the Legion's commander among them; the tier-1 towers and tanks are not.

A gun fires only when the line to its target is clear. Before each shot the engine traces a cone from the muzzle to
the aim point and refuses the shot if a friendly unit, a feature or the ground stands in it (the towers' lasers ignore
features). A body packed closer than its guns' spread therefore muzzles itself: the units behind cannot fire past the
units in front, and a shot that is fired can still hit a friend who steps into it. High-trajectory cannons (the
Shellshocker, the Gauntlet's lob) arc over what blocks a flat shot.

Sixty-two units pay energy to fire. The light laser tower draws 20 a shot (43 a second firing), the Beamer 6 a shot
but ten shots a second, the heavy laser tower 75, the commander's D-gun 500. A gun whose shot the side cannot pay
does not fire: a laser line stands silent in an energy stall.

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
radius. A Blitz bullet's splash is 4. No weapon damages the unit that fired it.

<!--meta
id: F-fight-sight
layer: fact
sources: {code: [upstream/RecoilEngine/rts/Sim/Misc/LosHandler.cpp:83-162, upstream/RecoilEngine/rts/Sim/Misc/LosMap.cpp:16, 545, 654, upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:480-491, upstream/RecoilEngine/rts/Sim/Misc/LosHandler.h:273-283, 324-325 (radar error 96, factor 2), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:131, 1494-1504, 2314, 2331 (targfac), upstream/Beyond-All-Reason/units (no unit sets targfac)], game: 54199a0d, claims: [K-mech-under-the-water-is-sonar-and-torpedoes]}
status: live
-->
### Sight, radar, sonar

Sight is a ray cast over the terrain from the unit's eye (its middle plus its sight height, with five elmos of grace),
out to its sight distance: a ridge hides what is behind it, and a unit on a rise sees over what a unit below cannot.
Aircraft are seen at one and a half times a unit's sight distance. Radar is cast the same way from the radar's own
height, so radar too is blocked by terrain and a radar on a hill covers more. A unit seen by radar alone is a contact
whose position is blurred by a random offset of up to 96 elmos, the same for every side all game, since no unit in
the game reduces it; the engine's rule is that each active targeting facility halves the error, and the game has
none. Jammers hide what is within their circle from radar; stealth units never show on radar; under water a unit is
seen by sonar alone, in a plain circle, and is hit by torpedoes alone.

<!--meta
id: F-fight-ground
layer: fact
sources: {code: [upstream/Beyond-All-Reason/gamedata/movedefs.lua:1-60 (SLOPE, DEPTH, CRUSH), 70-400 (the classes), upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:352 (crushResistance = mass), 442 (speed)], game: 54199a0d, claims: [K-engine-a-pushed-tank-crushes-a-light-unit, K-mech-a-low-muzzle-does-not-fire-from-shallow-water]}
status: live
-->
### Ground: slopes, water, crushing

Speeds in the unit tables are elmos a second on the flat; slopes slow a unit by its class's factor and stop it at its
limit. Tanks climb slopes to 27 degrees, hovercraft to 33, bots and the commander to 54, so ground a bot crosses can
wall a tank. Tanks and bots wade water to 20 deep (the Tick to 5); the commander, the amphibious bots and the amphibious
tanks walk the sea floor; hovercraft cross water at speed; ships need 8 of depth and submarines 15. A unit standing
in water with its muzzle under the surface cannot fire. A moving unit crushes a unit it pushes into whose mass is below
its crush strength: tanks crush light units (the Rover, the Tick) that are jostled under them, bots crush less.

<!--meta
id: F-fight-death
layer: fact
sources: {code: [upstream/Beyond-All-Reason/weapons/Unit_Explosions.lua:269-283 (energystorage), 675-689 (fusionExplosion), 737-752 (advancedFusionExplosion), 841-856 (commanderexplosion), upstream/Beyond-All-Reason/units/ArmBuildings/LandDefenceOffence/armsilo.lua (nuclear_missile), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:525-531], game: 54199a0d, claims: [K-mech-commander-blast, K-mech-buildings-explode, K-mech-missile-and-blast-deaths-unattributed]}
status: live
-->
### Death blasts

What a death leaves to reclaim is in the economy's section on wrecks: worn down, a wreck; killed by a burst, a heap
or nothing. Some deaths also explode. A commander's death does 5,000 damage at its centre falling to nothing at 350
elmos, to friend and foe alike; an energy storage 980 to 210; a fusion 2,650 to 240; an advanced fusion 10,600 to
640; a nuclear missile 12,500 to 640. A solar, a converter or a wind turbine bursts for nothing. Blasts chain where
such buildings stand together, and the engine names no attacker for a unit a blast or a missile kills.

<!--meta
id: F-fight-big-weapons
layer: fact
sources: {code: [upstream/Beyond-All-Reason/units/armcom.lua (disintegrator), upstream/Beyond-All-Reason/units/ArmBuildings/LandDefenceOffence/armsilo.lua, armamd.lua, upstream/Beyond-All-Reason/units/ArmBuildings/LandDefenceOffence/armgate.lua, upstream/RecoilEngine/rts/Sim/Weapons/WeaponDef.cpp:547, upstream/Beyond-All-Reason/gamedata/alldefs_post.lua:1098-1103], game: 54199a0d, claims: [K-mech-dgun, K-mech-shields-stop-shots-by-type]}
status: live
-->
### The D-gun, the nuke, the anti-nuke, the shield

The commander's D-gun reaches 250, costs 500 energy a shot, fires every 0.9 seconds and deletes anything it hits except
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
  17; a light laser tower kills a Pawn in 2.3, a Blitz in 4.5, a Stout in 11; a Bull kills a Stout in 7.3 and a Stout
  a Bull in 58.
- Range buys free fire. A light laser tower (430) against a Blitz (reach 180, speed 101) fires for 2.5 seconds before
  the Blitz can answer, 400 of its 730 health; a Bull (460) on a Stout (350, 75) fires 1.5 seconds first, 360 of 1,800.
  The free damage is the reach difference over the attacker's speed, times the defender's damage a second.
- When every unit is in range and firing, a side's weight is its count squared times one unit's damage a second times
  its health: twice the units is four times the weight. Per metal, damage a second times health over metal squared
  ranks tier-1 land units: Pawn and Grunt 11, Blitz and Incisor 5.4, Centurion 4.0, Rover 3.8, Mace 3.6, Thug 3.4,
  Pounder 3.1, Brute and Stout 2.9, Aggravator 2.5, Rocketeer 2.1, Janus 1.6, Shellshocker 1.4, Wolverine 1.1,
  Whistler 0.9, Lasher 0.9. The ranking ignores range, speed, splash, scatter and overkill, which is what the rest of
  this page and the derivations are about: the cheap swarm wins only when it reaches the fight whole.
- Splash against a packed body: a Pounder shell does nearly its full 190 to every unit within about 50 of the burst,
  so five Blitzes packed within that take 950 a shot and all die in the fourth shot, 7.2 seconds, where five spread
  out die one at a time over 35; a Stout's shell, radius 24 and falling fast, touches two at most.
- A beam at the end of its range does half: a light laser tower puts 161 a second on a Blitz at its muzzle and 80 at
  430; a Blitz that closes takes the full rate only at the end. A laser tower 300 above its target reaches 308 across
  the ground, so a tower on a rise covers less ground than its range says and a unit below it is in reach later.
- Two sides beat one: the second attacker on a unit's far side does 1.9 of its damage to the first's 0.9, more than
  twice, until the armour swings.
- Ten kills of its own worth make a unit 58 percent tougher and 29 percent faster firing; a veteran body is worth
  more than its metal and is worth repairing.
- Laser lines eat energy: ten light laser towers firing draw 430 a second, a Beamer alone 60; a side at zero energy
  defends with cannons and missiles only, and has no D-gun.
