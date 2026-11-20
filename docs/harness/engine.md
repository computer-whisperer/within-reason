# Engine, game and the native AI interface

## Sources (shallow clones in `upstream/`, git-ignored)
- `RecoilEngine` — engine (Spring fork). AI interface in `rts/ExternalAI/Interface/`. Submodules not fetched.
- `Beyond-All-Reason` — the game: Lua content, 5.2G. Lua AIs listed in `luaai.lua`.
- `CircuitAI` @ `barbarian` — BARb, the C++ AI BAR ships (`master` is an empty stub).
- `BYAR-Chobby`, `teiserver`, `SPADS`, `bar-lobby` — lobby client, lobby server, autohost, new lobby (not read).

## Engine releases (github beyond-all-reason/RecoilEngine)
A 33 MB Linux archive ships `spring`, `spring-headless`, `spring-dedicated`, `pr-downloader`, `libunitsync.so`,
`AI/Interfaces/C/0.1/libAIInterface.so`, `AI/Skirmish/{BARb,CircuitAI,NullAI}`.
- `run/engine` -> `run/engines/2026.09.03`: the release the lobby runs, since 2026-11-16 (the user's launcher fetched
  it that evening and his three lobby games of that night were played on it, the flagship in each). The arena's since
  2026-11-19; every row of `docs/experiments.md` before that day's engine row is 2026.07.04's. Use this.
- Which release the lobby runs is in the launcher's configuration,
  `https://launcher-config.beyondallreason.dev/config.json`: each setup's `launch.engine`. "Alpha" is the ordinary
  channel; "Engine Test" runs a pre-release the developers are trying (2026.11.02 on 2026-11-19). GitHub's "latest"
  release is the ordinary channel's; a release marked pre-release there is not the lobby's (2026.11.04 that day; the
  tag 2026.11.03 was never published as a release).
- `run/engines/2026.07.04`: the lobby's release until 2026-11-16. It stays: a demo plays only on the engine that
  recorded it, and `run/replay_match.py` (so `run/tape.py`) picks the engine a demo's header names from `run/engines`.
- `run/engines/2026.09.01`, a pre-release: DO NOT USE for batches. SIGSEGV in a pathfinder worker thread in 3 of 18
  headless matches, `QTPFS::IPath::SetPoint` (Path.h:239) via `PathSearch::Finalize` / `LoadPartialPath` <-
  `PathManager::ExecuteSearch` (symbolized with the release's `-dbgsym` archive); 0 crashes in 86 matches on
  2026.07.04 (as of 2026-09-19). Not reported upstream. The 2026.09 changelog lists the mend ("intermittent memory
  errors/crashes during pathing caused by a race between background path-finding threads and newly created path
  requests"), and 2026.09.03 played 96 games over twelve maps, 2,200 game minutes, with no crash (2026-11-19).
- `run/engines/2026.11.04`, a pre-release, fetched 2026-11-19 to see what the next release breaks: nothing in the
  benches below. Not for rows.
- The AI interface headers (`crates/recoil-ai-sys/vendor`) are byte-identical at 2026.07.04, 2026.09.01, 2026.09.03
  and 2026.11.04, so one build of the libraries serves them all; the vendor directory's README has the check.

### Moving to a new release
- `run/engine_fetch.sh <release>` unpacks the release beside the others and copies in the opponents we installed
  under the present engine's `AI/Skirmish` (every Apex build as it is; Barb3's game side with the new release's BARb
  library). `--use` also points `run/engine` at it. A worktree has its own `run/engines` and link.
- `WITHIN_REASON_ENGINE=<release>` before `arena` or `duel` plays a batch on that release without moving the link;
  `batch.json` records the release (`engine`) either way.
- The 2026-11-19 check of 2026.09.03 (and of 2026.11.04 beside it), one binary on each engine; its row in
  `docs/experiments.md` has the numbers:
  1. the vendored headers against the tag (above), and the sim files our source-read claims cite
     (`git -C upstream/RecoilEngine diff --stat <old> <new> -- rts/Sim rts/ExternalAI`);
  2. a short game for each AI we seat and each opponent (WRDistilled and WRTerritory against BARb, WRDistilled
     against Apex and against eight Barb3), reading `engine.log` for script errors, with the Apex record put back
     afterwards (`run/data/opponent-state/apex-v<version>-records/`);
  3. the flagship's shim by the duel runner: the ground army's property drills (`run/props_bench.sh`) and the air
     raids (the scenes in `run/data/air-intercept-kept/bench`), each beside the old engine;
  4. the twelve-map bench (`run/distilled_maps.sh barb`, `PORT` and `WITHIN_REASON_ENGINE` set) on each engine, for
     crashes and for the bot's and BARb's curves (`run/bot_maps.py`);
  5. a demo of the new engine through `run/tape.py`, `run/replay_match.py --audit` and `run/audit.py`, and the demo's
     header and team statistics through `run/replays/startscript.py`;
  6. a game and a drill on the lobby's present game version (below), the arena's own put back afterwards.
  No flagship game was played for it: the player and the hands never touch the engine but through the shim, and the
  lobby games of 2026-11-16 had already played the flagship on the release.
- What 2026.09 changes in the simulation that we lean on (its changelog; the command handling read in the source,
  `CommandAI.cpp`, `AirCAI.cpp`, `MobileCAI.cpp`, the rest taken from the changelog's words): the engine ships
  BARb 1.6.29, which plays water maps where 1.6.28 did not (K-barb-1-6-29-plays-water-maps); an attack order on a
  crashing aircraft now ends at once for every unit, not for aircraft alone (BAR's `fireAtCrashing = false`); a unit
  works through a run of orders that end as they start in one update (`CCommandAI::FinishCommand`, up to 64); a
  bomber with a long burst no longer flies a second run at the same target; a weapon whose muzzle is under the ground
  no longer believes its line of fire free; a map's terrain types above 1 now speed units up (the drag used the
  unit's base speed); the first blocked-site test of a building with an odd footprint is aligned as the final one is.
  None of these moved a bench figure beyond its scatter.

## Game data
**The game version the arena and the duel runner play is the tracked file `run/game_version`**: one line, the game's
full name. Currently `Beyond All Reason test-32018-ad135b9` (BAR's release 2026.11.19-2, the lobby's version that
day; the user, 2026-11-19: "agreed, move the arena to test-32018"). Before it: test-31836-b250a16 from 2026-11-06
(release 2026.11.04-2, on the user's word), test-31691 before that. It moves by a commit of that file on the user's
word and by nothing else. `WITHIN_REASON_GAME="<full name>"` plays a batch on another version in the pool without
moving it, and `batch.json` records `game`. Rows on different game versions are not comparable on balance, so after
a move the benches are played again as the new baseline (`docs/experiments.md`, the rows of 2026-11-19 and of
2026-11-06).

Until 2026-11-19 the arena named the rapid tag `byar:test` and resolved it at every match's start from the index
pr-downloader keeps (`run/data/rapid/.../byar/versions.gz`), which every game fetch rewrites to the newest build:
a replay's version fetched for a tape moved the arena with no word of it (2026-11-06, a bench's rows) or was put back
by hand (every lobby replay from 2026-11-10 on; docs/harness/pitfalls.md has the history). A game named in full needs no index: the engine names a package by
the `modinfo.lua` inside it and reads `versions.gz` for tags alone (`rts/System/FileSystem/RapidHandler.cpp`).

Fetching, with env `PRD_RAPID_USE_STREAMER=false PRD_RAPID_REPO_MASTER=https://repos-cdn.beyondallreason.dev/repos.gz
PRD_HTTP_SEARCH_URL=https://files-cdn.beyondallreason.dev/find`:
`run/engine/pr-downloader --filesystem-writepath run/data --download-game "$(cat run/game_version)"`, and
`--download-map "<name>"`. The lobby's version of the hour is what `byar:test` names upstream (`--download-game
byar:test` fetches it and prints its name; the launcher's setups download that tag, docs above). A build's source
is the commit in its name, which GitHub serves though it is on no branch we track (BAR cuts each release as its own
commit): `git -C upstream/Beyond-All-Reason fetch --depth 1 origin <the full hash of the build's `byar:git:` line in
versions.gz>`.

In the pool beside the arena's version, for replays of games recorded on them (a demo names its game in full):
the arena's earlier versions; test-31908 (the lobby's of 2026-11-10, the Erebos Lakes replay); test-31958-7ffdbab
(2026-11-15, that day's seven lobby games); test-31986-603f15d (2026-11-16, that night's three lobby games).

**The unit tables compiled into the binaries are not read from the arena's game.** `crates/bot/data/units.json`
(the glossary and, through `run/fact_units.py`, the brief's unit tables; written by `run/unit_stats.py`) and
`crates/combatsim/data/units.json` (the combat table of the micro engine, the combat estimates, the nuke code and
the bots; written by `crates/combatsim/tools/extract_units.py`) were read from the game's source at 54199a0d
(2026-09-19) and not again when the arena moved. Measured 2026-11-19 against the two builds' own commits, by running
both tools on each: against test-32018, 57 of the glossary's 574 entries differ in a number (Armada 7, Cortex 14,
Legion 36) and 81 of the combat table's 568 (18, 23, 40); 27 and 47 of them differed against test-31836 already,
which the arena played from 2026-11-06. No unit was added or removed. Among Armada and Cortex the differences are
small: the tier-1 turrets' sight (455 to 494 in the tables, 500 in the game), the Reaper (a shot 109 -> 120, reload
0.7 -> 0.8 s, sight 462 -> 410), the advanced fusions' health, the rocket vehicles' prices and missiles, flight
figures of some tier-3 and anti-nuke missiles, eight units' collision volumes (radius and height), the floating
radars' health (110 and 114 -> 180), a dummy first weapon the tier-1 anti-air bots no longer carry. Legion has the
larger ones (`legaheattank` health 7,700 -> 5,700 and metal 1,250 -> 1,050, the constructors' sight, the advanced
navy's armour class). Regenerating is
`run/unit_stats.py --checkout <the build's source>` and `crates/combatsim/tools/extract_units.py <the same>`, then
`run/fact_units.py` and `run/glossary_sheet.py`. Not done (2026-11-19): the texts written beside the old numbers
want reading with it, and it is the user's to order.

Maps: Quicksilver Remake 1.24; Isidis crack 1.1, Feast of Hades 1.0.1, Comet Catcher Remake 1.8 and Mithril Mountain
v2.0.1 (fetched 2026-09-20 as the most played on gex.honu.pw); the distilled bot's map bench (fetched
2026-11-06: Onyx Cauldron 2.2.2, Ravaged Remake v1.2, Avalanche 3.4, Zed Remake 3.3.1, Red Comet Remake 1.8, The
Halite Maze v1.4, Hooked 1.1.1, Flooded_Valley_V5, Argent Strata 1.1, Sertaleina_v9.2, TitanDuel_V3;
`run/distilled_maps.sh`); Great Divide V1 (the team-game test ground, fetched 2026-09-20: four starts, two north and
two south; `--boxes north-south`); the lobby games' maps (Erebos Lakes v1.0, Copper Hill v1.0.1, Ceasefire 1.1,
Faster Than Light 1.1, Canis River v1.4, Tangerine_Remake 1.0, Eye Of Horus 1.7.1). Pass the exact name to the
arena's `--map`. The arena's start boxes default to the north-west and south-east 30 % corners, which is not where
people start on a left-against-right map such as Comet Catcher (`--boxes west-east` is there for it, untried).

## Native AI interface
- Discovery: `<data dir>/AI/Skirmish/<ShortName>/<Version>/{AIInfo.lua,libSkirmishAI.so}` (AILibraryManager.cpp:183); any data
  dir counts, including the write dir.
- Exports: `init(aiId, SSkirmishAICallback*)`, `release(aiId)`, `handleEvent(aiId, topic, data)`.
- 27 event topics (AISEvents.h), ~94 command structs (AISCommands.h), ~596 callbacks (SSkirmishAICallback.h). Array
  callbacks return the size when passed a null buffer; metal spot arrays are counted in floats, `y` holds the metal value.
- One UPDATE event per sim frame (30/s). Dispatch is in-process in the game loop (`eoh->Update()`, Game.cpp:1748): a
  blocking AI stalls its host client.
- Lua bridge: COMMAND_CALL_LUA_RULES / COMMAND_CALL_LUA_UI out, EVENT_LUA_MESSAGE in (unused so far).
- AI orders travel as NETMSG_AICOMMAND from the hosting client through the server; other clients do not need the AI's files.
- The AI's stderr lands in the engine's stdout log.

## Start script (`doc/StartScriptFormat.txt`)
`[AI0] { ShortName=; Version=; Team=; Host=<player number>; [OPTIONS]{} }` — `Host` selects which player's client loads
and runs the AI. `FixedRNGSeed` exists. Lua AIs are set per team with `LuaAI=name`. `GameType` accepts a rapid tag (`byar:test`).

## Three ways to be a bot
1. Native skirmish AI (.so via the C interface) — full AI API, fog of war respected. What we do; how BARb works.
2. Lua AI gadget inside the game archive (SimpleAI, Scavengers, Raptors) — synced code, needs a game fork or mutator.
3. LuaUI widget driving a player slot — plays as a human; the only route that needs no AI slot.

## Running headless
`spring-headless --isolation --write-dir <dir> <script>`; extra read-only data via `SPRING_DATADIR` (honoured in isolation
mode, DataDirLocater.cpp:432). Load takes ~12 s. The engine log is block-buffered: complete only after a graceful exit.
Autohost channel (`AutohostIP`/`AutohostPort` in the script): server -> us UDP events (AutohostInterface.cpp: SERVER_STARTED 0,
QUIT 1, STARTPLAYING 2, GAMEOVER 3 `[id, size, player, winning ally teams...]`, MESSAGE 4, PLAYER_* 10-14, GAME_LUAMSG 20,
TEAMSTAT 60); us -> server plain-text commands (`/kill`, `/setminspeed`, `/setmaxspeed`, `/kick`, `/forcestart`, `/cheat`...).
Speed: `/setmaxspeed N` then `/setminspeed N` at STARTPLAYING forces the speed up; the `MaxSpeed` mod option alone does nothing.
