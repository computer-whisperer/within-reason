## The late game by the simulator: the frontier

The build-order simulator's fastest order to each late-game milestone, over thirty minutes, on maps of 11, 22, 40 spots a side (`run/frontier_set.py`; every run's order and curve is in `docs/briefs/reference/frontier-<map>.md`). **A ceiling, not a target:** no enemy, nothing dies, every spot on our half is free, and the simulator runs 15 to 25 % high on income by minute 8 against quiet recorded openings. The one enemy it knows is the tax: metal a second taken for soldiers before any build. The standard tax is 3 a second from 2:00, 5 from 4:00, 9 from 8:00, 12 from 12:00 and 15 from 16:00 (the experienced players' standing army on Comet Catcher grows at that rate, before losses); the heavy tax is one and a half times that. The arithmetic is the engine's: a frame builds at the sum of the power on it until the store empties, and a stockpile advances at its fixed rate, helped by nobody (K-t2-build-power-for-a-project-follows-the-spend, K-rules-a-stockpile-takes-no-build-power). The search is stochastic (simulated annealing, 20,000 steps x 8 restarts a cell); a cell out of order, a heavier tax ahead of a lighter one, is the search's noise, not the game's. `never`: not by 30:00.

| the fastest order to (no tax / standard tax / heavy tax) | Great Divide V1 (11 spots a side) | Quicksilver Remake 1.24 (22 spots a side) | Comet Catcher Remake 1.8 (40 spots a side) |
|---|---|---|---|
| income 50 a second | 9:52 / 10:49 / 12:23 | 9:05 / 9:39 / 11:11 | 4:47 / 5:36 / 6:22 |
| income 100 a second | 13:53 / 15:02 / 17:44 | 12:50 / 16:42 / 16:33 | 10:05 / 10:59 / 11:03 |
| income 200 a second | never / never / never | 28:17 / never / never | 19:33 / never / 27:33 |
| the first tier-2 extractor | 6:20 / 6:32 / 7:16 | 7:42 / 7:28 / 8:21 | 7:31 / 8:14 / 8:49 |
| the first fusion | 7:38 / 8:39 / 9:50 | 8:10 / 8:40 / 9:31 | 7:37 / 8:20 / 11:34 |
| the first advanced fusion | 12:09 / never / never | never / never / never | never / 12:38 / 14:09 |
| the first anti-nuke | 8:19 / 8:36 / 9:40 | 8:21 / 9:03 / 9:34 | 8:06 / 9:13 / 10:09 |
| the first missile | 14:22 / 16:50 / 18:01 | 14:43 / 15:30 / 17:08 | 14:17 / 16:25 / 17:54 |

**The nuke line.** BARb hard_aggressive's anti-nuke stands at minute 21 to 25 in 27 of 33 long games on Comet Catcher (K-barb-anti-nuke-by-minute; medium builds one in one long game of six, at 25 at the median), so a missile that is to land on him is stockpiled before about 21:00. Under the standard tax the first missile comes at 16:50 on Great Divide V1, 15:30 on Quicksilver Remake 1.24, 16:25 on Comet Catcher Remake 1.8. A missile is energy flow, 1,042 a second for its 120 s, and no build power.

**What stood at the first missile under the standard tax** (the state at the last whole minute before it).

| map | missile | silo stood | advanced lab | first advanced constructor | first tier-2 extractor | first fusion | extractors | constructors | turrets | build power | energy a second | income | army metal paid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Great Divide V1 | 16:50 | 14:21 | 6:16 | 6:38 | 7:33 | 10:04 | 11 | 6 | 2 | 2190 | 1170 | 43 | 6600 |
| Quicksilver Remake 1.24 | 15:30 | 12:59 | 7:31 | 7:53 | 15:32 | 15:02 | 21 | 5 | 2 | 1980 | 1025 | 44 | 5880 |
| Comet Catcher Remake 1.8 | 16:25 | 14:11 | 6:56 | 7:18 | 8:16 | 15:35 | 20 | 5 | 3 | 2310 | 2022 | 66 | 6600 |

**The missile's orders under the standard tax**, one queue a line (`com` the commander, `facN` the factories as they stand, `conN` the constructors as they leave; `help` puts the builder's power on the dearest frame begun; `assist` on the first factory; `moho` the tier-2 extractor on a standing one's spot; a constructor whose queue has run out upgrades the nearest standing extractor).

- **Great Divide V1** (16:50): `com: win mex@1856,664 lab win solar mex@2272,992 win mex` · `fac0: ck ck ck ck` · `fac1: ack ack` · `con0: mex mex@2832,384 nanotc` · `con1: solar win mex@1552,1504 mex@672,1360 nanotc win` · `con2: mex@1088,696 win advsol advsol makr win win assist win` · `con3: mex@600,128 mex mex alab` · `con4: assist moho assist assist fus silo moho moho ckfus assist fus silo moho` · `con5: moho moho moho help fus silo mmkr moho moho moho assist moho`
- **Quicksilver Remake 1.24** (15:30): `com: mex win lab solar solar mex mex` · `fac0: ck ck ck ck` · `fac1: ack` · `con0: solar mex mex nanotc advsol advsol mex mex mex mex` · `con1: mex@6096,3280 mex mex nanotc advsol advsol makr advsol estor advsol advsol` · `con2: mex@2336,3008 mex@1392,2880 mex mex mex alab makr advsol makr advsol advsol` · `con3: advsol mex mex mex mex` · `con4: silo fus moho ckfus moho moho moho moho moho silo moho`
- **Comet Catcher Remake 1.8** (16:25): `com: lab solar solar mex@1056,5632 mex mex@1184,1776 mex` · `fac0: ck ck ck` · `fac1: ack ack` · `con0: mex@2656,4672 mex@2464,3808 mex advsol advsol solar mex mex help mex mex@4208,4352 mex@4416,3760 mex mex@640,944 mex estor mex mex` · `con1: win solar mex mex nanotc solar mex mex mex mex mex@3360,2656 nanotc help mex@3008,1952 makr llt makr makr mex advsol mex mex mex mex mex` · `con2: mex advsol solar alab mex help advsol nanotc mex estor mex@1984,736 help assist advsol mex makr mex mex mex mex` · `con3: moho ckfus moho moho moho moho moho moho silo moho` · `con4: silo fus moho moho moho moho silo moho`
