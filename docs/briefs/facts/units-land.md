## Unit facts: land

The land fighters: bots, tanks, mechs, hovercraft, and the mobile support beside them (radar, jammers, minelayers, mines and bombs are under structures). From the game's unit definitions at commit 54199a0d3 (`run/unit_stats.py` into `crates/bot/data/units.json`; this document by `run/fact_units.py`). Metal and energy are the costs. Build time is in build units: a builder with build power P finishes the unit in build time / P seconds, and draws the costs over those seconds. Speed is elmos a second; range and sight are elmos; dps is damage a second against ground targets. The ratios are per metal of cost.

### Armada: land (51 units)

<!--meta
id: F-units-land-armada
layer: fact
ids: [F-unit-armaak, F-unit-armah, F-unit-armamph, F-unit-armanac, F-unit-armantiship, F-unit-armart, F-unit-armaser, F-unit-armbanth, F-unit-armbrtha, F-unit-armbull, F-unit-armcroc, F-unit-armfast, F-unit-armfav, F-unit-armfboy, F-unit-armfido, F-unit-armflash, F-unit-armflea, F-unit-armgremlin, F-unit-armham, F-unit-armjam, F-unit-armjanus, F-unit-armjeth, F-unit-armlatnk, F-unit-armlun, F-unit-armmanni, F-unit-armmar, F-unit-armmark, F-unit-armmart, F-unit-armmav, F-unit-armmerl, F-unit-armmh, F-unit-armpincer, F-unit-armpw, F-unit-armraz, F-unit-armrock, F-unit-armsam, F-unit-armscab, F-unit-armseer, F-unit-armsh, F-unit-armsnipe, F-unit-armspid, F-unit-armsptk, F-unit-armspy, F-unit-armstump, F-unit-armthor, F-unit-armvader, F-unit-armvang, F-unit-armvulc, F-unit-armwar, F-unit-armyork, F-unit-armzeus]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | range | dps | dps vs air | sight | health per metal | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| armjeth | Crossbow | 1 | anti-air bot | armamsub | 125 | 1100 | 1830 | 630 | 56.2 | 760 | 0.5 | 55 | 380 | 5.04 | 0.00 | amphibious, anti_air |
| armah | Sweeper | 1 | anti-air hovercraft | armfhp | 230 | 4500 | 3560 | 1070 | 88.5 | 700 | 1.4 | 84.5 | 509 | 4.65 | 0.01 | hover, anti_air, sonar, floats |
| armmh | Possum | 1 | artillery hovercraft | armfhp | 175 | 3000 | 3300 | 530 | 63.9 | 710 | 50 |  | 509 | 3.03 | 0.29 | hover, floats |
| armart | Shellshocker | 1 | artillery tank | armvp | 135 | 2200 | 3000 | 620 | 54 | 710 | 42.3 |  | 364 | 4.59 | 0.31 |  |
| armham | Mace | 1 | line bot | armlab | 130 | 1300 | 2200 | 1000 | 46.2 | 380 | 60 | 12.1 | 380 | 7.69 | 0.46 |  |
| armanac | Crocodile | 1 | line hovercraft | armfhp | 270 | 2400 | 3000 | 1530 | 70 | 370 | 70 | 20.7 | 509 | 5.67 | 0.26 | hover, floats |
| armpincer | Pincer | 1 | line tank | armamsub | 200 | 2000 | 2610 | 1340 | 63 | 305 | 77.3 | 16.7 | 500 | 6.70 | 0.39 | amphibious, sonar |
| armstump | Stout | 1 | line tank | armvp | 225 | 2100 | 3400 | 1800 | 75 | 350 | 80.8 | 15 | 350 | 8.00 | 0.36 |  |
| armpw | Pawn | 1 | raider bot | armconsul | 54 | 900 | 1650 | 370 | 87 | 180 | 90 | 20 | 429 | 6.85 | 1.67 |  |
| armsh | Seeker | 1 | raider hovercraft | armfhp | 85 | 1250 | 1700 | 290 | 113.7 | 230 | 60 | 11.7 | 550 | 3.41 | 0.71 | hover, floats |
| armflash | Blitz | 1 | raider tank | armvp | 110 | 900 | 2000 | 730 | 101 | 180 | 90 | 20 | 350 | 6.64 | 0.82 |  |
| armwar | Centurion | 1 | riot bot | armlab | 270 | 3100 | 4200 | 1590 | 45 | 325 | 183.3 | 30 | 380 | 5.89 | 0.68 |  |
| armjanus | Janus | 1 | riot tank | armvp | 240 | 2600 | 3550 | 1030 | 54 | 380 | 88 |  | 330 | 4.29 | 0.37 |  |
| armflea | Tick | 1 | scout bot | armlab | 21 | 300 | 800 | 60 | 132 | 140 | 40 | 3.3 | 600 | 2.86 | 1.90 |  |
| armfav | Rover | 1 | scout vehicle | armvp | 31 | 370 | 1100 | 105 | 168 | 180 | 35 | 2 | 635 | 3.39 | 1.13 |  |
| armrock | Rocketeer | 1 | skirmisher bot | armlab | 120 | 1000 | 2010 | 720 | 50.7 | 475 | 41.3 | 41.3 | 380 | 6.00 | 0.34 | anti_air |
| armsam | Whistler | 1 | skirmisher tank | armvp | 150 | 2100 | 3420 | 820 | 55 | 550 | 25.8 | 48 | 620 | 5.47 | 0.17 | anti_air |
| armspid | Webber | 2 | EMP bot | armalab | 250 | 5000 | 7000 | 1110 | 71.5 | 280 |  |  | 550 | 4.44 |  | paralyzer, emp, stealth |
| armamph | Platypus | 2 | amphibious assault bot | armalab | 260 | 2700 | 7000 | 1170 | 90 | 275 | 109.1 | 63.3 | 380 | 4.50 | 0.42 | needs_energy_to_fire, amphibious, anti_air |
| armaak | Archangel | 2 | anti-air bot | armalab | 520 | 5600 | 10000 | 2200 | 47.4 | 1200 | 2.2 | 258.2 | 400 | 4.23 | 0.00 | amphibious, anti_air |
| armyork | Shredder | 2 | anti-air vehicle | armavp | 450 | 9100 | 13000 | 2600 | 70.5 | 775 | 1.4 | 272.7 | 390 | 5.78 | 0.00 | anti_air |
| armfboy | Fatboy | 2 | artillery bot | armalab | 1400 | 20000 | 32000 | 7800 | 30 | 700 | 114.3 |  | 510 | 5.57 | 0.08 |  |
| armmart | Mauser | 2 | artillery tank | armavp | 320 | 4900 | 8500 | 1070 | 60 | 820 | 85.7 |  | 330 | 3.34 | 0.27 |  |
| armmerl | Ambassador | 2 | artillery tank | armavp | 920 | 6500 | 21000 | 1220 | 33 | 1300 | 105.6 |  | 330 | 1.33 | 0.11 | stealth |
| armzeus | Welder | 2 | assault bot | armalab | 350 | 6100 | 8000 | 3500 | 48 | 280 | 129.4 |  | 380 | 10.00 | 0.37 | needs_energy_to_fire |
| armcroc | Turtle | 2 | assault tank | armamsub | 750 | 15000 | 21000 | 5000 | 54 | 480 | 140.6 | 35.3 | 500 | 6.67 | 0.19 | amphibious, sonar |
| armbull | Bull | 2 | assault tank | armavp | 950 | 13000 | 23000 | 4650 | 62 | 460 | 245.5 |  | 494 | 4.89 | 0.26 |  |
| armvader | Tumbleweed | 2 | crawling bomb | armalab | 65 | 5800 | 8000 | 445 | 84 |  |  |  | 260 | 6.85 |  | amphibious |
| armaser | Smuggler | 2 | jammer bot | armalab | 78 | 1400 | 6000 | 340 | 37 |  |  |  | 380 | 4.36 |  | radar_jammer |
| armjam | Umbra | 2 | jammer vehicle | armavp | 105 | 2000 | 7000 | 510 | 43 |  |  |  | 330 | 4.86 |  | radar_jammer |
| armbrtha | Basilica | 2 | long-range cannon | armaca | 4500 | 64000 | 85000 | 4450 |  | 4650 | 123.1 |  | 273 | 0.99 | 0.03 | needs_energy_to_fire, long_range_cannon |
| armvulc | Ragnarok | 2 | long-range cannon | armaca | 70000 | 830000 | 1400000 | 32000 |  | 5750 | 2708.3 |  | 700 | 0.46 | 0.04 | needs_energy_to_fire, long_range_cannon |
| armscab | Umbrella | 2 | mobile anti-nuke | armalab | 1150 | 30000 | 49000 | 870 | 51 |  |  |  | 450 | 0.76 |  | antinuke |
| armantiship | Haven | 2 | mobile anti-nuke | armasy | 1400 | 12000 | 27000 | 5000 | 63 |  |  |  | 1000 | 3.57 |  | sonar, antinuke, floats |
| armmark | Compass | 2 | radar bot | armalab | 100 | 1250 | 5000 | 355 | 40.5 |  |  |  | 900 | 3.55 |  |  |
| armseer | Prophet | 2 | radar vehicle | armavp | 125 | 2000 | 7500 | 980 | 57 |  |  |  | 900 | 7.84 |  |  |
| armfast | Sprinter | 2 | raider bot | armalab | 160 | 3800 | 5000 | 690 | 111.3 | 230 | 120 | 40 | 380 | 4.31 | 0.75 |  |
| armlatnk | Jaguar | 2 | riot tank | armavp | 330 | 6400 | 8000 | 1060 | 90 | 300 | 146.7 | 42.9 | 390 | 3.21 | 0.44 | needs_energy_to_fire |
| armfido | Hound | 2 | skirmisher bot | armalab | 285 | 6300 | 6500 | 1200 | 69 | 650 | 77.3 | 10.6 | 400 | 4.21 | 0.27 |  |
| armsptk | Recluse | 2 | skirmisher bot | armalab | 400 | 4500 | 11500 | 1390 | 47 | 575 | 160.7 |  | 440 | 3.48 | 0.40 |  |
| armmav | Gunslinger | 2 | skirmisher bot | armalab | 650 | 11000 | 17000 | 1800 | 50 | 365 | 265 | 65 | 550 | 2.77 | 0.41 |  |
| armsnipe | Sharpshooter | 2 | sniper bot | armalab | 680 | 20000 | 24000 | 580 | 33 | 900 | 250 |  | 455 | 0.85 | 0.37 | needs_energy_to_fire |
| armmanni | Starlight | 2 | sniper tank | armavp | 1200 | 18500 | 33000 | 2800 | 42 | 950 | 531.9 |  | 650 | 2.33 | 0.44 | needs_energy_to_fire |
| armspy | Ghost | 2 | spy bot | armalab | 135 | 9000 | 12000 | 300 | 65 |  |  |  | 550 | 2.22 |  | stealth |
| armgremlin | Gremlin | 2 | stealth tank | armavp | 230 | 3700 | 8500 | 1060 | 70.5 | 220 | 87.5 | 8 | 494 | 4.61 | 0.38 | stealth |
| armthor | Thor | 3 | EMP assault mech | armshltx | 9000 | 240000 | 320000 | 56000 | 54 | 540 | 967.7 |  | 600 | 6.22 | 0.11 | paralyzer, emp, needs_energy_to_fire, gantry |
| armvang | Vanguard | 3 | artillery mech | armshltx | 3300 | 71000 | 120000 | 8500 | 33 | 1450 | 316.2 |  | 625 | 2.58 | 0.10 | gantry |
| armlun | Lunkhead | 3 | assault hovercraft | armshltx | 1150 | 23000 | 40000 | 5300 | 50.1 | 575 | 205.6 |  | 620 | 4.61 | 0.18 | hover, sonar, gantry, floats |
| armmar | Marauder | 3 | assault mech | armshltx | 970 | 21000 | 33000 | 4900 | 94.5 | 350 | 335.7 | 42.9 | 455 | 5.05 | 0.35 | amphibious, gantry |
| armraz | Razorback | 3 | assault mech | armshltx | 3800 | 68000 | 120000 | 13300 | 69 | 475 | 1050 | 200 | 450 | 3.50 | 0.28 | needs_energy_to_fire, gantry |
| armbanth | Titan | 3 | assault mech | armshltx | 13500 | 286000 | 360000 | 69000 | 45.6 | 860 | 1452.5 | 1052.5 | 617 | 5.11 | 0.11 | needs_energy_to_fire, amphibious, anti_air, gantry |

### Cortex: land (52 units)

<!--meta
id: F-units-land-cortex
layer: fact
ids: [F-unit-coraak, F-unit-corah, F-unit-corak, F-unit-coramph, F-unit-corantiship, F-unit-corban, F-unit-corbuzz, F-unit-corcan, F-unit-corcat, F-unit-corcrash, F-unit-cordemon, F-unit-coreter, F-unit-corfav, F-unit-corgarp, F-unit-corgator, F-unit-corgol, F-unit-corhal, F-unit-corhrk, F-unit-corint, F-unit-corjugg, F-unit-corkarg, F-unit-corkorg, F-unit-corlevlr, F-unit-cormabm, F-unit-cormando, F-unit-cormart, F-unit-cormh, F-unit-cormist, F-unit-cormort, F-unit-corparrow, F-unit-corpyro, F-unit-corraid, F-unit-correap, F-unit-corroach, F-unit-corsala, F-unit-corsent, F-unit-corsh, F-unit-corshiva, F-unit-corsktl, F-unit-corsnap, F-unit-corsok, F-unit-corspec, F-unit-corspy, F-unit-corstorm, F-unit-corsumo, F-unit-cortermite, F-unit-corthud, F-unit-cortrem, F-unit-corvoyr, F-unit-corvrad, F-unit-corvroc, F-unit-corwolv]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | range | dps | dps vs air | sight | health per metal | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| corcrash | Trasher | 1 | anti-air bot | coramsub | 125 | 1100 | 1900 | 640 | 53.5 | 760 | 0.5 | 55 | 380 | 5.12 | 0.00 | amphibious, anti_air |
| corah | Birdeater | 1 | anti-air hovercraft | corfhp | 250 | 4700 | 3650 | 1120 | 88.5 | 700 | 1.4 | 84.5 | 509 | 4.48 | 0.01 | hover, anti_air, sonar, floats |
| cormh | Mangonel | 1 | artillery hovercraft | corfhp | 200 | 3300 | 3500 | 550 | 63.9 | 700 | 61.1 |  | 509 | 2.75 | 0.31 | hover, floats |
| corwolv | Wolverine | 1 | artillery tank | corvp | 170 | 2500 | 3550 | 750 | 48 | 710 | 42.3 |  | 330 | 4.41 | 0.25 |  |
| corhal | Halberd | 1 | assault hovercraft | corfhp | 640 | 13000 | 12900 | 4350 | 45.3 | 300 | 343.8 | 31.2 | 450 | 6.80 | 0.54 | needs_energy_to_fire, hover, floats |
| corthud | Thug | 1 | line bot | corlab | 140 | 1150 | 2100 | 1100 | 45 | 380 | 60 | 12.1 | 380 | 7.86 | 0.43 |  |
| corsnap | Cayman | 1 | line hovercraft | corfhp | 300 | 3100 | 3500 | 1720 | 67 | 370 | 70 | 20.7 | 491 | 5.73 | 0.23 | hover, floats |
| corgarp | Garpike | 1 | line tank | coramsub | 220 | 2600 | 3100 | 1420 | 58.5 | 305 | 77.3 | 16.7 | 500 | 6.45 | 0.35 | amphibious, sonar |
| corraid | Brute | 1 | line tank | corvp | 235 | 2400 | 3900 | 2000 | 72 | 350 | 80.8 | 15 | 350 | 8.51 | 0.34 |  |
| corak | Grunt | 1 | raider bot | corfast | 43 | 820 | 1250 | 280 | 81 | 215 | 74 | 14 | 520 | 6.51 | 1.72 |  |
| corsh | Goon | 1 | raider hovercraft | corfhp | 75 | 1500 | 1500 | 255 | 109.5 | 230 | 60 | 11.7 | 550 | 3.40 | 0.80 | hover, floats |
| corgator | Incisor | 1 | raider tank | corvp | 120 | 1100 | 2300 | 820 | 85 | 230 | 93.8 | 17.5 | 330 | 6.83 | 0.78 |  |
| corlevlr | Pounder | 1 | riot tank | corvp | 220 | 2600 | 3100 | 1400 | 40 | 315 | 105.6 |  | 330 | 6.36 | 0.48 |  |
| corfav | Rascal | 1 | scout vehicle | corvp | 26 | 270 | 1150 | 90 | 153 | 180 | 31.8 | 1.8 | 600 | 3.46 | 1.22 |  |
| corstorm | Aggravator | 1 | skirmisher bot | corlab | 110 | 1150 | 1950 | 740 | 47.7 | 475 | 41.3 | 41.3 | 380 | 6.73 | 0.38 | anti_air |
| cormist | Lasher | 1 | skirmisher tank | corvp | 155 | 2400 | 3440 | 860 | 52 | 575 | 25.2 | 48 | 620 | 5.55 | 0.16 | anti_air |
| coramph | Duck | 2 | amphibious assault bot | coralab | 330 | 9500 | 12500 | 2350 | 55.5 | 300 | 132.4 | 33.5 | 400 | 7.12 | 0.40 | needs_energy_to_fire, amphibious, sonar |
| coraak | Manticore | 2 | anti-air bot | coralab | 650 | 6200 | 11000 | 3200 | 34.5 | 1300 | 0.9 | 233.1 | 390 | 4.92 | 0.00 | amphibious, anti_air |
| corsent | Fury | 2 | anti-air vehicle | coravp | 470 | 10500 | 16000 | 2700 | 69 | 775 | 1.4 | 272.7 | 338 | 5.74 | 0.00 | anti_air |
| cormort | Sheldon | 2 | artillery bot | coralab | 400 | 2800 | 7000 | 940 | 50 | 850 | 65.6 |  | 380 | 2.35 | 0.16 |  |
| corhrk | Arbiter | 2 | artillery bot | coralab | 600 | 5800 | 9500 | 610 | 54 | 1210 | 100 |  | 380 | 1.02 | 0.17 |  |
| cormart | Quaker | 2 | artillery tank | coravp | 400 | 4400 | 9000 | 1200 | 58 | 800 | 84 |  | 330 | 3.00 | 0.21 |  |
| corvroc | Negotiator | 2 | artillery tank | coravp | 880 | 6700 | 20000 | 1390 | 33 | 1310 | 106.2 |  | 330 | 1.58 | 0.12 | stealth |
| cortrem | Tremor | 2 | artillery tank | coravp | 1850 | 49000 | 43000 | 3000 | 40.5 | 1470 | 400 |  | 351 | 1.62 | 0.22 |  |
| cortermite | Termite | 2 | assault bot | coralab | 540 | 8100 | 17000 | 3100 | 50 | 340 | 331.2 |  | 380 | 5.74 | 0.61 | needs_energy_to_fire |
| corcan | Sumo | 2 | assault bot | coralab | 560 | 9300 | 12000 | 6000 | 38 | 275 | 343.8 | 68.8 | 380 | 10.71 | 0.61 | needs_energy_to_fire |
| corsumo | Mammoth | 2 | assault bot | coralab | 2200 | 35000 | 65000 | 15600 | 23 | 650 | 503.3 | 118.3 | 510 | 7.09 | 0.23 | needs_energy_to_fire |
| correap | Tiger | 2 | assault tank | coravp | 690 | 10000 | 16000 | 5300 | 69 | 410 | 155.7 |  | 462 | 7.68 | 0.23 |  |
| corban | Banisher | 2 | assault tank | coravp | 1000 | 23000 | 30000 | 2500 | 54 | 800 | 133.3 | 133.3 | 650 | 2.50 | 0.13 | anti_air |
| corparrow | Poison Arrow | 2 | assault tank | coramsub | 1050 | 21000 | 26000 | 6300 | 51 | 575 | 205.6 |  | 500 | 6.00 | 0.20 | amphibious, sonar |
| corgol | Tzar | 2 | assault tank | coravp | 1650 | 28000 | 40000 | 7800 | 39 | 650 | 257.1 |  | 395 | 4.73 | 0.16 |  |
| cormando | Commando | 2 | commando bot | coralab | 1200 | 15500 | 24000 | 1800 | 75 | 300 | 200 | 60 | 600 | 1.50 | 0.17 | amphibious, radar_jammer |
| corroach | Bedbug | 2 | crawling bomb | coralab | 69 | 5800 | 8000 | 620 | 76 |  |  |  | 260 | 8.99 |  | amphibious |
| corsktl | Skuttle | 2 | crawling bomb | coralab | 755 | 27000 | 23000 | 355 | 84 |  |  |  | 260 | 0.47 |  | amphibious |
| corspec | Deceiver | 2 | jammer bot | coralab | 75 | 1300 | 6500 | 350 | 35 |  |  |  | 380 | 4.67 |  | radar_jammer |
| coreter | Obscurer | 2 | jammer vehicle | coravp | 105 | 1900 | 7500 | 580 | 41 |  |  |  | 330 | 5.52 |  | radar_jammer |
| corint | Basilisk | 2 | long-range cannon | coraca | 4600 | 67000 | 93300 | 4700 |  | 4950 | 125 |  | 273 | 1.02 | 0.03 | needs_energy_to_fire, long_range_cannon |
| corbuzz | Calamity | 2 | long-range cannon | coraca | 68000 | 800000 | 1400000 | 33500 |  | 6100 | 2727.3 |  | 700 | 0.49 | 0.04 | needs_energy_to_fire, long_range_cannon |
| cormabm | Saviour | 2 | mobile anti-nuke | coravp | 1250 | 32000 | 52000 | 870 | 51 |  |  |  | 450 | 0.70 |  | antinuke |
| corantiship | Oasis | 2 | mobile anti-nuke | corasy | 1400 | 12000 | 27000 | 5000 | 63 |  |  |  | 1000 | 3.57 |  | sonar, antinuke, floats |
| corvoyr | Augur | 2 | radar bot | coralab | 99 | 1350 | 5000 | 390 | 38 |  |  |  | 925 | 3.94 |  |  |
| corvrad | Omen | 2 | radar vehicle | coravp | 100 | 1300 | 5000 | 570 | 50 |  |  |  | 900 | 5.70 |  |  |
| corpyro | Fiend | 2 | raider bot | coralab | 200 | 3000 | 6500 | 1060 | 82.5 | 230 | 240 |  | 380 | 5.30 | 1.20 |  |
| corsala | Salamander | 2 | riot tank | coramsub | 350 | 7000 | 10500 | 2100 | 72 | 340 | 163.6 |  | 385 | 6.00 | 0.47 | amphibious, sonar |
| corspy | Spectre | 2 | spy bot | coralab | 150 | 9000 | 12000 | 450 | 62 |  |  |  | 550 | 3.00 |  | stealth |
| corcat | Catapult | 3 | artillery mech | corgant | 4900 | 80000 | 160000 | 6100 | 48 | 1350 | 600 |  | 700 | 1.24 | 0.12 | gantry |
| corsok | Cataphract | 3 | assault hovercraft | corgant | 950 | 19000 | 34000 | 4200 | 45.3 | 725 | 166.7 | 55.6 | 650 | 4.42 | 0.18 | needs_energy_to_fire, hover, sonar, gantry, floats |
| corshiva | Shiva | 3 | assault mech | corgant | 1600 | 23000 | 40000 | 9400 | 48 | 800 | 407.1 |  | 520 | 5.88 | 0.25 | amphibious, gantry |
| corkarg | Karganeth | 3 | assault mech | corgant | 2500 | 57000 | 94000 | 12500 | 45 | 600 | 600 | 250 | 455 | 5.00 | 0.24 | gantry |
| cordemon | Demon | 3 | assault mech | corgant | 6000 | 90000 | 160000 | 18000 | 63 | 500 | 1440 | 250 | 500 | 3.00 | 0.24 | gantry |
| corjugg | Behemoth | 3 | assault mech | corgant | 20000 | 513000 | 780000 | 335000 | 16.5 | 590 | 1121.4 | 621.4 | 720 | 16.75 | 0.06 | needs_energy_to_fire, anti_air, gantry |
| corkorg | Juggernaut | 3 | assault mech | corgant | 29000 | 615000 | 730000 | 149000 | 37 | 950 | 4186.9 | 2187.5 | 845 | 5.14 | 0.14 | needs_energy_to_fire, amphibious, anti_air, gantry |

### Legion: land (53 units)

<!--meta
id: F-units-land-legion
layer: fact
ids: [F-unit-legaabot, F-unit-legadvaabot, F-unit-legah, F-unit-legaheattank, F-unit-legajamk, F-unit-legamcluster, F-unit-legamph, F-unit-legamphtank, F-unit-leganavyantinukecarrier, F-unit-legaradk, F-unit-legaskirmtank, F-unit-legaspy, F-unit-legavantinuke, F-unit-legavjam, F-unit-legavrad, F-unit-legavroc, F-unit-legbal, F-unit-legbar, F-unit-legbart, F-unit-legcar, F-unit-legcen, F-unit-legeheatraymech, F-unit-legehovertank, F-unit-legelrpcmech, F-unit-legerailtank, F-unit-legeshotgunmech, F-unit-legfloat, F-unit-leggat, F-unit-leggob, F-unit-leghades, F-unit-leghelios, F-unit-leghrk, F-unit-leginc, F-unit-leginf, F-unit-leginfestor, F-unit-legjav, F-unit-legkark, F-unit-legkeres, F-unit-leglob, F-unit-leglrpc, F-unit-legmed, F-unit-legmh, F-unit-legmrv, F-unit-legner, F-unit-legrail, F-unit-legscout, F-unit-legsh, F-unit-legshot, F-unit-legsnapper, F-unit-legsrail, F-unit-legstarfall, F-unit-legstr, F-unit-legvflak]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | range | dps | dps vs air | sight | health per metal | dps per metal | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| legaabot | Toxotai | 1 | anti-air bot | legaceb | 120 | 950 | 1800 | 610 | 55 | 760 | 1.5 | 55.5 | 380 | 5.08 | 0.01 | amphibious, anti_air |
| legah | Alpheus | 1 | anti-air hovercraft | legfhp | 210 | 4300 | 3300 | 1120 | 88.5 | 700 | 75 | 120 | 509 | 5.33 | 0.36 | hover, anti_air, sonar, floats |
| legmh | Salacia | 1 | artillery hovercraft | legfhp | 190 | 3100 | 3300 | 520 | 70 | 700 | 53.3 |  | 509 | 2.74 | 0.28 | hover, floats |
| legbar | Barrage | 1 | artillery tank | legvp | 250 | 3200 | 3600 | 1500 | 40.5 | 535 | 12 |  | 400 | 6.00 | 0.05 |  |
| legkark | Karkinos | 1 | assault bot | leglab | 310 | 2400 | 4400 | 1725 | 42 | 360 | 188.9 | 43.9 | 400 | 5.56 | 0.61 |  |
| legcar | Cardea | 1 | assault hovercraft | legfhp | 550 | 11000 | 11000 | 3600 | 51 | 321 | 280 | 91 | 450 | 6.55 | 0.51 | hover, floats |
| leggat | Decurion | 1 | assault tank | legvp | 300 | 3600 | 4800 | 2550 | 45 | 361 | 180 | 45 | 400 | 8.50 | 0.60 |  |
| leglob | Satyr | 1 | line bot | leglab | 60 | 500 | 1200 | 550 | 51 | 400 | 30 | 8.4 | 380 | 9.17 | 0.50 |  |
| legner | Nereus | 1 | line hovercraft | legfhp | 220 | 2000 | 2400 | 1100 | 76 | 370 | 61.1 | 21.1 | 509 | 5.00 | 0.28 | hover, floats |
| legamphtank | Cetus | 1 | line tank | legamphlab | 200 | 2000 | 2610 | 1340 | 63 | 305 | 77.3 | 16.7 | 500 | 6.70 | 0.39 | amphibious, sonar |
| legcen | Phobos | 1 | raider bot | leglab | 140 | 2200 | 2800 | 840 | 90 | 180 | 107.1 | 35.7 | 400 | 6.00 | 0.77 |  |
| legsh | Glaucus | 1 | raider hovercraft | legfhp | 60 | 1200 | 1200 | 220 | 102 | 260 | 60 | 60 | 550 | 3.67 | 1.00 | hover, anti_air, floats |
| leghades | Alaris | 1 | raider tank | legvp | 65 | 850 | 1650 | 425 | 99 | 211 | 35.7 | 14.3 | 330 | 6.54 | 0.55 |  |
| legscout | Wheelie | 1 | scout vehicle | legvp | 23 | 350 | 900 | 75 | 160 | 169 | 23.6 | 2.1 | 635 | 3.26 | 1.03 |  |
| leggob | Goblin | 1 | skirmisher bot | legaceb | 30 | 420 | 900 | 200 | 75 | 261 | 26.7 | 8.9 | 500 | 6.67 | 0.89 |  |
| legbal | Ballista | 1 | skirmisher bot | leglab | 160 | 1920 | 2400 | 900 | 43.5 | 460 | 45.5 | 45.5 | 380 | 5.62 | 0.28 | anti_air |
| leghelios | Helios | 1 | skirmisher tank | legvp | 160 | 2400 | 2600 | 800 | 75 | 320 | 123.1 | 19.2 | 360 | 5.00 | 0.77 |  |
| legrail | Lance | 1 | skirmisher tank | legvp | 240 | 3600 | 3600 | 1100 | 46.5 | 650 | 26.7 | 75 | 525 | 4.58 | 0.11 | anti_air |
| legamph | Telchine | 2 | amphibious assault bot | legaceb | 600 | 13200 | 16980 | 2750 | 48 | 450 | 990 |  | 450 | 4.58 | 1.65 | needs_energy_to_fire, amphibious, sonar |
| legadvaabot | Aquilon | 2 | anti-air bot | legalab | 750 | 6650 | 11270 | 2400 | 34.5 | 1151 | 7.8 | 375 | 390 | 3.20 | 0.01 | amphibious, anti_air |
| legvflak | Charon | 2 | anti-air vehicle | legavp | 470 | 10500 | 15130 | 2700 | 70 | 800 | 18 | 720 | 338 | 5.74 | 0.04 | anti_air |
| leghrk | Thanatos | 2 | artillery bot | legalab | 750 | 9000 | 12600 | 1200 | 45 | 1200 | 112.5 |  | 380 | 1.60 | 0.15 |  |
| legamcluster | Cleaver | 2 | artillery tank | legavp | 460 | 5500 | 10450 | 1250 | 44 | 930 | 41.7 |  | 330 | 2.72 | 0.09 |  |
| legavroc | Boreas | 2 | artillery tank | legavp | 920 | 6500 | 20130 | 1220 | 33 | 1300 | 105.6 |  | 330 | 1.33 | 0.11 | stealth |
| legmed | Medusa | 2 | artillery tank | legavp | 1500 | 22500 | 30370 | 3000 | 39 | 1000 | 250 |  | 600 | 2.00 | 0.17 |  |
| leginf | Inferno | 2 | artillery tank | legavp | 1700 | 30000 | 42900 | 3650 | 37.5 | 1200 | 51.4 |  | 351 | 2.15 | 0.03 |  |
| leginc | Incinerator | 2 | assault bot | legalab | 2300 | 46000 | 69700 | 9000 | 24 | 725 | 990 |  | 650 | 3.91 | 0.43 | needs_energy_to_fire |
| legfloat | Triton | 2 | assault tank | legamphlab | 650 | 12000 | 12000 | 3500 | 55 | 550 | 205 | 105 | 500 | 5.38 | 0.32 | hover, anti_air, floats |
| legaheattank | Prometheus | 2 | assault tank | legavp | 1250 | 19000 | 25600 | 7700 | 52 | 450 | 990 |  | 450 | 6.16 | 0.79 | needs_energy_to_fire |
| legsnapper | Snapper | 2 | crawling bomb | legalab | 60 | 5600 | 9260 | 550 | 81 |  |  |  | 280 | 9.17 |  | amphibious |
| legajamk | Tiresias | 2 | jammer bot | legalab | 75 | 1300 | 6280 | 350 | 35 |  |  |  | 380 | 4.67 |  | radar_jammer |
| legavjam | Cicero | 2 | jammer vehicle | legavp | 105 | 1900 | 6930 | 510 | 41 |  |  |  | 330 | 4.86 |  | radar_jammer |
| leglrpc | Olympus | 2 | long-range cannon | legaca | 5200 | 76000 | 93000 | 4450 |  | 4800 | 100 |  | 273 | 0.86 | 0.02 | needs_energy_to_fire, long_range_cannon |
| legstarfall | Starfall | 2 | long-range cannon | legaca | 63000 | 720000 | 1400000 | 26000 |  | 6100 | 2700 |  | 700 | 0.41 | 0.04 | needs_energy_to_fire, long_range_cannon |
| legavantinuke | Hera | 2 | mobile anti-nuke | legavp | 1100 | 36000 | 51300 | 720 | 58 |  |  |  | 450 | 0.65 |  | antinuke |
| leganavyantinukecarrier | Hecate | 2 | mobile anti-nuke | legadvshipyard | 1600 | 13000 | 26800 | 5000 | 63 |  |  |  | 1000 | 3.12 |  | sonar, antinuke, floats |
| legaradk | Euclid | 2 | radar bot | legalab | 99 | 1350 | 4700 | 390 | 45 |  |  |  | 925 | 3.94 |  |  |
| legavrad | Pheme | 2 | radar vehicle | legavp | 125 | 2000 | 7290 | 980 | 57 |  |  |  | 900 | 7.84 |  |  |
| legstr | Hoplite | 2 | raider bot | legaceb | 355 | 5250 | 9240 | 1780 | 84 | 280 | 240 | 90 | 400 | 5.01 | 0.68 |  |
| legmrv | Quickshot | 2 | raider tank | legavp | 250 | 4100 | 5900 | 970 | 135 | 240 | 120 |  | 500 | 3.88 | 0.48 |  |
| legshot | Phalanx | 2 | riot bot | legalab | 470 | 5000 | 11000 | 3350 | 44 | 280 | 105 |  | 350 | 7.13 | 0.22 |  |
| legbart | Belcher | 2 | skirmisher bot | legalab | 550 | 5500 | 10000 | 1780 | 60 | 625 | 30 |  | 400 | 3.24 | 0.05 |  |
| legaskirmtank | Gladiator | 2 | skirmisher tank | legavp | 450 | 6800 | 10490 | 2500 | 65 | 600 | 105 |  | 440 | 5.56 | 0.23 |  |
| legsrail | Arquebus | 2 | sniper bot | legalab | 750 | 15000 | 19500 | 2200 | 42 | 850 | 121.4 | 121.4 | 700 | 2.93 | 0.16 | needs_energy_to_fire, anti_air |
| leginfestor | Infestor | 2 | spider assault bot | legalab | 250 | 5200 | 4310 | 1200 | 54 | 300 | 120 |  | 400 | 4.80 | 0.48 | stealth |
| legaspy | Eidolon | 2 | spy bot | legalab | 125 | 8500 | 12900 | 300 | 62.5 |  |  |  | 550 | 2.40 |  | stealth |
| legelrpcmech | Astraeus | 3 | artillery mech | leggant | 11000 | 150000 | 178000 | 17000 | 25 | 3100 | 166.7 |  | 625 | 1.55 | 0.02 | needs_energy_to_fire, long_range_cannon, gantry |
| legehovertank | Charybdis | 3 | assault hovercraft | leggant | 1350 | 25000 | 41600 | 4900 | 63 | 500 | 108.3 |  | 600 | 3.63 | 0.08 | needs_energy_to_fire, hover, sonar, gantry, floats |
| legerailtank | Daedalus | 3 | assault mech | leggant | 6500 | 165000 | 125000 | 16500 | 61 | 1000 | 800 | 800 | 750 | 2.54 | 0.12 | needs_energy_to_fire, anti_air, gantry |
| legeshotgunmech | Praetorian | 3 | assault mech | leggant | 7000 | 120000 | 159000 | 22000 | 72 | 700 | 1233.3 | 385.7 | 617 | 3.14 | 0.18 | gantry |
| legeheatraymech | Sol Invictus | 3 | assault mech | leggant | 23500 | 615000 | 585250 | 110000 | 40 | 800 | 2680 | 211.1 | 845 | 4.68 | 0.11 | needs_energy_to_fire, amphibious, gantry |
| legjav | Javelin | 3 | raider mech | leggant | 1200 | 21000 | 39850 | 7200 | 68 | 450 | 360 |  | 455 | 6.00 | 0.30 | amphibious, gantry |
| legkeres | Keres | 3 | riot mech | leggant | 2600 | 57000 | 60000 | 21000 | 48 | 481 | 535.3 |  | 650 | 8.08 | 0.21 | gantry |
