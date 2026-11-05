<!--meta
id: M-leg-intro
layer: meta
population: Legion play by people and by the stock AIs: two duels at OS 49 (Canis River v1.4, 2026-10-03, from the bar-replays processing of the public replay bucket), two at OS 27 (Great Divide V1, our cards), fourteen 8v8 lane seats above OS 30 on All That Glitters (our cards), BARb and Apex seated as Legion in our arena
dates: 2026-10-14 to 2026-11-05
ground: the engine replay's events (bar-replays), our cards, the arena's truth files
sources: {code: [run/replays/bar_replays.py, run/data/replays/manifest.jsonl, docs/knowledge/legion.md], web: [http://b-dtop.b.gecko.network:8770 (selection 6, legion-1v1-os40)], claims: [K-leg-pro-pool-is-thin, K-leg-both-stock-ais-play-legion, K-leg-lab-energy-inverts]}
status: live
-->
## Legion play: what the few games show

Legion is a lobby option and about one public duel in forty at OS 40 has a Legion side (39 of 14,212), so there is no
Legion pool at the Comet pool's standard. What there is: two duels at OS 49 by one player on one wind map, two at
OS 27, fourteen 8v8 lane seats above OS 30, and the stock AIs seated as Legion against us. Everything below is from
those and says which. The pool's numbers (M-pool-intro) are Armada's and Cortex's; a Legion seat reads the economy's
Legion section (F-eco-legion) for the prices and this document for the play.

<!--meta
id: M-leg-duel-opening
layer: meta
population: ozvideo (OS 49.0 and 48.4) as Legion against maratus (OS 49.1 and 49.5) as Armada, Canis River v1.4, two games of 2026-10-03 on test-31479, both won by Legion at 13:25 and 12:11; Warmoob (OS 27) as Legion against Learnedprawn (OS 25) as Cortex, Great Divide V1, two games, both won
ground: bar-replays events for the OS-49 games (every unit created with its clock, every death with its killer, team stats by half-minute); our cards for the OS-27 games
sources: {claims: [K-leg-open-os49-canis-river, K-leg-open-os27-great-divide, K-leg-lab-energy-inverts], web: [http://b-dtop.b.gecko.network:8770/match/d011c16a8ebd392b093b7dcb82c9d784, http://b-dtop.b.gecko.network:8770/match/9d0ec16a69d1a0c01178d7a69ef203ea]}
status: live
-->
**The Legion duel opening at OS 49: the bot lab and a Goblin stream.** One player, one opponent, one map whose wind runs 4
to 14, two games the same to the second:

| clock | what |
|---|---|
| 0:03, 0:10, 0:17 | three extractors |
| 0:24 | solar |
| 0:34, 0:40 | two wind turbines |
| 0:47 | bot lab |
| 1:04 to 1:57 | seven to nine more turbines and nothing else |
| 1:26 | the lab's first product a constructor (a second at 1:43 in one game) |
| 1:48 on | Goblins without a pause, one every three to six seconds: 173 and 179 by 10:00 |
| 2:09 | the first construction turret on the lab; three or four by 10:00 |
| 2:26 or 7:58 | Zagreus, four and seven in all |
| 4:11 on | eight or nine solars, one advanced solar at 7:11 or 8:57, energy storages at 5:29 and 8:15 |
| 5:08 and 6:36 | the second bot lab; a third at 6:21 and 7:56, a fourth at 9:29 |
| 7:25 | Satyrs join the stream in the longer game, 26 by 10:00 |

No vehicle plant in either game; four to fifteen Pharos and four to nine radars. The opponent opened the bot lab at
0:28 for Fleas and Pawns and a vehicle plant at 3:13 and 3:45 for Blitzes (28 and 47 by 10:00) and Stouts.

How it was won: not on income. At 10:00 the Legion side had produced less metal than Armada in both games (16,750
against 17,630; 12,850 against 15,490), a little more energy, and 326 and 286 units against 192 and 176. Goblins died
by the dozen, mostly to Blitzes (61 and 73 of them a game) and Stouts, and the Legion army value was behind at 8:00 in
both games (2,700 against 3,863; 1,530 against 1,943). Then the stream outgrew the Blitz count (7,210 against 3,300 at
12:00; 3,174 against 1,665 at 10:00) and the last minute of each game is the Armada base dying, 25 to 28 extractors
and 18 to 21 solars killed in one minute. The energy bill of the labs is the other way round for Legion (F-eco-legion):
a Goblin stream draws what a Blitz stream does, and on a map with wind to 14 the turbines pay it.

The OS-27 games are the same shape 22 OS lower: extractor, extractor, wind, wind, solar, bot lab at 0:42; the lab's
constructor, then two to four Goblins, a Zagreus and a second constructor; Goblins to 2:00, then Satyrs with one to
three Ballistas from 4:00, the advanced bot lab at 8:08 in the long game, and Hive pickets from 10:00. What neither
sample says: whether the vehicle plant ever opens a Legion duel at this level, and what the shape is on a solar map;
the 36 other Legion duels above OS 40 are queued for processing and are the test.

<!--meta
id: M-leg-ais
layer: meta
population: BARb medium seated as Legion (leg-smoke-barb, Comet, six minutes), Apex standard as Legion (leg-smoke-apex and eight 40-minute games of apex-v024-legion-distilled-comet-1)
ground: the arena's truth files and census
sources: {code: [docs/experiments.md], claims: [K-leg-both-stock-ais-play-legion]}
status: live
-->
**The AIs as Legion.** BARb medium opens the vehicle plant: at 6:00 a Legion commander, 3 extractors, 4 solars, 2
advanced solars, a vehicle plant, a constructor vehicle, 6 Alaris, 3 Pharos and a radar; its hard profiles as Legion
are unseen. Apex standard opens the bot lab in all eight games: Goblins, Phobos and Satyrs, a Ballista, two Pharos
and an advanced solar by 8:00, and fields 5 to 14 soldiers at 10:00 (median 9) against the 1 to 25 (median 15) it
fields as Armada or Cortex on the same seeds; its nets are trained on Armada and Cortex mirrors, so its Legion is its
untrained side.
