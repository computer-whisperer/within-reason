# Map dependence

Seeded 2026-09-19. "Map archive" = `run/data/maps/quicksilver_remake_1.24.sd7` (read `mapinfo.lua` and three feature defs
out of it). Unit paths relative to `upstream/Beyond-All-Reason/units/`.

### K-maps-quicksilver-facts
**Claim.** Quicksilver Remake 1.24: 14x14 (7168 elmos square), 2 players, an island surrounded by water (water level 0,
heights -80…325), wind 3-17, tidal strength 10, maxMetal 1.30, extractor radius 120. The site classifies it as a flat,
grassy 1v1 island map; average wind 12.7.
**Status.** supported (2026-09-19) — map archive; terrain description and average wind reported
**Evidence.** Map archive `mapinfo.lua:23-33,118-119`. https://www.beyondallreason.info/map/quicksilver (updated 2026-06-26).
**Would be wrong if.** The engine reported different wind limits at match start.
**Used by.** (see K-open-quicksilver-is-a-wind-map)

### K-maps-read-wind-at-start
**Claim.** Wind income per turbine equals current wind speed, which wanders between the map's minWind and maxWind and is
capped at 25 per turbine; the map limits are available to an AI at game start, so the wind/solar choice can be made per
map rather than hard-coded. Rule-of-thumb thresholds are in K-open-wind-threshold.
**Status.** reported (2026-09-19); cap supported from local source
**Evidence.** `ArmBuildings/LandEconomy/armwin.lua` (windgenerator 25); official economy guide
https://www.beyondallreason.info/guide/in-depth-look-at-economy. Whether our shim exposes `Map_getMinWind/MaxWind` is unchecked.
**Would be wrong if.** Logged energy income from N turbines was not ≈ N × current wind.
**Used by.** (candidate: H-ECO-ENERGY-CHOICE; needs min/max wind in bot-protocol's map info)

### K-maps-wind-needs-a-floor
**Claim.** With wind 3-17 the income of a wind-only economy swings almost 6:1; guides say to overbuild energy and keep the
bar above 50%. One energy storage (170 M, +6000 E) buffers ~60 s of a 100 E/s shortfall and is the cheap fix for wind troughs.
**Status.** reported (2026-09-19); storage numbers supported from local source
**Evidence.** https://www.crdhq.com/articles/bar-economy-guide ("energy bar above 50% at all times", "always overbuild
energy"); `ArmBuildings/LandEconomy/armestor.lua` (metalcost 170, energystorage 6000).
**Would be wrong if.** A wind brain with one storage still hit 0 energy for >10 s stretches in most matches.
**Used by.** (candidate: build one energy storage when turbines ≥ 8; note H-ECO-ENERGY-BY-STORAGE's 40% test scales with storage)

### K-maps-tidal-is-poor-here
**Claim.** A tidal generator makes the map's tidal strength in E/s, steadily. On Quicksilver that is 10 E/s for 90 M
(9 M per E/s) — worse than solar (7.75) and far worse than wind (~3.1) — so water energy is not worth it on this map. It only
wins on maps with tidal ≥ ~20 and low wind.
**Status.** supported (2026-09-19) for inputs (map archive tidal 10; `ArmBuildings/SeaEconomy/armtide.lua` 90 M, tidalgenerator 1);
the engine rule "output = tidal strength" is reported
**Evidence.** As above; https://www.beyondallreason.info/map/quicksilver (tidal 10).
**Would be wrong if.** A tidal generator on Quicksilver produced more than 10 E/s.
**Used by.** (none)

### K-maps-features-are-early-energy
**Claim.** Map features hold reclaimable resources that arrive instantly and cost nothing: on Quicksilver the palm trees
checked hold 250 energy each (0 metal) and the rock checked 10 metal. Four trees equal the whole 1000 E starting stock,
which is what the first minute is short of (K-open-energy-binds-first). Guides call reclaim of features "incredibly
important"; any builder can do it and a constructor's idle time near trees is free energy.
**Status.** supported (2026-09-19) for the sampled feature values (map archive `features/ad0_senegal_1.lua`,
`ad0_senegal_1_large.lua`: energy 250; `agorm_rock1.lua`: metal 10); density near start positions not measured; advice reported
**Evidence.** Map archive; https://www.beyondallreason.info/guide/reclaim-resurrect-repair.
**Would be wrong if.** An area-reclaim order by a constructor near the base yielded no energy, or took so long that a solar
would have paid more.
**Used by.** (candidate: H-ECO-RECLAIM-FEATURES — when stored energy < 40% and features are within ~400 of a builder,
area-reclaim before building another generator; needs a reclaim command in bot-protocol)

### K-maps-factory-by-terrain
**Claim.** Factory choice follows terrain: bots for hills and steep approaches, vehicles for flat open ground, hovercraft
or amphibious units where water separates the players, ships only where the enemy is reachable by sea. The official
starter guide says to pick the factory from the start location. Quicksilver's land is flat, which favours vehicles on paper;
both start positions are on the same island, so land units reach the enemy.
**Status.** reported (2026-09-19)
**Evidence.** https://www.beyondallreason.info/guide/how-to-start-manage-your-economy;
https://www.beyondallreason.info/guide/important-knowledge-on-advanced-mechanics (slope tolerance);
https://masterbel2.wordpress.com/beyond-all-reason-units-guide-uses-tactics-and-counters/ (vehicles favoured in 1v1).
That land units path between the starts is our own arena observation (waves meet mid-map, K-army-piecemeal-midmap).
**Would be wrong if.** A vehicle-plant opening did not beat the bot-lab opening on Quicksilver at equal economy rules.
**Used by.** (candidate: factory type chosen by mean slope between the two start positions)

### K-maps-comet-barb-opens-bots
**Claim.** On Comet Catcher Remake, a flat open vehicles map, BARb opens with a bot lab most of the time, at every
profile the arena has run there: 91 bot labs (Armada 75, Cortex 16) against 13 vehicle plants in 104 recorded games.
Its terrain rule, if it has one, does not pick the plant here; expect Pawns and Ticks early on this map.
**Status.** measured (2026-09-23) from the arena's results.
**Evidence.** `opponent_first_factory` in `run/matches/*comet*/results.jsonl` (v33-comet, now-comet, rush-5 to
rush-7 batches, medium; two games with no truth file).
**Would be wrong if.** The hard profiles, which those batches did not run, opened with the plant.
**Used by.** the player's brief (the Comet Catcher section).

### K-maps-comet-tanks-stall-in-a-packed-base
**Claim.** Tanks path badly out of a base packed with buildings: on Comet Catcher with 18 solars and 5 vehicle plants
within 800 of home, a group of 8 Blitzes stood at spot_50 "not nearer its goal" for over 35 s and a group of 48 for
67 s (the player's note at 15:04), both ordered to spots far east. Generators belong on named spots away from the
plants, and the plants apart, when the army is vehicles.
**Status.** observed (2026-09-23), one game; the cause (buildings, not the footwork's march hold) is inferred from
the geometry, not shown.
**Evidence.** `run/matches/1790051208-comet-1-easy`: the picture's `progress` words for group_P at 14:30 to 15:04;
the record's building positions.
**Would be wrong if.** The same groups stalled with the base spread out, or the stall came from the march rule.
**Used by.** the player's brief (the Comet Catcher section).

### K-maps-comet-middle-spots-die-first
**Claim.** On Comet Catcher the spots nearest home by walking time are not the strip's own: from a west-strip start
at (1176, 3497) the menu lists spot_43 (1992, 3288) and spot_52 (2424, 3736) in the open middle before spot_28 and
spot_30 of the strip, and a constructor told to "expand along our strip" took them; against medium each died within
27 to 70 s of standing, to Ticks and Pawns, while the strip's spot_36, spot_54, spot_28 and spot_30 stood untaken
until 5:38 (comet-3). The strip's spots are named in the brief, so the player lists them by name.
**Status.** observed (2026-09-23), one game.
**Evidence.** `run/matches/1790052453-comet-3-medium`: the extractor timeline (2:28 spot_43 died 3:17, 2:59 spot_52
died 3:27, 3:27 spot_43 again died 6:02); `jev-0.jsonl` the constructor's picks.
**Update 2026-09-22 (the players' replay, `run/matches/1790122851-replay-vak-vs-artur-comet`).** Experienced players take exactly these spots first: the west
player's commander built spot_43 at 2:33 and spot_38 and spot_39 by 3:36, a light turret beside each pair, a solar or
two and a radar at the outpost, and none died in seven minutes (K-open-comet-commander-expands). The claim is about an
unguarded constructor at an open spot against Ticks, not about the spots: taken by the commander with a turret beside
them, they are the first expansion, not the last.
**Would be wrong if.** Middle spots held as long as strip spots against the same raids.
**Used by.** the player's brief (Comet Catcher section).

### K-maps-metal-share-wins
**Claim.** Holding more metal spots is the main predictor of winning; one guide puts it at "60%+ of the spots usually wins".
On a symmetric 1v1 map this means every spot in our half plus some contested middle spots, and denying the opponent's
outer spots by raiding.
**Status.** reported (2026-09-19)
**Evidence.** https://www.crdhq.com/articles/bar-economy-guide (2026; the 60% figure has no stated basis).
**Would be wrong if.** Across our match logs, the side with more extractors at minute 10 did not win clearly more often.
**Used by.** H-ECO-OWN-HALF (candidate: relax to "own half + middle band" once an army group holds the middle)

### K-maps-forward-factory
**Claim.** Slow heavy units are worth more the closer they are built to the enemy; a forward factory after expanding
shortens reinforcement paths and forces the opponent into slow units too. This is the community answer to reinforcements
trickling across the map.
**Status.** reported (2026-09-19)
**Evidence.** https://masterbel2.wordpress.com/beyond-all-reason-units-guide-uses-tactics-and-counters/.
**Would be wrong if.** A second lab placed at ~40% of the way to the enemy was lost in most matches before repaying itself.
**Used by.** (candidate: H-ECO-MORE-LABS places lab 2 forward, behind the outpost turrets; relates to K-army-piecemeal-midmap)

### K-maps-quicksilver-corner-asymmetry
**Claim.** Quicksilver Remake 1.24 is an asymmetric island with cliffs and inlets, and the two corner starts the arena
uses are different games: the north-west start sits on a narrow peninsula and has 15 metal spots nearer to it on foot,
the south-east start 19.
**Status.** supported (2026-09-19). An earlier version of this entry called the metal spots mirror-symmetric; that was
wrong (only the two start positions mirror, because the arena's start boxes do): 42 of 44 spots have no mirror partner.
The user saw the asymmetry at once in the replay.
**Evidence.** Terrain survey at game start (`brain/routes.rs` log lines, terrain-check): 38 of 44 spots reachable on
foot; the six cut off are (3224,520) (6184,520) (6632,984) (504,6184) (936,6664) (4632,6664), the first of which is the
spot constructors kept failing to reach. Results by corner: v11 SE 10-0-2 / NW 3-7-2; v13-medium SE 10-2 / NW 1-11;
with the start boxes swapped the gap stayed with the corner. The viewer's terrain layer shows the shape.
**Would be wrong if.** A symmetric map showed the same gap between its two starts.
**Used by.** Evaluation: results are read per corner; a symmetric 1v1 map is wanted in the pool.

### K-maps-terrain-not-straight-lines
**Claim.** Geometry by straight line ("nearer to us than to the enemy", "500 towards the enemy", "is this target
reachable") is wrong on maps with cliffs and water. A walking-distance field over the engine's slope and height maps,
for the soldiers' movement class, agrees with what the engine lets units do.
**Status.** supported (2026-09-19) on one map
**Evidence.** The field's unreachable spots include the one the engine refused all day; with stations, lab yard,
staging points and "our half" taken from the field, a 12-minute game had 3 move failures and no "unreachable" give-ups
(terrain-check-2), against hundreds to thousands before.
**Would be wrong if.** Move failures or unreachable give-ups came back in numbers on another map.
**Used by.** `brain/routes.rs`: `spot_is_ours`, `forward_of_home`, attack-target filter, staging point.

### K-maps-quicksilver-spot-value-and-wind
**Claim.** On Quicksilver Remake 1.24 a tier-1 extractor yields 2.0 metal/s on every spot we have built on, and wind per
turbine averages 12.8 over the first ten minutes (single games 8.2 to 15.5).
**Status.** supported (2026-09-19)
**Evidence.** Income steps at extractor completion in 12 records of `v15-terrain-medium` (home spots 18 of 18 exactly
2.0; outer spots median 1.8-2.7 with converter noise); wind inferred as (energy income - constant producers) / turbines.
The map site lists 12.7.
**Would be wrong if.** A spot's first extractor raised income by something other than 2.0 with no converter running.
**Used by.** `crates/buildorder` (`map::SPOT_METAL`, `map::WIND_MEAN`).

### K-maps-our-half-shrinks
**Claim.** Placing the enemy's base at the mean of every enemy building we remember drags it toward us, because what we
see is mostly its forward turrets and extractors; the walking-distance line between "its" spots and "ours" follows, and
the constructors (H-ECO-OWN-HALF) are left with almost nothing to take.
**Status.** supported for the south-east (2026-09-20, v24: 14.8 v 4.2 extractors at minute 15 with the fix, 9.6 v 9.8 without; 5-0-1 against 4-0-2);
not the north-west's limit, whose extractor curve did not move.
**Evidence.** commander-5-tempo-brief, minute 23: "free spots on our side 2" with 7 extractors held, of 38 reachable
spots (15 are ours at game start); army 11,200 v 4,100 and extractors flat for 19 minutes; the commander's
`expansion_radius` 3000 changed nothing because the own-half rule applied beneath it; constructors fell through to
converters (30). North-west heuristic batches stall at 4-5 extractors from minute 6 (v22, v23).
**Would be wrong if.** The A/B shows the same extractor curve with the base located by factories only.
**Used by.** H-MAP-ENEMY-BASE; the commander's `expansion_radius` now replaces the own-half rule.

### K-maps-mirror-start-is-a-beach
**Claim.** On Quicksilver the point mirror of our north-west start, (5136, 5980), is a beach across an inlet from the
real south-east base at about (5576, 5360): armies ordered "to the enemy start" walk there and stand on the shore, out
of sight of the base. The walkable metal spot nearest the mirror point, (5448, 5240), is 140 from the real start.
**Status.** observed (2026-09-20); the user recognised it as a recurring failure while watching commander-7.
**Evidence.** commander-7-raid-expansion/00, minute 25: 57 soldiers of squad `farguard` at (4750-5250, 5750-6250) on a
`fight` order to (5136, 5980), the report still saying "its commander has never been seen; its factories seen: none".
commander-5: a strike "reached presumed enemy start F7 but found nothing". The heuristic's fallback target and the
scout use the same point.
**Would be wrong if.** On other maps the nearest-spot rule lands farther from the real start than the mirror does;
start positions are not exposed to a skirmish AI (`Map_getStartPos` gives only our own), so this stays a guess until
a factory is seen (H-MAP-ENEMY-BASE).
**Used by.** H-MAP-ENEMY-START.

### K-team-a-mean-of-two-bases-is-nobodys
**Claim.** With several opponents, any single "enemy start" (a mirror point, a mean of factories) is a place where
nobody lives; ground, direction and targets have to be asked of the nearest live base.
**Status.** reasoned, not measured (2026-09-20): the single-base code was replaced before a team game was played.
**Used by.** H-MAP-ENEMY-BASE.

### K-team-allies-are-invisible-by-default
**Claim.** The AI interface's unit list is our own team's; allied units must be asked for (`getFriendlyUnits`), and no
call tells where another team started: an ally's start is where its commander is first seen, an enemy's is somewhere in
its ally team's start box, which only the setup script (`Game_getSetupScript`) gives.
**Status.** supported (2026-09-20), read from `SSkirmishAICallback.h` and seen in team-2v2-smoke.
**Used by.** H-TEAM-ALLIED-SPOTS, H-TEAM-ALLY-GROUND, H-MAP-ENEMY-BASE.

### K-map-ground-is-who-answers-first
**Claim.** Whether a metal spot can be held is decided by who can bring force there sooner, not by its distance from home
or by a clock since the last loss: spots beside our army are safe far from home, and spots near home are not while the
opponent's army stands among them.
**Status.** supported (2026-09-20), terr-2 against the distance-and-clock rules on the same 8 north-west seeds:
extractors 7.0 / 7.4 / 7.8 at minutes 5 / 7 / 11 against 5.0 / 6.2 / 6.9, soldiers 24-28 against 22 at minutes 11-15; on
Mithril Mountain 12.2 / 14.6 extractors at minutes 11 / 15 against 8.4 / 8.1 (v33, 16 games) and 4-2-2 against 5-10-1.
Wins north-west did not move (1-7-0 against 0-6-2; that corner has won 10-25 % for days, as it does for BARb).
**Evidence.** `run/matches/*terr-2-*`, `*terr-1-nw-base`; curves from the per-minute lines of bot.log.
**Used by.** H-MAP-TERRITORY.

### K-map-presence-is-not-memory
**Claim.** "May a constructor go there" and "where do raids come from" need different memories. With every enemy soldier
seen in three minutes counted against a spot, one visit by the opponent's army closed most of our half for minutes and
nothing was rebuilt; an extractor pays for itself in about half a minute, so only who is there now should stop it.
**Status.** observed (2026-09-20), terr-1-nw: 31-41 of 44 spots classed theirs from minute 8, extractors 4-5 from minute
7 against 6-7 without the grid; with the class forgetting in 60 s (terr-2) the curve went above the old rules'.
**Evidence.** `run/matches/1789915396-terr-1-nw/*/bot.log`, the `ground:` lines.
**Used by.** H-MAP-TERRITORY.

### K-maps-quicksilver-corner-decides-both-economies
**Claim.** The Quicksilver corner asymmetry is a property of the ground, not of our play: whoever holds the south-east
start has roughly twice the extractors of whoever holds the north-west one. At minute 20 over 430 games, when we start
north-west we have 3.9 extractors and BARb (south-east) has 20.8; when we start south-east we have 11.6 and BARb
(north-west) has 8.9. Which start we drew is accordingly the single most informative thing we know about the
opponent's economy: used alone beside the clock it removes 30 % of the error in estimating their extractor count and
32 % of the error in the income proxy, more than any observation of them.
**Status.** measured (2026-09-20). Sharpens K-maps-quicksilver-corner-asymmetry, which established the asymmetry from
the terrain (15 spots near the north-west start on foot, 19 near the south-east one) and from our own results.
**Evidence.** `docs/studies/tempo-model.md`, sections "Models, simple to less simple" and "What evidence actually
moves the estimate"; opponent ground truth in every Quicksilver batch from v17 to v34.
**Would be wrong if.** The two corners' extractor counts evened out once our own play stopped plateauing in the
north-west (K-eco-reach-stops-at-the-first-gap): part of BARb's 20.8 is spots we never contest.
**Used by.** (none yet) — candidate: the tempo model conditions on it; expansion and defence rules could too.

### K-maps-early-walks-nearly-straight
**Claim.** In the first six minutes our builders' ways are 4-7 % longer on foot than in a straight line on Quicksilver
and on Mithril Mountain (median 1.04-1.07, longest 1.35): the home plateau is open. The map's ground matters to the
opening where a start sits behind a choke (not measured yet: Great Divide).
**Status.** measured (2026-09-20), 8 games, 247 trips.
**Evidence.** `buildorder walks` on `open-cal2-*`, last column.
**Would be wrong if.** The same report on Great Divide or Comet Catcher gave the same ratio.
**Used by.** (none) — the search uses the map's ground anyway (`buildorder::game::Walked`).


### K-maps-quicksilver-is-an-island
**Claim.** Quicksilver Remake is an island: the sea surrounds it, the commander (ours and theirs) is amphibious and
walks on the sea floor, and the enemy commander retreats into the water and builds on the shore from it when its base
is gone. Nothing our bot lab builds fights in the water (the Crossbow and the Tumbleweed can enter it; neither
fights ground units), so a commander in the sea is out of reach until the referee ends the game on its economy.
**Status.** stated by the user 2026-09-22 from watching pianist-player-7; the amphibian list is from the unit
definitions' move classes.
**Evidence.** pianist-player-7: the enemy commander ended at (6825, 5906) on the south-east shore beside a shipyard
with a ball of 226 Maces stalled at the water's edge; the referee called the win with 8 enemy units left.
**Used by.** the map tool's `water` entry, the picture's and the report's enemy-commander lines.

### K-maps-barb-starts-where-the-guesser-puts-it
**Claim.** An AI seat with no start position in the script is placed at 0:00 by the game's own start-point guesser
(`common/lib_startpoint_guesser.lua` `GuessOne`, called from `game_initial_spawn.lua` for unplaced teams): among the
free metal spots inside its box, the one whose neighbours within 575 elmos and 300 height score best, so a cluster's
middle, not the box centre and not the mirror of ours. On Comet Catcher's standard boxes (the west and east strips)
that is the north end of the east strip, (6899, 660-680), while the mirror of our start snapped to metal is the middle
of the strip, (6936, 2680): 2,000 elmos off. With the arena's earlier custom boxes ("frontier") the two agreed within
120.
**Status.** demonstrated (2026-09-22): 86 surviving Comet games' truth files against the record's `intent`
`enemy_start`; every game since rush-7 (2026-09-20, standard boxes) 1,999-2,020 off, every game before within 120.
**Evidence.** `run/matches/*comet*/00/truth-0.jsonl` frame 0 (`armcom` position) against `record-0.jsonl` `intent`;
the scorecard rows through 2026-09-22 read "base never found" in 11 of 14 while `look_min` (then measured against
the guess) said a unit of ours stood there by minute 4-9.
**Would be wrong if.** BARb moved its commander after placement (the truth shows it within 20 of the placement all
game), or a lobby set start positions explicitly.
**Used by.** H-HANDS-ENEMY-EVIDENCE (why no guess is shown); H-MAP-ENEMY-START's status. The bot's own modes could
replicate the guesser; under the player nothing is guessed.

## Comet Catcher Remake 1.8: the replay survey (2026-09-23)

Claims from the game cards of 29 public 1v1 duels (both players OS 25 and above, games of 2026-09-20 to 2026-09-23 on BAR test-31357 to 31383), filed by the synthesis agent against `docs/knowledge/questions.md` and checked with `run/replays/check.py`; the map file `docs/knowledge/maps/comet_catcher_remake_1.8.md` and the index `docs/knowledge/replays.md` point here. Spot numbers are the cards' (the game's own 80-spot list), not the bot's (the engine's 75); they map to the bot's by position (nearest within 130), and every spot carries its grid cell (8x8 over 8192x6144). Unit names: armfav Rover, corfav Rascal, armflash Blitz, corgator Incisor, corak Grunt, armpw Pawn, armflea Tick, armstump Stout, corraid Brute, armllt/corllt the light laser turret. "W"/"L" are the winners' and the losers' sides. Remaining card limits: `fights` is the eight costliest cell-minutes per game; the cards carry no unit positions over time (held back or chasing cannot be read); the killer of the first extractor is "?" on 10 sides; one manifest start (bcaeb16a, Chronopolize) disagrees with the card's B5 and the card is used. Fixed after the survey (cards re-cut 2026-09-23 evening): the faction field, the spots' cells, the first-enemy-seen field (removed: a replay sees everything), the factories' order, the last 15 s of kills counted apart, cells on every building, the first tier-2 unit.

**Re-checked 2026-09-24 on 33 games with both players at OS 40 and above** (66 sides; 25 fetched that day with
`run/replays/pick.py --min-os 40 --pages 12`, the tables from `run/replays/recheck.py --floors 25,40`, the players'
advice to the user being that 25 is too low to learn from). What held at the higher floor: the plant first in 77% of
sides (all: 77%), the fast plant before 0:45 in 52% (51%), the commander's turret before 0:30 in 32% (35%),
extractors at 12:00 winners 29 against losers 17 (the same), income at 8:00 41 against 35 (41 against 34), light
turrets by 8:00 9 against 7 (9 against 6), a second factory in 79% at 7:56 (the same), tier 2 in 5% (6%), the first
extractor lost at 4:12 (4:15), 60% of the fight-list deaths on rows 4 and 5 (62%). What moved: army metal at 8:00 is
even at the higher floor, 1763 against 1702 (all: 1800 against 1602), so the winners' lead at 8:00 is extractors and
turrets, not soldiers; the higher-OS side won 18 of 33 (55%) against 36 of 54 (67%), as it should when both are strong;
games are shorter, 12:23 median against 13:34. The solar count is the new claim below.

### K-map-comet-catcher-remake-1-8-two-solars-before-the-plant
**Claim.** The opening builds two solar collectors before the first factory: 2 in 44 of 66 sides at OS 40 and above
(0 or 1 in 15, 3 in 7, 4 or more in none; the whole pool 80 of 108, 17, 11, 0). The third and fourth come after the
plant: by 2:00 the median is 4 (4 or more in 49 of 66) and by 3:00 5, by 5:00 8, winners and losers alike (4 or more
by 2:00: 23 of 33 winners, 26 of 33 losers). The five build-order entries after the plant are, at each position, a
solar (29 of 66), a constructor (15) or a solar (25), a solar (24), a solar (18) or a Rover (11), a Rover (15), with two
solars among the five for winners and losers both; the one sequence that repeats is solar, constructor, solar, Rover,
Rover (8 of 66). So the user's high-OS friend's rule, "more than three solars in the early opening is a major
mistake", is the pool's habit read strictly: nobody builds a fourth before the plant, and few a third; the plant at
0:43 (median) comes on two, and the energy for it is built while it stands. Our brief's opening (three solars, the
plant at 0:58, from the VAK-Artur game) is the minority line.
**Status.** observed (2026-09-24, 33 games at OS 40+; the whole pool agrees); no result signal either way past the
plant (the count by 2:00 does not separate winners from losers).
**Evidence.** `run/replays/recheck.py --floors 25,40` (the solar rows and the five actions); the cards' `build_order`.
**Would be wrong if.** The cards missed solars started by the commander before the plant (the build order is every
`created` event with its builder; a solar begun and cancelled would be counted, a solar assisted not started would not).
**Used by.** `docs/briefs/player.md` (the Comet Catcher opening: two solars, the plant, then solar, constructor, solar,
Rovers).

### K-map-comet-catcher-remake-1-8-two-openings
**Claim.** The sides open one of two ways. The fast plant (no turret; the first factory started 0:34-0:39 after
extractor, extractor, solar, solar) in 22 of 58 sides, 16 of them Armada; and the turret opening (extractor 0:03, extractor
0:10, solar 0:17, light turret 0:25-0:28, a third extractor 0:44-0:48, a solar, the factory 0:58-1:10) in 19 sides,
14 of them Cortex, with seven variants around the two. Split at 0:55: the 29 later-plant sides won 18, the 29
earlier-plant sides won 11. The later plant won in both factions (Armada 5 of 8 against 6 of 17; Cortex 13 of 21 against
5 of 12) and at both OS levels (the higher-OS player 14 of 17 against 8 of 12; the lower-OS player 4 of 12 against 3 of
17). In the 17 games where the two sides opened different ways the later plant won 12, but in 10 of those 12 it was
the higher-OS player. Within a game the first factory's time does not pick the winner (winner earlier in 14, later in 14,
tied in 1). The later plant carries the turret and the third home extractor (26 of its 29 sides built the turret before
1:00; none of the 29 fast sides did), so this and K-map-comet-catcher-remake-1-8-turret-at-0-27 are one habit.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 (the build order to the first factory's start per side).
**Would be wrong if.** The later plant's winning share fell to the earlier's once the OS gap is held equal (for example
in games between players within 3 OS). **Refines** K-open-comet-pro-order (one game, a plant at 0:58 on three solars):
that game's timing is the later family, but most later-family sides put one solar and a turret before the plant, not three
solars.
**Used by.** (candidate: the player's brief, Comet Catcher section, the opening to copy)

### K-map-comet-catcher-remake-1-8-first-factory-kind
**Claim.** The first factory is a vehicle plant in 45 of 58 sides (Armada 22 of 25, Cortex 23 of 33) and a bot lab in 13
(Cortex 10, Armada 3); none opened with an aircraft plant, and no side built a wind generator before 8:00 (solars only,
two before the factory in most sides). Vehicle-plant sides won 22 of 45, bot-lab sides 7 of 13. By faction and kind:
the Armada plant started 0:34 (median, 22 sides) and made its first soldier at 0:54 and first constructor at 1:30 (the
soldier first in 18 of 22; the most common first four: Rover, Blitz, Rover, Rover in 6); the Cortex plant started 1:01
(23 sides) with a constructor first in 16 of 23 (constructor, then three Rascals in 10), constructor 1:23, first soldier
1:43; the Cortex lab started 0:59 (10 sides) with two constructors then two Grunts in 7 of 10.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29.
**Would be wrong if.** Games on later versions opened with bot labs in more than a quarter of sides. **Refines**
K-maps-comet-barb-opens-bots: BARb opens a bot lab in 91 of 104 games here; the people open a vehicle plant in 45 of 58.
**Used by.** (candidate: the player's brief, Comet Catcher section)

### K-map-comet-catcher-remake-1-8-turret-at-0-27
**Claim.** A light laser turret built by the commander at 0:25-0:28, before the third extractor and the factory, is in 26
of 58 sides: 18 of 29 winners and 8 of 29 losers (18 Cortex, 8 Armada; the winners' first turret at 0:27 median against
the losers' 2:49). In the 16 games where only one side had it, that side won 13 (10 of them the higher-OS player). By
8:00 the winner had built more light turrets than the loser in 19 of 29 games (median 9 against 6).
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 for the counts; the 16 games with one early turret: 02bdb16aa5ed8ab92635aa50d3ff80ff,
25e2b26a0ca652a85ae574d70c1335ae, 4dfdaf6ac5933a177c5180e7e41eae49, 58ffaf6a9ac117a532729b2e845bcce1,
70d6b16afe1c617409a3d8c434292704, 77c1b16aae254c3828e41af2948cb0a8, 7fa2b26a71c0cfe4bf78e42b6932bd57,
8ab1b16ade373a81104fe441581fa6b4, 9bb4b26a7a2b55ea413fed5f3bd82758, 9d32b36a73aa57673fa0fe2b404888ce,
c9e7b26a5471b1e62ada12b90ad9378b, caddb26aacbf1134776b8c7388718e63, d7a7b16ab0e5e842a8d3181204f3b87d,
e139b36a1f3ec62af45fdd67e03baaa7, e3dab26a287fbfb1c149fcaf82179003, e7abb16a46e9a36d9a1616ed1d886394.
**Would be wrong if.** The early-turret side's wins disappeared among games between players of equal OS. The card has
no position for non-extractor buildings, so where the turret stands (at the home spots, it would seem from the
commander's clock) is not shown. It does not delay the first raid on an extractor (4:18 median with it, 4:24 without).
**Used by.** (candidate: the player's brief, Comet Catcher section; K-open-comet-rascals-lose-to-the-commander had one
turret at the plant by 2:16)

### K-map-comet-catcher-remake-1-8-home-spots-then-flanks
**Claim.** Every start has three home spots, and the commander takes them: A4 spot_30 (A4) 0:03, spot_28 (A4) 0:10,
spot_36 (B4) 0:45 (commander in 13 of 16); B5 spot_50 (B5), spot_45 (B5), spot_43 (B5) 1:17 (11 of 12); H5 spot_48 (H5),
spot_51 (H5), spot_42 (G5) 0:48 (18 of 19); G4 spot_34 (G4), spot_29 (G4), spot_35 (G4) 1:25 (8 of 8). The commander
then often takes the next pair toward the middle (A4: spot_45 and spot_50 (B5) at 2:06-2:33, commander in 10 of 16;
H5: spot_34 and spot_29 (G4) at 2:12-2:36, 10 of 18; G4: spot_41 and spot_40 (F5) at 2:56-3:09, 6 of 7; B5: spot_38 and
spot_39 (C4), 4 of 7) and constructors take the flank strips: the commander built a median 5 of the winners' extractors
to 8:00 (a quarter; half or more in 7 of 29 winners) and 6 of the losers'.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 (`build_order[].by` and `spots_taken`).
**Would be wrong if.** Constructors took the third home spot in more than a fifth of sides. **Refines**
K-open-comet-commander-expands (one game, the commander built 12 of 21 extractors walking to the middle): that is the
minority here; in 22 of 29 winners the constructors built most of the extractors.
**Used by.** (candidate: the player's brief, the Comet Catcher expansion list)

### K-map-comet-catcher-remake-1-8-a4-expansion
**Claim.** From A4 (17 sides) the order by median first-take clock is: spot_30 (A4) 0:03, spot_28 (A4) 0:10, spot_36 (B4)
0:45, spot_45 (B5) 2:06, spot_50 (B5) 2:33, spot_19 (B3) 2:51 (17 of 17), spot_22 (B3) 3:50 (12), spot_14 (A2) 4:10 (16),
spot_54 (A6) 4:30 (13), spot_62 (A7) 4:30 (10), spot_43 (B5) 4:32 (11), spot_26 (C3) 4:46 (11), spot_7 (A1) 4:46 (16),
spot_2 (B1) 5:12 (16), spot_38 (C4) 5:16 (10), spot_24 (C3) 5:36 (10), spot_9 (B1) 5:57 (14), spot_4 (C1) 6:14 (14).
The north-west strip (spot_19, spot_14, spot_7, spot_2, spot_9, spot_4: B3 to C1) is built by constructors (13 to 16 of
the sides that took each, the commander once). Winners (10) and losers (7) take the same spots at the same clock (spot_19
at 2:58 against 2:51; spot_14 4:12 against 3:46); the start cell's result is not in the order.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** A4 sides: 25e2b26a0ca652a85ae574d70c1335ae, 7fa2b26a71c0cfe4bf78e42b6932bd57, 9bb4b26a7a2b55ea413fed5f3bd82758,
a6b0b26afebc29c4c1140288a6e24143, aea5b16ada5db98bee2b319fa34f8ec1, c9e7b26a5471b1e62ada12b90ad9378b,
d7a7b16ab0e5e842a8d3181204f3b87d, ddd5b26ab214d493b449ca397998298c, e7abb16a46e9a36d9a1616ed1d886394,
fec4b06a503f97922bf620d7ac25e728 (won); 1abab26ad021944255a890ffee3cfaf8, 322bb36ac73bdb1d7f4c5ea8f23508a7,
4632b06a6b0aec0595a36a7025ab7f4b, 68f7b36acc1035b103b8cce339f4e4d1, 70d6b16afe1c617409a3d8c434292704,
e139b36a1f3ec62af45fdd67e03baaa7, e3dab26a287fbfb1c149fcaf82179003 (lost).
**Would be wrong if.** More A4 games showed the south corner (spot_54, spot_62 in A6) before the north strip in most.
**Used by.** (candidate: the player's brief, the Comet Catcher expansion list from the west)

### K-map-comet-catcher-remake-1-8-h5-winners-go-south
**Claim.** From H5 (19 sides) the winners took the south strip first and the losers the north: spot_60 (G6) at 2:06 in 7
of 7 winners against 3:00 in 10 of 12 losers; spot_65 (H7) 3:14 in 7 of 7 against 4:53 in 9 of 12; spot_72 (H8) 4:00
(7 of 7) against 5:33 (8 of 12); spot_57 (G6) 4:09 (7 of 7) against 6:08 (8 of 12); while spot_25 (H3) went at 4:42 in
10 of 12 losers against 6:07 in 4 of 7 winners. The common order: spot_48 (H5), spot_51 (H5), spot_42 (G5) 0:48,
spot_34 (G4) 2:12, spot_60 (G6), spot_29 (G4), then spot_65 (H7), spot_57 (G6), spot_72 (H8), spot_77 (G8), spot_53 (F6).
The south strip is the mirror of A4's north strip (the map is point-symmetric), and constructors built it (spot_60 17
of 17, spot_65 16 of 16).
**Status.** observed (2026-09-23, replay survey)
**Evidence.** H5 winners: 02bdb16aa5ed8ab92635aa50d3ff80ff, 322bb36ac73bdb1d7f4c5ea8f23508a7, 4632b06a6b0aec0595a36a7025ab7f4b,
70d6b16afe1c617409a3d8c434292704, 77c1b16aae254c3828e41af2948cb0a8, 9d32b36a73aa57673fa0fe2b404888ce,
e139b36a1f3ec62af45fdd67e03baaa7; losers: 17c9b26aebb2301d97c86728aa03ec47, 4dfdaf6ac5933a177c5180e7e41eae49,
58ffaf6a9ac117a532729b2e845bcce1, 7fa2b26a71c0cfe4bf78e42b6932bd57, 8ab1b16ade373a81104fe441581fa6b4,
9bb4b26a7a2b55ea413fed5f3bd82758, a6b0b26afebc29c4c1140288a6e24143, aea5b16ada5db98bee2b319fa34f8ec1,
d7a7b16ab0e5e842a8d3181204f3b87d, d867b06aede3ee9a9b3c58f57054970a, ddd5b26ab214d493b449ca397998298c,
fec4b06a503f97922bf620d7ac25e728.
**Would be wrong if.** The pattern is the players, not the spots: four of the seven H5 wins are one player
([APM]Hellontoast, Cortex, 77c1, 02bd, e139, 70d6); more H5 games by other players could erase it.
**Used by.** (candidate: the player's brief, the Comet Catcher expansion list from the east)

### K-map-comet-catcher-remake-1-8-b5-winners-go-south-east
**Claim.** From B5 (12 sides) the winners went south-east along spot_52 (C5) 2:47 (7 of 7), spot_61 (C6) 3:17 (7 of 7),
spot_63 (C7) 3:32 (6 of 7), spot_64 (D7) 3:44 (5 of 7), spot_59 (D6) 4:44 (6 of 7), and took the back corner (spot_54,
spot_62 in A6) only at 5:56-6:19; the losers took the back corner first (spot_54 (A6) 2:40, spot_62 (A7) 3:12, in 3 of
5) and spot_52 (C5) at 3:54 (4 of 5), spot_61 (C6) in 2 of 5.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** B5 winners: 17c9b26aebb2301d97c86728aa03ec47, 340db46a51552a0cf03dfc39998a1fa2, 4dfdaf6ac5933a177c5180e7e41eae49,
58ffaf6a9ac117a532729b2e845bcce1, 8ab1b16ade373a81104fe441581fa6b4, caddb26aacbf1134776b8c7388718e63,
d867b06aede3ee9a9b3c58f57054970a; losers: 02bdb16aa5ed8ab92635aa50d3ff80ff, 34e4b26a49e860e2f5b9c3bc9e6a75e1,
77c1b16aae254c3828e41af2948cb0a8, 9d32b36a73aa57673fa0fe2b404888ce, bcaeb16a38fe69a4b9575cffbfd459e8.
**Would be wrong if.** Other players from B5 won as often by the back corner: three of the seven winners are one player
([APM]Nekuodah: 17c9, 4dfd, 58ff), and five sides is thin for the losers.
**Used by.** (candidate: the player's brief, the Comet Catcher expansion list from the west)

### K-map-comet-catcher-remake-1-8-g4-and-h1-expansion
**Claim.** From G4 (8 sides): spot_34, spot_29, spot_35 (all G4) by 1:25, then spot_27 (F4) 2:48 (7 of 8), spot_41 and
spot_40 (F5) 2:56-3:09 (7 of 8, the commander in 6), spot_42 (G5) 3:45 (6), spot_25 (H3) 4:09 and spot_17 (H3) 4:44
(8 of 8, constructors), spot_12 (H2) 5:43 and spot_5 (G1) 6:27 (6). From H1 (2 sides, both lost): spot_12 (H2), spot_5
(G1), spot_10 (G1) 1:06, spot_6 (G1) 2:09, spot_17 (H3), spot_11 (F2), spot_25 (H3), spot_3 (F1), spot_29 (G4) 4:34.
Five and three G4 sides are too few to split winners from losers.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** G4: 1abab26ad021944255a890ffee3cfaf8, 34e4b26a49e860e2f5b9c3bc9e6a75e1, 68f7b36acc1035b103b8cce339f4e4d1,
bcaeb16a38fe69a4b9575cffbfd459e8, e3dab26a287fbfb1c149fcaf82179003 (won), 25e2b26a0ca652a85ae574d70c1335ae,
c9e7b26a5471b1e62ada12b90ad9378b, caddb26aacbf1134776b8c7388718e63 (lost); H1: 340db46a51552a0cf03dfc39998a1fa2,
e7abb16a46e9a36d9a1616ed1d886394.
**Would be wrong if.** More G4 games put the north spots (H3) before F5.
**Used by.** (candidate: the player's brief)

### K-map-comet-catcher-remake-1-8-extractor-curve
**Claim.** Extractors standing and metal income (median, winners against losers): 2:00 3 and 3 (8.9 and 8.9, 29 each);
4:00 8 against 6 (18.1 against 15.8, 29); 6:00 12 against 11 (27.2 against 24.9, 29); 8:00 18 against 15 (38.7 against
33.0, 25 games still running); 10:00 23 against 15.5 (55.9 against 40.4, 22); 12:00 28 against 14.5 (68.1 against 36.4,
18). Constructors: 4 against 3 at 5:00 (29), 7 against 5 at 10:00 (22), 9.5 against 6 at 12:00 (18). By start cell at
8:00 (winners/losers): A4 18/18 (8 and 7 sides), B5 16/11 (5 and 5), G4 18/12 (5 and 2), H5 17/16 (7 and 9).
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 (`curves` per minute).
**Would be wrong if.** A recount by `run/replays/check.py` gave other medians (these match the design's 15:45 first pass:
winners 8/18/28, losers 6/15/14.5).
**Used by.** (candidate: the bot's economy targets on Comet Catcher; the plateau at 22-25 from 16:00 in pace-1 sits under
the winners' 28 at 12:00)

### K-map-comet-catcher-remake-1-8-second-factory
**Claim.** A second factory was finished in 44 of 58 sides, started at 8:30 (median; winners 8:45 in 23 of 29, losers
7:56 in 21 of 29). Of the 22 sides of each in games reaching 10:00, 18 winners and 13 losers had a second factory by
10:00. The winners' second was a vehicle plant in 15 of 23 (air plant 4, bot lab 4); the Armada losers' was a bot lab in 6
of 9; every Cortex lab opener that built a second built a vehicle plant (9 of 9). The higher-OS player started a second
factory before the lower in 17 of 29 games (later in 8, neither in 4).
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 (`factories`, counting only factories the card shows finished; see Card defects for the unfinished ones).
**Would be wrong if.** A second factory by 9:00 were as common among losers once games under 10:00 are excluded.
**Used by.** (candidate: the player's brief, second-factory timing)

### K-map-comet-catcher-remake-1-8-tier-2-is-rare
**Claim.** Tier 2 comes in 4 of 58 sides and 3 of 29 games: a Cortex advanced vehicle plant at 12:51 (H5, won), a
Cortex advanced aircraft plant at 14:57 (B5, won), and both sides of one 20-minute game (Armada advanced vehicle plant
16:08, B5, won; Cortex 19:42, H1, lost). Of the 10 games that reached 15:00, 2 had a tier-2 factory by then. The games are
decided on tier 1: 26 of 29 ended with no tier-2 factory on either side.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** 4632b06a6b0aec0595a36a7025ab7f4b, 17c9b26aebb2301d97c86728aa03ec47, 340db46a51552a0cf03dfc39998a1fa2; all 29 for
the absence. The first tier-2 unit is not on the cards (see Q3 note below).
**Would be wrong if.** Longer games (none past 21:31 here) went to tier 2 as a rule.
**Used by.** (candidate: the tier-2 timing in the brief: none before 12:00 on this map)

The cards carry the first tier-2 factory but not the first tier-2 unit's clock; the army every two minutes shows one
(armlatnk, 5 at 18:00 in 340db46a51552a0cf03dfc39998a1fa2). A `first_tier2_unit` field would answer it.


The cards cannot give the first contact: `first_enemy_seen` is 0:00 on all 58 sides (the other commander is seen at
the start). The card would need the first enemy unit other than the commander seen, and the first damage dealt or taken,
with clock and cell.

### K-map-comet-catcher-remake-1-8-first-extractor-lost
**Claim.** Every one of the 58 sides lost an extractor; the first at 4:18 (median; before 3:00 in 18 sides, before 5:00 in
37). Winners lost their first at 4:20 (7 before 3:00), losers at 3:41 (11 before 3:00). By start cell: G4 3:22 (8), H1 3:40
(2), H5 4:15 (19), A4 4:35 (17), B5 5:10 (12). The killer where the card names it (48 of 58): scout cars 21 (Rover 11,
Rascal 10), Incisor 10, Blitz 8, Grunt 6, other 3. The spot lost first most often from A4 is spot_19 (B3) (4 of 17) and
spot_36 (B4) (3); from H5 spot_57 (G6) (3 of 19) and spot_34 (G4) (3). Before 8:00 both winners and losers rebuilt a
median of 2 extractors (67 and 71 rebuilds in all).
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 (`first_extractor_lost`); the 10 sides with killer "?" are listed under Card defects.
**Would be wrong if.** The raid clock differed by more than a minute between the mirrored starts in more games (G4 3:22
against B5 5:10 over 8 and 12 sides is the only start difference here).
**Used by.** (candidate: the bot's first-turret timing: a raid on an extractor from 2:00)

### K-map-comet-catcher-remake-1-8-raids-met-by-turrets
**Claim.** Raids are met by turrets and a scouting swarm, not by holding the army home: light turrets by 8:00 median 9
for winners against 6 for losers (the winner more in 19 of 29 games), 26 sides with one before 1:00 (18 winners); and
the vehicle plants send five or more scout cars by 3:00 in 38 of 45 sides, winners and losers alike (median 7.5
against 8). The fights of minutes 0-5 are even (winners lost 96 units, losers 100, in the eight top fight cells per game)
and fall in each side's own half about equally (99 against 97 deaths), which is raiding both ways, not a battle.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29.
**Would be wrong if.** A card with unit orders showed the winners holding units at their extractors. The cards carry no
positions of units over time, so "held back or chasing" cannot be read; the card would need the army's centroid or the
soldiers' cells per minute beside the fights.
**Used by.** (candidate: the brief's raid response on Comet Catcher)

### K-map-comet-catcher-remake-1-8-fights-on-rows-4-5
**Claim.** The deaths in the cards' fight lists (the eight costliest cell-minutes per game, 3,477 deaths in all) are on
rows 4 and 5 (2,370, 68%): by column G 764, B 692, D 608, E 386, F 313, C 259, H 257, A 198; the costliest cells G5 354,
B5 350, B4 296, D5 268, G4 202, D4 179, E5 165. The routes between the bases are the middle band B4/B5, C4/C5, D4/D5,
E4/E5, F5, G4/G5; rows 1-3 and 6-8 hold 13% and 18%. The fighting moves into the loser's half as the game goes: minutes
0-5 half and half (99 against 97 deaths), 6-9 479 of 697 in the loser's half, 10-13 668 of 939, 14 on 1,390 of 1,645,
and from minute 6 the loser loses two to three and a half times what the winner does (481 against 216, 622 against 317, 1,285 against
360).
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 (`fights`, taken once per game).
**Would be wrong if.** The full fight record (not the top eight) put most deaths off rows 4-5. **Refines**
K-maps-comet-middle-spots-die-first: the middle column D (D4, D5) is the third-costliest after the two home columns.
**Used by.** (candidate: routes for the bot's army on Comet Catcher)

### K-map-comet-catcher-remake-1-8-decided-by-the-8-to-12-gap
**Claim.** Games are decided by the extractor gap opening between 8:00 and 12:00, with the army ahead with it, not by a
raid or tier 2. Winner minus loser (median): extractors 0 at 2:00 (winner ahead in 4 of 29), 1 at 4:00 (16 ahead of 29),
2 at 6:00 (17), 3 at 8:00 (17 of 25), 6.5 at 10:00 (17 of 22), 12 at 12:00 (15 of 18); army metal ahead in 14 of 29 at
4:00, 18 of 29 at 6:00, 18 of 25 at 8:00, 16 of 18 at 12:00. From 8:00 to 12:00 the winners added a median 11 extractors
and the losers 1 (18 games reaching 12:00); the winners had 25 or more at 12:00 in 14 of 18, the losers in 3.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29; the 18 games reaching 12:00: 17c9b26aebb2301d97c86728aa03ec47, 1abab26ad021944255a890ffee3cfaf8,
322bb36ac73bdb1d7f4c5ea8f23508a7, 340db46a51552a0cf03dfc39998a1fa2, 34e4b26a49e860e2f5b9c3bc9e6a75e1,
4632b06a6b0aec0595a36a7025ab7f4b, 58ffaf6a9ac117a532729b2e845bcce1, 68f7b36acc1035b103b8cce339f4e4d1,
70d6b16afe1c617409a3d8c434292704, 7fa2b26a71c0cfe4bf78e42b6932bd57, 9bb4b26a7a2b55ea413fed5f3bd82758,
9d32b36a73aa57673fa0fe2b404888ce, a6b0b26afebc29c4c1140288a6e24143, c9e7b26a5471b1e62ada12b90ad9378b,
d7a7b16ab0e5e842a8d3181204f3b87d, d867b06aede3ee9a9b3c58f57054970a, e139b36a1f3ec62af45fdd67e03baaa7,
e7abb16a46e9a36d9a1616ed1d886394.
**Would be wrong if.** The losers' flat 8-12 curve were the result of losing (extractors killed) rather than a cause: the
card's standing count cannot tell built-and-lost from never built; per-minute extractors built and lost would.
**Used by.** (candidate: the bot's economy target past 8:00; see K-eco-winner-pulls-away-8-to-12)

### K-map-comet-catcher-remake-1-8-endings
**Claim.** Games last 13:39 (median; 6:57 to 21:31): 7 under 10:00, 12 from 10 to 15, 10 over 15. Every card lists the
loser's commander and all its factories among the winner's kills (29 of 29), and in 28 of 29 the last minute's fights are
in or beside the loser's start cell (the one other: the loser's army died in the winner's B4). At the end the winner had
more extractors in 26 of 29 (median 29 against 8) and more army metal in 27 (median 2.3 times; twice or more in 20).
Three winners closed while behind or level on extractors (02bd 13 against 16, 70d6 18 against 29, d7a7 26 against
26), each with the larger army (2.2, 0.8 and 2.6 times; 70d6 won a D4/D5 battle at 16:00 with less).
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29.
**Would be wrong if.** The card's kill table counts a resign's self-destruct as kills: then "the commander hunted" and "a
resign" cannot be told apart here (see Card defects), and the claim is only that the loser's base went in the last
minute. K-open-comet-flash-raids-take-the-resign had a resign by self-destruct at 6:45.
**Used by.** (candidate: when the bot should expect the other side to fold)

### K-map-comet-catcher-remake-1-8-higher-os-differences
**Claim.** The higher-OS player won 22 of 29. Paired within a game, the higher-OS side: started a second factory earlier
in 17 (later in 8); built more solars by 8:00 in 18 (fewer in 8; median 2 more); had more energy stored at 6:00 in 20 of
29; had the turret before 1:00 when the other did not in 10 games (the reverse in 6); had more extractors at 10:00 in 14
of 22 (median 4.5 more) but at 4:00 and 6:00 only 12 and 13 of 29. It did not plant earlier (earlier in 13, later in
15) or make more scout cars by 3:00 (12 against 13).
**Status.** observed (2026-09-23, replay survey); at OS 40 and above (2026-09-24, 33 games) the higher-OS side won 18 of 33, 55%: the habits below are the pool's, the result at the top is closer to even.
**Evidence.** all 29; `run/replays/recheck.py`.
**Would be wrong if.** The differences held between winners and losers of equal OS but not by OS (that is, skill shows as
the result, not as habits). Paired by result instead: the winner had more solars in 20 of 29, more energy income at 8:00
in 19 of 25, more extractors at 4:00 in 16 (fewer in 4).
**Used by.** (candidate: the brief's energy and second-factory lines)

