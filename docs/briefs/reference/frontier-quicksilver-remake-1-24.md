### The late-game frontier: Quicksilver Remake 1.24

22 spots a side (44 on the map), the arm side, from the recorded game `1790451669-onepass-player-nopenalty-3`; the build-order simulator's best orders over 30 minutes (`run/frontier_set.py`). **A ceiling, not a target:** no enemy, nothing dies, every spot on our half is free, and the simulator runs 15 to 25 % high on income by minute 8 against quiet recorded openings. The tax is the enemy it knows: metal a second taken for soldiers before any build, from each time. Standard: 3 a second from 2:00, 5 from 4:00, 9 from 8:00, 12 from 12:00, 15 from 16:00 (the experienced players' standing army on Comet grows at that rate, before losses); heavy: one and a half times that. The arithmetic the frontier rests on is the engine's (a frame builds at the sum of the power on it until the store empties; a stockpile at its fixed rate, helped by nobody: K-t2-build-power-for-a-project-follows-the-spend, K-rules-a-stockpile-takes-no-build-power). The search is stochastic (simulated annealing, 20,000 steps x 8 restarts a cell, the reach rows from two seeds); a cell a heavier tax beat was searched again from other seeds (1 cells improved so). A cell still out of order is the search's noise, not the game's.

| the fastest order to | no tax | standard tax | heavy tax |
|---|---|---|---|
| income 50 a second | 9:05 | 9:39 | 11:11 |
| income 100 a second | 12:50 | 16:42 | 16:33 |
| income 200 a second | 28:17 | not by 30:00 | not by 30:00 |
| the first tier-2 extractor | 7:42 | 7:28 | 8:21 |
| the first fusion | 8:10 | 8:40 | 9:31 |
| the first advanced fusion | not by 30:00 | not by 30:00 | not by 30:00 |
| the first anti-nuke | 8:21 | 9:03 | 9:34 |
| the first missile | 14:43 | 15:30 | 17:08 |

**The nuke line against BARb hard_aggressive**, whose anti-nuke stands at minute 21 to 25 in 27 of 33 long games (K-barb-anti-nuke-by-minute; medium builds one in one long game of six): none: 14:43, inside his anti-nuke's window; standard: 15:30, inside his anti-nuke's window; heavy: 17:08, inside his anti-nuke's window. A missile is energy flow, 1,042 a second for its 120 s, not build power.

**What stands at each milestone under the standard tax** (the state at the last whole minute before it).

| milestone | at | extractors | constructors | turrets | build power | energy a second | income | army metal paid | metal wasted |
|---|---|---|---|---|---|---|---|---|---|
| income 50 a second | 9:39 | 16 | 4 | 1 | 1760 | 388 | 39 | 2100 | 7 |
| income 100 a second | 16:42 | 21 | 6 | 0 | 1850 | 756 | 94 | 6600 | 8132 |
| income 200 a second | not by 30:00 | 21 | 3 | 1 | 1850 | 572 | 158 | 19200 | 73006 |
| the first tier-2 extractor | 7:28 | 18 | 5 | 2 | 1980 | 401 | 38 | 1260 | 7 |
| the first fusion | 8:40 | 19 | 6 | 2 | 2060 | 352 | 41 | 1560 | 7 |
| the first advanced fusion | not by 30:00 | 21 | 4 | 1 | 1700 | 605 | 124 | 19200 | 61541 |
| the first anti-nuke | 9:03 | 12 | 3 | 1 | 1620 | 389 | 26 | 2100 | 7 |
| the first missile | 15:30 | 21 | 5 | 2 | 1980 | 1025 | 44 | 5880 | 7 |

**The orders under the standard tax**, one queue a line (`com` the commander, `facN` the factories as they stand, `conN` the constructors as they leave; `help` puts the builder's power on the dearest frame begun; `assist` on the first factory; `amex` the tier-2 extractor on a standing one's spot).

- **income 50 a second** (9:39): `com: win mex@3904,2160 lab win solar solar mex win mex@5264,2880 mex mex mex makr makr mex mex` · `fac0: ck ck` · `fac1: fark ack` · `con0: mex advsol advsol nanotc advsol mex mex mstor estor makr advsol help` · `con1: mex mex alab mex mex mex solar help solar advsol advsol advsol` · `con2: mex mex mex mex makr assist mex` · `con3: moho moho moho moho moho moho moho moho moho moho moho moho moho moho fus`
- **income 100 a second** (16:42): `com: solar mex win lab solar win mex@4096,2848 mex@6096,3280 mex fmkr` · `fac0: ck ck ck` · `fac1: fark ack ack` · `con0: mex mex assist advsol alab mex mex advsol help advsol` · `con1: mex@5264,2880 solar advsol mex mex advsol makr advsol advsol` · `con2: mex help mex@3264,592 mex@6672,1056 mex makr` · `con3: solar mex mex mex mex mex mex` · `con4: moho moho moho moho moho moho moho moho moho moho fus` · `con5: moho moho moho moho moho moho moho moho moho moho moho`
- **income 200 a second** (not by 30:00): `com: solar solar mex solar mex mex win mex solar solar solar mex@3376,3328 win sy llt estor mex win solar estor mex@3264,592 solar mex` · `fac0: cs cs` · `fac1: antiship acsub` · `con0: mex asy uwes nanotcplat mex uwms` · `con1: mex mex mex mex mex uwes mex mex mex mex mex mex` · `con2: uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme uwmme`
- **the first tier-2 extractor** (7:28): `com: mex mex solar lab win solar mex solar win win mex mex mex mex@2032,1072 mex mex win` · `fac0: ck ck ck ck` · `fac1: ack ack` · `con0: win mex@2352,2416 mex mex nanotc win solar alab assist win` · `con1: assist makr nanotc estor makr advsol mex mex makr` · `con2: win advsol mex mex` · `con3: mex@5136,1408 mex mex mex mex` · `con4: moho ckfus moho moho mmkr moho moho moho moho moho moho moho moho` · `con5: moho moho moho moho moho moho moho moho fus moho moho`
- **the first fusion** (8:40): `com: mex win mex lab solar solar solar mex mex help mex@3264,592` · `fac0: ck ck ck ck ck` · `fac1: ack` · `con0: mex@2192,880 advsol alab` · `con1: mex mex mex mex mex mex mex` · `con2: mex mex mex mex mex` · `con3: advsol nanotc solar makr mex` · `con4: solar mex nanotc mex llt win` · `con5: fus moho moho moho moho mmkr moho moho moho moho moho moho moho moho moho moho`
- **the first advanced fusion** (not by 30:00): `com: solar solar mex lab solar mex` · `fac0: ck ck ck` · `fac1: ack` · `con0: mex advsol advsol alab mex makr mstor mstor makr mex mex` · `con1: mex estor mex advsol help advsol advsol advsol mex mex mex` · `con2: mex mex mex mex mex mex nanotc makr mex mex mex mex` · `con3: moho moho moho moho moho moho moho moho moho moho moho moho moho moho`
- **the first anti-nuke** (9:03): `com: mex mex win win lab solar mex@4176,2288 solar win mex mex mex mex mex mex` · `fac0: ck ck` · `fac1: ack` · `con0: mex mex nanotc alab assist win help mex makr makr makr win mex@2032,1072 mex@2192,880 mex mex mex` · `con1: advsol advsol mex advsol assist mex@6224,592 nanotc estor win mex mex advsol` · `con2: amd fus mmkr moho moho moho moho moho moho moho moho moho moho moho moho moho`
- **the first missile** (15:30): `com: mex win lab solar solar mex mex` · `fac0: ck ck ck ck` · `fac1: ack` · `con0: solar mex mex nanotc advsol advsol mex mex mex mex` · `con1: mex@6096,3280 mex mex nanotc advsol advsol makr advsol estor advsol advsol` · `con2: mex@2336,3008 mex@1392,2880 mex mex mex alab makr advsol makr advsol advsol` · `con3: advsol mex mex mex mex` · `con4: silo fus moho ckfus moho moho moho moho moho silo moho`
