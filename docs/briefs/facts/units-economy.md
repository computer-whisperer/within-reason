## Unit facts: economy

What produces, stores and builds: extractors, energy makers, converters, storages, construction turrets, factories and constructors of every faction, with the commanders. From the game's unit definitions at commit 54199a0d3 (`run/unit_stats.py` into `crates/bot/data/units.json`; this document by `run/fact_units.py`). Metal and energy are the costs. Build time is in build units: a builder with build power P finishes the unit in build time / P seconds, and draws the costs over those seconds. Speed is elmos a second; range and sight are elmos; dps is damage a second against ground targets. The ratios are per metal of cost.

### Armada: economy (61 units)

<!--meta
id: F-units-economy-armada
layer: fact
ids: [F-unit-armaap, F-unit-armaca, F-unit-armack, F-unit-armacsub, F-unit-armacv, F-unit-armadvsol, F-unit-armafus, F-unit-armageo, F-unit-armalab, F-unit-armamex, F-unit-armamsub, F-unit-armap, F-unit-armasy, F-unit-armavp, F-unit-armbeaver, F-unit-armca, F-unit-armch, F-unit-armck, F-unit-armckfus, F-unit-armcom, F-unit-armconsul, F-unit-armcs, F-unit-armcsa, F-unit-armcv, F-unit-armdecom, F-unit-armestor, F-unit-armfark, F-unit-armfhp, F-unit-armfmkr, F-unit-armfus, F-unit-armgeo, F-unit-armgmm, F-unit-armhp, F-unit-armlab, F-unit-armmakr, F-unit-armmex, F-unit-armmls, F-unit-armmmkr, F-unit-armmoho, F-unit-armmstor, F-unit-armnanotc, F-unit-armnanotcplat, F-unit-armplat, F-unit-armrecl, F-unit-armrectr, F-unit-armshltx, F-unit-armshltxuw, F-unit-armsolar, F-unit-armsy, F-unit-armtide, F-unit-armuwadves, F-unit-armuwadvms, F-unit-armuwageo, F-unit-armuwes, F-unit-armuwfus, F-unit-armuwgeo, F-unit-armuwmme, F-unit-armuwmmm, F-unit-armuwms, F-unit-armvp, F-unit-armwin]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | build power | energy make | energy upkeep | e storage | m storage | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| armap | Aircraft Plant | 1 | aircraft factory | armaca | 650 | 1100 | 5450 | 2050 |  | 150 |  |  |  |  |  |
| armamsub | Amphibious Complex | 1 | amphibious factory | armbeaver | 1200 | 5500 | 11100 | 2650 |  | 300 |  |  |  |  | submerged, on_water |
| armlab | Bot Lab | 1 | bot factory | armack | 500 | 950 | 5000 | 2900 |  | 150 |  |  |  |  |  |
| armcom | Armada Commander | 1 | commander |  | 2700 | 26000 | 75000 | 3700 | 37.5 | 300 | 30 |  |  |  | cloak, amphibious, anti_air, sonar |
| armnanotc | Construction Turret | 1 | construction turret | armbeaver | 230 | 3200 | 5300 | 560 |  | 200 |  |  |  |  |  |
| armnanotcplat | Naval Construction Turret | 1 | construction turret | armbeaver | 230 | 3200 | 5300 | 560 |  | 200 |  |  |  |  | on_water |
| armca | Construction Aircraft | 1 | constructor aircraft | armap | 110 | 2100 | 5000 | 156 | 136 | 50 | 5 |  |  |  | flies |
| armck | Construction Bot | 1 | constructor bot | armlab | 110 | 1600 | 3450 | 760 | 36 | 80 | 7 |  |  |  |  |
| armch | Construction Hovercraft | 1 | constructor hovercraft | armfhp | 200 | 2700 | 4470 | 1440 | 66.9 | 110 | 11 |  |  |  | hover, floats |
| armcsa | Construction Seaplane | 1 | constructor seaplane | armplat | 150 | 5000 | 9500 | 405 | 124 | 75 | 20 |  |  |  | flies |
| armcs | Construction Ship | 1 | constructor ship | armconsul | 200 | 2000 | 3460 | 1040 | 65.4 | 125 | 7 |  |  |  | floats |
| armcv | Construction Vehicle | 1 | constructor vehicle | armconsul | 135 | 1950 | 4050 | 1380 | 54 | 90 | 10 |  |  |  |  |
| armbeaver | Beaver | 1 | constructor vehicle | armamsub | 150 | 3100 | 6700 | 1030 | 42 | 80 | 8 |  |  |  | amphibious |
| armfmkr | Naval Energy Converter | 1 | energy converter | armbeaver | 1 | 1150 | 2600 | 167 |  |  |  |  |  |  | submerged, on_water |
| armmakr | Energy Converter | 1 | energy converter | armbeaver | 1 | 1150 | 2600 | 167 |  |  |  |  |  |  |  |
| armestor | Energy Storage | 1 | energy storage | armbeaver | 170 | 1700 | 4110 | 1890 |  |  |  |  | 6000 |  |  |
| armuwes | Naval Energy Storage | 1 | energy storage | armbeaver | 170 | 1700 | 4110 | 1890 |  |  |  |  | 6000 |  | submerged, on_water |
| armgeo | Geothermal Powerplant | 1 | geothermal plant | armbeaver | 560 | 13000 | 13100 | 1940 |  |  | 300 |  | 1000 |  |  |
| armuwgeo | Offshore Geothermal Powerplant | 1 | geothermal plant | armbeaver | 560 | 13000 | 13100 | 1940 |  |  | 300 |  | 1000 |  |  |
| armfhp | Naval Hovercraft Platform | 1 | hovercraft factory | armbeaver | 670 | 2000 | 8700 | 3350 |  | 150 |  |  |  |  | submerged, on_water |
| armhp | Hovercraft Platform | 1 | hovercraft factory | armbeaver | 670 | 2000 | 8700 | 3350 |  | 150 |  |  |  |  |  |
| armmex | Metal Extractor | 1 | metal extractor | armbeaver | 50 | 500 | 1800 | 270 |  |  |  | 3 |  |  |  |
| armamex | Twilight | 1 | metal extractor | armbeaver | 200 | 1500 | 1800 | 1610 |  |  |  | 3 |  |  | stealth |
| armmstor | Metal Storage | 1 | metal storage | armbeaver | 330 | 570 | 2920 | 2100 |  |  |  |  |  | 3000 |  |
| armuwms | Naval Metal Storage | 1 | metal storage | armbeaver | 330 | 570 | 2920 | 3600 |  |  |  |  |  | 3000 | submerged, on_water |
| armrectr | Lazarus | 1 | resurrection bot | armlab | 130 | 1400 | 2800 | 220 | 78 | 200 |  |  |  |  | resurrect, stealth |
| armrecl | Grim Reaper | 1 | resurrection submarine | armsy | 240 | 3500 | 6200 | 420 | 66 | 150 |  |  |  |  | resurrect, submerged, sonar |
| armplat | Seaplane Platform | 1 | seaplane factory | armaca | 1450 | 5000 | 12000 | 2000 |  | 300 |  |  |  |  | submerged, sonar, on_water |
| armsy | Shipyard | 1 | shipyard | armacsub | 450 | 950 | 5160 | 4100 |  | 150 |  |  |  |  | submerged, on_water |
| armsolar | Solar Collector | 1 | solar collector | armbeaver | 155 | 0 | 2600 | 340 |  |  | 20 |  |  |  |  |
| armadvsol | Advanced Solar Collector | 1 | solar collector | armbeaver | 350 | 5000 | 7950 | 1130 |  |  | 80 |  |  |  |  |
| armtide | Tidal Generator | 1 | tidal generator | armbeaver | 90 | 200 | 2190 | 400 |  |  |  |  |  |  | submerged, on_water |
| armvp | Vehicle Plant | 1 | vehicle factory | armacv | 590 | 1550 | 5700 | 3000 |  | 150 |  |  |  |  |  |
| armwin | Wind Turbine | 1 | wind generator | armbeaver | 40 | 175 | 1600 | 196 |  |  |  |  |  |  |  |
| armaap | Advanced Aircraft Plant | 2 | aircraft factory | armaca | 2900 | 29000 | 32000 | 3750 |  | 600 |  |  |  |  |  |
| armfark | Butler | 2 | assist bot | armalab | 210 | 3000 | 6000 | 335 | 75 | 140 | 12 |  |  |  |  |
| armalab | Advanced Bot Lab | 2 | bot factory | armack | 2600 | 15000 | 25000 | 4250 |  | 600 |  |  |  |  |  |
| armaca | Advanced Construction Aircraft | 2 | constructor aircraft | armaap | 340 | 12000 | 22000 | 200 | 192 | 120 | 10 |  |  |  | flies |
| armack | Advanced Construction Bot | 2 | constructor bot | armalab | 430 | 6900 | 12500 | 930 | 33 | 210 | 14 |  |  |  |  |
| armmls | Voyager | 2 | constructor ship | armasy | 190 | 3200 | 6000 | 1460 | 66 | 200 |  |  |  |  | floats |
| armacsub | Advanced Construction Sub | 2 | constructor submarine | armasy | 700 | 9000 | 23000 | 1110 | 72 | 350 | 30 |  |  |  | submerged |
| armconsul | Consul | 2 | constructor vehicle | armavp | 250 | 4300 | 8500 | 1080 | 78 | 150 | 15 |  |  |  |  |
| armacv | Advanced Construction Vehicle | 2 | constructor vehicle | armavp | 550 | 6800 | 16000 | 2050 | 52.5 | 290 | 20 |  |  |  |  |
| armdecom | Commander | 2 | decoy commander | armalab | 770 | 12000 | 30000 | 3700 | 37.5 | 300 | 15 |  |  |  | amphibious, anti_air, sonar |
| armmmkr | Advanced Energy Converter | 2 | energy converter | armaca | 380 | 21000 | 35000 | 445 |  |  |  |  |  |  |  |
| armuwmmm | Naval Advanced Energy Converter | 2 | energy converter | armacsub | 380 | 21000 | 35000 | 445 |  |  |  |  |  |  | on_water |
| armuwadves | Hardened Energy Storage | 2 | energy storage | armaca | 830 | 10000 | 20300 | 11700 |  |  |  |  | 40000 |  |  |
| armfus | Fusion Reactor | 2 | fusion reactor | armaca | 3350 | 18000 | 54000 | 3800 |  |  | 750 |  | 2500 |  |  |
| armckfus | Cloakable Fusion Reactor | 2 | fusion reactor | armaca | 3550 | 22000 | 65000 | 3800 |  |  | 750 |  | 2500 |  |  |
| armuwfus | Naval Fusion Reactor | 2 | fusion reactor | armacsub | 5200 | 33500 | 99900 | 5600 |  |  | 1200 |  | 2500 |  | submerged, on_water |
| armafus | Advanced Fusion Reactor | 2 | fusion reactor | armaca | 9700 | 69000 | 312500 | 8300 |  |  | 3000 |  | 9000 |  |  |
| armgmm | Prude | 2 | geothermal plant | armaca | 1150 | 25000 | 41500 | 13900 |  |  | 750 |  | 1500 |  |  |
| armageo | Advanced Geothermal Powerplant | 2 | geothermal plant | armaca | 1600 | 27000 | 50000 | 3600 |  |  | 1250 |  | 12000 |  |  |
| armuwageo | Advanced Geothermal Powerplant | 2 | geothermal plant | armaca | 1600 | 27000 | 50000 | 3600 |  |  | 1250 |  | 12000 |  | submerged, on_water |
| armmoho | Advanced Metal Extractor | 2 | metal extractor | armaca | 620 | 7700 | 14900 | 2800 |  |  |  | 20 |  |  |  |
| armuwmme | Naval Advanced Metal Extractor | 2 | metal extractor | armacsub | 620 | 7700 | 14900 | 2800 |  |  |  | 20 |  |  | submerged, on_water |
| armuwadvms | Hardened Metal Storage | 2 | metal storage | armaca | 750 | 11500 | 20400 | 10300 |  |  |  |  |  | 10000 |  |
| armasy | Advanced Shipyard | 2 | shipyard | armacsub | 2900 | 9700 | 24000 | 6000 |  | 600 |  |  |  |  | submerged, on_water |
| armavp | Advanced Vehicle Plant | 2 | vehicle factory | armacv | 2600 | 14000 | 27000 | 4750 |  | 600 |  |  |  |  |  |
| armshltx | Experimental Gantry | 3 | experimental gantry | armaca | 7900 | 58000 | 62000 | 16000 |  | 1800 |  |  | 1400 |  | gantry |
| armshltxuw | Experimental Gantry | 3 | experimental gantry | armacsub | 7900 | 58000 | 62000 | 16000 |  | 1800 |  |  | 1400 |  | submerged, gantry, on_water |

### Cortex: economy (60 units)

<!--meta
id: F-units-economy-cortex
layer: fact
ids: [F-unit-coraap, F-unit-coraca, F-unit-corack, F-unit-coracsub, F-unit-coracv, F-unit-coradvsol, F-unit-corafus, F-unit-corageo, F-unit-coralab, F-unit-coramsub, F-unit-corap, F-unit-corasy, F-unit-coravp, F-unit-corbhmth, F-unit-corca, F-unit-corch, F-unit-corck, F-unit-corcom, F-unit-corcs, F-unit-corcsa, F-unit-corcv, F-unit-cordecom, F-unit-corestor, F-unit-corexp, F-unit-corfast, F-unit-corfhp, F-unit-corfmkr, F-unit-corfus, F-unit-corgant, F-unit-corgantuw, F-unit-corgeo, F-unit-corhp, F-unit-corlab, F-unit-cormakr, F-unit-cormex, F-unit-cormexp, F-unit-cormls, F-unit-cormmkr, F-unit-cormoho, F-unit-cormstor, F-unit-cormuskrat, F-unit-cornanotc, F-unit-cornanotcplat, F-unit-cornecro, F-unit-corplat, F-unit-correcl, F-unit-corsolar, F-unit-corsy, F-unit-cortide, F-unit-coruwadves, F-unit-coruwadvms, F-unit-coruwageo, F-unit-coruwes, F-unit-coruwfus, F-unit-coruwgeo, F-unit-coruwmme, F-unit-coruwmmm, F-unit-coruwms, F-unit-corvp, F-unit-corwin]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | build power | energy make | energy upkeep | e storage | m storage | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| corap | Aircraft Plant | 1 | aircraft factory | coraca | 630 | 1100 | 5380 | 2150 |  | 150 |  |  |  |  |  |
| coramsub | Amphibious Complex | 1 | amphibious factory | corch | 1200 | 5600 | 11400 | 2800 |  | 300 |  |  |  |  | submerged, on_water |
| corexp | Exploiter | 1 | armed metal extractor | corca | 240 | 1900 | 2900 | 1440 |  |  |  | 3 |  |  | needs_energy_to_fire |
| corlab | Bot Lab | 1 | bot factory | corack | 470 | 1050 | 5000 | 2900 |  | 150 |  |  |  |  |  |
| corcom | Cortex Commander | 1 | commander |  | 2700 | 26000 | 75000 | 3700 | 37.5 | 300 | 30 |  |  |  | cloak, amphibious, anti_air, sonar |
| cornanotc | Construction Turret | 1 | construction turret | corca | 230 | 3200 | 5300 | 560 |  | 200 |  |  |  |  |  |
| cornanotcplat | Naval Construction Turret | 1 | construction turret | corch | 230 | 3200 | 5300 | 560 |  | 200 |  |  |  |  | on_water |
| corca | Construction Aircraft | 1 | constructor aircraft | corap | 115 | 2200 | 5200 | 161 | 131 | 55 | 5 |  |  |  | flies |
| corck | Construction Bot | 1 | constructor bot | corfast | 120 | 1750 | 3550 | 820 | 34.5 | 85 | 7 |  |  |  |  |
| corch | Construction Hovercraft | 1 | constructor hovercraft | corfhp | 190 | 3000 | 4580 | 1490 | 60.9 | 115 | 11 |  |  |  | hover, floats |
| corcsa | Construction Seaplane | 1 | constructor seaplane | corplat | 140 | 5300 | 9500 | 435 | 140 | 75 | 20 |  |  |  | flies |
| corcs | Construction Ship | 1 | constructor ship | corfast | 200 | 2000 | 3700 | 1080 | 60 | 125 | 7 |  |  |  | floats |
| corcv | Construction Vehicle | 1 | constructor vehicle | corvp | 145 | 2100 | 4160 | 1430 | 51 | 95 | 10 |  |  |  |  |
| cormuskrat | Muskrat | 1 | constructor vehicle | coramsub | 170 | 3500 | 6860 | 1110 | 40.5 | 85 | 8 |  |  |  | amphibious |
| corfmkr | Naval Energy Converter | 1 | energy converter | corch | 1 | 1250 | 2680 | 167 |  |  |  |  |  |  | submerged, on_water |
| cormakr | Energy Converter | 1 | energy converter | corca | 1 | 1250 | 2680 | 167 |  |  |  |  |  |  |  |
| corestor | Energy Storage | 1 | energy storage | corca | 175 | 1800 | 4260 | 2000 |  |  |  |  | 6000 |  |  |
| coruwes | Naval Energy Storage | 1 | energy storage | corch | 175 | 1800 | 4260 | 2000 |  |  |  |  | 6000 |  | submerged, on_water |
| corgeo | Geothermal Powerplant | 1 | geothermal plant | corca | 540 | 13000 | 12900 | 2050 |  |  | 300 |  | 1000 |  |  |
| coruwgeo | Offshore Geothermal Powerplant | 1 | geothermal plant | corca | 540 | 13000 | 12900 | 2050 |  |  | 300 |  | 1000 |  | submerged, on_water |
| corfhp | Naval Hovercraft Platform | 1 | hovercraft factory | corch | 670 | 2000 | 8700 | 3750 |  | 150 |  |  |  |  | submerged, on_water |
| corhp | Hovercraft Platform | 1 | hovercraft factory | corca | 670 | 2000 | 8700 | 3750 |  | 150 |  |  |  |  |  |
| cormex | Metal Extractor | 1 | metal extractor | corca | 50 | 500 | 1870 | 275 |  |  |  | 3 |  |  |  |
| cormstor | Metal Storage | 1 | metal storage | corca | 340 | 590 | 2920 | 2100 |  |  |  |  |  | 3000 |  |
| coruwms | Naval Metal Storage | 1 | metal storage | corch | 340 | 590 | 2920 | 2100 |  |  |  |  |  | 3000 | submerged, on_water |
| cornecro | Graverobber | 1 | resurrection bot | corlab | 130 | 1400 | 2800 | 220 | 78 | 200 |  |  |  |  | resurrect, stealth |
| correcl | Death Cavalry | 1 | resurrection submarine | corsy | 240 | 3500 | 6200 | 420 | 66 | 150 |  |  |  |  | resurrect, submerged, sonar |
| corplat | Seaplane Platform | 1 | seaplane factory | coraca | 1400 | 5500 | 11800 | 2200 |  | 300 |  |  |  |  | submerged, sonar, on_water |
| corsy | Shipyard | 1 | shipyard | coracsub | 450 | 950 | 5100 | 4300 |  | 150 |  |  |  |  | submerged, on_water |
| corsolar | Solar Collector | 1 | solar collector | corca | 150 | 0 | 2800 | 355 |  |  | 20 |  |  |  |  |
| coradvsol | Advanced Solar Collector | 1 | solar collector | corca | 370 | 4000 | 8150 | 1200 |  |  | 80 |  |  |  |  |
| cortide | Tidal Generator | 1 | tidal generator | corch | 85 | 250 | 2100 | 395 |  |  |  |  |  |  | submerged, on_water |
| corvp | Vehicle Plant | 1 | vehicle factory | coracv | 570 | 1550 | 5650 | 3000 |  | 150 |  |  |  |  |  |
| corwin | Wind Turbine | 1 | wind generator | corca | 43 | 175 | 1680 | 220 |  |  |  |  |  |  |  |
| coraap | Advanced Aircraft Plant | 2 | aircraft factory | coraca | 2900 | 28000 | 32000 | 3900 |  | 600 |  |  |  |  |  |
| corbhmth | Cerberus | 2 | armed geothermal plant | coraca | 3100 | 35000 | 59600 | 8300 |  |  | 450 |  |  |  | needs_energy_to_fire |
| cormexp | Advanced Exploiter | 2 | armed metal extractor | coraca | 2400 | 12000 | 32500 | 7800 |  |  |  | 20 |  |  | needs_energy_to_fire |
| coralab | Advanced Bot Lab | 2 | bot factory | corack | 2600 | 16000 | 26000 | 4500 |  | 600 |  |  |  |  |  |
| coraca | Advanced Construction Aircraft | 2 | constructor aircraft | coraap | 360 | 11000 | 22000 | 205 | 181.5 | 130 | 10 |  |  |  | flies |
| corfast | Twitcher | 2 | constructor bot | coralab | 210 | 3800 | 8000 | 830 | 90 | 125 | 15 |  |  |  |  |
| corack | Advanced Construction Bot | 2 | constructor bot | coralab | 470 | 6900 | 12500 | 1000 | 33 | 220 | 14 |  |  |  |  |
| cormls | Pathfinder | 2 | constructor ship | corasy | 200 | 3300 | 6500 | 1600 | 57.8 | 200 |  |  |  |  | floats |
| coracsub | Advanced Construction Sub | 2 | constructor submarine | corasy | 700 | 9000 | 23000 | 1110 | 64.8 | 350 | 30 |  |  |  | submerged |
| coracv | Advanced Construction Vehicle | 2 | constructor vehicle | coravp | 580 | 7000 | 17000 | 2150 | 49.5 | 310 | 20 |  |  |  |  |
| cordecom | Commander | 2 | decoy commander | coralab | 750 | 13000 | 33000 | 3700 | 37.5 | 300 | 15 |  |  |  | amphibious, anti_air, sonar |
| cormmkr | Advanced Energy Converter | 2 | energy converter | coraca | 370 | 21000 | 31300 | 560 |  |  |  |  |  |  |  |
| coruwmmm | Naval Advanced Energy Converter | 2 | energy converter | coracsub | 370 | 21000 | 31300 | 560 |  |  |  |  |  |  | on_water |
| coruwadves | Hardened Energy Storage | 2 | energy storage | coraca | 840 | 11000 | 20400 | 12700 |  |  |  |  | 40000 |  |  |
| corfus | Fusion Reactor | 2 | fusion reactor | coraca | 3600 | 22000 | 59000 | 4300 |  |  | 850 |  | 2500 |  |  |
| coruwfus | Naval Fusion Reactor | 2 | fusion reactor | coracsub | 5400 | 34000 | 105000 | 5900 |  |  | 1220 |  | 2500 |  | submerged, on_water |
| corafus | Advanced Fusion Reactor | 2 | fusion reactor | coraca | 9700 | 48000 | 329200 | 9800 |  |  | 3000 |  | 9000 |  |  |
| corageo | Advanced Geothermal Powerplant | 2 | geothermal plant | coraca | 1500 | 27000 | 48000 | 4150 |  |  | 1250 |  | 12000 |  |  |
| coruwageo | Advanced Geothermal Powerplant | 2 | geothermal plant | coraca | 1500 | 27000 | 48000 | 4150 |  |  | 1250 |  | 12000 |  | submerged, on_water |
| cormoho | Advanced Metal Extractor | 2 | metal extractor | coraca | 640 | 8100 | 14100 | 3900 |  |  |  | 20 |  |  |  |
| coruwmme | Naval Advanced Metal Extractor | 2 | metal extractor | coracsub | 640 | 8100 | 14100 | 3900 |  |  |  | 20 |  |  | submerged, on_water |
| coruwadvms | Hardened Metal Storage | 2 | metal storage | coraca | 760 | 11000 | 20500 | 11200 |  |  |  |  |  | 10000 |  |
| corasy | Advanced Shipyard | 2 | shipyard | coracsub | 2800 | 10000 | 24000 | 5900 |  | 600 |  |  |  |  | submerged, on_water |
| coravp | Advanced Vehicle Plant | 2 | vehicle factory | coracv | 2600 | 16000 | 28000 | 5100 |  | 600 |  |  |  |  |  |
| corgantuw | Experimental Gantry | 3 | experimental gantry | coracsub | 8350 | 62500 | 68000 | 17800 |  | 1800 |  |  | 1400 |  | submerged, gantry, on_water |
| corgant | Experimental Gantry | 3 | experimental gantry | coraca | 8400 | 62000 | 68000 | 17800 |  | 1800 |  |  | 1400 |  | gantry |

### Legion: economy (60 units)

<!--meta
id: F-units-economy-legion
layer: fact
ids: [F-unit-legaap, F-unit-legaca, F-unit-legaceb, F-unit-legack, F-unit-legacv, F-unit-legadveconv, F-unit-legadvestore, F-unit-legadvshipyard, F-unit-legadvsol, F-unit-legafcv, F-unit-legafus, F-unit-legageo, F-unit-legalab, F-unit-legamphlab, F-unit-legamstor, F-unit-leganavaladvgeo, F-unit-leganavaleconv, F-unit-leganavalfusion, F-unit-leganavalmex, F-unit-leganavyconsub, F-unit-leganavyengineer, F-unit-legap, F-unit-legavp, F-unit-legca, F-unit-legch, F-unit-legck, F-unit-legcom, F-unit-legcv, F-unit-legdecom, F-unit-legeconv, F-unit-legestor, F-unit-legfeconv, F-unit-legfhp, F-unit-legfus, F-unit-leggant, F-unit-leggantuw, F-unit-leggeo, F-unit-leghp, F-unit-leglab, F-unit-legmex, F-unit-legmoho, F-unit-legmohocon, F-unit-legmstor, F-unit-legnanotc, F-unit-legnanotcplat, F-unit-legnavyconship, F-unit-legnavyrezsub, F-unit-legotter, F-unit-legrampart, F-unit-legrezbot, F-unit-legsolar, F-unit-legspcon, F-unit-legsplab, F-unit-legsy, F-unit-legtide, F-unit-leguwestore, F-unit-leguwgeo, F-unit-leguwmstore, F-unit-legvp, F-unit-legwin]
grounding: crates/bot/data/units.json at game commit 54199a0d3f9a3aadd18cc731a44f6bba6e0f92aa (run/unit_stats.py)
generated: run/fact_units.py
status: live
-->
| unit | name | tier | class | from | metal | energy | build time | health | speed | build power | energy make | energy upkeep | e storage | m storage | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| legap | Legion Drone Plant | 1 | aircraft factory | legaca | 430 | 1100 | 6380 | 1780 |  | 150 |  |  |  |  |  |
| legamphlab | Amphibious Complex | 1 | amphibious factory | legch | 1200 | 5600 | 11400 | 2800 |  | 150 |  |  |  |  | submerged, on_water |
| leglab | Legion Bot Lab | 1 | bot factory | legaceb | 470 | 1050 | 5000 | 2900 |  | 150 |  |  |  |  |  |
| legcom | Legion Commander | 1 | commander |  | 2700 | 26000 | 75000 | 3700 | 37.5 | 300 | 30 |  |  |  | cloak, amphibious, anti_air, sonar |
| legnanotc | Construction Turret | 1 | construction turret | legaceb | 230 | 3200 | 5300 | 560 |  | 200 |  |  |  |  |  |
| legnanotcplat | Naval Construction Turret | 1 | construction turret | leganavyengineer | 230 | 3200 | 5300 | 560 |  | 200 |  |  |  |  | on_water |
| legca | Legion Construction Aircraft | 1 | constructor aircraft | legap | 100 | 2200 | 5000 | 151 | 139 | 45 | 5 |  |  |  | flies |
| legck | Legion Construction Bot | 1 | constructor bot | legaceb | 100 | 1600 | 3250 | 700 | 39 | 75 | 5 |  |  |  |  |
| legch | Construction Hovercraft | 1 | constructor hovercraft | legfhp | 185 | 2600 | 4360 | 1390 | 72.9 | 105 | 11 |  |  |  | hover, floats |
| legspcon | Construction Seaplane | 1 | constructor seaplane | legsplab | 150 | 5100 | 8500 | 405 | 124 | 75 | 20 |  |  |  | flies |
| legnavyconship | Construction Ship | 1 | constructor ship | legaceb | 180 | 2000 | 3500 | 1080 | 71 | 115 | 5 |  |  |  | floats |
| legcv | Legion Construction Vehicle | 1 | constructor vehicle | legvp | 125 | 2100 | 3900 | 1330 | 57 | 85 | 7 |  |  |  |  |
| legotter | Otter | 1 | constructor vehicle | legamphlab | 150 | 3100 | 6540 | 950 | 43.5 | 75 | 8 |  |  |  | amphibious |
| legeconv | Energy Converter | 1 | energy converter | legca | 1 | 1150 | 2600 | 167 |  |  |  |  |  |  |  |
| legfeconv | Naval Energy Converter | 1 | energy converter | legch | 1 | 1250 | 2680 | 167 |  |  |  |  |  |  | submerged, on_water |
| legestor | Energy Storage | 1 | energy storage | legca | 175 | 1800 | 4260 | 2000 |  |  |  |  | 6000 |  |  |
| leguwestore | Naval Energy Storage | 1 | energy storage | legch | 175 | 1800 | 4260 | 2000 |  |  |  |  | 6000 |  | submerged, on_water |
| leguwgeo | Offshore Geothermal Powerplant | 1 | geothermal plant | legca | 540 | 13000 | 12900 | 2050 |  |  | 300 |  | 1000 |  | submerged, on_water |
| leggeo | Geothermal Powerplant | 1 | geothermal plant | legca | 560 | 13000 | 12900 | 2050 |  |  | 300 |  | 1000 |  |  |
| legfhp | Offshore Hovercraft Platform | 1 | hovercraft factory | legch | 670 | 2000 | 8700 | 3750 |  | 150 |  |  |  |  | submerged, on_water |
| leghp | Hovercraft Platform | 1 | hovercraft factory | legca | 670 | 2000 | 8700 | 3750 |  | 150 |  |  |  |  |  |
| legmex | Metal Extractor | 1 | metal extractor | legaceb | 50 | 500 | 1880 | 275 |  |  |  | 3 |  |  |  |
| legmstor | Metal Storage | 1 | metal storage | legca | 340 | 590 | 2920 | 2100 |  |  |  |  |  | 3000 |  |
| leguwmstore | Naval Metal Storage | 1 | metal storage | legch | 340 | 590 | 2920 | 2100 |  |  |  |  |  | 3000 | submerged, on_water |
| legrezbot | Zagreus | 1 | resurrection bot | leglab | 130 | 1400 | 2800 | 220 | 78 | 200 |  |  |  |  | resurrect, stealth |
| legnavyrezsub | Dionysus | 1 | resurrection submarine | legsy | 240 | 3500 | 5960 | 420 | 66.9 | 150 |  |  |  |  | resurrect, submerged, sonar |
| legsplab | Offshore Seaplane Platform | 1 | seaplane factory | legaca | 1400 | 5500 | 11800 | 2200 |  | 300 |  |  |  |  | submerged, sonar, on_water |
| legsy | Shipyard | 1 | shipyard | leganavyconsub | 450 | 950 | 5100 | 4300 |  | 150 |  |  |  |  | submerged, on_water |
| legsolar | Solar Collector | 1 | solar collector | legaceb | 150 | 0 | 2800 | 340 |  |  | 20 |  |  |  |  |
| legadvsol | Advanced Solar Collector | 1 | solar collector | legca | 465 | 4080 | 12500 | 1100 |  |  | 100 |  |  |  |  |
| legtide | Tidal Generator | 1 | tidal generator | leganavyengineer | 85 | 250 | 2100 | 395 |  |  |  |  |  |  | submerged, on_water |
| legvp | Legion Vehicle Plant | 1 | vehicle factory | legacv | 570 | 1650 | 5700 | 3000 |  | 150 |  |  |  |  |  |
| legwin | Wind Turbine | 1 | wind generator | legafcv | 43 | 175 | 1680 | 230 |  |  |  |  |  |  |  |
| legaap | Legion Advanced Aircraft Plant | 2 | aircraft factory | legaca | 2900 | 28000 | 31050 | 3900 |  | 600 |  |  |  |  |  |
| legmohocon | Advanced Metal Fortifier | 2 | armed metal extractor | legaca | 1060 | 14500 | 19400 | 3900 |  | 400 |  | 20 |  |  |  |
| legalab | Legion Advanced Bot Lab | 2 | bot factory | legack | 2600 | 16000 | 25200 | 4500 |  | 600 |  |  |  |  |  |
| legaca | Advanced Construction Aircraft | 2 | constructor aircraft | legaap | 330 | 11000 | 20000 | 195 | 203 | 115 | 10 |  |  |  | flies |
| legaceb | Proteus | 2 | constructor bot | legalab | 310 | 5600 | 9500 | 830 | 75 | 125 | 15 |  |  |  |  |
| legack | Advanced Construction Bot | 2 | constructor bot | legalab | 410 | 6900 | 11500 | 860 | 36 | 195 | 14 |  |  |  |  |
| leganavyengineer | Artifex | 2 | constructor ship | legadvshipyard | 200 | 3300 | 4800 | 1600 | 57.8 | 200 |  |  |  |  | floats |
| leganavyconsub | Advanced Construction Submarine | 2 | constructor submarine | legadvshipyard | 700 | 9000 | 23000 | 1110 | 64.8 | 345 | 30 |  |  |  | submerged |
| legafcv | Aceso | 2 | constructor vehicle | legavp | 200 | 2900 | 4000 | 450 | 110 | 120 | 10 |  |  |  |  |
| legacv | Advanced Construction Vehicle | 2 | constructor vehicle | legavp | 530 | 6600 | 15000 | 1950 | 55.5 | 270 | 20 |  |  |  |  |
| legdecom | Legion Commander | 2 | decoy commander | legalab | 750 | 13000 | 32600 | 3700 | 37.5 | 300 | 15 |  |  |  | amphibious, anti_air, sonar |
| legadveconv | Advanced Energy Converter | 2 | energy converter | legaca | 370 | 21000 | 31300 | 560 |  |  |  |  |  |  |  |
| leganavaleconv | Naval Advanced Energy Converter | 2 | energy converter | leganavyconsub | 370 | 21000 | 31300 | 560 |  |  |  |  |  |  | on_water |
| legadvestore | Hardened Energy Storage | 2 | energy storage | legaca | 840 | 11000 | 20400 | 12700 |  |  |  |  | 40000 |  |  |
| legfus | Fusion Reactor | 2 | fusion reactor | legaca | 4000 | 25000 | 66000 | 4600 |  |  | 950 |  | 2500 |  |  |
| leganavalfusion | Naval Fusion Reactor | 2 | fusion reactor | leganavyconsub | 5400 | 34000 | 105000 | 5900 |  |  | 1220 |  | 2500 |  | submerged, on_water |
| legafus | Advanced Fusion Reactor | 2 | fusion reactor | legaca | 10500 | 69000 | 340000 | 9800 |  |  | 3300 |  | 9000 |  |  |
| legrampart | Rampart | 2 | geothermal anti-nuke platform | legaca | 2600 | 38000 | 36000 | 8600 |  |  | 600 |  | 4000 |  | radar_jammer |
| leganavaladvgeo | Advanced Underwater Geothermal Powerplant | 2 | geothermal plant | legaca | 1500 | 27000 | 32000 | 4150 |  |  | 1250 |  | 12000 |  | submerged, on_water |
| legageo | Advanced Geothermal Powerplant | 2 | geothermal plant | legaca | 1600 | 27000 | 49950 | 4150 |  |  | 1250 |  | 12000 |  |  |
| leganavalmex | Advanced Underwater Metal Extractor | 2 | metal extractor | leganavyconsub | 640 | 8100 | 14100 | 3900 |  |  |  | 20 |  |  | submerged, on_water |
| legmoho | Advanced Metal Extractor | 2 | metal extractor | legaca | 640 | 8100 | 14100 | 3900 |  |  |  | 20 |  |  |  |
| legamstor | Hardened Metal Storage | 2 | metal storage | legaca | 760 | 11000 | 20500 | 11200 |  |  |  |  |  | 10000 |  |
| legadvshipyard | Advanced Shipyard | 2 | shipyard | leganavyconsub | 2800 | 10000 | 23550 | 5900 |  | 600 |  |  |  |  | submerged, on_water |
| legavp | Advanced Vehicle Plant | 2 | vehicle factory | legacv | 2500 | 16000 | 27750 | 5100 |  | 600 |  |  |  |  |  |
| leggant | Experimental Gantry | 3 | experimental gantry | legaca | 8400 | 62000 | 67300 | 17800 |  | 1800 |  |  | 1400 |  | gantry |
| leggantuw | Experimental Gantry | 3 | experimental gantry | leganavyconsub | 8400 | 62000 | 67300 | 17800 |  | 1800 |  |  | 1400 |  | submerged, gantry, on_water |
