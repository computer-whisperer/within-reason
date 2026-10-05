# Within Reason

*An AI opponent for Beyond All Reason that plays within reason. Lobby name: `WReason`. (Formerly `bar_bots`.)*

AI players for [Beyond All Reason](https://www.beyondallreason.info/), the open-source RTS on the Recoil engine.
The flagship is a model-driven player: a Claude session reads the game and writes standing instructions in prose,
and Jev (TypeSafe's System One model) plays every unit from them, second by second. Heuristic bots that cost nothing
to run are a younger second line, fitted to the play the flagship has shown. This is an early-stage hobby project and
is not affiliated with the BAR team.

## How it works

```
 Recoil engine (spring / spring-headless)                 bot process
 ┌──────────────────────────────────────────┐   Unix     ┌──────────────────────────────────┐
 │ libSkirmishAI.so  (crates/ai-shim)       │  socket    │ crates/bot                       │
 │  sends events + state snapshots  ────────┼──────────► │  the player: a Claude session    │
 │  applies the commands it gets back ◄─────┼─────────── │  the hands: Jev, a menu a second │
 └──────────────────────────────────────────┘            │  the control lane: 10 Hz footwork│
                                                         └──────────────────────────────────┘
```

The engine loads a thin Rust library through its C skirmish-AI interface. The library holds no strategy: ten times a
second it sends the bot process what happened and what the world looks like, and applies the orders that come back. In
the bot process three things decide, each at its own pace:

- **The player** (a Claude Code session, Opus or Sonnet) takes a turn every few seconds to tens of seconds: it reads a
  report of the game and writes a *packet*, prose instructions for every group, builder and factory, plus build lists.
- **The hands** (Jev) answer, about once a game second, typed questions over a picture of the game: should this group
  change course, which of the moves on its menu fits the instructions now. Code builds the menu and carries out the
  pick; it does not choose.
- **The control lane** (`crates/micro`) is the one place code decides alone, because it must be faster than a model
  call: per-unit footwork in a fight (step out of fire, kite, spread from a D-gun, hold formation).

A heuristic bot skips the socket and the models: the same engine-side host calls the bot inside the library
(`bots/`, [`bots/README.md`](bots/README.md)). [`DESIGN.md`](DESIGN.md) has the details of both.

| Crate | What it is |
|---|---|
| `recoil-ai-sys` | bindgen bindings over the engine's AI interface headers (vendored) |
| `bot-protocol` | the messages between engine side and bot, their framing, and the trait an in-library bot implements |
| `ai-host` | the engine's side of an AI library: snapshots out, commands in, over any transport |
| `ai-shim` | the library the engine loads for the flagship: the host over a socket to the bot process |
| `bot` | the flagship's bot process: world model, the hands, the player's session, the match recorder |
| `jev` | client for TypeSafe's Jev |
| `micro` | the control lane: per-unit footwork at 10 Hz |
| `terrain` | walking distances over the map's ground per movement class |
| `combatsim`, `buildorder` | study tools: a combat simulator and a build-order search over the game's own unit numbers |
| `arena` | batch evaluation: parallel headless matches, a referee, logs, records and replays per match; a duel director for unit-level fights |
| `bots/*` | heuristic bots that run inside their own library, with no bot process ([`bots/README.md`](bots/README.md)) |

`viewer/` replays a match record beside the player's transcript and the hands' log (`run/view_match.py`).

## Status

As of early October 2026 ([`docs/experiments.md`](docs/experiments.md) is the running ledger, one row a batch):

- **Against BARb, 1v1 on Comet Catcher:** the standing duel against the `hard_aggressive` profile, with an Opus player
  at medium effort, was won in each of the last six single games, in 12 to 20 game minutes, at $0.32 to $0.82 of Jev a
  game. These are one-game rows on one seed, not a win rate.
- **Large team games:** two 8v8 games against `hard_aggressive` on All That Glitters were lost.
- **Against people:** mostly losses. An experienced player beats it in 1v1; one 2v1 was won.
- **Heuristic bots:** only the template exists (`bots/template`, which builds extractors and nothing else).

A flagship game needs a TypeSafe API key for Jev and a Claude Code subscription for the player, so it is not something
to drop into a lobby as it stands.

## Knowledge base

Most of the work is learning the game, so the documentation is structured as a loop: observe a game, write down a
checkable claim, build on it, verify with another game, retire it when it stops holding. Claims live in
[`docs/knowledge/`](docs/knowledge/README.md) with their evidence and status, the rules in play are registered in
[`docs/heuristics.md`](docs/heuristics.md), what code decides in a flagship game is listed in
[`docs/code-decisions.md`](docs/code-decisions.md), and [`docs/README.md`](docs/README.md) describes the process. The
analysis of how BARb plays, from its source and configs, is in
[`docs/knowledge/opponents-barb.md`](docs/knowledge/opponents-barb.md).

## Running it

Requirements: Linux, a Rust toolchain, libclang (for bindgen), `7z`, and about 3 GB of disk for the engine and game data.
The flagship also needs a TypeSafe API key (`TYPESAFE_API_KEY`, or the file `~/.config/within-reason/jev.env`) and, with `--player`, the
`claude` CLI signed in.

```sh
cargo build --release

# engine (use the version BAR ships; see docs/harness/engine.md) and game data
mkdir -p run/engines && cd run
gh release download 2026.07.04 -R beyond-all-reason/RecoilEngine -p 'recoil_*_amd64-linux.7z'
7z x -oengines/2026.07.04 recoil_*_amd64-linux.7z && ln -s engines/2026.07.04 engine
export PRD_RAPID_USE_STREAMER=false PRD_RAPID_REPO_MASTER=https://repos-cdn.beyondallreason.dev/repos.gz \
       PRD_HTTP_SEARCH_URL=https://files-cdn.beyondallreason.dev/find
./engine/pr-downloader --filesystem-writepath "$PWD/data" --download-game byar:test
./engine/pr-downloader --filesystem-writepath "$PWD/data" --download-map "Comet Catcher Remake 1.8"
cd ..

# no key needed: the template heuristic bot against BARb easy, five game minutes
target/release/arena --our-ai WRTemplate:0.1 --matches 1 --speed 50 --profile easy \
    --map "Comet Catcher Remake 1.8" --max-minutes 5 --label my-first-game

# the flagship, one headless game: an Opus player over the hands (each engine needs ~4 GB of memory)
target/release/arena --pianist --player --matches 1 --speed 50 --profile medium \
    --map "Comet Catcher Remake 1.8" --label my-first-flagship-game
```

To play against the flagship in the BAR client: `run/install_to_bar.sh` copies the AI into a local BAR install, then
`run/human_game.sh` starts the bot and you add `WReason` to a skirmish. The lobby only lists unknown AIs with
"Simplified AI list" turned off (Developer settings tab; see [`docs/harness/lobby.md`](docs/harness/lobby.md)). Please
ask the people you are playing with before bringing an experimental AI into a shared lobby.

## Licence

GPL-2.0-or-later, the same as the engine whose interface headers are vendored in `crates/recoil-ai-sys/vendor/`.
See [`LICENSE`](LICENSE).
