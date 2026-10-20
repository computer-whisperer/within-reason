## Units and costs

Internal names are what `produce` and `queue` take. Metal / health / speed / range.

**Tier-1 bots (the bot lab, `armlab` 500 metal).**

| Armada | | Cortex | |
|---|---|---|---|
| Tick `armflea` | 21 / 60 / 132 / 140, scout | | |
| Pawn `armpw` | 54 / 370 / 87 / 180, raider | Grunt `corak` | 43 / 280 / 81 / 215, raider |
| Rocketeer `armrock` | 120 / 720 / 51 / 475 | Aggravator `corstorm` | 110 / 740 / 48 / 475 |
| Mace `armham` | 130 / 1000 / 46 / 380, line | Thug `corthud` | 140 / 1100 / 45 / 380, line |
| Centurion `armwar` | 270 / 1590 / 45 / 325, brawler | | |
| Crossbow `armjeth` | anti-air only: it cannot hit ground units | Trasher `corcrash` | anti-air only |
| Lazarus `armrectr` | 130; repairs, reclaims, resurrects; cannot fight | Graverobber `cornecro` | resurrects |

**Tier-1 vehicles (the vehicle plant, `armvp` 590 metal, `corvp` 570).**

| Armada | | Cortex | |
|---|---|---|---|
| Rover `armfav` | 31 / 105 / 168 / 180, scout car | Rascal `corfav` | 26 / 90 / 153 / 180 |
| Blitz `armflash` | 110 / 730 / 101 / 180, raider tank | Incisor `corgator` | 120 / 820 / 85 / 230 |
| Stout `armstump` | 225 / 1800 / 75 / 350, line tank | Brute `corraid` | 235 / 2000 / 72 / 350 |
| Janus `armjanus` | 240 / 1030 / 54 / 380, rocket burst | Pounder `corlevlr` | 220 / 1400 / 40 / 315, riot tank |
| Whistler `armsam` | 150 / 820 / 55 / 550, rockets (its anti-air missiles reach 700) | Lasher `cormist` | 155 / 860 / 52 / 575 (anti-air 700) |
| Shellshocker `armart` | 135 / 620 / 54 / 710, artillery | Wolverine `corwolv` | 170 / 750 / 48 / 710 |
| Pincer `armpincer` | 200 / 1340 / 63 / 305, amphibious | Garpike `corgarp` | amphibious |
| Constructor `armcv` | 135 / 1380 / 54 | Constructor `corcv` | 145 / 1430 / 51 |

A light laser tower (Sentry, `armllt`/`corllt`) reaches 430. The roster line of every unit says what it does to
aircraft: tier-1 tanks and bots do a fifth of their damage to aircraft and fire up only when nothing on the ground
is in range.

**Head on, at equal metal (the simulator and unit duels).**
- Mace and Centurion (Thug for Cortex) win most tier-1 bot matchups, including against light turrets. A Pawn beats
  only a Grunt and loses badly to a Thug. A Rocketeer loses to every mobile unit it cannot out-range; it out-ranges
  the light laser tower (475 against 430).
- A Blitz is two Pawns' metal with twice a Pawn's health and speed. A Stout beats any tier-1 bot head on and takes a
  light turret with a few friends, but does not out-range it (350 against 430); Whistlers and Shellshockers do.
- A fight is decided by value on the spot: the side with more metal of soldiers there, a turret counting about three
  times its metal, loses less nine times in ten; beyond two to one almost always.
- Units move at different speeds: a group sent across the map with a plain move arrives strung out; `fight to`
  marches it together.

**Splash** (`docs/studies/2026-09-28-body-sizes.md`). The radius within which a shot hurts, its damage, its range:

| Unit | Hurts within | Damage a shot | Range |
|---|---|---|---|
| Fatboy `armfboy` | 150 | 800 | 700 |
| Pounder `corlevlr` | 72 | 190 (full to the edge) | 315 |
| Bull `armbull` | 65 | 270 | 460 |
| Janus `armjanus` | 64 | two of 330 | 380 |
| Mauser `armmart` | 60 | 260 | 820 |
| Stout `armstump` | 24 | 97 | 350 |
| Blitz, light turret, Centurion, Sharpshooter | 6 to 8 | single target in effect | |

Our bodies stand 25 to 40 elmos between neighbours; the experienced players' 67 to 70. At equal metal, 25 Stouts
against 4 Fatboys lose by 0.55 at 56 spacing and by 0.22 at 160; 21 Stouts against 5 Bulls lose by 0.34 at 56 and
draw at 160; 6 Stouts against 1 Fatboy go from +0.12 to +0.52. Nothing in the hands holds a spacing.

**Energy and metal.** The table of extractors and solars is in the cases under "Income past the spot count". A
factory draws about 80 energy a second while it is being built. Vehicles cost eight to fourteen energy per metal
(Blitz 900 for 110, Stout 2,100 for 225, the constructor 1,950 for 135). Laser turrets stop firing when the energy
store is empty; the picture says STALLING when it is. An energy converter (`armmakr`) turns 70 energy a second into
1 metal.

**Tier 2, what it costs, in every faction a game can hold.** Every seat reads this whole table, not its own
column: a seat's allies and its opponents are often of another faction (the lobby's "Random" is decided at the
start), an advanced constructor an ally hands over builds its own faction's buildings (u6bkep built Cortex mohos
with thebluegecko's `corack` on Center Command), and his tier-2 units are named by his faction when they appear.
Metal / energy (the glossary, game source 54199a0d; the `units` tool has every unit's full entry by name):

| | Armada | Cortex | Legion |
|---|---|---|---|
| advanced bot lab | `armalab` 2,600/15,000 | `coralab` 2,600/16,000 | `legalab` 2,600/16,000 |
| advanced vehicle plant | `armavp` 2,600/14,000 | `coravp` 2,600/16,000 | `legavp` 2,500/16,000 |
| advanced air plant | `armaap` 2,900/29,000 | `coraap` 2,900/28,000 | `legaap` 2,900/28,000 |
| advanced constructor bot | `armack` 430/6,900 | `corack` 470/6,900 | `legack` 410/6,900 |
| advanced constructor vehicle | `armacv` 550/6,800 | `coracv` 580/7,000 | `legacv` 530/6,600 |
| advanced extractor (moho) | `armmoho` 620/7,700 | `cormoho` 640/8,100 | `legmoho` 640/8,100 |
| fusion | `armfus` 3,350/18,000 | `corfus` 3,600/22,000 | `legfus` 4,000/25,000 |
| advanced fusion | `armafus` 9,700/69,000 | `corafus` 9,700/48,000 | `legafus` 10,500/69,000 |
| advanced solar | `armadvsol` 350/5,000 | `coradvsol` 370/4,000 | `legadvsol` 465/4,080 |
| converter | `armmakr` 1/1,150 | `cormakr` 1/1,250 | `legeconv` 1/1,150 |
| advanced converter | `armmmkr` 380/21,000 | `cormmkr` 370/21,000 | `legadveconv` 370/21,000 |
| anti-nuke | `armamd` 1,500/38,000 | `corfmd` 1,500/40,000 | `legabm` 1,500/40,000 |
| silo | `armsilo` 8,100/90,000 | `corsilo` 7,700/82,000 | `legsilo` 7,700/82,000 |
| construction turret | `armnanotc` 230/3,200 | `cornanotc` 230/3,200 | `legnanotc` 230/3,200 |
| metal storage | `armmstor` 330/570 | `cormstor` 340/590 | `legmstor` 340/590 |
| energy storage | `armestor` 170/1,700 | `corestor` 175/1,800 | `legestor` 175/1,800 |
| geothermal (on a vent) | `armgeo` 560/13,000 | `corgeo` 540/13,000 | `leggeo` 560/13,000 |

On a Comet spot an advanced extractor adds 6.9 metal a second over the extractor it replaces: 90 seconds of its
output is its own metal. The plant and one constructor before the first advanced extractor: 3,150 metal, 61 seconds
of income at 52 a second with nothing else built, or three and a half minutes at 9 a second (the Center Command
sprint seat, unfed).

**Builders and build power.** Build power is what turns metal into things: a frame builds at the sum of the power on
it, and its draw a second is that power times its cost over its build time. Metal / build power / metal per point /
speed / health; Armada, then Cortex (`docs/knowledge/tier2.md`, K-t2-mobile-build-power-by-cost).

| Builder | Armada | Cortex |
|---|---|---|
| Construction turret `armnanotc` / `cornanotc` | 230 / 200 / 1.15 / stands still, reach 400 / 560 | the same |
| Bot constructor `armck` / `corck` | 110 / 80 / 1.38 / 36 / 760 | 120 / 85 / 1.41 / 34 / 820 |
| Vehicle constructor `armcv` / `corcv` | 135 / 90 / 1.50 / 54 / 1,380 | 145 / 95 / 1.53 / 51 / 1,430 |
| Butler `armfark` (advanced bot lab; builds only solars, wind, extractors, converters and radar) / Twitcher `corfast` (advanced bot lab; a tier-1 constructor's list) | 210 / 140 / 1.50 / 75 / 335 | 210 / 125 / 1.68 / 90 / 830 |
| Consul `armconsul` (advanced vehicle plant; a tier-1 constructor's list) | 250 / 150 / 1.67 / 78 / 1,080 | |
| Advanced vehicle constructor `armacv` / `coracv` | 550 / 290 / 1.90 / 52 / 2,050 | 580 / 310 / 1.87 / 50 / 2,150 |
| Advanced bot constructor `armack` / `corack` | 430 / 210 / 2.05 / 33 / 930 | 470 / 220 / 2.14 / 33 / 1,000 |
| Construction aircraft `armca` / `corca` | 110 / 50 / 2.20 / 136 / 156 | 115 / 55 / 2.09 / 131 / 161 |
| Advanced construction aircraft `armaca` / `coraca` | 340 / 120 / 2.83 / 192 / 200 | 360 / 130 / 2.77 / 182 / 205 |
| Decoy commander `armdecom` / `cordecom` (advanced bot lab) | 770 / 300 / 2.57 / 38 / 3,700 | 750 / 300 / 2.50 / 38 / 3,700 |
| On water: Voyager `armmls` / Pathfinder `cormls` (advanced shipyard) | 190 / 200 / 0.95 / 66 / 1,460 | 200 / 200 / 1.00 / 58 / 1,600 |
| Construction seaplane `armcsa` / `corcsa` (seaplane platform) | 150 / 75 / 2.00 / 124 / 405 | 140 / 75 / 1.87 / 140 / 435 |
| Sea constructor `armcs` / `corcs`; construction hovercraft `armch` / `corch`; Beaver `armbeaver` / Muskrat `cormuskrat` (amphibious, vehicle plant) | 1.60; 1.82; 1.88 a point | 1.60; 1.65; 2.00 |

The commander's 300 is already paid for. Resurrection bots cannot assist. The experienced players: the minelayer
ships assist (they are the naval assist units); the land minelayers (`armmlv`, `cormlv`) do not. A constructor must
stand within about 130 of the frame and a turret within 400; ground constructors jostle at a frame, air constructors
do not. The tier-2 construction turret is a lobby option ("Extra Units Pack") not used in serious play.

The power a project can use is set by what you can spend on it: its build time, times the metal a second you give
it, over its metal; beyond that it stalls (the store empty, everything on it slows in proportion). Armada's advanced
fusion (9,700 metal, 69,000 energy, build time 312,500) at 100 metal a second: about 3,200 power, 97 s, 710 energy a
second; at 50 metal a second half the power in twice the time. The fusion (3,350 / 18,000 / 54,000) at 50 a second:
810 power, 67 s. The advanced plant (2,600 / 14,000 / 27,000) at 30 a second: 310 power (an advanced constructor and
a turret), 87 s. A ring of turrets is built once and serves what stands within its 400; mobile power moves to the
next project (`docs/knowledge/tier2.md`, K-t2-build-power-for-a-project-follows-the-spend).

**Wrecks.** In 29 Comet Catcher duels, 24 of 58 sides built resurrection bots (11 winners, 13 losers), a median of
ten a side, the first between minutes 4 and 10; they worked wrecks on their own ground, not the field a fight had
just left. Winners' income ran above what their extractors give in 12% of minutes, losers' in 7%. The report has a
`wrecks` line; `produce` armrectr makes them and they work the richest held field on their own; a constructor's
`reclaim` option appears when a field is within 1,800 of it.

**Aircraft.** An enemy aircraft plant in sight has meant gunships about ninety seconds later (human-9: the plant
seen at 2:16, the first Banshee at 3:54, six of them killed eight Blitzes at 8:22). What hits aircraft: the Nettle
turret, Whistlers, Crossbows.

**Legion** (a seat whose `seats:` line says Legion; nothing below has been measured in a game of ours). Its tier-1
economy costs what Cortex's does (extractor `legmex` 50, solar `legsolar` 150, wind `legwin` 43, bot lab `leglab` 470,
vehicle plant `legvp` 570, converter `legeconv` 70 energy to 1 metal), so the opening arithmetic above holds. The
vehicles do not: they cost 30 to 60 % more energy per metal than their twins (Alaris 850 energy for 65 metal against
the Blitz's 900 for 110; Decurion 3,600 for 300 against the Stout's 2,100 for 225; Lance 3,600 for 240), so a Legion
vehicle army wants more energy per metal of income, and in the first Legion game the plants ran at the energy income
from 10:00 to 14:00 with the metal store capped while the advanced solars waited (advanced solar `legadvsol` 465 metal
for 100 a second, fusion `legfus` 4,000 for 950). The
names differ where it matters: the light tower is the Pharos `leglht` (70 metal, 550 health, heat ray at 430 with no
energy per shot, so it fires through an energy stall); the scout is the vehicle plant's Wheelie `legscout` (23 metal,
speed 160; there is no scout bot); the resurrection bot is the Zagreus `legrezbot`; the air factory is a Drone Plant
`legap` (430 metal) whose aircraft are drones (the Noctua `legfig` fighter at 40, a constructor aircraft `legca`). The
role words resolve to Goblin, Satyr, Pharos, Alaris and Decurion on a Legion seat. Hives (`leghive`, 350 metal, six
drones at 15 metal each) and a few tier-2 units spawn their own drones, which appear as units no factory made.

| Bot lab `leglab` | metal / health / speed / range | Vehicle plant `legvp` | |
|---|---|---|---|
| Goblin `leggob` | 30 / 200 / 75 / 261, two 12-damage lasers per 0.9 s; the cheap swarm unit | Wheelie `legscout` | 23 / 75 / 160 / 169, scout |
| Satyr `leglob` | 60 / 550 / 51 / 400, plasma 75 per 2.5 s; the Mace's place at under half the metal | Alaris `leghades` | 65 / 425 / 99 / 211, raider (the Blitz's place at 60 % of its metal) |
| Phobos `legcen` | 140 / 840 / 90 / 180, two 75-damage gauss shots per 1.4 s; fast assault | Helios `leghelios` | 160 / 800 / 75 / 320, heat-ray skirmisher |
| Ballista `legbal` | 160 / 900 / 43.5 / 460, one 250 rocket per 5.5 s; outranges the 430 towers | Decurion `leggat` | 300 / 2550 / 45 / 361, six-shot gatling; the assault tank (the Stout's place at 300) |
| Karkinos `legkark` | 310 / 1725 / 42 / 360, heat ray plus a shotgun; the Centurion's place | Barrage `legbar` | 250 / 1500 / 40 / 535, napalm artillery, area 120; short of the others' 710 |
| Toxotai `legaabot` | 120, anti-air only, 760 | Lance `legrail` | 240 / 1100 / 46 / 650, railgun 200 per 7.5 s that also shoots aircraft |
| Zagreus `legrezbot` | 130; repairs, reclaims, resurrects | Constructor `legcv` | 125 / 1330 / 57, 85 build power |

The one Legion player in the pool above OS 25 (OS 27, Great Divide, two wins) opened extractor, extractor, wind, wind,
solar, bot lab at 0:42; the lab made a constructor, Goblins, a Zagreus and a second constructor; his army from 4:00 was
Satyrs with a few Ballistas, and after 10:00 drones from Hives. An opponent may be Legion too (BARb and Apex both play
it; BARb medium opened with the vehicle plant, Alaris and Pharos): its tier-1 towers reach the same 430; its Cacophony `legmg` (380 metal, 575, land and air) and Dragon's
Jaw `legdtr` (290 metal, pop-up riot cannon at 400, stealthed) stand where the others' heavy laser would, its
Amputator `legcluster` (850 metal, 1,000) where the Gauntlet would. [K-leg-economy-is-the-cortex-economy,
K-leg-t1-bot-roster, K-leg-t1-vehicle-roster, K-leg-defences, K-leg-drone-plant-is-the-air-plant,
K-leg-open-os27-great-divide, K-leg-both-stock-ais-play-legion]
