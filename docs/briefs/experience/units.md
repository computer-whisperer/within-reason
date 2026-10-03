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
| Whistler `armsam` | 150 / 820 / 55 / 700, rockets | Lasher `cormist` | 155 / 860 / 52 / 700 |
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

**Tier 2, what it costs (Armada).** The advanced vehicle plant `armavp` 2,600 metal and 14,000 energy; the advanced
bot lab `armalab` 2,600 and 15,000; an advanced constructor 550 and 6,800 (vehicle) or 430 and 6,900 (bot); an
advanced extractor 620 and 7,700; a fusion reactor `armfus` 3,350 and 18,000 for 750 energy a second. On a Comet
spot an advanced extractor adds 6.9 metal a second over the extractor it replaces: 90 seconds of its output is its
own metal. The plant and one constructor before the first advanced extractor: 3,150 metal, 61 seconds of income at
52 a second with nothing else built.

**Wrecks.** In 29 Comet Catcher duels, 24 of 58 sides built resurrection bots (11 winners, 13 losers), a median of
ten a side, the first between minutes 4 and 10; they worked wrecks on their own ground, not the field a fight had
just left. Winners' income ran above what their extractors give in 12% of minutes, losers' in 7%. The report has a
`wrecks` line; `produce` armrectr makes them and they work the richest held field on their own; a constructor's
`reclaim` option appears when a field is within 1,800 of it.

**Aircraft.** An enemy aircraft plant in sight has meant gunships about ninety seconds later (human-9: the plant
seen at 2:16, the first Banshee at 3:54, six of them killed eight Blitzes at 8:22). What hits aircraft: the Nettle
turret, Whistlers, Crossbows.
