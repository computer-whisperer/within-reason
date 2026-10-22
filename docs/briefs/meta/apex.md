<!--meta
id: M-apex-intro
layer: meta
population: Apex (apexearth's fork of BARb with trained nets; its author played against us in the lobby 5v5 of 2026-10-18), versions v0.2.2 (2026-10-13), v0.2.4, v0.2.9 (2026-10-17), v0.3.0 (the lobby ally, 2026-10-18), v0.3.2 (2026-10-18), v0.3.3 (2026-10-20)
dates: 2026-10-13 to 2026-10-20
ground: truth (the arena's truth files), the author's own issue list, and the ledger
sources: {code: [docs/experiments.md, docs/studies/2026-10-18-distilled-apex-losses.md, run/opponent_set.py], claims: [K-opp-people-nuke-late-or-never]}
status: live
-->
## Apex

Apex is a fork of BARb's engine (CircuitAI) by the player apexearth, with his own scripts and trained nets choosing
openings and builds; a new version every few days, so every number below carries its version. It plays Cortex in our
games unless the lobby sets otherwise. It has been our opponent in 1v1s on Comet Catcher (the flagship twice, the
distilled bot in batteries of eight) and in 3v3s on Altored Divide with a bonus to it, and our ally once (three seats of
v0.3.0 beside our two in the lobby 5v5).

<!--meta
id: M-apex-opening
layer: meta
premises: [F-eco-energy, F-eco-multipliers]
population: Apex v0.2.2 (the opening read from its own log and the truth file, 2026-10-13), v0.2.9 and v0.3.3 (the two flagship 1v1s on Comet), v0.3.2 (seven 3v3s on Altored, +0 to +60), v0.3.0 (the lobby ally at +50)
map: Comet Catcher Remake 1.8; Altored Divide Bar Remake 1.6.2
dates: 2026-10-13 to 2026-10-20
ground: truth
sources: {code: [run/opponent_set.py, docs/experiments.md]}
status: live
-->
### The opening

At +0 it opens a bot lab at 0:26 to 0:40 (v0.2.9 at 0:40 on Comet, v0.3.2 at 0:26-0:38 on Altored, v0.3.3 at 1:14) after
two extractors and no energy: v0.2.2's log showed the rule (its net trusts the "two extractors then the lab" plan and
never picks the one plan with a generator), the lab idle from 0:44 to 1:28 for want of energy, the first constructor out
at 1:52 (ours 1:28), four solars in a row from 1:16, the third extractor at 2:28, and its commander stopping fifteen
seconds with a job and no order. With a bonus the energy gap is hidden and it opens lab-first at 0:05-0:06 (every Altored
game at +50 and +60, the lobby allies at +50). Its first soldier is a Grunt or a Pawn in our half at 1:14 to 1:54 (Altored
+60 1:14, +50 1:16, +0 1:50, +20 1:54; Comet 1:54 and 2:26) and our first extractor dies at 2:41 to 3:12 (4:15 and 3:48 on
Comet). The author's own list names what it abandons: 37% of every build it starts, nearly all the big ones, when the
want behind the build vanishes between ticks.

<!--meta
id: M-apex-economy
layer: meta
premises: [F-eco-extractors, F-eco-build]
population: Apex v0.3.2 as three seats on Altored Divide (seven games, +0 to +60, the side summed); v0.3.0 as our ally at +50 in the lobby 5v5 (one seat read in detail); v0.2.9 against the distilled bot on Comet (eight games)
map: Altored Divide Bar Remake 1.6.2; Comet Catcher Remake 1.8
dates: 2026-10-18 to 2026-10-19
ground: truth
sources: {code: [run/opponent_set.py, docs/studies/2026-10-18-distilled-apex-losses.md, run/matches/1792302895-human-8v8-1]}
status: live
-->
### Economy, build power, tier 2

It builds economy and turrets, not army, in the first fifteen minutes. The three Altored seats together (summed; the
seven games mix bonuses from +0 to +60): 10 extractors at 2:00, 14 at 4:00, 17 at 6:00 to 8:00, 18 at 10:00, 23 at 12:00;
constructors 3 at 2:00, 9 at 4:00, 11 at 6:00, 14 at 8:00, 21 at 10:00, 22 at 12:00; turrets 4 at 4:00, 13 at 6:00, 21 at
8:00, 23 at 10:00, 28 at 12:00; army metal flat at 1,250 to 1,900 from 4:00 to 12:00 (Grunts half of it to 6:00, then
Thuds a third), with soldiers in our half 4 to 6 at 2:00-3:00 and none from 6:00. Its advanced factory came at 9:24
(+20, on 13 extractors), 12:32 (+40, on 25), 12:16 (+50, on 23), 10:58 and 11:42 (+60, on 23 and 27), an advanced air
plant in three of five, with 800 to 4,200 of army when placed. As our ally at +50 (v0.3.0) one seat had its advanced lab
at 5:20 on eight spots, the first advanced constructor at 7:50, the first moho at 9:00, 6 mohos at 15:00 and 17 by 30:00,
21 constructors at 8:00 and 14 construction turrets at 18:00, and it upgraded our extractors to mohos as it passed (20
given to one of our seats); its income was 104 at 15:00 against our 42 and 57 on more spots. Against the distilled bot
on Comet (v0.2.9, +0) it began an advanced plant at 6:00-9:30 in seven of eight games and stood 14 to 23 extractors with
19 to 32 soldiers at 10:00 in the games it won.

<!--meta
id: M-apex-ladder
layer: meta
premises: [F-eco-multipliers, M-apex-economy]
population: Apex v0.3.2 as three seats against three of ours on Altored Divide with a bonus to it: +20, +40, +50, +60 twice (2026-10-18/19); v0.2.9 and v0.3.3 1v1s on Comet (2026-10-17, 10-20)
map: Altored Divide Bar Remake 1.6.2; Comet Catcher Remake 1.8
dates: 2026-10-17 to 2026-10-20
ground: truth and the ledger
sources: {code: [docs/experiments.md, run/matches/1792374652-apex-v032-altored-3v3-apex50-1, run/matches/1792383969-apex-v032-altored-3v3-apex60-1, run/matches/1792421655-apex-v032-altored-3v3-apex60-2], claims: [K-mech-lazarus-raises-commanders]}
status: live
-->
### What it takes to even the score, and what it brings late

The Altored 3v3 ladder: +0 won 13:00 and +50 both sides won 12:30 (referee's calls on 56 extractors against 14);
+20 won 17:30 (his commanders dead 14:16 and 16:48); +40 won at the user's call at 21:02 (he kept 23 to 25 extractors to
16:00); +50 won 33:24 outright, the first contested rung (his 23 extractors against our 53 at 16:00, then his push and
air cut ours to 29 by 24:00 before our army of 468 won); +60 lost on the timeout with our side broken, then won 29:36 on
the next game once our sprint wording was in. So v0.3.2 needs about +60 to hold level on Altored. What it brings when it
has the income: air waves early (Shadows 6:34, Rapiers 13:18, Hurricanes 15:46 at +50), Sumos, Pyros and Juggernauts,
an experimental gantry at 22:00 to 24:34 with Juggernauts from 32:51, Karganeths and Shivas (338 of our units to the
Juggernauts after 44:00 in the lost game), 51 Lazarus that raised two of our dead commanders, and a silo standing at
24:28 in one game and none in the next. On Comet at +0 the flagship beat v0.2.9 (16:34) and v0.3.3 (17:01): its
advanced air plant at 16:00 on five extractors in the first, its air plant at 12:50 and no tier 2 in the second, nine
extractors against our 24 at 10:00. The distilled bot stood 1-7 against v0.3.2 on Comet before its rebuild and 4-4
after (2026-10-20).
