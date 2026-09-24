# The replay survey

Written 2026-09-23 (the user: mine the public replays of high-level players "to flesh out our game space knowledge";
Fable builds the pipeline and the card, Opus agents read the cards; "Quicksilver and comet catcher are just the
bootstrap maps for now. I intend to have us rapidly scale up the variety of maps we use over time, and part of this
survey should be preparing us for that").

## The spike (2026-09-23, verified)

- **Source.** BAR's own replay service: `https://api.bar-rts.com/replays?page=N&limit=N&preset=duel&hasBots=false&maps=<script name>`
  lists matches (id, start time, duration, map script and file name, ally teams with player names and the winner;
  `totalResults` is -1, page until empty). `https://api.bar-rts.com/replays/<id>` gives the detail: `fileName`,
  `gameVersion` ("Beyond All Reason test-31383-33cc827"), `engineVersion`, `preset`, `gameEndedNormally`, `hasBots`,
  `gameSettings` (`ranked_game`, `tweakunits`, `tweakdefs`, `startmetal`...), `Map` (id, script name, file name, size)
  and per player `name`, `faction`, `skill` ("[35.66]", the OS), `skillUncertainty`, `rank`, `startPos`, `teamId`.
  The demo downloads from `https://storage.uk.cloud.ovh.net/v1/AUTH_10286efc0d334efd917d476d7183232e/BAR/demos/<fileName>`
  (URL-encoded; 522 KB for a 17-minute duel). Gex (gex.honu.pw) browses the same replays with OS, per-map pages,
  match pools and unit statistics, rendered in the browser with no documented API: a place to look, not to fetch from.
- **Local pieces.** `run/replay_match.py <demo>` plays a demo through the headless engine with a Lua widget writing
  `record-<team>.jsonl` and `truth-<team>.jsonl` per team into `run/matches/<time>-replay-<label>/` (about real time:
  329 s for a 6:52 game); the demo's game version must be in `run/data` (`pr-downloader --download-game`, then
  `byar:test` again: docs/harness/pitfalls.md); the map must be in `run/data/maps`. `run/replay_plan.py` transcribes a
  builder's opening from a record; the arena readers (`run/floor.py`, `run/army_use.py`, `run/batch_trade.py`,
  `run/contact_use.py`, the viewer) read the records as arena matches.

## Decisions

1. **Select by manifest, not by hand.** `run/replays/pick.py` pages the API for a map (or every map), keeps duels
   without bots that ended normally, unmodified (`tweakunits`, `tweakdefs` empty; `startmetal` 1000), 6-40 minutes,
   both players' OS at or above a floor (25 to start), and writes `run/data/replays/manifest.jsonl`: one line per
   match with the detail fields that matter (id, file, map script and file name, game and engine version, duration,
   winner, players with OS, rank, faction and start position). Re-running adds new matches and keeps the old lines.
2. **Fetch by version.** `run/replays/fetch.py` downloads the manifest's demos into `run/data/replays/<map file
   name>/<fileName>` and the maps not yet in the pool (`pr-downloader --download-map`), then the game versions the
   demos need, one batch per version, `byar:test` refetched after; a version the engine cannot play (Gex marks
   "broken game version") is recorded in the manifest and skipped.
3. **Replay in parallel.** `run/replays/replay.py` runs `replay_match.py` for every fetched demo not yet replayed,
   N engines at once on distinct ports from a base, never while an arena batch runs on the same ports; the match
   directory's name is `<time>-replay-<match id>`; the manifest line gets the directory.
4. **The card is the unit of knowledge.** `run/replays/card.py <match dir>` writes `card.json` and `card.md` beside
   the records: the map and its facts (size, spot count, the start positions as grid cells), the players (OS,
   faction, side), the result and how it ended; per player the build order to 8:00 by the clock with each
   extractor's spot number and grid, the factory and tier timings, the constructor count by minute, the extractor,
   income, energy and army-metal curves by minute, the first contact, the first raid on an extractor and what did
   it, the army composition by minute, the losses and kills by type, and where the fights were. No prose beyond a
   few hundred words; numbers Opus can quote and a script can check. Spot numbers are the engine's centroid spots
   (H-ECO-SPOT-CENTROID), so cards from different games of one map speak of the same spots.
5. **Synthesis is a question list, then claims.** `docs/knowledge/questions.md` holds what we want from the
   replays (per map: the opening by the clock and where it expands; when tier 2 comes; how raids are met; where the
   fights happen; how a game is closed; and across maps: what a start position's spot geometry decides). An Opus
   agent reads a batch of cards for one map and matchup against that list and files claims in `docs/knowledge/`
   (`K-open-<map>-...`, `K-map-<map>-...`) with the match ids as evidence and counts ("plant before 1:00 in 18 of 20"),
   status observed; `docs/knowledge/replays.md` indexes the replays ingested and the claims resting on them.
   Agents read cards only: never records, demos or the engine.
6. **Claims with numbers are checked by script** (`run/replays/check.py`) against the cards before a claim moves
   from observed to supported.
7. **Maps are first-class.** Every map we ingest gets a `docs/knowledge/maps/<map file name>.md` from its cards' map
   facts and the claims about it, so a new map enters the bot's brief from the survey, not from play alone; the
   arena's map list grows from that directory.

## Order

The spike, then the manifest and fetcher (decisions 1-2), the card proven on the five replays already ingested and
the spike's (4), a twenty-game Comet Catcher pilot through the chain with one synthesis agent whose claims we judge
against K-open-comet-pro-order (3, 5, 6), then Quicksilver, then the maps the replays say people play most.

## Status

The spike done 2026-09-23 14:00: the API read, one 17-minute Comet Catcher duel (OS 28 against 36, test-31383,
already in the pool) downloaded (522 KB) and replayed in 608 s of wall clock, 1.7 times faster than the game (eight
engines would do about 27 such games an hour); both players' records read (`run/matches/1790185785-replay-spike-comet-duel`):
the west player (OS 28, lost) 16 extractors and income 36 at 8:00 with the plant at 0:35 and a second constructor
early, the east player (OS 36, won) 12 extractors and income 30 with the plant at 0:34 and eleven Rovers before 2:30,
a Rover raid opening the card must make visible. A replay record's header has no `seats` (the widget writes it, not
the shim): the card takes the players, OS and factions from the API detail, kept in the manifest.

**2026-09-23 14:45.** The card is complete on the spike duel re-replayed with the widget writing the game's own metal
spots (`mex_count`, `mex_x<i>`, `mex_z<i>`, `mex_metal<i>` game rules params) and fuller unit rows (class, build
speed, extraction): 80 spots, every extractor with its spot and grid, roles (commander, constructor N, factory) on the
build order, factories with start and finish, curves, the first extractor lost (the west player's spot_19 to a Rover at
2:08, rebuilt at 2:13, 2:23 and 2:36). **The game's spot list has 80 spots on Comet Catcher where the engine's, which
the shim reads, has 75: the cards' spot numbers are not the bot's.** The shim should read the game's list from the same
rules params (the positions people's extractors snap to, which settles the extractor placement the user asked about
on 2026-09-23 too), so the brief, the picture and the cards name the same spots; until then a claim that names spots
for the bot maps them by position (nearest within 130). `run/replays/pick.py`, `fetch.py`, `card.py` and `replay.py`
stand; the 29 kept Comet Catcher duels are on disk with their versions in the pool, and the parallel replay of the
other 28 was started at 14:45.

**2026-09-23 15:00, the first batch.** 17 of the 29 Comet Catcher duels replayed and carded in 25 minutes on four
engines (320-770 s each); 12 died in a second to a port collision in the runner (ports were given by index, not per
worker: fixed, and those 12 are replaying). A first pass over the 17 cards, 34 sides: the first factory at 0:51 (median)
and a vehicle plant in 25 of 34 sides (a bot lab in 9); a second factory in 23 of 34 at 8.9 min (median); tier 2 in
only 3 sides, at 16 min; 16 extractors and income 39 at 8:00 (winners 18, losers 15); five or more Rovers before 3:00
in 13 of 34 sides, a raid opening the pros' replay never showed; the higher-OS player won 14 of 17. Starts vary inside
the box (A4 and B5 west, G4, H5 and H1 east), so "from the west start" claims must say which cell.

**2026-09-23 15:45, all 29 carded.** The 12 replayed again on the fixed runner (343-684 s each); 29 of 29 kept Comet
Catcher duels have cards. `run/replays/aggregate.py` gives the first pass over the 58 sides: the first factory at 0:54
(median), a vehicle plant in 40 of 58 (a bot lab in 14, an aircraft plant in 4); a second factory in 44 of 58 at 9.2
min; tier 2 in 4 sides at 15.5 min; extractors at 4:00 median 7 (winners 8, losers 6), at 8:00 median 16 (winners 18,
losers 15, income 39), at 12:00 median 21.5 (winners 28, losers 14.5, income 54; 36 sides still playing); five or more
Rovers at the 2:00 sample in 14 of 58; **every side lost an extractor**, the first at 4:18 (median); the higher-OS
player won 22 of 29. Start cells A4 17, B5 12 in the west; H5 19, G4 8, H1 2 in the east. The winners' curve pulls
away between 8 and 12 minutes (18 to 28 extractors against 15 to 14.5), which is the window our tier-hard games
plateau in (22-25 extractors flat from 16:00 in pace-1). Next: the synthesis pilot, after the spot-numbering decision.

**2026-09-23 23:30, the synthesis pilot.** One Opus agent read the 29 cards against `docs/knowledge/questions.md` and
filed 22 claims (the tally by script over 58 sides), a map file `docs/knowledge/maps/comet_catcher_remake_1.8.md` with
a spot-by-cell table and a "what our bot should copy" list, and the index `docs/knowledge/replays.md`; the claims were
checked with the new `run/replays/check.py` (a per-side table from the cards and `--count EXPR`; decision 6) and merged
into `maps.md` (17 per-map claims), `openings.md`, `army.md` and `economy.md`. What the agent found that the first pass
had not: the game is decided between 8:00 and 12:00 (winners add 11 extractors, losers 1, over 18 games), winners spend
in that window (banked 500+ at 10:00 in 7 of 22 against 15 of 22 losers), raids are met with turrets (9 against 6 by
8:00; a turret at 0:27 before the factory in 18 of 29 winners), the later plant with the turret won 18 of 29 sides
against the 0:34 plant's 11 (confounded with OS), and the expansion direction differs by start and result (H5 winners
south first, 5 of 7 with spot_60 by 2:10). The agent also found card defects, all fixed and the cards re-cut: the
record header's `side` was wrong on 22 of 58 sides (now the faction from what was built), 28 spots had two cells (now
the spot's own), `first_enemy_seen` was 0:00 everywhere (removed), `factories` were sorted as strings, kills in the last
15 s (a resign's self-destruct) are counted apart, every building has its cell, the first tier-2 unit is on the card.
Still open: the spot-number bridge to the bot (the user's decision), the brief's Comet section citing the new claims,
Quicksilver next.

**2026-09-24, the spot-number bridge.** The 14:45 note above was wrong about the count: the shim's list has had 80
spots on Comet Catcher all along, and since the centroid snapping (H-ECO-SPOT-CENTROID, records from pace-1 on) it has
been identical to the game's list index for index (every position within 1 elmo, the same order; checked on pace-1,
wake-1 and wake-3 against the replay records' `mex_x<i>`/`mex_z<i>`; comet-1 of 2026-09-22, before the snapping, had
the same 80 at other positions). So the cards' spot numbers are the bot's already, and the brief's Comet spot names
needed no renumbering (spot_28 and spot_30 at A4, spot_45, spot_50 and spot_43 at B5, the strip's thirteen with x
under 1,400, as the brief has them). The bridge is now explicit: the shim reads the game's rules params (`mex_count`,
`mex_x<i>`, `mex_z<i>`, the same the replay widget reads) and publishes that list when it exists, the engine's
centroid-snapped list otherwise, and logs the source and any spot of ours with no engine spot within 130. Smoke games
spots-comet and spots-quick (easy, four minutes, heuristic brain): 80 and 44 spots, 14 and 8 extractors finished, no
refused site. Quicksilver's game list is checked the same way when a Quicksilver replay is carded.
