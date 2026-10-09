### The late-game frontier: Great Divide V1

11 spots a side (22 on the map), the arm side, from the recorded game `1791068837-seats-1-seat`; the build-order simulator's best orders over 30 minutes (`run/frontier_set.py`). **A ceiling, not a target:** no enemy, nothing dies, every spot on our half is free, and the simulator runs 15 to 25 % high on income by minute 8 against quiet recorded openings. The tax is the enemy it knows: metal a second taken for soldiers before any build, from each time. Standard: 3 a second from 2:00, 5 from 4:00, 9 from 8:00, 12 from 12:00, 15 from 16:00 (the experienced players' standing army on Comet grows at that rate, before losses); heavy: one and a half times that. The arithmetic the frontier rests on is the engine's (a frame builds at the sum of the power on it until the store empties; a stockpile at its fixed rate, helped by nobody: K-t2-build-power-for-a-project-follows-the-spend, K-rules-a-stockpile-takes-no-build-power). The search is stochastic (simulated annealing, 20,000 steps x 8 restarts a cell, the reach rows from two seeds); a cell a heavier tax beat was searched again from other seeds (0 cells improved so). A cell still out of order is the search's noise, not the game's.

| the fastest order to | no tax | standard tax | heavy tax |
|---|---|---|---|
| income 50 a second | 9:52 | 10:49 | 12:23 |
| income 100 a second | 13:53 | 15:02 | 17:44 |
| income 200 a second | not by 30:00 | not by 30:00 | not by 30:00 |
| the first tier-2 extractor | 6:20 | 6:32 | 7:16 |
| the first fusion | 7:38 | 8:39 | 9:50 |
| the first advanced fusion | 12:09 | not by 30:00 | not by 30:00 |
| the first anti-nuke | 8:19 | 8:36 | 9:40 |
| the first missile | 14:22 | 16:50 | 18:01 |

**The nuke line against BARb hard_aggressive**, whose anti-nuke stands at minute 21 to 25 in 27 of 33 long games (K-barb-anti-nuke-by-minute; medium builds one in one long game of six): none: 14:22, inside his anti-nuke's window; standard: 16:50, inside his anti-nuke's window; heavy: 18:01, inside his anti-nuke's window. A missile is energy flow, 1,042 a second for its 120 s, not build power.

**What stands at each milestone under the standard tax** (the state at the last whole minute before it).

| milestone | at | extractors | constructors | turrets | build power | energy a second | income | army metal paid | metal wasted |
|---|---|---|---|---|---|---|---|---|---|
| income 50 a second | 10:49 | 11 | 3 | 1 | 1720 | 339 | 45 | 2640 | 7 |
| income 100 a second | 15:02 | 11 | 5 | 2 | 2500 | 1500 | 98 | 5880 | 7 |
| income 200 a second | not by 30:00 | 11 | 4 | 2 | 2000 | 2488 | 129 | 19200 | 33367 |
| the first tier-2 extractor | 6:32 | 11 | 4 | 1 | 1700 | 284 | 23 | 960 | 7 |
| the first fusion | 8:39 | 11 | 3 | 1 | 1620 | 298 | 26 | 1560 | 34 |
| the first advanced fusion | not by 30:00 | 11 | 4 | 0 | 1500 | 500 | 100 | 19200 | 43668 |
| the first anti-nuke | 8:36 | 11 | 6 | 2 | 2060 | 453 | 26 | 1560 | 7 |
| the first missile | 16:50 | 11 | 6 | 2 | 2190 | 1170 | 43 | 6600 | 7 |

**The orders under the standard tax**, one queue a line (`com` the commander, `facN` the factories as they stand, `conN` the constructors as they leave; `help` puts the builder's power on the dearest frame begun; `assist` on the first factory; `amex` the tier-2 extractor on a standing one's spot).

- **income 50 a second** (10:49): `com: win win vp solar win mex mex@416,392 mex mex` · `fac0: cv cv` · `fac1: acv` · `con0: mex mex estor solar mex nanotc mex mex@672,1360 win mex makr` · `con1: mex@2520,464 advsol avp win makr win makr solar help solar solar` · `con2: moho moho moho moho fus mmkr ckfus mmkr mmkr fus moho moho moho moho moho moho moho`
- **income 100 a second** (15:02): `com: mex mex solar vp solar solar win win mex mex@176,208 help mex@600,128 makr solar makr solar fmkr win fmkr fmkr makr estor fmkr fmkr fmkr win makr fmkr` · `fac0: cv cv` · `fac1: acv acv acv` · `con0: mex mex mex avp mex@416,392 nanotc makr makr help advsol` · `con1: mex advsol advsol advsol advsol advsol mex@672,1360 help nanotc makr makr` · `con2: mmkr moho moho moho mmkr ckfus moho` · `con3: moho moho moho fus assist ckfus assist fus fus mmkr mmkr mmkr` · `con4: help moho moho moho moho fus mmkr mmkr ckfus mmkr ckfus`
- **income 200 a second** (not by 30:00): `com: win solar fmkr mex win mex makr solar solar vp mex solar` · `fac0: beaver cv cv` · `fac1: acv latnk` · `con0: mex fmkr makr advsol makr avp makr mstor solar nanotc advsol makr makr fmkr estor fmkr fmkr fmkr fmkr advsol fmkr nanotc makr fmkr fmkr win advsol advsol mstor solar fmkr makr makr makr advsol solar fmkr makr advsol fmkr advsol makr makr solar win advsol` · `con1: advsol mex mex advsol makr advsol mex advsol mex mex mex` · `con2: makr mex advsol makr makr advsol advsol advsol estor advsol makr` · `con3: moho moho moho moho moho ckfus moho moho moho moho moho moho`
- **the first tier-2 extractor** (6:32): `com: mex@1856,664 win lab win win solar mex mex mex mex win mex` · `fac0: ck ck ck` · `fac1: ack` · `con0: mex@1552,1504 nanotc advsol win alab` · `con1: win mex mex mex` · `con2: win win help mex` · `con3: moho fus moho moho moho moho mmkr fus mmkr moho moho moho moho moho moho amd`
- **the first fusion** (8:39): `com: lab win solar mex@416,392 mex mex mex mex solar solar win solar solar` · `fac0: ck ck` · `fac1: ack` · `con0: mex@2520,464 mex@2448,160 mex nanotc mex help` · `con1: mex win mex@1552,1504 win advsol alab` · `con2: fus mmkr moho moho moho moho moho mmkr ckfus moho moho moho moho moho moho`
- **the first advanced fusion** (not by 30:00): `com: mex solar lab solar mex` · `fac0: ck ck ck` · `fac1: ack` · `con0: mex@2832,384 makr mex solar advsol mex mex win` · `con1: solar advsol estor mex mex makr mex mex` · `con2: mex@600,128 advsol alab solar advsol` · `con3: assist moho moho moho moho assist moho moho moho moho moho moho moho`
- **the first anti-nuke** (8:36): `com: mex win lab win win solar mex@1088,696 mex@176,208` · `fac0: ck ck ck ck ck` · `fac1: ack` · `con0: help mex mex mex mex` · `con1: mex@600,128 mex nanotc makr makr` · `con2: solar advsol alab` · `con3: solar win advsol win advsol nanotc` · `con4: mex mex` · `con5: amd moho moho ckfus moho mmkr moho moho moho moho moho moho moho moho`
- **the first missile** (16:50): `com: win mex@1856,664 lab win solar mex@2272,992 win mex` · `fac0: ck ck ck ck` · `fac1: ack ack` · `con0: mex mex@2832,384 nanotc` · `con1: solar win mex@1552,1504 mex@672,1360 nanotc win` · `con2: mex@1088,696 win advsol advsol makr win win assist win` · `con3: mex@600,128 mex mex alab` · `con4: assist moho assist assist fus silo moho moho ckfus assist fus silo moho` · `con5: moho moho moho help fus silo mmkr moho moho moho assist moho`
