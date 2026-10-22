## Unit facts: air

What flies: fighters, bombers, gunships, radar and transport planes, drones. From the game's unit definitions at commit 54199a0d3 (`run/unit_stats.py` into `crates/bot/data/units.json`; this document by `run/fact_units.py`). Metal and energy are the costs. Build time is in build units: a builder with build power P finishes the unit in build time / P seconds, and draws the costs over those seconds. Speed is elmos a second; range and sight are elmos; dps is damage a second against ground targets. The ratios are per metal of cost.

### Armada: air (20 units)

<!--meta
id: F-units-air-armada
layer: fact
ids: [F-unit-armatlas, F-unit-armawac, F-unit-armblade, F-unit-armbrawl, F-unit-armdfly, F-unit-armfig, F-unit-armhawk, F-unit-armhvytrans, F-unit-armkam, F-unit-armlance, F-unit-armliche, F-unit-armpeep, F-unit-armpnix, F-unit-armsaber, F-unit-armsb, F-unit-armseap, F-unit-armsehak, F-unit-armsfig, F-unit-armstil, F-unit-armthund]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | range | dps | dps vs air | sight | health per metal | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| armthund | Stormbringer | 1 | bomber | armap | 150 | 4400 | 5000 | 670 | 250 | 1280 | 91.7 | 91.7 | 430 | 4.47 | 0.61 | flies, anti_air |
| armsb | Tsunami | 1 | bomber | armplat | 240 | 7500 | 10000 | 1170 | 267.3 | 1280 | 142.9 | 142.9 | 455 | 4.88 | 0.60 | flies, anti_air |
| armfig | Falcon | 1 | fighter aircraft | armap | 73 | 2900 | 3500 | 167 | 289.2 | 680 | 13.3 | 138.9 | 430 | 2.29 | 0.18 | flies, anti_air |
| armsfig | Cyclone | 1 | fighter aircraft | armplat | 90 | 4500 | 7000 | 220 | 310.8 | 710 | 14.4 | 252 | 430 | 2.44 | 0.16 | flies, anti_air |
| armkam | Banshee | 1 | gunship | armap | 135 | 2300 | 5050 | 560 | 184.8 | 350 | 38.6 |  | 520 | 4.15 | 0.29 | flies |
| armsaber | Sabre | 1 | gunship | armplat | 210 | 6400 | 11000 | 1010 | 156.9 | 425 | 66.7 |  | 595 | 4.81 | 0.32 | needs_energy_to_fire, flies |
| armpeep | Blink | 1 | radar plane | armap | 52 | 1550 | 2800 | 89 | 375 |  |  |  | 865 | 1.71 |  | flies |
| armsehak | Horizon | 1 | radar plane | armplat | 125 | 7000 | 11000 | 580 | 338.1 |  |  |  | 1100 | 4.64 |  | flies, sonar |
| armseap | Puffin | 1 | torpedo bomber | armplat | 290 | 6500 | 15000 | 1300 | 210 | 600 | 114 |  | 535 | 4.48 | 0.39 | flies, sonar |
| armatlas | Stork | 1 | transport aircraft | armap | 68 | 1300 | 3850 | 265 | 207 |  |  |  | 430 | 3.90 |  | transport, flies |
| armhvytrans | Osprey | 1 | transport aircraft | armap | 190 | 4000 | 8500 | 630 | 110 |  |  |  | 430 | 3.32 |  | transport, flies |
| armstil | Stiletto | 2 | EMP bomber | armaap | 460 | 38000 | 39000 | 1880 | 300 | 1280 |  |  | 430 | 4.09 |  | paralyzer, emp, stealth, flies |
| armdfly | Abductor | 2 | EMP transport aircraft | armaap | 320 | 10000 | 19000 | 1170 | 210 | 520 |  |  | 520 | 3.66 |  | paralyzer, emp, transport, stealth, needs_energy_to_fire, flies |
| armliche | Liche | 2 | atomic bomber | armaap | 2200 | 46000 | 73000 | 2300 | 294.9 | 250 | 625 |  | 455 | 1.05 | 0.28 | flies |
| armpnix | Blizzard | 2 | bomber | armaap | 230 | 12500 | 25000 | 1130 | 258 | 1280 | 190 | 190 | 430 | 4.91 | 0.83 | flies, anti_air |
| armhawk | Highwind | 2 | fighter aircraft | armaap | 150 | 6100 | 11500 | 370 | 358 | 740 | 17.1 | 535.7 | 430 | 2.47 | 0.11 | stealth, flies, anti_air |
| armbrawl | Roughneck | 2 | gunship | armaap | 310 | 6200 | 17000 | 1780 | 160.8 | 380 | 106.7 |  | 550 | 5.74 | 0.34 | flies |
| armblade | Hornet | 2 | gunship | armaap | 1250 | 22000 | 32000 | 3000 | 204 | 420 | 167.6 |  | 624 | 2.40 | 0.13 | flies |
| armawac | Oracle | 2 | radar plane | armaap | 175 | 8600 | 16000 | 890 | 317.4 |  |  |  | 1275 | 5.09 |  | flies, sonar |
| armlance | Cormorant | 2 | torpedo bomber | armaap | 400 | 8000 | 19000 | 1920 | 285 | 400 | 187.5 |  | 500 | 4.80 | 0.47 | flies, sonar |

### Cortex: air (18 units)

<!--meta
id: F-units-air-cortex
layer: fact
ids: [F-unit-corape, F-unit-corawac, F-unit-corbw, F-unit-corcrwh, F-unit-corcut, F-unit-corfink, F-unit-corhunt, F-unit-corhurc, F-unit-corhvytrans, F-unit-corsb, F-unit-corseah, F-unit-corseap, F-unit-corsfig, F-unit-corshad, F-unit-cortitan, F-unit-corvalk, F-unit-corvamp, F-unit-corveng]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | range | dps | dps vs air | sight | health per metal | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| corbw | Shuriken | 1 | EMP drone | corap | 58 | 1300 | 2070 | 83 | 280.5 | 220 |  |  | 430 | 1.43 |  | paralyzer, emp, flies |
| corshad | Whirlwind | 1 | bomber | corap | 150 | 4600 | 5050 | 680 | 234 | 1280 | 93.3 | 93.3 | 430 | 4.53 | 0.62 | flies, anti_air |
| corsb | Dam Buster | 1 | bomber | corplat | 200 | 8200 | 9000 | 940 | 240 | 1280 | 150 | 150 | 455 | 4.70 | 0.75 | flies, anti_air |
| corveng | Valiant | 1 | fighter aircraft | corap | 73 | 2800 | 3330 | 156 | 297.6 | 680 | 13.3 | 138.9 | 430 | 2.14 | 0.18 | flies, anti_air |
| corsfig | Bat | 1 | fighter aircraft | corplat | 90 | 4600 | 6500 | 220 | 315.6 | 710 | 14.4 | 240 | 430 | 2.44 | 0.16 | flies, anti_air |
| corcut | Cutlass | 1 | gunship | corplat | 230 | 5400 | 11500 | 1010 | 152.4 | 430 | 72.3 |  | 595 | 4.39 | 0.31 | flies |
| corfink | Finch | 1 | radar plane | corap | 51 | 1450 | 2400 | 100 | 360 |  |  |  | 835 | 1.96 |  | flies |
| corhunt | Watcher | 1 | radar plane | corplat | 130 | 6900 | 11500 | 730 | 324.3 |  |  |  | 1130 | 5.62 |  | flies, sonar |
| corseap | Monsoon | 1 | torpedo bomber | corplat | 230 | 5500 | 11000 | 1000 | 200 | 600 | 85.5 |  | 535 | 4.35 | 0.37 | flies, sonar |
| corvalk | Hercules | 1 | transport aircraft | corap | 74 | 1450 | 4120 | 280 | 198 |  |  |  | 430 | 3.78 |  | transport, flies |
| corhvytrans | Hephaestus | 1 | transport aircraft | corap | 190 | 4000 | 8500 | 800 | 100 |  |  |  | 430 | 4.21 |  | transport, flies |
| corhurc | Hailstorm | 2 | bomber | coraap | 310 | 18500 | 36000 | 1520 | 248.4 | 1280 | 237.9 | 237.9 | 430 | 4.90 | 0.77 | flies, anti_air |
| corvamp | Nighthawk | 2 | fighter aircraft | coraap | 145 | 5100 | 10500 | 290 | 379 | 690 | 24 | 500 | 430 | 2.00 | 0.17 | stealth, flies, anti_air |
| corape | Wasp | 2 | gunship | coraap | 370 | 6800 | 18000 | 1560 | 159 | 410 | 110.9 |  | 550 | 4.22 | 0.30 | flies |
| corcrwh | Dragon | 2 | gunship | coraap | 5100 | 72000 | 120000 | 16700 | 114.9 | 575 | 630 | 95 | 494 | 3.27 | 0.12 | needs_energy_to_fire, flies |
| corawac | Condor | 2 | radar plane | coraap | 180 | 8300 | 16000 | 990 | 321 |  |  |  | 1250 | 5.50 |  | flies, sonar |
| cortitan | Angler | 2 | torpedo bomber | coraap | 410 | 8200 | 18000 | 1960 | 279 | 400 | 150 |  | 500 | 4.78 | 0.37 | flies, sonar |
| corseah | Skyhook | 2 | transport aircraft | coraap | 370 | 9000 | 12500 | 2200 | 185 |  |  |  | 500 | 5.95 |  | transport, flies |

### Legion: air (24 units)

<!--meta
id: F-units-air-legion
layer: fact
ids: [F-unit-legafigdef, F-unit-legatorpbomber, F-unit-legatrans, F-unit-legcib, F-unit-legeallterrainmech, F-unit-legfhive, F-unit-legfig, F-unit-legfort, F-unit-leghive, F-unit-legkam, F-unit-leglts, F-unit-legmineb, F-unit-legmos, F-unit-legphoenix, F-unit-legspbomber, F-unit-legspcarrier, F-unit-legspfighter, F-unit-legspradarsonarplane, F-unit-legspsurfacegunship, F-unit-legsptorpgunship, F-unit-legstronghold, F-unit-legvcarry, F-unit-legvenator, F-unit-legwhisper]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | range | dps | dps vs air | sight | health per metal | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| legspbomber | Pyrphoros | 1 | bomber | legsplab | 240 | 7500 | 9890 | 1170 | 267.3 | 1280 | 150 | 150 | 455 | 4.88 | 0.62 | flies, anti_air |
| legspcarrier | Hecatoncheir | 1 | drone carrier aircraft | legsplab | 250 | 5000 | 9250 | 1500 | 93 |  |  |  | 600 | 6.00 |  | flies |
| legfhive | Naval Hive | 1 | drone hive | leganavyengineer | 350 | 9000 | 7500 | 1200 |  |  |  |  | 455 | 3.43 |  | submerged, on_water |
| leghive | Hive | 1 | drone hive | legca | 350 | 9000 | 6600 | 1200 |  |  |  |  | 455 | 3.43 |  |  |
| legfig | Noctua | 1 | fighter aircraft | legap | 40 | 1600 | 2000 | 220 | 270 | 600 | 6.7 | 120 | 700 | 5.50 | 0.17 | flies, anti_air |
| legspfighter | Astrapios | 1 | fighter aircraft | legsplab | 90 | 4500 | 6540 | 220 | 310.8 | 760 | 3.8 | 138.8 | 430 | 2.44 | 0.04 | flies, anti_air |
| legmos | Mosquito | 1 | gunship | legap | 110 | 2000 | 4000 | 415 | 165 | 380 |  |  | 520 | 3.77 |  | flies |
| legspsurfacegunship | Enyo | 1 | gunship | legsplab | 270 | 6700 | 12140 | 1200 | 148 | 400 | 57.1 |  | 595 | 4.44 | 0.21 | flies |
| legcib | Blindfold | 1 | juno drone | legap | 100 | 3500 | 4200 | 600 | 300 | 450 | 0.1 | 0.1 | 835 | 6.00 | 0.00 | flies, anti_air |
| legspradarsonarplane | Okeanos | 1 | radar plane | legsplab | 125 | 7000 | 10680 | 580 | 338.1 |  |  |  | 1100 | 4.64 |  | flies, sonar |
| legkam | Martyr | 1 | suicide drone | legap | 65 | 2000 | 2200 | 160 | 220 | 1280 | 46.3 | 2000.3 | 430 | 2.46 | 0.71 | flies, anti_air |
| legsptorpgunship | Ladon | 1 | torpedo bomber | legsplab | 190 | 5000 | 10720 | 900 | 273 | 460 | 62.2 |  | 535 | 4.74 | 0.33 | flies, sonar |
| leglts | Aeolus | 1 | transport aircraft | legap | 68 | 1300 | 3850 | 265 | 207 |  |  |  | 430 | 3.90 |  | transport, flies |
| legatrans | Hippotes | 1 | transport aircraft | legap | 190 | 4000 | 8500 | 800 | 100 |  |  |  | 430 | 4.21 |  | transport, flies |
| legfort | Tyrannus | 2 | assault gunship | legaap | 5600 | 79000 | 119750 | 16700 | 93 | 915 | 555.3 | 95 | 600 | 2.98 | 0.10 | flies |
| legphoenix | Phoenix | 2 | bomber | legaap | 450 | 25000 | 46600 | 2000 | 270 | 950 | 18.1 |  | 430 | 4.44 | 0.04 | needs_energy_to_fire, flies |
| legvcarry | Mantis | 2 | drone carrier vehicle | legavp | 450 | 9000 | 11550 | 1400 | 45 |  |  |  | 400 | 3.11 |  |  |
| legvenator | Venator | 2 | fighter aircraft | legaap | 110 | 3200 | 9730 | 210 | 450 | 700 | 28.8 | 576 | 430 | 1.91 | 0.26 | stealth, flies, anti_air |
| legafigdef | Ajax | 2 | fighter aircraft | legaap | 180 | 6200 | 11850 | 480 | 270 | 600 | 20 | 800 | 430 | 2.67 | 0.11 | flies, anti_air |
| legmineb | Harbinger | 2 | minelayer bomber | legaap | 300 | 21000 | 30550 | 1110 | 210 | 1600 |  |  | 455 | 3.70 |  | needs_energy_to_fire, flies |
| legwhisper | Whisper | 2 | radar plane | legaap | 210 | 9000 | 16480 | 990 | 321 |  |  |  | 1250 | 4.71 |  | stealth, flies, sonar |
| legatorpbomber | Aesacus | 2 | torpedo bomber | legaap | 480 | 9480 | 21970 | 1900 | 290 | 400 | 214.3 |  | 500 | 3.96 | 0.45 | flies, sonar |
| legstronghold | Stronghold | 2 | transport gunship | legaap | 550 | 13000 | 24200 | 2600 | 160 | 450 | 150 | 60 | 430 | 4.73 | 0.27 | transport, flies |
| legeallterrainmech | Myrmidon | 3 | drone carrier mech | leggant | 3950 | 78000 | 120250 | 9200 | 33 | 1100 | 300 | 55.5 | 625 | 2.33 | 0.08 | gantry |
