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
`pr-downloader --filesystem-writepath run/data --download-game byar:test` (and `--download-map "<name>"`) with env
`PRD_RAPID_USE_STREAMER=false PRD_RAPID_REPO_MASTER=https://repos-cdn.beyondallreason.dev/repos.gz
PRD_HTTP_SEARCH_URL=https://files-cdn.beyondallreason.dev/find`. Currently: test-31836 (2026-11-06, the lobby's version, on the user's word; test-31691 before it; the older ones stay in the pool for replays; test-31908, the lobby's version of 2026-11-10, is in the pool for the Erebos Lakes replay with `versions.gz` put back, so `byar:test` still names test-31836, and Erebos Lakes v1.0 is among the maps; test-31958-7ffdbab, the lobby's version of 2026-11-15, the same way for
the tapes of that day's seven lobby games, with Copper Hill v1.0.1, Ceasefire 1.1, Faster Than Light 1.1, Canis River
v1.4 and Tangerine_Remake 1.0 among the maps; test-31986-603f15d, the lobby's version of 2026-11-16, the same way
for the tapes of that night's three lobby games, with Eye Of Horus 1.7.1 among the maps; test-32018-ad135b9, the
lobby's version of 2026-11-19, for the engine check's smoke, the tag put back to test-31836 after it); maps Quicksilver Remake 1.24, Isidis crack 1.1, Feast of Hades 1.0.1, Comet Catcher Remake 1.8,
Mithril Mountain v2.0.1, the distilled bot's map bench (fetched 2026-11-06: Onyx Cauldron 2.2.2, Ravaged Remake v1.2, Avalanche 3.4,
Zed Remake 3.3.1, Red Comet Remake 1.8, The Halite Maze v1.4, Hooked 1.1.1, Flooded_Valley_V5, Argent Strata 1.1, Sertaleina_v9.2,
TitanDuel_V3; `run/distilled_maps.sh`), Great Divide V1 (the team-game test ground, fetched 2026-09-20: four starts, two north and two
south; `--boxes north-south`) (the middle four fetched 2026-09-20 as the most played on gex.honu.pw; pass the exact name to the
arena's `--map`). The arena's start boxes default to the north-west and south-east 30 % corners, which is not
where people start on a left-against-right map such as Comet Catcher (`--boxes west-east` is there for it, untried).

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
