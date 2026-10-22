# Part d: units, hands, opponent-barb, players-words, team-games, other-maps; the reference tables

Reader D, 2026-10-21, working tree at d44e318. Rows in `part-d.jsonl` (152 rows; every non-blank line of the fourteen files is inside a row; bare `###` headings of team-games.md are folded into the first row under them). The reference files are annotated one row per section, as the README asks, with the tool that makes each section named in the row (`run/reference_set.py` for the two faction tables, `run/timelines.py` for the whole games, `run/opponent_set.py` for the BARb table, `run/frontier_set.py` for the frontier files); the per-map frontier files share one shape, so their rows repeat with the map and the source game changed.

## Counts

By layer (what the passage is now): meta 38, episode 30, derived 25, mixed 23, fact 23, tool 12, ruling 1.

By fate (what it would become): meta 37, episode 29, fact 29, rederive 27, tool 25, discard 5.

By file: exp/hands.md 12, exp/opponent-barb.md 7, exp/other-maps.md 27, exp/players-words.md 14, exp/team-games.md 23, exp/units.md 23, ref/comet-catcher-remake-1.8-arm-games.md 4, ref/comet-catcher-remake-1.8-arm.md 10, ref/comet-catcher-remake-1.8-barb-hard_aggressive.md 2, ref/comet-catcher-remake-1.8-cor.md 10, ref/frontier-comet-catcher-remake-1-8.md 5, ref/frontier-great-divide-v1.md 5, ref/frontier-quicksilver-remake-1-24.md 5, ref/frontier.md 5.

Doctrine flags: rule-from-one-case 8, rule-from-ruling 5, rule-from-pool 2. Rows with a conflict: 21. Rows with a glossary check: 26, of which 3 carry a mismatch.

The shape of this part: the hand-written files split three ways. `units.md` is nearly all fact (23 rows, 18 of them tables or arithmetic that check against the glossary) with a few derivations that already carry their premises. `hands.md` is tool text that should move whole to the role prompt, with four episodes riding on it as justification. `opponent-barb.md` is fact read from BARb's source, with two one-game observations stated as rules. `players-words.md` is meta of named people on named days, the cleanest meta in the brief. `team-games.md` and `other-maps.md` are episodes, most of them from the week before the hands rebuild (2026-09-27/28), each carrying a defect since fixed. The reference tables are meta of the OS 40+ pool with the derivation of the opening numbers attached (the one section in the brief already in the rework's shape), the BARb table is meta of the profile against an older us, and the frontier is a derivation with its premises stated and its ceiling caveat in the first line.

## The derivation backlog

Subjects whose passages are `rederive`, with the lessons the derivation must reproduce or explain away.

| subject | where | the lessons it must reproduce | premises |
|---|---|---|---|
| The odds on the spot | units.md:34-43 part 3 | metal of soldiers on the spot decides; a turret counts about three times its metal; the heavier side loses less nine times in ten, beyond two to one almost always | units.json dps/health/reach; the combat simulator; our fight records (which games measured the nine-in-ten is not stated) |
| Tier-1 matchups | units.md:34-43 parts 1-2 | Mace/Centurion/Thug win most bot matchups and beat light turrets; a Pawn beats only a Grunt; a Rocketeer loses to what it cannot out-range and out-ranges the Sentry (475 vs 430); a Stout takes a turret with friends but does not out-range it; Whistlers and Shellshockers do; a Blitz is two Pawns' metal with twice the health (the 'twice the speed' is false: 101 vs 87) | units.json through the simulator |
| Spacing against splash | units.md:57-59 | 25 Stouts vs 4 Fatboys lose by 0.55 at 56 spacing, 0.22 at 160; 21 vs 5 Bulls 0.34 to a draw; 6 vs 1 Fatboy +0.12 to +0.52 | the splash table (units.md:45-55); the pros' spacing (64 in formations.md, 67-70 here) |
| The moho's payback and the sprint's entry cost | units.md:93-96; late-game.md:22 (reader C) | a Comet moho adds 6.9 a second, 90 s of its output repays it; plant + constructor 3,150 = 61 s at 52 a second; the '3.5 minutes at 9 a second' must come out as 5:50 | armavp 2,600 + armacv 550; the sheet's spot value and the moho multiplier |
| Build power a project can use follows the spend | units.md:123-129 | armafus at 100/s: ~3,200 power, 97 s, 710 E/s; fusion at 50/s: 810, 67 s; plant at 30/s: 310, 87 s; a nano ring serves what stands within 400 | K-t2-build-power-for-a-project-follows-the-spend (already a derivation; keep) |
| The commander assists the lab at birth | players-words.md:12 | thebluegecko's word after game 10; mechanics.md:487 'demonstrated' | commander 300 build power against the lab's 150; the first units' build times; the store at the lab's birth |
| The team sprint | players-words.md:52-57; team-games.md:51-70; winning.md/cases.md (reader C) | two seats gift to one, one plant, build power pushes it through (game 10's plant at 0% with one constructor; Center Command's 11 extractors + 1,165 bank at 7:06, standing 1:46 later, five advanced constructors by 10:15, mohos at 19-33 s with nanos) | build power follows the spend; the moho's payback; transfer's cap; the pool's tier-2 rarity in duels (1 of 41, 3 of 39) |
| Ahead on the trade and lost | team-games.md:25 | three 3v1s lost while ahead on seen-destroyed metal | the trade counts seen deaths and summed the seats (the ledger's corrections of 2026-09-27); extractors lost a minute (reference: winners 0.99, losers 2.52); a loss is the commander or the economy |
| No anti-air before the first aircraft | team-games.md:45-49; units.md:137-139 | an air plant in sight meant Banshees ~90 s later (human-9); bombers at 17:00 with no anti-air (3v1-1); Liche killed two commanders (Cape Violet) | armap/armkam/armliche build times at his build power; the Nettle's 80 metal; the commander's 3,700 health against the bomb |
| Few spots per seat and a yard first | other-maps.md:80-81; team-games.md:83-85 | a seat with one or two extractors for eight minutes had no economy (human-17, SailAway 12) | spots reachable per seat x income per spot; shipyard/hover platform cost; the start store |
| A ball into standing turrets it cannot out-range dies | other-maps.md:97-98; player.md's Great Divide section (reader B) | the Comet nest with a Gauntlet took 9k and 11k of tier-1 tanks (player-8/9, 15.9k for 5.6k and 10.2k for 1.3k) | Gauntlet 1,220 and heavy laser 620 against Stout 350; turret dps per metal in reach |
| The frontier (keep as is) | ref/frontier.md and the three per-map files | fastest times to 50/100/200 income, moho, fusion, anti-nuke, missile under three taxes; the nuke line against BARb's 21-25 window | the engine's build and stockpile arithmetic; the map's spots; the tax from the pros' army growth (a meta premise, Comet's, applied to all three maps) |
| The opening numbers from the sheet (keep as is) | ref/*-arm.md:81-84, ref/*-cor.md:81-84 | the factory at 0:34/0:59 after 2-3 extractors and 2 generators within reach; crossing times from speeds over half the start distance; 2.0-2.3 income per extractor against the sheet's 2.30 | the map sheet; unit speeds (checked) |

## The meta catalogue

Entries for the meta layer: what a population does, with population, map and date.

| population, map, date | where | what they do |
|---|---|---|
| the OS 40+ pool, Armada, Comet Catcher Remake 1.8 (41 sides of 30 duels, OS 41-51, regenerated 2026-10-17) | ref/comet-catcher-remake-1.8-arm.md | openings (22 of 41 mex-mex-solar-solar-vp), first factory armvp 40 of 41, the clock of milestones, the game by the minute, where the extractors stand, nano before the second factory 23 of 24, tier 2 in 1 of 41, Blitz 84-88% of the army minutes 6-10 |
| the OS 40+ pool, Cortex, Comet Catcher (39 sides of 29 duels, OS 40-54) | ref/comet-catcher-remake-1.8-cor.md | openings split (11 of 39 with an early Guard; corvp 24 to corlab 15), later first soldier in his half (2:48 vs Armada's 1:40), nano at 5:06, tier 2 in 3 of 39, Incisors 67-76% minutes 6-10, Brutes from 14 |
| named pool players (Artur91, Baldric, f1sher, Chronopolize, thebluegecko in a duel) | ref/*-arm.md:43-55, ref/*-cor.md:43-55, ref/*-arm-games.md | per-player quirks: Artur91's five rows in the top quarter; thebluegecko beating Artur91 at 20:59 in the longest Armada loss |
| BARb hard_aggressive on Comet (110 of our games to 2026-10-03, most before the hands rebuild) | ref/comet-catcher-remake-1.8-barb-hard_aggressive.md; exp/opponent-barb.md | bot lab 97 of 110 at 0:54, first soldier in our half 1:50, our first extractor lost 3:11, the advanced factory 9:57/10:44/11:45 in 83 of 110, 15-17 extractors and 2,154-3,307 army at that moment, Centurions/Rocketeers/Pawns to minute 12 then Sharpshooters, Fatboys, Razorbacks |
| thebluegecko (OS 48 in duels, 34 in team lobbies), Comet Catcher 3v1s and lobbies, 2026-09-27/28, 10-03, 10-19 | exp/players-words.md; exp/team-games.md:9-25, 59-70; exp/other-maps.md:42-46, 59-68, 72-79 | single raiders from 2:06, Stout/Janus blocks, bombers at 17:00, Welders and Liche on water maps; his words on Lashers, Brutes, Mangonels, hovers, the t2 transition, chat conduct; his team sprint on Center Command |
| Irishstud14 (OS 48, Cortex), Comet Catcher 3v1 game 4, 2026-09-28 | exp/team-games.md:20-22; exp/players-words.md:17-22 | Pounders at home, Incisor swarms of 10-15 past the Brute body; 'metal heavy map', 'add nanos' |
| u6bkep (OS 19), games 7-8 and Center Command, 2026-09-27 and 10-19 | exp/players-words.md:21; exp/team-games.md:59-65 | '3v1 is about fair'; built Cortex mohos with a handed-over corack |
| Matt (one replay), Quicksilver, 2026-09-20 | exp/other-maps.md:13-20 | two extractors, three winds, the lab at 0:33, seven winds and a solar in 37 s, one constructor first then Pawns every 11 s, raiders out at 2:30, the commander never left |
| players on Quicksilver (human-1..6 chat), 2026-09-22 | exp/players-words.md:13-15; exp/other-maps.md:10-12, 32-34 | wind averages 12; about twenty energy per metal spent; two constructors is greedy; 30 Pawns before Mace; moving the commander costs build power |
| the one Legion player above OS 25 (OS 27), Great Divide, two wins | exp/units.md:166-168 | extractor, extractor, wind, wind, solar, bot lab at 0:42; Goblins, a Zagreus; Satyrs with Ballistas from 4:00; drones from Hives after 10:00 |
| what people are said to do on Great Divide (second-hand, the user 2026-09-27) | exp/other-maps.md:92-94 | camp the pass with light turrets by 3:00, heavy by 6:00, Pit Bulls/Gauntlets with artillery; the one game there had the person not fortify it |
| the pool's wreck use (29 Comet duels) | exp/units.md:131-135 | 24 of 58 sides built rez bots, a median of ten, first at minutes 4-10, on their own ground |

## The episode index

Every `episode` row (and the mixed rows that carry one), with the harness defects of its time and the review lens.

| where | game(s) | date | lens | defects of the time |
|---|---|---|---|---|
| exp/hands.md:21-22 | human-7 | 2026-09-22 | ledger | human-7 (2026-09-22): produce had no count cap (the `corck:1` cap was built from this game); the old hands before the 2026-10-02 rebuild |
| exp/hands.md:23-27 | realtime-2 | 2026-09-21 | ledger | realtime-2 (2026-09-21): the pre-rebuild hands with no standing raid rule (the detachment rule was built after; chase recall fixed 2026-10-17); the lesson is against a defect since fixed |
| exp/hands.md:36-41 (part 2) | rove-task-1, player-20-live, player-21-leash, player-9 (series unknown), player-10 (series unknown) | 2026-10-08, 2026-09-29 | ledger | rove-task-1 (2026-10-08): the brief lacked 'leave it to rove' (fixed 12a8db8), the blocked-site watchdog's false alarms (fixed 08599ab); player-20/21 (2026-09-29, before the hands rebuild and before rove was a group task): lane rove's solo tours, the follow leash |
| exp/other-maps.md:21-31 | human-1, human-3, human-5, human-8 | 2026-09-22 | ledger | 2026-09-22 builds: the lab off the spots and the mirror guess (36cbfd7), no queue-ahead until human-6, the produce cap only from human-8; the old hands sent raiders in ones and twos |
| exp/other-maps.md:35-37 | pianist-player-6, pianist-player-3 | 2026-09-21 | ledger | the pianist's first week: fight_to committed to everything (fixed in game 2), no raid answer in the hands |
| exp/other-maps.md:42-46 | bluegecko-3v1-cape-violet | 2026-09-28 | ledger | bluegecko-3v1-cape-violet (2026-09-28): under-water spots unlisted and the shipyard site snapped to the builder's ground (both fixed b9a43ff); a false lane wake; the advice block's (f) sent the army into the camp |
| exp/other-maps.md:47-48 | human-16 | 2026-10-03 | unknown | unknown (no ledger row; the cannot-fire record was new) |
| exp/other-maps.md:59-62 | bluegecko-3v1-comet-catcher-11 (SailAway 2) | 2026-09-28 | model | the build before the tidal, start-box and under-water work (b6e5819): water seats had no tidal words |
| exp/other-maps.md:63-68 | bluegecko-3v1-comet-catcher-12 (SailAway 2) | 2026-09-28 | model | the water words new (b6e5819); the brief's sonar rule absent; the player's own 4:09 produce list |
| exp/other-maps.md:72-77 | human-17 | 2026-10-03 | ledger | the first game on the map with the people block and the map sheet; not reviewed |
| exp/other-maps.md:95-96 | team-2v2-smoke, team-2v2-board, commander-team-1, bluegecko-2v1-great-divide | 2026-09-20, 2026-09-27 | ledger | the heuristic bot before the pianist (another regime); K-team-seats-alone-split-the-army, K-team-a-partner-dies-beside-an-idle-army |
| exp/other-maps.md:97-98 | onepass-player-8, onepass-player-9 | 2026-09-27 | model | the one-pass build of 2026-09-27: the fall-back home walks, the thrash at a nest (player-40-era churn), no march |
| exp/players-words.md:63-68 | player-9 (series unknown), comet-1-easy, escalate-4-hard-aggressive, player-50 | 2026-09-22, 2026-10-03 | person | comet-1 (2026-09-22): the plant deterred by the draw warning (K-hands-plant-deterred-by-the-draw-warning, fixed); escalate-4 (2026-09-22): an old assist list kept the commander off the menu; player-50 (2026-10-03): the march fixes of that day, friendly fire 23% |
| exp/team-games.md:9-14 | bluegecko-3v1-comet-catcher | 2026-09-27 | ledger | bluegecko-3v1-comet-catcher (2026-09-27): the standing tool stored rules on whichever seat read first (fixed 8cc9ac6); no station in the ball's slot; the old hands |
| exp/team-games.md:15-16 | bluegecko-3v1-comet-catcher-2 | 2026-09-27 | ledger | bluegecko-3v1-comet-catcher-2 (2026-09-27): packet paragraphs to tagged groups missed (fixed d64864a); the retreat walked a builder home into the party (fixed a9cd2f0); the old hands |
| exp/team-games.md:17-19 | bluegecko-3v1-comet-catcher-3 | 2026-09-28 | ledger | bluegecko-3v1-comet-catcher-3 (2026-09-28): the Cortex lists refused (the lead seat's roster only, fixed 6872ff4); attack orders refused on stale party names; remove knew one seat (fixed c42f6f6); constructors in lab exit lanes |
| exp/team-games.md:20-22 | bluegecko-3v1-comet-catcher-4 | 2026-09-28 | ledger | bluegecko-3v1-comet-catcher-4 (2026-09-28): the first turn fired with two of three seats; three of five factories' exit lanes blocked; the old hands with no raid answer |
| exp/team-games.md:23-23 | human-15 | 2026-10-03 | unknown | none |
| exp/team-games.md:27-31 | bluegecko-3v1-comet-catcher | 2026-09-27 | model | the standing tool per seat broken (fixed 8cc9ac6); no raid rule in the old hands; the station slot empty |
| exp/team-games.md:32-34 | bluegecko-3v1-comet-catcher-2 | 2026-09-27 | model | pre-rebuild hands; the retreat that walked a builder into the party |
| exp/team-games.md:35-37 | bluegecko-3v1-comet-catcher-3 | 2026-09-28 | ledger | stale party names refused attack orders (the hands attack what stands at a place); a fallback walked a group home instead of to spot_31 |
| exp/team-games.md:38-38 | bluegecko-3v1-comet-catcher | 2026-09-27 | ledger | no march order in the old hands (the march's fixes 9ba2889, 2026-10-03) |
| exp/team-games.md:40-43 | bluegecko-3v1-comet-catcher-2, bluegecko-3v1-comet-catcher | 2026-09-27 | ledger | the retreat rule walked the commander into the party (fixed a9cd2f0); a queued list held the commander (fixed); 'told to stay home' was the player's order |
| exp/team-games.md:45-49 | bluegecko-3v1-comet-catcher, bluegecko-3v1-cape-violet, human-9 | 2026-09-27, 2026-09-28, 2026-09-22 | ledger | the old hands; Cape Violet's under-water spots unlisted (fixed b9a43ff) |
| exp/team-games.md:51-56 | bluegecko-3v1-comet-catcher-10 | 2026-09-28 | ledger | a list step naming a frame started a second frame (frames looked up by type and distance, fixed 2026-10-17); the lobby boxes not honoured in this game; the old hands |
| exp/team-games.md:57-58 | bluegecko-3v1-comet-catcher-12 (SailAway 2), human-10 | 2026-09-28, 2026-10-03 | ledger | none |
| exp/team-games.md:59-65 | Center Command 2v4 | 2026-10-19, 2026-10-20 | ledger | cross-faction constructor words and a tagged plant step fired twice (two harness faults named in the review, nothing changed) |
| exp/team-games.md:65-70 (part 2) | Center Command 2v4 | 2026-10-19, 2026-10-20 | ledger | the commander's three asks unanswered by the seats (the standing transfer built 2026-10-19/20 after); a tagged plant step fired twice; mohos each on one constructor |
| exp/team-games.md:74-79 | human-10 | 2026-10-03 | ledger | human-10 (2026-10-03): queue/produce took internal names only (role words built 6b28423 the same night); the hands played prose before the lists landed |
| exp/team-games.md:80-82 | human-11 | 2026-10-03 | ledger | human-11 (2026-10-03): builders without a list defaulted to helping (the human-11 fixes: staging lists, help words, built 2026-10-03) |
| exp/team-games.md:83-85 | bluegecko-3v1-comet-catcher-12 (SailAway 2) | 2026-09-28 | person | the player's own produce list of 4:09 never lifted; no sonar; the water words newly built (b6e5819) |
| exp/team-games.md:88-95 | human-12 | 2026-10-03 | ledger | human-12 (2026-10-03): the attack/walk thrash at one-second turns (the thrash pair built 2026-10-03; the shell standoff left); Crossbows counted as long-reach soldiers |
| exp/units.md:67-71 | Center Command 2v4 | 2026-10-19 | ledger | none |
| exp/units.md:137-139 | human-9 | 2026-09-22 | ledger | human-9 (2026-09-22): the spot table off at that start (an extractor refused, H-HANDS-REFUSED), the old hands; the lesson's timing does not depend on them |
| ref/comet-catcher-remake-1.8-arm-games.md:5-24 | unknown | 2026-10-03 | ledger | none |
| ref/comet-catcher-remake-1.8-arm-games.md:26-39 | unknown | 2026-10-03 | ledger | none |
| ref/comet-catcher-remake-1.8-arm-games.md:41-68 | unknown | 2026-10-03 | ledger | none |

## Numbers checked against docs/glossary/units.csv

About 130 single lookups, all in units.md, other-maps.md and the reference derivations. Everything in the tier-1 tables, the tier-2 cost table (57 cells), the builders table (every ratio, speed and health), the Legion roster, the splash ranges, the Great Divide reach list and the naval reaches matches, with these exceptions:

- `units.md:38`: "twice a Pawn's speed" is false (Blitz 101, Pawn 87); the metal and health halves hold.
- `units.md:94-96`: 3,150 metal at 9 a second is 350 s (5:50), not "three and a half minutes".
- `other-maps.md:57`: Ellysaw `armpship` reaches 470 in the csv, not 500.
- `units.md:30`: the Cortex Guard `corllt` is 435 where the text gives 430 for both towers (minor).
- Names: the csv calls `corpun` the Agitator and `armmerl` the Ambassador where `other-maps.md:89-91` says Punisher and Pillager (the game renamed them; the numbers match).
- `units.md:118`: resurrection bots "cannot assist" while the csv lists their build power as 200; the claim rests on a canAssist flag the csv does not carry (unchecked).
- `units.md:62`: "a factory draws about 80 energy a second": the csv's arithmetic at the commander's 300 power gives 82 for a vehicle plant and 57-63 for a bot lab; economy.md:145 has the 80 as observed in human-3 (a Cortex lab, 56 a second net of the commander's 30).
- Arithmetic that checks: the build-power-follows-the-spend worked examples (3,221 power / 97 s / 711 E/s; 806 / 67 s; 311 / 87 s), the vehicles' 8-14 energy per metal, the moho's 90 s payback, the reference derivations' crossing times (18/29/40/55 s Armada, 21/38/63 s Cortex).

## Conflicts

Within my files and against the knowledge and the ledger; the merge pass joins them with the other parts.

- exp/hands.md:36-41 (part 2): docs/experiments.md player-20-live row: the roving Rovers 'killed about 420' by 4:36; this passage says a roving Rover 'killed nothing (player-20, player-21)'
- exp/opponent-barb.md:6-8: docs/knowledge/opponents-barb.md K-barb-raid-timing: the squad quota is 9 on easy/medium and 6 on hard, so 'about four' is medium's figure; the brief now faces hard_aggressive
- exp/opponent-barb.md:15-16: docs/knowledge/opponents-barb.md summary: on hard the commander roams to 2000 elmos until minute 7, so 'builds at home' is medium's rule
- exp/opponent-barb.md:17-17: docs/knowledge/opponents-barb.md K-barb-tempo-is-half-clock-half-sighting (its tempo is half sighting) against 'whatever we do'
- exp/opponent-barb.md:17-17: docs/knowledge/opponents-barb.md K-barb-tier2: the second factory comes when spare income covers it or at the switch time, so our raids on its income can move the clock
- exp/other-maps.md:85-88: docs/knowledge/maps/great_divide_v1.md:29 gives the E5 pass as 644 wide; this says 700 to 840
- exp/other-maps.md:92-94: docs/experiments.md bluegecko-2v1-great-divide row: 'The person did not fortify the pass' (two light turrets at 7:05; his commander came through at 4:26) against the camp this passage expects
- exp/other-maps.md:95-96: docs/experiments.md rows 113-118: the 2026-09-20 team games were 2v2s at 20- and 30-minute caps and commander-team-1 stood at 10 extractors at its 35-minute cap, not five
- exp/other-maps.md:97-98: docs/knowledge/maps/great_divide_v1.md K-map-great-divide-v1-the-choke-is-the-plan places the nest at F3; this and cases.md:328 say E3
- exp/players-words.md:1-6: docs/experiments.md human-10 and Center Command rows give thebluegecko 34 OS; this says about 48 (bluegecko-3v1-comet-catcher-10 says 48)
- exp/players-words.md:13-15 (part 3): docs/briefs/experience/units.md:62-63 gives vehicles at 8 to 14 energy per metal; this gives about twenty per metal spent (a rule of thumb for a wind map, a different quantity)
- exp/players-words.md:17-22: the attribution 'the opponent, human-10' for 'not a unit choice problem': docs/experiments.md human-10 row names thebluegecko at 22:38
- exp/team-games.md:25-25: docs/experiments.md rows 420-423: 'Trade corrected 2026-09-27 evening' (seen figures summed the seats and counted deaths twice), so the trades quoted in lines 15-22 overstate our side
- exp/team-games.md:80-82: docs/briefs/experience/team-games.md:74 heads this 'The opening with several seats'; human-11 was a 1v1 (docs/experiments.md row 617)
- exp/team-games.md:88-95: docs/briefs/experience/team-games.md:88 heads this 'Writing one packet for several seats'; human-12 was a 1v1 (docs/experiments.md row 631)
- exp/units.md:57-59: docs/knowledge/formations.md:16 gives the pros' spacing median as 64 elmos; this passage says 67 to 70
- exp/units.md:141-155: docs/briefs/experience/units.md:141 says nothing below was measured in a game of ours; lines 146-148 report the first Legion game (leg-player-1) and docs/experiments.md has its row (WON 25:54)
- ref/comet-catcher-remake-1.8-arm.md:148-154: docs/briefs/experience/winning.md and cases.md: the tier-2 sprint as the way to win, where the pool's duels show tier 2 in 1 of 41 Armada and 3 of 39 Cortex sides (the games end before)
- ref/comet-catcher-remake-1.8-barb-hard_aggressive.md:1-7: docs/briefs/experience/opponent-barb.md:17 'techs on a clock' is the same clock measured (9:57 / 10:44 / 11:45 in 83 of 110 games)
- ref/comet-catcher-remake-1.8-cor.md:151-157: docs/briefs/experience/winning.md and cases.md: the tier-2 sprint as the way to win, where the pool's duels show tier 2 in 1 of 41 Armada and 3 of 39 Cortex sides (the games end before)
- ref/frontier-great-divide-v1.md:16-16: docs/knowledge/opponents-barb.md K-barb-anti-nuke-by-minute is measured on Comet Catcher; this file applies its 21-25 window to Great Divide V1 without a measurement there
- ref/frontier-quicksilver-remake-1-24.md:16-16: docs/knowledge/opponents-barb.md K-barb-anti-nuke-by-minute is measured on Comet Catcher; this file applies its 21-25 window to Quicksilver Remake 1.24 without a measurement there

The five that matter most:

1. `other-maps.md:85-94` against `docs/knowledge/maps/great_divide_v1.md` and the ledger: the pass is 644 wide there and 700-840 here, and the "camp at the pass" the brief expects is a conjecture the one game with a person (bluegecko-2v1-great-divide) did not show: he did not fortify it and came through with his commander at 4:26.
2. `units.md:141-148`: "nothing below has been measured in a game of ours" in the same paragraph as the first Legion game's energy stall (leg-player-1, a game of ours, won 25:54).
3. `hands.md:36-41` against the ledger: "a roving Rover ... killed nothing (player-20, player-21)" where player-20's row has the lane-rove Rovers killing about 420 by 4:36; and the player-9/10 scouting pair cannot be placed in a series.
4. `opponent-barb.md:6-8, 15-17` against the BARb source reading: the squad of "about four" and "builds at home" are medium's rules (hard's raid quota is 6 and its commander roams to 2,000 until minute 7); "techs on a clock, whatever we do" against K-barb-tempo-is-half-clock-half-sighting and K-barb-tier2's income trigger.
5. `other-maps.md:95-96` against the ledger: the old bot's Great Divide games were 2v2s at 20/30/35-minute caps with 10 extractors at the last cap, not "three games on five extractors for 35 minutes"; and the "games with people since have not been read" is stale (bluegecko-2v1-great-divide has a row).

Lesser: the pros' spacing 67-70 here against formations.md's median 64; the Comet nest at E3 here and F3 in the knowledge entry; thebluegecko "about OS 48" against 34 OS in the human-10 and Center Command rows; "twenty energy for every metal" (a wind-map rule of thumb) beside "8 to 14 energy per metal" (a vehicle's cost); the 3v1 trades quoted before the ledger's double-counting correction; two "several seats" lessons (human-11, human-12) from 1v1s.

## Era-bound passages

- Pre-hands-rebuild (before 2026-10-02), with people: `team-games.md` lines 9-49 and 51-56 (the 3v1 series, 2026-09-27/28), `other-maps.md` 21-31 (Quicksilver human-1..8, 2026-09-22), 42-46 (Cape Violet), 59-68 (SailAway 11-12), `hands.md` 21-27 (human-7, realtime-2). Each lesson there rides on a defect fixed since (the standing tool per seat, the retreat that walked builders into the party, the seat roster union, lists without a step, no raid rule, no march, the under-water spots).
- Pre-hands-rebuild against BARb: `other-maps.md` 35-37 (pianist-player-3/6, BARb medium, 2026-09-21), 95-96 (the heuristic bot, 2026-09-20), 97-98 (onepass-player-8/9, hard_aggressive 2026-09-27); `hands.md` 36-41 (player-20/21, 2026-09-29); `players-words.md` 63-68 (comet-1, escalate-4, 2026-09-22; player-50, 2026-10-03).
- BARb-profile-bound: `opponent-barb.md` whole (medium's constants in places); the BARb reference table (110 games to 2026-10-03, hard_aggressive, "across many versions of our side"); `ref/frontier*.md:16` (the anti-nuke window measured on Comet, applied to Great Divide and Quicksilver).
- People, current hands: `team-games.md` 59-70 and `units.md` 67-70 (Center Command 2v4, 2026-10-19), 74-95 (human-10/11/12, 2026-10-03), `other-maps.md` 72-81 (human-17, 2026-10-03), `players-words.md` 8-11, 39-41 (2026-10-03).
- Pool-bound: the reference tables (Comet Catcher only; regenerated 2026-10-17 from the survey of 2026-09-23 on), `units.md` 131-135, 166-168, `other-maps.md` 13-20 (one replay).
- Undated facts: the unit tables, the sea's rules, the map headings, the frontier arithmetic.

## What I could not classify

- `hands.md:28-29`, the scout's sentence "that held under the enemy commander's eyes": which game is not said; lens unknown.
- `hands.md:40-41`, "player-9" and "player-10" (three scouts, none arrived; three roving Rovers at his lab at 3:45): the ledger has pianist-player-9/10, player-10-routes and onepass-player-9 and none carries these lines.
- `opponent-barb.md:15-16`, "in one easy game it walked into our half with a party": the game is not named.
- `other-maps.md:47-48`, human-16's Incisors firing in 2% of their in-reach seconds: no ledger row; whether the cause is the weapon or the hands is not said.
- `players-words.md:63-68`, the user's "home guard" (escalate-4) and "light encampments" (player-50) readings: the ledger rows for those games do not carry them, so the lens is the user's word with no row to check against.
- `other-maps.md:93`, "an experienced player's word: an all-air side loses": no source found.

