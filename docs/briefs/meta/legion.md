<!--meta
id: M-leg-intro
layer: meta
population: Legion play by people and by the stock AIs: 36 Legion sides of 29 public duels with both players above OS 40 (2026-02 to 2026-10, nine maps, from the bar-replays processing of the public replay bucket), two duels at OS 27 (Great Divide V1, our cards), fourteen 8v8 lane seats above OS 30 on All That Glitters (our cards), BARb and Apex seated as Legion in our arena
dates: 2026-10-14 to 2026-11-05
ground: the engine replay's events (bar-replays), our cards, the arena's truth files
sources: {code: [run/replays/bar_replays.py, run/replays/legion_openings.py, run/data/replays/manifest.jsonl, docs/knowledge/legion.md], web: [http://b-dtop.b.gecko.network:8770 (selection 6, legion-1v1-os40)], claims: [K-leg-pro-pool-is-thin, K-leg-both-stock-ais-play-legion, K-leg-lab-energy-inverts, K-leg-duel-openers-os40]}
status: live
-->
## Legion play: what the few games show

Legion is a lobby option and about one public duel in forty at OS 40 has a Legion side (39 of 14,212), so there is no
Legion pool at the Comet pool's standard and no Comet game in it. What there is: 36 Legion sides of 29 duels above
OS 40 on nine maps, read by the engine's events (sixteen players, OS 43 to 60, seven of the duels Legion mirrors); two
duels at OS 27; fourteen 8v8 lane seats above OS 30; and the stock AIs seated as Legion against us. The pool's
numbers (M-pool-intro) are Armada's and Cortex's; a Legion seat reads the economy's Legion section (F-eco-legion) for
the prices and this document for the play.

<!--meta
id: M-leg-duel-opening
layer: meta
population: the 36 Legion sides above OS 40 (K-leg-duel-openers-os40): Flats and Forests 8, Eternal Consequences 7, Hades Ponds 7, Rosetta 4, Canis River 3, Carrot Mountains, Archsimkats Valley and Argent Strata 2 each, Erebos Lakes 1; PRO_Fiddler112 six sides, Glados and Altf4_ubivator four, ozvideo two; plus the two OS-27 duels on Great Divide
ground: bar-replays events (every unit created with its clock, every death with its killer, team stats by half-minute); our cards for the OS-27 games
sources: {code: [run/replays/legion_openings.py], claims: [K-leg-duel-openers-os40, K-leg-open-os49-canis-river, K-leg-open-os27-great-divide, K-leg-lab-energy-inverts]}
status: live
-->
**The Legion duel opens the vehicle plant two times in three, the bot lab once; never air.** Over the 36 sides the
vehicle plant stands at 0:41 (0:31 to 0:52) after three or four extractors and a solar or two turbines, the bot lab
at 0:31 (0:20 to 0:47) after one to three extractors and one solar or turbine. The opener follows the player more
than the map: Flats and Forests (wind to 12, solars) is vehicles in 8 of 8, Eternal Consequences (16) in 6 of 7,
Hades Ponds (14) splits 4 vehicles against 3 bot labs, Rosetta (a solar map) 3 against 1; Canis River, Carrot
Mountains and Argent Strata are bot labs. The energy arithmetic that favours Legion's bot lab (F-eco-legion) is not
what decides it; the raiders' speed and the player's habit are. Wins by opener, 10 of 24 and 7 of 12, are within
noise, with mirrors and OS gaps mixed in; in the two Legion mirrors where the openers differed the bot lab won both.

| the opener | what it does in the first five minutes | then |
|---|---|---|
| vehicle plant, the Wheelie mass (the Legion form of the Rover mass; PRO_Fiddler112 OS 54, Chomolungma, phosphophyllite) | two or three constructors, then Wheelies without a pause, 34 to 49 by 5:00; they kill his scout cars and bare extractors and die by the dozen (59 to 117 lost a game) | the first Alaris at 4:44 to 7:09, 45 to 88 by 10:00, killing his raiders and extractors by the hundred; a second vehicle plant at 5:49 to 8:46 |
| vehicle plant, Alaris first (Glados OS 51, four of four won; GlorpChud, modulo) | a constructor, one Wheelie, Alaris from 1:15 to 1:53, 14 to 23 by 5:00 | Helios from 5:00, 51 to 111 Alaris by 10:00; an Alaris army killed 231 Goblins in one game and still lost to the stream |
| bot lab (ozvideo OS 49, Altf4_ubivator OS 57, Haundreyy, iyc) | two to five constructors first, then Goblins from 1:07 to 2:18, 24 to 57 by 5:00 | 50 to 242 Goblins by 10:00; Altf4 adds Phobos, 8 to 15 by 10:00; Satyrs in the long games; a second bot lab at 5:08 to 7:29 |

Common to both: five constructors by 5:00, 15 extractors by 5:00 and 34 to 40 by 10:00, one or two construction
turrets and two or three Pharos by 5:00, the second factory at 5:49 to 11:08 (a second vehicle plant in 14 sides, the
drone plant in 8, a bot lab in 7), the energy the map's (turbines by the dozen where the wind tops 14, solars on
Flats and Rosetta), no Hive in any side by 10:00, one advanced solar in about a third of the sides by 10:00.

How the Goblin stream wins when it wins (ozvideo's two games against an Armada at OS 49, Canis River, 13:25 and
12:11): not on income. At 10:00 the Legion side had produced less metal than Armada in both games, a little more
energy, and 326 and 286 units against 192 and 176; Goblins died by the dozen to Blitzes (61 and 73 a game), the
army value was behind at 8:00, and then the stream outgrew the Blitz count and the last minute of each game is the
Armada base dying. Against a Legion Alaris army the same stream lost 231 Goblins and won the game (Rosetta).

The OS-27 games are the bot lab's shape 22 OS lower: extractor, extractor, wind, wind, solar, bot lab at 0:42; the
lab's constructor, then Goblins, a Zagreus and a second constructor; Satyrs with one to three Ballistas from 4:00,
the advanced bot lab at 8:08 in the long game, and Hive pickets from 10:00. What the sample cannot say: nothing in
it is on Comet, and the maps are mostly wind maps, so the solar-map form rests on Rosetta's four sides.

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
