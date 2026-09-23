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

1. **Select by manifest, not by hand.** `run/replays/select.py` pages the API for a map (or every map), keeps duels
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
