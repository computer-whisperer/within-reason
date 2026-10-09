### The late-game frontier: Comet Catcher Remake 1.8

40 spots a side (80 on the map), the arm side, from the recorded game `1791511650-rove-task-2`; the build-order simulator's best orders over 30 minutes (`run/frontier_set.py`). **A ceiling, not a target:** no enemy, nothing dies, every spot on our half is free, and the simulator runs 15 to 25 % high on income by minute 8 against quiet recorded openings. The tax is the enemy it knows: metal a second taken for soldiers before any build, from each time. Standard: 3 a second from 2:00, 5 from 4:00, 9 from 8:00, 12 from 12:00, 15 from 16:00 (the experienced players' standing army on Comet grows at that rate, before losses); heavy: one and a half times that. The arithmetic the frontier rests on is the engine's (a frame builds at the sum of the power on it until the store empties; a stockpile at its fixed rate, helped by nobody: K-t2-build-power-for-a-project-follows-the-spend, K-rules-a-stockpile-takes-no-build-power).

| the fastest order to | no tax | standard tax | heavy tax |
|---|---|---|---|
| income 50 a second | 5:20 | 5:30 | 6:20 |
| income 100 a second | 11:21 | 11:41 | 12:28 |
| income 200 a second | not by 30:00 | not by 30:00 | not by 30:00 |
| the first tier-2 extractor | 6:24 | 7:12 | 8:49 |
| the first fusion | 7:25 | 8:28 | 9:59 |
| the first advanced fusion | not by 30:00 | 17:48 | 15:08 |
| the first anti-nuke | 8:12 | 9:45 | 10:25 |
| the first missile | 12:39 | 16:00 | 16:19 |

**The nuke line against BARb hard_aggressive**, whose anti-nuke stands at minute 21 to 25 in 27 of 33 long games (K-barb-anti-nuke-by-minute; medium builds one in one long game of six): none: 12:39, inside his anti-nuke's window; standard: 16:00, inside his anti-nuke's window; heavy: 16:19, inside his anti-nuke's window. A missile is energy flow, 1,042 a second for its 120 s, not build power.

**What stands at each milestone under the standard tax** (the state at the last whole minute before it).

| milestone | at | extractors | constructors | turrets | build power | energy a second | income | army metal paid | metal wasted |
|---|---|---|---|---|---|---|---|---|---|
| income 50 a second | 5:30 | 19 | 2 | 0 | 630 | 130 | 44 | 660 | 891 |
| income 100 a second | 11:41 | 38 | 4 | 1 | 1160 | 673 | 92 | 3180 | 13000 |
| income 200 a second | not by 30:00 | 40 | 4 | 2 | 1790 | 1439 | 134 | 19200 | 39769 |
| the first tier-2 extractor | 7:12 | 11 | 2 | 1 | 1540 | 271 | 27 | 1260 | 7 |
| the first fusion | 8:28 | 15 | 3 | 1 | 1620 | 298 | 36 | 1560 | 7 |
| the first advanced fusion | 17:48 | 23 | 4 | 1 | 1830 | 1031 | 55 | 7500 | 871 |
| the first anti-nuke | 9:45 | 11 | 2 | 1 | 1540 | 331 | 26 | 2100 | 102 |
| the first missile | 16:00 | 23 | 2 | 2 | 1740 | 1754 | 53 | 6600 | 7 |

**The orders under the standard tax**, one queue a line (`com` the commander, `facN` the factories as they stand, `conN` the constructors as they leave; `help` puts the builder's power on the dearest frame begun; `assist` on the first factory; `amex` the tier-2 extractor on a standing one's spot).

- **income 50 a second** (5:30): `com: solar solar solar mex vp mex mex mex mex mex mex mex mex mex mex mex` · `fac0: cv cv` · `con0: mex mex amex mex mex mex win mex solar makr lab advsol makr advsol advsol makr llt makr advsol advsol nanotc amex mex mex mex mex mex mex makr mex mex mex mex mex mex` · `con1: solar mex@2656,4672 mex mex mex solar advsol advsol nanotc makr mex makr advsol solar makr advsol advsol nanotc makr mex@880,608 advsol estor advsol makr advsol makr makr solar llt mex mex`
- **income 100 a second** (11:41): `com: solar mex solar mex vp solar mex mex@3824,4272 makr solar` · `fac0: cv cv cv cv` · `con0: win mex advsol makr mex amex mex@1984,736 mex mex mex makr makr` · `con1: solar makr solar nanotc advsol mex llt advsol advsol lab makr makr advsol advsol mex` · `con2: mex@624,4160 mex mex mex mex mex mex mex mex mex@2480,2880 mex mex mex mex` · `con3: mex mex mex mex mex mex mex mex mex mex mex mex mex estor solar`
- **income 200 a second** (not by 30:00): `com: mex mex@1824,5584 solar solar mex solar solar estor ap solar mex@944,2368` · `fac0: ca ca` · `fac1: aca aca blade` · `con0: estor llt solar solar win mex solar win solar advsol solar mstor advsol advsol mex solar win mex mex@640,944 mex mex mex mex mex mex mex solar mex amex mex mex mex mex mex mex` · `con1: mex@3360,2656 solar mstor nanotc makr makr solar nanotc solar amex mex advsol mex@2608,2176 mex aap mex@1408,2800 makr makr mex mex mex mex mex mex mex mex mex mex mex` · `con2: moho ckfus moho moho mmkr mmkr`
- **the first tier-2 extractor** (7:12): `com: solar lab mex solar mex@1824,5584 mex@1056,5632 solar mex mex mex mex mex mex@2656,4672 mex mex` · `fac0: ck` · `fac1: ack` · `con0: advsol nanotc advsol alab mex@2480,2880 mex mex mex mex mex mex@1984,736 mex mex mex mex amex mex amex amex mex mex mex mex mex mex amex mex mex mex mex mex mex mex` · `con1: moho fus mmkr moho moho moho moho moho moho moho moho moho moho moho moho`
- **the first fusion** (8:28): `com: solar mex lab solar mex solar mex@1296,3776 mex mex mex mex mex@2464,3808 mex` · `fac0: ck ck` · `fac1: ack` · `con0: advsol mex advsol alab mex help makr amex mex mex mex@1936,1840 mex mex llt makr mex mex mex mex mex mex` · `con1: assist mex nanotc mex solar mex mex mex nanotc nanotc makr advsol advsol mex estor mex mex mex mex mex mex mex mex mex mex mex` · `con2: fus mmkr fus fus fus fus fus fus moho fus mmkr moho fus fus fus fus fus fus mmkr fus mmkr`
- **the first advanced fusion** (17:48): `com: solar lab solar mex win mex@2464,3808 mex@5200,5952 win` · `fac0: ck ck` · `fac1: ack ack` · `con0: mex advsol advsol mex win estor mex mex mex mex help makr advsol mex mex advsol advsol mex mex` · `con1: mex mex nanotc mex@2656,4672 alab mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex mex` · `con2: fus help mmkr moho assist moho mmkr moho` · `con3: afus mmkr mmkr moho mmkr mmkr`
- **the first anti-nuke** (9:45): `com: solar mex solar lab mex solar estor mex mex mex mex mex help solar mex@4208,4352 solar mex@3504,3584` · `fac0: ck` · `fac1: ack` · `con0: mex advsol nanotc advsol alab solar mex mex mex mex mex@3360,2656 mex mex mex mex amex mex mex@1984,736 mex mex mex mex mex mex mex mex mex mex mex mex mex mex solar mex` · `con1: amd fus amd moho amd amd moho amd amd moho mmkr`
- **the first missile** (16:00): `com: mex solar lab solar mex mex estor solar solar mex mex mex mex mex@2032,3360 mex mex mex mex@1408,2800 mex@1936,1840 win estor estor mex` · `fac0: ck` · `fac1: ack` · `con0: mex nanotc solar advsol assist solar alab nanotc mex mex mex@4272,5904 mex@2480,2880 mex@1984,736 mex mex mex mex mex amex mex mex mex mex mex mex mex mex mex mex mex mex mex mex` · `con1: assist fus silo fus moho silo fus mmkr moho moho ckfus`
