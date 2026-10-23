## Unit facts: sea

What floats or dives: warships of every class and submarines. From the game's unit definitions at commit 54199a0d3 (`run/unit_stats.py` into `crates/bot/data/units.json`; this document by `run/fact_units.py`). Metal and energy are the costs. Build time is in build units: a builder with build power P finishes the unit in build time / P seconds, and draws the costs over those seconds. Speed is elmos a second; range and sight are elmos; dps is damage a second against ground targets. The ratios are per metal of cost.

### Armada: sea (14 units)

<!--meta
id: F-units-sea-armada
layer: fact
ids: [F-unit-armaas, F-unit-armbats, F-unit-armcrus, F-unit-armdecade, F-unit-armepoch, F-unit-armlship, F-unit-armmship, F-unit-armpship, F-unit-armpt, F-unit-armroy, F-unit-armserp, F-unit-armsjam, F-unit-armsub, F-unit-armsubk]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | range | dps | dps vs air | sight | health per metal | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| armroy | Corsair | 1 | destroyer | armconsul/armmls/armsy | 880 | 6600 | 10200 | 3700 | 67.2 | 700 | 146.9 |  | 500 | 4.20 | 0.17 | sonar, floats |
| armpship | Ellysaw | 1 | frigate | armsy | 380 | 2550 | 4250 | 2150 | 81 | 470 | 116.7 | 61.7 | 500 | 5.66 | 0.31 | anti_air, floats |
| armpt | Skater | 1 | patrol boat | armmls/armsy | 150 | 1000 | 2700 | 780 | 93 | 300 | 65.5 | 70.8 | 750 | 5.20 | 0.44 | stealth, anti_air, sonar, floats |
| armdecade | Dolphin | 1 | raider ship | armmls/armsy | 175 | 1500 | 2450 | 970 | 104.7 | 280 | 135 | 30 | 375 | 5.54 | 0.77 | floats |
| armsub | Eel | 1 | submarine | armsy | 440 | 3100 | 5800 | 840 | 66 | 500 | 85.5 |  | 400 | 1.91 | 0.19 | submerged, sonar |
| armaas | Dragonslayer | 2 | anti-air ship | armasy | 1000 | 12000 | 21000 | 3900 | 63 | 840 | 3.9 | 430.2 | 475 | 3.90 | 0.00 | anti_air, floats |
| armbats | Dreadnought | 2 | battleship | armasy | 3300 | 22000 | 50000 | 9800 | 57.9 | 880 | 600 |  | 600 | 2.97 | 0.18 | floats |
| armcrus | Paladin | 2 | cruiser | armasy | 1000 | 12000 | 23000 | 5600 | 60 | 500 | 357 | 24 | 600 | 5.60 | 0.36 | needs_energy_to_fire, sonar, floats |
| armepoch | Epoch | 2 | flagship | armasy | 20000 | 190000 | 290000 | 50000 | 53.9 | 2450 | 1248.8 | 352.9 | 689 | 2.50 | 0.06 | floats |
| armsjam | Bermuda | 2 | jammer ship | armasy | 310 | 5000 | 20000 | 1350 | 45 |  |  |  | 390 | 4.35 |  | radar_jammer, floats |
| armmship | Longbow | 2 | missile ship | armasy | 2000 | 10000 | 24000 | 3350 | 63 | 1550 | 108.3 | 69 | 318.5 | 1.68 | 0.05 | anti_air, floats |
| armlship | Maelstrom | 2 | raider ship | armasy | 570 | 11400 | 15000 | 2200 | 99 | 400 | 250 | 69 | 500 | 3.86 | 0.44 | needs_energy_to_fire, floats |
| armsubk | Barracuda | 2 | submarine | armasy | 950 | 17000 | 28000 | 2350 | 81 | 400 | 175 |  | 390 | 2.47 | 0.18 | submerged, sonar |
| armserp | Serpent | 2 | submarine | armasy | 1800 | 27000 | 32000 | 3550 | 45 | 800 | 206.2 |  | 550 | 1.97 | 0.11 | submerged, stealth, sonar |

### Cortex: sea (14 units)

<!--meta
id: F-units-sea-cortex
layer: fact
ids: [F-unit-corarch, F-unit-corbats, F-unit-corblackhy, F-unit-corcrus, F-unit-coresupp, F-unit-corfship, F-unit-cormship, F-unit-corpship, F-unit-corpt, F-unit-corroy, F-unit-corshark, F-unit-corsjam, F-unit-corssub, F-unit-corsub]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | range | dps | dps vs air | sight | health per metal | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| corroy | Oppressor | 1 | destroyer | corfast/cormls/corsy | 960 | 7200 | 10600 | 3950 | 62.4 | 710 | 154 |  | 500 | 4.11 | 0.16 | sonar, floats |
| corpship | Riptide | 1 | frigate | corsy | 420 | 2800 | 4550 | 2350 | 82.5 | 480 | 120 | 38.8 | 500 | 5.60 | 0.29 | floats |
| corpt | Herring | 1 | patrol boat | cormls/corsy | 210 | 1400 | 2500 | 890 | 69 | 650 | 23.5 | 60 | 670 | 4.24 | 0.11 | anti_air, sonar, floats |
| coresupp | Supporter | 1 | raider ship | cormls/corsy | 100 | 1000 | 1400 | 480 | 109.8 | 280 | 91.1 | 13.3 | 600 | 4.80 | 0.91 | floats |
| corsub | Orca | 1 | submarine | corsy | 580 | 4000 | 7200 | 1060 | 57 | 500 | 136.8 |  | 400 | 1.83 | 0.24 | submerged, sonar |
| corarch | Arrow Storm | 2 | anti-air ship | corasy | 1000 | 12000 | 21000 | 3900 | 60.5 | 870 | 157.6 | 429 | 465 | 3.90 | 0.16 | anti_air, floats |
| corbats | Despot | 2 | battleship | corasy | 3400 | 23000 | 51000 | 12000 | 53.1 | 950 | 658.9 |  | 600 | 3.53 | 0.19 | needs_energy_to_fire, floats |
| corcrus | Buccaneer | 2 | cruiser | corasy | 1000 | 12000 | 23000 | 6200 | 54 | 500 | 406.8 | 45 | 600 | 6.20 | 0.41 | needs_energy_to_fire, sonar, floats |
| corblackhy | Black Hydra | 2 | flagship | corasy | 21000 | 190000 | 310000 | 53000 | 49.5 | 2550 | 1304.2 | 352.9 | 650 | 2.52 | 0.06 | needs_energy_to_fire, floats |
| corsjam | Phantasm | 2 | jammer ship | corasy | 280 | 4200 | 17000 | 1450 | 40 |  |  |  | 403 | 5.18 |  | radar_jammer, floats |
| cormship | Messenger | 2 | missile ship | corasy | 2000 | 10000 | 24000 | 3350 | 61.4 | 1550 | 137.5 | 69 | 318.5 | 1.68 | 0.07 | anti_air, floats |
| corfship | Brimstone | 2 | riot ship | corasy | 630 | 9400 | 13000 | 2700 | 93 | 450 | 375 | 69 | 500 | 4.29 | 0.60 | floats |
| corshark | Predator | 2 | submarine | corasy | 730 | 14000 | 23000 | 1610 | 84 | 400 | 159.1 |  | 390 | 2.21 | 0.22 | submerged, sonar |
| corssub | Kraken | 2 | submarine | corasy | 1900 | 29000 | 35000 | 3900 | 39 | 850 | 206.2 |  | 570 | 2.05 | 0.11 | submerged, stealth, sonar |

### Legion: sea (16 units)

<!--meta
id: F-units-sea-legion
layer: fact
ids: [F-unit-leganavyaaship, F-unit-leganavyantiswarm, F-unit-leganavyartyship, F-unit-leganavybattleship, F-unit-leganavybattlesub, F-unit-leganavycruiser, F-unit-leganavyflagship, F-unit-leganavyheavysub, F-unit-leganavymissileship, F-unit-leganavyradjamship, F-unit-legnavyaaship, F-unit-legnavyartyship, F-unit-legnavydestro, F-unit-legnavyfrigate, F-unit-legnavyscout, F-unit-legnavysub]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | range | dps | dps vs air | sight | health per metal | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| legnavyaaship | Iapetus | 1 | anti-air ship | leganavyengineer/legsy | 250 | 3600 | 4000 | 1200 | 60 | 760 | 5 | 120 | 750 | 4.80 | 0.02 | anti_air, floats |
| legnavydestro | Syracusia | 1 | destroyer | leganavyengineer/legsy | 875 | 9500 | 11500 | 3800 | 58 | 700 | 170.8 |  | 500 | 4.34 | 0.20 | needs_energy_to_fire, floats |
| legnavyfrigate | Argonaut | 1 | frigate | legaceb/legsy | 370 | 3700 | 4800 | 2600 | 72 | 460 | 215.4 |  | 500 | 7.03 | 0.58 | sonar, floats |
| legnavyscout | Hippocampus | 1 | patrol boat | leganavyengineer/legsy | 120 | 1600 | 1800 | 700 | 97 | 351 | 60 | 15 | 600 | 5.83 | 0.50 | floats |
| legnavysub | Ketea | 1 | submarine | legsy | 320 | 2400 | 4600 | 600 | 57 | 400 | 93.3 |  | 400 | 1.88 | 0.29 | submerged, sonar |
| legnavyartyship | Octeres | 1 | warship | legsy | 1450 | 15000 | 20000 | 4500 | 45 | 960 | 59.7 |  | 500 | 3.10 | 0.04 | floats |
| leganavyaaship | Notus | 2 | anti-air ship | legadvshipyard | 1000 | 12000 | 20100 | 3900 | 57 | 860 | 24 | 840 | 465 | 3.90 | 0.02 | anti_air, floats |
| leganavybattleship | Scylla | 2 | battleship | legadvshipyard/leggantuw | 2900 | 21000 | 49350 | 10350 | 25 | 750 | 875 |  | 600 | 3.57 | 0.30 | hover, floats |
| leganavycruiser | Thalassa | 2 | cruiser | legadvshipyard | 900 | 10800 | 20290 | 5600 | 57 | 500 | 360 |  | 600 | 6.22 | 0.40 | sonar, floats |
| leganavymissileship | Ultor | 2 | cruiser | legadvshipyard | 2000 | 10000 | 23000 | 3350 | 61.4 | 1550 | 300 | 30 | 318.5 | 1.68 | 0.15 | floats |
| leganavyflagship | Neptune | 2 | flagship | legadvshipyard | 16000 | 240000 | 269000 | 48000 | 63 | 1000 | 3046.7 | 1066.7 | 650 | 3.00 | 0.19 | needs_energy_to_fire, floats |
| leganavyradjamship | Dolus | 2 | jammer ship | legadvshipyard | 450 | 6000 | 19800 | 1700 | 42 |  |  |  | 600 | 3.78 |  | radar_jammer, floats |
| leganavybattlesub | Architeuthis | 2 | submarine | legadvshipyard | 1100 | 16000 | 25000 | 2350 | 72 | 600 | 180 |  | 450 | 2.14 | 0.16 | submerged, sonar |
| leganavyheavysub | Sphyrna | 2 | submarine | legadvshipyard | 1900 | 29000 | 34370 | 3900 | 39 | 850 | 206.2 |  | 570 | 2.05 | 0.11 | submerged, stealth, sonar |
| leganavyantiswarm | Leocampus | 2 | warship | legadvshipyard | 500 | 8500 | 11270 | 2300 | 105 | 350 | 190.9 | 60 | 500 | 4.60 | 0.38 | floats |
| leganavyartyship | Corinth | 2 | warship | legadvshipyard | 13000 | 125000 | 204750 | 22000 | 48 | 2100 | 434.5 |  | 689 | 1.69 | 0.03 | needs_energy_to_fire, floats |
