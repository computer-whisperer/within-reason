### The late-game frontier: Great Divide V1

11 spots a side (22 on the map), the arm side, from the recorded game `1791068837-seats-1-seat`; the build-order simulator's best orders over 30 minutes (`run/frontier_set.py`). **A ceiling, not a target:** no enemy, nothing dies, every spot on our half is free, and the simulator runs 15 to 25 % high on income by minute 8 against quiet recorded openings. The tax is the enemy it knows: metal a second taken for soldiers before any build, from each time. Standard: 3 a second from 2:00, 5 from 4:00, 9 from 8:00, 12 from 12:00, 15 from 16:00 (the experienced players' standing army on Comet grows at that rate, before losses); heavy: one and a half times that. The arithmetic the frontier rests on is the engine's (a frame builds at the sum of the power on it until the store empties; a stockpile at its fixed rate, helped by nobody: K-t2-build-power-for-a-project-follows-the-spend, K-rules-a-stockpile-takes-no-build-power).

| the fastest order to | no tax | standard tax | heavy tax |
|---|---|---|---|
| income 50 a second | 10:58 | 11:34 | 13:29 |
| income 100 a second | not by 30:00 | not by 30:00 | not by 30:00 |
| income 200 a second | not by 30:00 | not by 30:00 | not by 30:00 |
| the first tier-2 extractor | 6:03 | 6:44 | 8:06 |
| the first fusion | 7:58 | 8:57 | 9:44 |
| the first advanced fusion | 12:24 | not by 30:00 | not by 30:00 |
| the first anti-nuke | 7:58 | 8:15 | 9:23 |
| the first missile | 15:18 | 17:01 | 19:04 |

**The nuke line against BARb hard_aggressive**, whose anti-nuke stands at minute 21 to 25 in 27 of 33 long games (K-barb-anti-nuke-by-minute; medium builds one in one long game of six): none: 15:18, inside his anti-nuke's window; standard: 17:01, inside his anti-nuke's window; heavy: 19:04, inside his anti-nuke's window. A missile is energy flow, 1,042 a second for its 120 s, not build power.

**What stands at each milestone under the standard tax** (the state at the last whole minute before it).

| milestone | at | extractors | constructors | turrets | build power | energy a second | income | army metal paid | metal wasted |
|---|---|---|---|---|---|---|---|---|---|
| income 50 a second | 11:34 | 11 | 5 | 4 | 1890 | 1741 | 40 | 3180 | 7 |
| income 100 a second | not by 30:00 | 11 | 6 | 11 | 3460 | 4414 | 87 | 19200 | 31 |
| income 200 a second | not by 30:00 | 11 | 6 | 11 | 3460 | 4414 | 87 | 19200 | 31 |
| the first tier-2 extractor | 6:44 | 11 | 3 | 1 | 1490 | 295 | 25 | 960 | 7 |
| the first fusion | 8:57 | 11 | 3 | 1 | 1620 | 238 | 26 | 1560 | 7 |
| the first advanced fusion | not by 30:00 | 11 | 6 | 0 | 1920 | 1068 | 64 | 19200 | 31495 |
| the first anti-nuke | 8:15 | 11 | 4 | 2 | 1900 | 444 | 26 | 1560 | 7 |
| the first missile | 17:01 | 11 | 3 | 2 | 1820 | 2022 | 40 | 7500 | 123 |

**The orders under the standard tax**, one queue a line (`com` the commander, `facN` the factories as they stand, `conN` the constructors as they leave; `help` puts the builder's power on the dearest frame begun; `assist` on the first factory; `amex` the tier-2 extractor on a standing one's spot).

- **income 50 a second** (11:34): `com: mex win win lab win solar assist makr win solar win mex solar solar win win makr makr makr win` · `fac0: ck ck` · `fac1: ch ch ch` · `con0: mex mex mex advsol advsol win mex@600,128 mex mex makr makr makr advsol assist makr` · `con1: mex@672,1360 nanotc mex advsol hp advsol advsol makr advsol advsol nanotc makr advsol nanotc makr makr advsol makr makr` · `con2: fmkr advsol makr solar makr makr fmkr solar makr advsol makr advsol` · `con3: nanotc makr advsol solar win makr fmkr advsol assist advsol advsol advsol makr solar win win solar` · `con4: assist win assist mex`
- **income 100 a second** (not by 30:00): `com: fmkr solar mex solar solar solar solar makr win fmkr makr mex fmkr win mex win win hp mex@2832,384 llt@176,208 mex` · `fac0: ch ch` · `fac1: ch ch ch ch` · `con0: makr advsol makr win fmkr mstor advsol amex nanotcplat fmkr solar makr advsol amex solar solar fmkr advsol fmkr advsol nanotcplat fmkr advsol fmkr mstor amex assist solar advsol fmkr advsol advsol fmkr fmkr advsol fmkr nanotcplat advsol solar win makr makr nanotcplat nanotcplat nanotc win` · `con1: makr win advsol advsol advsol advsol makr fmkr advsol fmkr makr solar mex makr makr win win fmkr mstor advsol fmkr llt mstor solar mex nanotc win makr advsol advsol hp assist nanotc advsol advsol advsol makr advsol fmkr makr fmkr makr mex` · `con2: advsol fmkr llt makr fmkr advsol advsol solar makr fmkr makr solar makr fmkr makr estor makr win win` · `con3: fmkr advsol fmkr makr advsol win solar advsol estor nanotcplat advsol advsol mstor makr help solar advsol solar makr advsol makr advsol advsol` · `con4: win advsol advsol nanotc makr fmkr makr advsol makr advsol win fmkr fmkr advsol solar advsol nanotc fmkr solar advsol` · `con5: advsol makr fmkr assist advsol advsol mstor advsol solar makr`
- **income 200 a second** (not by 30:00): `com: fmkr solar mex solar solar solar solar makr win fmkr makr mex fmkr win mex win win hp mex@2832,384 llt@176,208 mex` · `fac0: ch ch` · `fac1: ch ch ch ch` · `con0: makr advsol makr win fmkr mstor advsol amex nanotcplat fmkr solar makr advsol amex solar solar fmkr advsol fmkr advsol nanotcplat fmkr advsol fmkr mstor amex assist solar advsol fmkr advsol advsol fmkr fmkr advsol fmkr nanotcplat advsol solar win makr makr nanotcplat nanotcplat nanotc win` · `con1: makr win advsol advsol advsol advsol makr fmkr advsol fmkr makr solar mex makr makr win win fmkr mstor advsol fmkr llt mstor solar mex nanotc win makr advsol advsol hp assist nanotc advsol advsol advsol makr advsol fmkr makr fmkr makr mex` · `con2: advsol fmkr llt makr fmkr advsol advsol solar makr fmkr makr solar makr fmkr makr estor makr win win` · `con3: fmkr advsol fmkr makr advsol win solar advsol estor nanotcplat advsol advsol mstor makr help solar advsol solar makr advsol makr advsol advsol` · `con4: win advsol advsol nanotc makr fmkr makr advsol makr advsol win fmkr fmkr advsol solar advsol nanotc fmkr solar advsol` · `con5: advsol makr fmkr assist advsol advsol mstor advsol solar makr`
- **the first tier-2 extractor** (6:44): `com: mex win lab win solar mex solar mex@1552,1504 mex fmkr win mex` · `fac0: ck ck ck` · `fac1: ack` · `con0: mex mex mex mex` · `con1: mex win solar win advsol alab` · `con2: nanotc win mex win` · `con3: moho ckfus moho moho moho mmkr moho moho moho moho moho moho moho`
- **the first fusion** (8:57): `com: mex win lab win solar mex@600,128 mex mex@1088,696 mex estor solar makr win mex` · `fac0: ck ck` · `fac1: ack` · `con0: win mex mex mex nanotc alab` · `con1: advsol mex mex` · `con2: fus mmkr mmkr moho fus fus fus mmkr fus fus fus fus fus`
- **the first advanced fusion** (not by 30:00): `com: assist mex win solar solar lab mex@2520,464` · `fac0: ck ck ck` · `fac1: ack ack ack` · `con0: win mex solar mex solar solar estor mex win mex` · `con1: mex@176,208 mex estor makr assist mex mex mex` · `con2: estor advsol alab` · `con3: moho moho` · `con4: ckfus moho mmkr moho`
- **the first anti-nuke** (8:15): `com: mex win lab win solar win mex mex mex mex mex mex` · `fac0: ck ck ck` · `fac1: ack` · `con0: win advsol nanotc win advsol` · `con1: mex@600,128 mex@416,392 mex alab win` · `con2: win mex advsol win nanotc` · `con3: amd moho afus mmkr amd amd amd amd amd`
- **the first missile** (17:01): `com: mex win lab win solar win mex win win mex@600,128 mex@176,208 mex mex@2832,384` · `fac0: ck ck` · `fac1: ack` · `con0: mex mex mex mex advsol advsol alab makr assist advsol solar` · `con1: nanotc win advsol makr win nanotc mex` · `con2: moho mmkr fus silo fus silo`
