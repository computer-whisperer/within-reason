### The late-game frontier: Quicksilver Remake 1.24

22 spots a side (44 on the map), the arm side, from the recorded game `1790451669-onepass-player-nopenalty-3`; the build-order simulator's best orders over 30 minutes (`run/frontier_set.py`). **A ceiling, not a target:** no enemy, nothing dies, every spot on our half is free, and the simulator runs 15 to 25 % high on income by minute 8 against quiet recorded openings. The tax is the enemy it knows: metal a second taken for soldiers before any build, from each time. Standard: 3 a second from 2:00, 5 from 4:00, 9 from 8:00, 12 from 12:00, 15 from 16:00 (the experienced players' standing army on Comet grows at that rate, before losses); heavy: one and a half times that. The arithmetic the frontier rests on is the engine's (a frame builds at the sum of the power on it until the store empties; a stockpile at its fixed rate, helped by nobody: K-t2-build-power-for-a-project-follows-the-spend, K-rules-a-stockpile-takes-no-build-power).

| the fastest order to | no tax | standard tax | heavy tax |
|---|---|---|---|
| income 50 a second | 8:39 | 9:10 | 9:06 |
| income 100 a second | not by 30:00 | not by 30:00 | 24:47 |
| income 200 a second | not by 30:00 | not by 30:00 | not by 30:00 |
| the first tier-2 extractor | 6:43 | 7:20 | 7:58 |
| the first fusion | 7:58 | 9:48 | 9:51 |
| the first advanced fusion | not by 30:00 | not by 30:00 | 17:00 |
| the first anti-nuke | 8:01 | 9:02 | 9:58 |
| the first missile | 14:16 | 15:40 | 17:14 |

**The nuke line against BARb hard_aggressive**, whose anti-nuke stands at minute 21 to 25 in 27 of 33 long games (K-barb-anti-nuke-by-minute; medium builds one in one long game of six): none: 14:16, inside his anti-nuke's window; standard: 15:40, inside his anti-nuke's window; heavy: 17:14, inside his anti-nuke's window. A missile is energy flow, 1,042 a second for its 120 s, not build power.

**What stands at each milestone under the standard tax** (the state at the last whole minute before it).

| milestone | at | extractors | constructors | turrets | build power | energy a second | income | army metal paid | metal wasted |
|---|---|---|---|---|---|---|---|---|---|
| income 50 a second | 9:10 | 21 | 3 | 1 | 910 | 676 | 48 | 2100 | 2539 |
| income 100 a second | not by 30:00 | 21 | 3 | 5 | 1870 | 1629 | 65 | 19200 | 23 |
| income 200 a second | not by 30:00 | 21 | 3 | 5 | 1870 | 1629 | 65 | 19200 | 23 |
| the first tier-2 extractor | 7:20 | 13 | 3 | 1 | 1620 | 303 | 26 | 1260 | 7 |
| the first fusion | 9:48 | 13 | 3 | 0 | 1420 | 211 | 28 | 2100 | 7 |
| the first advanced fusion | not by 30:00 | 21 | 6 | 1 | 2250 | 2180 | 130 | 19200 | 34129 |
| the first anti-nuke | 9:02 | 18 | 4 | 2 | 1900 | 463 | 36 | 2100 | 7 |
| the first missile | 15:40 | 21 | 5 | 3 | 2180 | 1160 | 40 | 5880 | 7 |

**The orders under the standard tax**, one queue a line (`com` the commander, `facN` the factories as they stand, `conN` the constructors as they leave; `help` puts the builder's power on the dearest frame begun; `assist` on the first factory; `amex` the tier-2 extractor on a standing one's spot).

- **income 50 a second** (9:10): `com: mex mex solar win solar vp mex solar solar mex makr mex solar makr fmkr uwes mex mex win makr mex mex` · `fac0: beaver cv cv` · `con0: win nanotc mex fmkr advsol advsol advsol advsol advsol advsol makr makr fmkr plat` · `con1: mex mex@6672,1056 mex mex advsol advsol` · `con2: mex mex mex mex mex mex mex makr`
- **income 100 a second** (not by 30:00): `com: solar win mex mex mex solar fhp win` · `fac0: ch` · `fac1: ck ck` · `con0: mex fmkr advsol advsol nanotcplat makr fmkr mstor win makr fmkr advsol makr mex solar advsol nanotcplat mex fmkr mex mex makr fmkr uwms makr advsol advsol nanotc mstor fmkr fmkr makr win fmkr makr advsol assist mstor lab mex advsol mex win makr uwes solar advsol mex uwms advsol advsol advsol makr mex mex mex mex` · `con1: nanotc nanotc mex solar win advsol advsol advsol mex makr win makr` · `con2: advsol makr advsol win mex mex makr win mex mex`
- **income 200 a second** (not by 30:00): `com: solar win mex mex mex solar fhp win` · `fac0: ch` · `fac1: ck ck` · `con0: mex fmkr advsol advsol nanotcplat makr fmkr mstor win makr fmkr advsol makr mex solar advsol nanotcplat mex fmkr mex mex makr fmkr uwms makr advsol advsol nanotc mstor fmkr fmkr makr win fmkr makr advsol assist mstor lab mex advsol mex win makr uwes solar advsol mex uwms advsol advsol advsol makr mex mex mex mex` · `con1: nanotc nanotc mex solar win advsol advsol advsol mex makr win makr` · `con2: advsol makr advsol win mex mex makr win mex mex`
- **the first tier-2 extractor** (7:20): `com: mex solar lab mex solar solar mex solar mex mex mex mex mex@2176,2208 mex mex solar win win` · `fac0: ck ck` · `fac1: ack` · `con0: mex@5136,1408 mex solar mex mstor advsol mex makr mex mex nanotc mex` · `con1: advsol nanotc solar alab mex nanotc win mex mex makr makr help makr makr mex` · `con2: moho afus moho moho moho moho moho moho moho moho moho`
- **the first fusion** (9:48): `com: lab solar solar mex mex@3376,3328 mex mex mex mex mex@2192,880 mex mex win` · `fac0: ck ck` · `fac1: ack` · `con0: mex@5888,2112 alab mex help mex@4336,1488 nanotc solar mex mex` · `con1: solar advsol help mex mex help nanotc makr mex@6096,3280 mex mex mex mex` · `con2: fus fus mmkr mmkr mmkr fus moho fus fus fus fus fus fus moho fus`
- **the first advanced fusion** (not by 30:00): `com: solar lab solar mex@2192,880 mex solar` · `fac0: ck ck` · `fac1: ack ack ack ack` · `con0: mex mex makr advsol advsol makr advsol mex mex amex solar mex mex assist makr makr makr nanotc mex mex mex mex` · `con1: mex@6224,592 advsol mex alab solar mex mex makr mex mex mex advsol mex advsol` · `con3: moho ckfus moho moho moho moho moho moho` · `con4: mmkr fus mmkr moho` · `con5: moho moho`
- **the first anti-nuke** (9:02): `com: mex solar lab win solar mex mex@2976,2880 mex@3376,3328 mex@4640,1744 mex mex mex` · `fac0: ck ck ck` · `fac1: ack` · `con0: mex mex help nanotc win advsol nanotc mex mex assist assist` · `con1: win advsol estor advsol mex advsol mex@2192,880 mex mex mex` · `con2: mex@6096,3280 mex alab mex makr nanotc mex` · `con3: amd ckfus mmkr amd amd amd fus amd amd moho amd amd`
- **the first missile** (15:40): `com: win lab solar win mex mex mex mex@6672,1056 mex` · `fac0: ck ck ck ck` · `fac1: ack` · `con0: solar mex mex nanotc advsol advsol win mex mex` · `con1: mex@3264,592 mex mex mex mex mex mex` · `con2: mex solar nanotc alab help help` · `con3: advsol mex mex mex mex nanotc` · `con4: fus silo fus moho moho mmkr fus silo moho`
