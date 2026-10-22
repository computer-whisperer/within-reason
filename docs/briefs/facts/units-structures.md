## Unit facts: structures

What stands still and fights, sees or hides: defence and anti-air turrets, artillery and long-range cannon, missile launchers and silos, anti-nuke, shields, walls, radar, jammers, cameras, sonar, mines. From the game's unit definitions at commit 54199a0d3 (`run/unit_stats.py` into `crates/bot/data/units.json`; this document by `run/fact_units.py`). Metal and energy are the costs. Build time is in build units: a builder with build power P finishes the unit in build time / P seconds, and draws the costs over those seconds. Speed is elmos a second; range and sight are elmos; dps is damage a second against ground targets. The ratios are per metal of cost.

### Armada: structures (44 units)

<!--meta
id: F-units-structures-armada
layer: fact
ids: [F-unit-armamb, F-unit-armamd, F-unit-armanni, F-unit-armarad, F-unit-armason, F-unit-armatl, F-unit-armbeamer, F-unit-armcir, F-unit-armclaw, F-unit-armdf, F-unit-armdl, F-unit-armdrag, F-unit-armemp, F-unit-armeyes, F-unit-armfatf, F-unit-armfdrag, F-unit-armferret, F-unit-armfflak, F-unit-armfhlt, F-unit-armflak, F-unit-armfmine3, F-unit-armfort, F-unit-armfrad, F-unit-armfrt, F-unit-armgate, F-unit-armguard, F-unit-armhlt, F-unit-armjamt, F-unit-armjuno, F-unit-armkraken, F-unit-armllt, F-unit-armmercury, F-unit-armmine1, F-unit-armmine2, F-unit-armmine3, F-unit-armmlv, F-unit-armpb, F-unit-armrad, F-unit-armrl, F-unit-armsd, F-unit-armsilo, F-unit-armtarg, F-unit-armtl, F-unit-armveil]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | metal | energy | build time | health | range | dps | dps vs air | sight | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| armfrt | Naval Nettle | 1 | anti-air turret | 80 | 900 | 1800 | 330 | 765 | 0.6 | 67.6 | 455 | 0.01 | submerged, anti_air, on_water |
| armrl | Nettle | 1 | anti-air turret | 80 | 900 | 1800 | 330 | 765 | 0.6 | 67.6 | 455 | 0.01 | anti_air |
| armferret | Ferret | 1 | anti-air turret | 330 | 5700 | 5000 | 1600 | 950 | 1.7 | 208.3 | 375 | 0.01 | stealth, anti_air |
| armcir | Chainsaw | 1 | anti-air turret | 750 | 7300 | 12000 | 4450 | 1200 | 2.5 | 625 | 380 | 0.00 | anti_air |
| armguard | Gauntlet | 1 | artillery turret | 1250 | 12500 | 19300 | 3050 | 1220 | 105.9 |  | 455 | 0.08 |  |
| armeyes | Beholder | 1 | camera tower | 32 | 850 | 750 | 280 |  |  |  | 560 |  | stealth |
| armllt | Sentry | 1 | defence turret | 85 | 680 | 2400 | 620 | 430 | 160.7 | 10.7 | 494 | 1.89 | needs_energy_to_fire |
| armbeamer | Beamer | 1 | defence turret | 190 | 1500 | 4800 | 1430 | 490 | 266 | 20 | 475 | 1.40 | needs_energy_to_fire |
| armclaw | Dragon's Claw | 1 | defence turret | 340 | 1600 | 4650 | 1330 | 430 | 158.8 |  | 440 | 0.47 | stealth, needs_energy_to_fire |
| armhlt | Overwatch | 1 | defence turret | 440 | 4700 | 11300 | 2600 | 620 | 215 | 19.4 | 494 | 0.49 | needs_energy_to_fire |
| armfhlt | Manta | 1 | defence turret | 470 | 6200 | 9670 | 4250 | 630 | 233.3 | 57.8 | 630 | 0.50 | submerged, needs_energy_to_fire, on_water |
| armjamt | Sneaky Pete | 1 | jammer tower | 240 | 8500 | 9950 | 790 |  |  |  | 195 |  | radar_jammer |
| armjuno | Juno | 1 | juno launcher | 640 | 17000 | 21800 | 2350 |  |  |  | 494 |  | stealth |
| armmine1 | Light Mine | 1 | mine | 7 | 270 | 100 | 11 |  |  |  | 83.2 |  | stealth |
| armmine2 | Medium Mine | 1 | mine | 25 | 1050 | 300 | 11 |  |  |  | 83.2 |  | stealth |
| armmine3 | Heavy Mine | 1 | mine | 50 | 2800 | 700 | 11 |  |  |  | 83.2 |  | stealth |
| armmlv | Groundhog | 1 | minelayer vehicle | 57 | 1100 | 3520 | 335 | 220 | 5 |  | 450 | 0.09 | stealth |
| armfrad | Naval Radar / Sonar Tower | 1 | radar and sonar tower | 130 | 1000 | 1800 | 110 |  |  |  | 760 |  | submerged, sonar, on_water |
| armrad | Radar Tower | 1 | radar tower | 60 | 630 | 1140 | 180 |  |  |  | 680 |  |  |
| armtl | Harpoon | 1 | torpedo launcher | 170 | 1500 | 3700 | 1300 | 500 | 186.7 |  | 494 | 1.10 | submerged, sonar, on_water |
| armdl | Anemone | 1 | torpedo launcher | 240 | 2200 | 6000 | 2200 | 600 | 150 |  | 617 | 0.62 | sonar |
| armdrag | Dragon's Teeth | 1 | wall | 8 | 0 | 255 | 2800 |  |  |  | 1 |  |  |
| armfdrag | Shark's Teeth | 1 | wall | 15 | 0 | 370 | 4450 |  |  |  | 1 |  | submerged, on_water |
| armemp | Paralyzer | 2 | EMP missile launcher | 1600 | 29000 | 79300 | 3350 |  |  |  | 455 |  | emp |
| armfflak | Naval Arbalest | 2 | anti-air turret | 820 | 13000 | 19000 | 1750 | 850 | 2 | 500 | 525 | 0.00 | submerged, anti_air, on_water |
| armflak | Arbalest | 2 | anti-air turret | 820 | 13000 | 19000 | 1750 | 850 | 2 | 500 | 525 | 0.00 | anti_air |
| armmercury | Mercury | 2 | anti-air turret | 1600 | 33000 | 28000 | 1670 | 2400 |  |  | 600 |  | needs_energy_to_fire |
| armamd | Citadel | 2 | anti-nuke | 1500 | 38000 | 60000 | 3300 |  |  |  | 201 |  | antinuke |
| armamb | Rattlesnake | 2 | artillery turret | 2500 | 17000 | 27000 | 4000 | 1380 | 194.4 |  | 442 | 0.08 |  |
| armdf | Decoy Fusion Reactor | 2 | decoy building | 270 | 100 | 7500 | 3800 |  |  |  | 210 |  |  |
| armpb | Pit Bull | 2 | defence turret | 680 | 14000 | 15000 | 2800 | 730 | 421.9 |  | 598 | 0.62 |  |
| armkraken | Gorgon | 2 | defence turret | 1000 | 13000 | 20000 | 4450 | 800 | 540 |  | 700 | 0.54 | submerged, on_water |
| armanni | Pulsar | 2 | defence turret | 3500 | 74000 | 52000 | 6100 | 1400 | 1090.9 |  | 780 | 0.31 | needs_energy_to_fire |
| armveil | Veil | 2 | jammer tower | 125 | 19000 | 9100 | 830 |  |  |  | 155 |  | radar_jammer |
| armfmine3 | Heavy Mine | 2 | mine | 32 | 1600 | 400 | 11 |  |  |  | 78 |  | stealth |
| armsilo | Armageddon | 2 | nuclear silo | 8100 | 90000 | 178500 | 5900 |  |  |  | 455 |  | nuke |
| armarad | Advanced Radar Tower | 2 | radar tower | 400 | 14000 | 8000 | 500 |  |  |  | 1000 |  |  |
| armgate | Keeper | 2 | shield generator | 3000 | 54000 | 55000 | 3550 |  |  |  | 273 |  |  |
| armason | Advanced Sonar Station | 2 | sonar station | 170 | 2600 | 6150 | 2350 |  |  |  | 215 |  | submerged, sonar, on_water |
| armsd | Tracer | 2 | targeting facility | 710 | 7100 | 11900 | 2650 |  |  |  | 240 |  |  |
| armfatf | Naval Pinpointer | 2 | targeting facility | 800 | 7300 | 9880 | 1610 |  |  |  | 273 |  | submerged, on_water |
| armtarg | Pinpointer | 2 | targeting facility | 810 | 7200 | 8700 | 2100 |  |  |  | 273 |  |  |
| armatl | Moray | 2 | torpedo launcher | 1000 | 8500 | 9260 | 2800 | 915 | 360 |  | 585 | 0.36 | submerged, sonar, on_water |
| armfort | Fortification Wall | 2 | wall | 40 | 500 | 900 | 8900 |  |  |  | 1 |  |  |

### Cortex: structures (43 units)

<!--meta
id: F-units-structures-cortex
layer: fact
ids: [F-unit-corarad, F-unit-corason, F-unit-coratl, F-unit-cordl, F-unit-cordoom, F-unit-cordrag, F-unit-corenaa, F-unit-corerad, F-unit-coreyes, F-unit-corfatf, F-unit-corfdoom, F-unit-corfdrag, F-unit-corfhlt, F-unit-corflak, F-unit-corfmd, F-unit-corfmine3, F-unit-corfort, F-unit-corfrad, F-unit-corfrt, F-unit-corgate, F-unit-corhllt, F-unit-corhlt, F-unit-corjamt, F-unit-corjuno, F-unit-corllt, F-unit-cormadsam, F-unit-cormaw, F-unit-cormine1, F-unit-cormine2, F-unit-cormine3, F-unit-cormlv, F-unit-corpun, F-unit-corrad, F-unit-corrl, F-unit-corscreamer, F-unit-corsd, F-unit-corshroud, F-unit-corsilo, F-unit-cortarg, F-unit-cortl, F-unit-cortoast, F-unit-cortron, F-unit-corvipe]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | metal | energy | build time | health | range | dps | dps vs air | sight | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| corfrt | Slingshot | 1 | anti-air turret | 80 | 850 | 1750 | 335 | 765 | 0.6 | 67.6 | 455 | 0.01 | submerged, anti_air, on_water |
| corrl | Thistle | 1 | anti-air turret | 80 | 850 | 1750 | 335 | 765 | 0.6 | 67.6 | 455 | 0.01 | anti_air |
| cormadsam | SAM | 1 | anti-air turret | 350 | 5500 | 4500 | 2500 | 950 | 2.5 | 225 | 375 | 0.01 | anti_air |
| corerad | Eradicator | 1 | anti-air turret | 800 | 7900 | 12000 | 4450 | 1200 | 2.5 | 625 | 380 | 0.00 | anti_air |
| corpun | Agitator | 1 | artillery turret | 1300 | 13500 | 17400 | 3250 | 1245 | 110.5 |  | 455 | 0.09 |  |
| coreyes | Beholder | 1 | camera tower | 32 | 850 | 750 | 280 |  |  |  | 540 |  | stealth |
| corllt | Guard | 1 | defence turret | 90 | 700 | 2500 | 650 | 435 | 160.7 | 10.7 | 494 | 1.79 | needs_energy_to_fire |
| corhllt | Twin Guard | 1 | defence turret | 195 | 1750 | 4900 | 1670 | 490 | 321.4 | 21.4 | 475 | 1.65 | needs_energy_to_fire |
| cormaw | Dragon's Maw | 1 | defence turret | 290 | 1550 | 4420 | 1610 | 410 | 320 |  | 422 | 1.10 | stealth |
| corfhlt | Coral | 1 | defence turret | 480 | 6200 | 11400 | 4350 | 630 | 231 | 52 | 630 | 0.48 | submerged, needs_energy_to_fire, on_water |
| corhlt | Warden | 1 | defence turret | 480 | 4700 | 11400 | 2750 | 620 | 217.5 | 29.2 | 455 | 0.45 | needs_energy_to_fire |
| corjamt | Castro | 1 | jammer tower | 100 | 4500 | 4000 | 1070 |  |  |  | 104 |  | radar_jammer |
| corjuno | Juno | 1 | juno launcher | 660 | 16000 | 27700 | 2500 |  |  |  | 494 |  | stealth |
| cormine1 | Light Mine | 1 | mine | 7 | 270 | 100 | 11 |  |  |  | 83.2 |  | stealth |
| cormine2 | Medium Mine | 1 | mine | 25 | 1050 | 300 | 11 |  |  |  | 83.2 |  | stealth |
| cormine3 | Heavy Mine | 1 | mine | 50 | 2800 | 700 | 11 |  |  |  | 83.2 |  | stealth |
| cormlv | Trapper | 1 | minelayer vehicle | 65 | 1300 | 3640 | 450 | 220 | 5 |  | 450 | 0.08 | stealth |
| corfrad | Naval Radar / Sonar Tower | 1 | radar and sonar tower | 130 | 1000 | 1800 | 114 |  |  |  | 740 |  | submerged, sonar, on_water |
| corrad | Radar Tower | 1 | radar tower | 60 | 630 | 1140 | 180 |  |  |  | 680 |  |  |
| cortl | Urchin | 1 | torpedo launcher | 170 | 1800 | 3750 | 1440 | 500 | 186.7 |  | 495 | 1.10 | submerged, sonar, on_water |
| cordl | Jellyfish | 1 | torpedo launcher | 250 | 2200 | 6200 | 2350 | 600 | 150 |  | 611 | 0.60 | sonar |
| cordrag | Dragon's Teeth | 1 | wall | 8 | 0 | 255 | 2800 |  |  |  | 1 |  |  |
| corfdrag | Shark's Teeth | 1 | wall | 15 | 0 | 400 | 4450 |  |  |  | 1 |  | submerged, on_water |
| corenaa | Naval Birdshot | 2 | anti-air turret | 850 | 14000 | 20000 | 1840 | 850 | 2 | 500 | 525 | 0.00 | submerged, anti_air, on_water |
| corflak | Birdshot | 2 | anti-air turret | 850 | 14000 | 20000 | 1840 | 850 | 2 | 500 | 525 | 0.00 | anti_air |
| corscreamer | Screamer | 2 | anti-air turret | 1650 | 32000 | 28000 | 1670 | 2400 |  |  | 600 |  | needs_energy_to_fire |
| corfmd | Prevailer | 2 | anti-nuke | 1500 | 40000 | 60000 | 3300 |  |  |  | 195 |  | antinuke |
| cortoast | Persecutor | 2 | artillery turret | 2500 | 17000 | 25700 | 4250 | 1390 | 200 |  | 416 | 0.08 |  |
| corvipe | Scorpion | 2 | defence turret | 730 | 14000 | 15000 | 3000 | 730 | 412.5 |  | 546 | 0.57 |  |
| corfdoom | Devastator | 2 | defence turret | 1200 | 15000 | 24000 | 6700 | 900 | 731 | 352 | 780 | 0.61 | submerged, needs_energy_to_fire, on_water |
| cordoom | Bulwark | 2 | defence turret | 3000 | 37000 | 55200 | 9400 | 950 | 1513.1 | 421.9 | 780 | 0.50 | needs_energy_to_fire |
| corshroud | Shroud | 2 | jammer tower | 130 | 20000 | 9400 | 890 |  |  |  | 155 |  | radar_jammer |
| corfmine3 | Heavy Mine | 2 | mine | 32 | 1600 | 400 | 11 |  |  |  | 71.5 |  | stealth |
| corsilo | Apocalypse | 2 | nuclear silo | 7700 | 82000 | 181000 | 6200 |  |  |  | 455 |  | nuke |
| corarad | Advanced Radar Tower | 2 | radar tower | 400 | 14000 | 8000 | 500 |  |  |  | 1000 |  |  |
| corgate | Overseer | 2 | shield generator | 3200 | 55000 | 55000 | 3550 |  |  |  | 273 |  |  |
| corason | Advanced Sonar Station | 2 | sonar station | 160 | 2400 | 6100 | 2400 |  |  |  | 210 |  | submerged, sonar, on_water |
| cortron | Catalyst | 2 | tactical missile launcher | 1200 | 14000 | 59000 | 3550 |  |  |  | 455 |  |  |
| corsd | Nemesis | 2 | targeting facility | 750 | 6700 | 12000 | 2800 |  |  |  | 225 |  |  |
| corfatf | Naval Pinpointer | 2 | targeting facility | 800 | 7500 | 10300 | 1530 |  |  |  | 273 |  | submerged, on_water |
| cortarg | Pinpointer | 2 | targeting facility | 800 | 7500 | 10900 | 2000 |  |  |  | 273 |  |  |
| coratl | Lamprey | 2 | torpedo launcher | 1050 | 8500 | 10900 | 2800 | 890 | 362.6 |  | 585 | 0.35 | submerged, sonar, on_water |
| corfort | Fortification Wall | 2 | wall | 40 | 500 | 900 | 8900 |  |  |  | 1 |  |  |

### Legion: structures (41 units)

<!--meta
id: F-units-structures-legion
layer: fact
ids: [F-unit-legabm, F-unit-legacluster, F-unit-legajam, F-unit-leganavalaaturret, F-unit-leganavaldefturret, F-unit-leganavalpinpointer, F-unit-leganavalsonarstation, F-unit-leganavaltorpturret, F-unit-legapopupdef, F-unit-legarad, F-unit-legbastion, F-unit-legcluster, F-unit-legctl, F-unit-legdeflector, F-unit-legdrag, F-unit-legdtr, F-unit-legeyes, F-unit-legfdrag, F-unit-legflak, F-unit-legfmg, F-unit-legforti, F-unit-legfrad, F-unit-legfrl, F-unit-legjam, F-unit-legjuno, F-unit-leglht, F-unit-leglraa, F-unit-leglupara, F-unit-legmg, F-unit-legmine1, F-unit-legmine2, F-unit-legmine3, F-unit-legmlv, F-unit-legperdition, F-unit-legrad, F-unit-legrhapsis, F-unit-legrl, F-unit-legsd, F-unit-legsilo, F-unit-legtarg, F-unit-legtl]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | metal | energy | build time | health | range | dps | dps vs air | sight | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| legfrl | Polybolos | 1 | anti-air turret | 80 | 850 | 1750 | 335 | 765 | 0.6 | 67.6 | 455 | 0.01 | submerged, anti_air, on_water |
| legrl | Bramble | 1 | anti-air turret | 80 | 850 | 1750 | 335 | 765 | 0.6 | 67.6 | 455 | 0.01 | anti_air |
| legrhapsis | Rhapsis | 1 | anti-air turret | 280 | 6500 | 4700 | 1900 | 950 | 12 | 180 | 375 | 0.04 | anti_air |
| leglupara | Lupara | 1 | anti-air turret | 900 | 8200 | 12500 | 4000 | 1125 | 1.1 | 283.3 | 380 | 0.00 | anti_air |
| legcluster | Amputator | 1 | artillery turret | 850 | 11000 | 17400 | 2900 | 1000 | 50 |  | 455 | 0.06 |  |
| legeyes | Argus | 1 | camera tower | 32 | 850 | 750 | 280 |  |  |  | 540 |  | stealth |
| leglht | Pharos | 1 | defence turret | 70 | 550 | 2200 | 550 | 430 | 181.8 | 22.7 | 494 | 2.60 |  |
| legdtr | Dragon's Jaw | 1 | defence turret | 290 | 1550 | 4420 | 1610 | 400 | 120 |  | 440 | 0.41 | stealth |
| legmg | Cacophony | 1 | defence turret | 380 | 5500 | 7600 | 2200 | 575 | 255 | 255 | 525 | 0.67 | anti_air |
| legfmg | Gelasma | 1 | defence turret | 450 | 5500 | 7600 | 3900 | 575 | 270 | 270 | 525 | 0.60 | submerged, anti_air, on_water |
| legjam | Nyx | 1 | jammer tower | 140 | 7200 | 4570 | 1070 |  |  |  | 104 |  | radar_jammer |
| legjuno | Juno | 1 | juno launcher | 660 | 16000 | 27700 | 2500 |  |  |  | 494 |  | stealth |
| legmine1 | Light Mine | 1 | mine | 7 | 270 | 100 | 11 |  |  |  | 83.2 |  | stealth |
| legmine2 | Medium Mine | 1 | mine | 25 | 1050 | 300 | 11 |  |  |  | 83.2 |  | stealth |
| legmine3 | Heavy Mine | 1 | mine | 50 | 2800 | 700 | 11 |  |  |  | 83.2 |  | stealth |
| legmlv | Sapper | 1 | minelayer vehicle | 52 | 900 | 3640 | 210 | 220 | 5 |  | 450 | 0.10 | stealth |
| legfrad | Naval Radar / Sonar Tower | 1 | radar and sonar tower | 130 | 1000 | 1800 | 110 |  |  |  | 760 |  | submerged, sonar, on_water |
| legrad | Radar Tower | 1 | radar tower | 60 | 630 | 1140 | 180 |  |  |  | 680 |  |  |
| legtl | Stheno | 1 | torpedo launcher | 170 | 1500 | 3700 | 1300 | 500 | 186.7 |  | 494 | 1.10 | submerged, sonar, on_water |
| legctl | Euryale | 1 | torpedo launcher | 240 | 2200 | 5900 | 2200 | 600 | 150 |  | 617 | 0.62 | sonar |
| legdrag | Dragon's Teeth | 1 | wall | 8 | 0 | 255 | 2800 |  |  |  | 1 |  |  |
| legfdrag | Shark's Teeth | 1 | wall | 15 | 0 | 400 | 4450 |  |  |  | 1 |  | submerged, on_water |
| leganavalaaturret | Fulmen | 2 | anti-air turret | 820 | 13000 | 19000 | 1750 | 875 | 18 | 1044 | 525 | 0.02 | submerged, anti_air, on_water |
| legflak | Pluto | 2 | anti-air turret | 820 | 13000 | 19000 | 1750 | 875 | 18 | 1044 | 525 | 0.02 | anti_air |
| leglraa | Xyston | 2 | anti-air turret | 1600 | 36000 | 28000 | 1670 | 2000 | 0.4 | 280 | 600 | 0.00 | needs_energy_to_fire, anti_air |
| legabm | Aegis | 2 | anti-nuke | 1500 | 40000 | 60000 | 3650 |  |  |  | 195 |  | antinuke |
| legacluster | Eviscerator | 2 | artillery turret | 2300 | 15000 | 25700 | 3700 | 1380 | 118.3 |  | 455 | 0.05 |  |
| legapopupdef | Chimera | 2 | defence turret | 750 | 15000 | 17500 | 2800 | 730 | 464.7 |  | 598 | 0.62 |  |
| leganavaldefturret | Ionia | 2 | defence turret | 1250 | 16500 | 24000 | 6300 | 700 | 735 |  | 630 | 0.59 | submerged, on_water |
| legbastion | Bastion | 2 | defence turret | 4200 | 58000 | 79000 | 12000 | 1100 | 51.7 |  | 780 | 0.01 | needs_energy_to_fire |
| legajam | Erebus | 2 | jammer tower | 125 | 19000 | 9100 | 830 |  |  |  | 155 |  | radar_jammer |
| legperdition | Perdition | 2 | napalm missile launcher | 1250 | 15000 | 62000 | 4000 |  |  |  | 500 |  |  |
| legsilo | Supernova | 2 | nuclear silo | 7700 | 82000 | 181000 | 6200 |  |  |  | 455 |  | nuke |
| legarad | Advanced Radar Tower | 2 | radar tower | 400 | 14000 | 8000 | 500 |  |  |  | 1000 |  |  |
| legdeflector | Soteria | 2 | shield generator | 3200 | 55000 | 55000 | 3550 |  |  |  | 273 |  |  |
| leganavalsonarstation | Auscultor | 2 | sonar station | 160 | 2400 | 6100 | 2400 |  |  |  | 210 |  | submerged, sonar, on_water |
| legsd | Ichnaea | 2 | targeting facility | 710 | 7100 | 11900 | 2650 |  |  |  | 240 |  |  |
| leganavalpinpointer | Naval Pinpointer | 2 | targeting facility | 800 | 7500 | 10300 | 1530 |  |  |  | 273 |  | submerged, on_water |
| legtarg | Pinpointer | 2 | targeting facility | 810 | 7200 | 8700 | 2100 |  |  |  | 273 |  |  |
| leganavaltorpturret | Delphinus | 2 | torpedo launcher | 1050 | 8500 | 10900 | 2800 | 890 | 362.6 |  | 585 | 0.35 | submerged, sonar, on_water |
| legforti | Fortification Wall | 2 | wall | 40 | 500 | 1000 | 8900 |  |  |  | 1 |  |  |
