# Pitfalls that cost time once already

- **Never overwrite a loaded `.so` in place.** `cp` onto `libSkirmishAI.so` while a game had it mapped crashed the user's GUI
  game 24 s later (SIGSEGV, PC=0 on the main thread; coredumpctl). Installs replace by rename (`run/install_ai.sh`), and the
  BAR install holds its own copy, never a symlink to the development build.
- **`pkill -f <pattern>` / `pgrep -f` match the invoking shell's own command line** and kill it. Use PIDs or `pkill -x`.
- **The engine's process name is `recoil-main`,** not `spring-headless`: `pgrep -x spring-headless` finds nothing (it made a
  memory check report "0 engines" while 24 were running). Use `pgrep -x recoil-main`.
- **Do not end the engine with a signal.** SIGTERM/SIGINT leave a 0-byte replay and a truncated `infolog.txt`; end matches with
  `/kill` on the autohost channel. (Retired 2026-09-19: the earlier claim that the stdout log is block-buffered. It is
  line-flushed; the first smoke test looked cut off at frame 31 because the engine prints almost nothing during play.)
- **Factory build orders: SHIFT means "build five".** Enforced in the shim; see K-rules-factory-shift-means-five.
- **The arena rebuilds the workspace at start.** Do not leave `crates/` half-edited while a batch is starting.
- **Prompts, briefs and the hands' rules are read from the checkout at run time** (`crates/bot/src/texts.rs`), so an
  edit to one reaches a running game: the hands within a second, the player at its next turn as a fresh session. Edit
  them deliberately while a game runs; a half-written brief is what the next session reads.
- **Requested game speed changes outcomes** (see arena.md, OPEN). Evaluate at `--speed 50`.
- **12 matches is noise-level.** ±14 points at 50%. Confirm with 24+ before believing a gain.
- After a bot restart the brain takes the commander's current position as home; brain state is not persisted.
- **Unix socket paths max out at ~108 bytes.** A long batch label once pushed `bot.sock` past it: the bot died at start-up and
  BARb beat an idle team 11 times (batch strategist-refactor-regression, 0-1-11 — not a brain result). Arena sockets now live in
  `$XDG_RUNTIME_DIR`, and the arena aborts a match whose bot process exits.
- **Look for gross mistakes before measuring small ones** (user, 2026-09-19). BARb easy is trivial for a human; while we lose to
  it, the bot is doing something absurd, and a 24-match batch cannot resolve rule-sized effects anyway (the same binary scored
  13-6-5 and 10-9-5; a behaviourally identical build 6-10-8). The per-minute `rules:` line is the detector: a rule firing
  hundreds of times per match means its orders are not executing. `DROPPED order` lines in `bot.log` and
  `WITHIN_REASON_TRACE_BUILDS` placements in `engine.log` say which and where.

## Fetching one game version for a replay strands the arena until `byar:test` is fetched too

`pr-downloader --download-game "Beyond All Reason test-NNNNN-xxxxxxx"` (what `run/replay_match.py` needs for a demo
from another build) also refreshes `run/data/rapid/.../versions.gz`, so the `byar:test` tag now names whatever build
is current upstream. The arena resolves the tag from that file and the engine dies in two seconds with `Dependent
archive "beyond all reason test-NNNNN" not found` (escalate-1-easy, 2026-09-22: the aborted match, no game). After
fetching a version for a replay, fetch `byar:test` too (docs/harness/engine.md, the same command with the tag) before
the next batch; the pool dedups, so it is a few hundred files.

## A replay that names the game by rapid tag will not open in the lobby

The engine accepts `GameType=byar:test` and resolves the tag itself, but the name goes into the replay's header as
written. The BAR lobby reads the header, cannot resolve a tag, and sits at "byar:test: 0%". The arena now resolves the
tag to the full name (`resolve_game`, from `run/data/rapid/*/byar/versions.gz`) before writing the start script.
Replays recorded before that: `run/fix_replay.py <match dir>` writes a `.fixed.sdfz` copy with the header corrected
(checked: the copy loads and plays in the headless engine; the game stream is untouched).


## Engine calls that re-enter the AI

`COMMAND_CHEATS_GIVE_ME_NEW_UNIT` creates the unit before it returns, and the engine delivers the unit's events to
`handleEvent` from inside that call. The shim's exports hold the instance table's mutex, so issuing it from inside an
export hung the engine at the first spawn (watchdog stack trace through `Mutex::lock_contended`). The shim now queues
spawns and issues them after unlocking (`engine::Spawner`). Ordinary unit orders travel through the network layer and
do not re-enter. Any future synchronous engine call (other cheats, Lua calls) needs the same treatment.

## Mass self-destruct is cancelled twice per team

See `duels.md`, "Clearing": BAR cancels self-destruct orders covering 95% of a team's units, two times per team per game.

## Start boxes are the lobby's, per map

Until 2026-09-20 the arena's default boxes were 30 % corner squares of its own, which is not any map's layout in the
lobby: Comet Catcher is played W against E in full-height 20 % strips, Quicksilver N against S in full-width strips.
Results on the corners are balanced (BARb had the same handicap) but not the players' game: the corners leave out most
of a strip's extractor clusters. `--boxes standard` (the default) reads the lobby's saved boxes from
`crates/arena/startboxes.dat`; a map missing there fails at argument parsing, so copy its line from BYAR-Chobby's
`savedBoxes.dat` (or `mapDetails.lua`'s `StartboxesSet`, zlib and base64) rather than falling back to `corners`.

- The bot protocol changed on 2026-09-22 (`UnitDamaged` gained `from` and `weapon`; later that day `Snapshot` gained `wind`; `Command::MoveState`; late
  that night `UnitDefInfo` gained `footprint` and the blasts, `OwnUnit` `facing`, `BuildSite` `keep_out`, and `Command::ReclaimUnit`, and on 2026-09-23 `TeamInfo.controller` (who plays each seat)): the arena builds both
  sides, but GUI play needs `run/install_to_bar.sh` re-run so the installed shim matches the bot.

- The engine's watchdog (`HangTimeout`, 60 s by default, 600 at most) kills a game whose main thread stalls that long,
  and a lockstep turn is such a stall: a hung `claude -p` session took pianist-player-10 with it. The arena writes
  `HangTimeout = 600` into the match's springsettings.cfg, and the driver abandons a turn after 45 s and replaces the
  session (`strategist/mod.rs` `TURN_CAP`).

- A game against people is realtime: `run/human_game.sh` (`WITHIN_REASON_REALTIME=1` beside `--pianist --player`), or
  nothing waits for it and nothing pauses. In lockstep runs (the arena's default) a wall-clock wait costs game time at
  the arena's speed, which is why the pianist's Jev call stays in place there.

- **The hands' token diet is a level, not a fact of the build.** `WITHIN_REASON_HANDS_EFFORT` (lean | normal | full) sets
  how much of the picture a Jev call carries; the arena passes `lean` unless `--hands-effort` says otherwise, `run/human_game.sh`
  passes `normal` unless its third argument says otherwise, and the record's and the jev log's headers carry the level. Read a
  game's Jev cost against its level; compare games at the same level.

- **The arena rebuilds everything but itself.** `run/install_ai.sh` runs `cargo build --release` at batch start, so the shim and
  the bot of a batch are the tree's, but the arena process that launched the batch is the binary that existed before the build:
  a new arena flag or default (diet-1: `--hands-effort`, default lean) is not in effect until the next arena start after a build.
  After changing `crates/arena`, run `cargo build --release` (or one batch) before the batch that needs it.

## Every player game before 2026-09-23 15:00 ran without the think penalty, and the penalty covered only the commander mode

The arena holds the game during a turn. `--think-penalty X` makes a turn's orders land X game seconds late per wall
second of thinking, the latency of a live game, but it defaulted to 0 and no series script passed it, so every
pianist and Lua player game in the ledger up to and including pace-1 was played with the world frozen while the player
thought: the player never met an event that arrived while it was thinking (in the real-time human games 55 % of the
critical events did, and the next turn came a median 6 s after them). And as coded the penalty held back only the
commander mode's directives, field orders and wake; the player's packet, lists, production, marks, policy changes,
footwork and removals landed at once, so passing it would have changed nothing. Since 2026-09-23 15:00 the penalty
holds back everything a turn changes (`shared.rs` `TurnOutputs`, `hold_for_turn`, `apply_delayed`) and `--player`
defaults it to 1. Read the earlier player rows with that in mind; the penalty series (penalty-1..4) is the first
representative set. `--think-penalty 0` turns it off deliberately.

## The player stopped at 0:13 on a usage warning (2026-09-23 evening)

The strategist's session guard ended the player's session on the first `rate_limit_event` whose status was not
`allowed`. The CLI also sends `allowed_warning` when a window passes a threshold (the 7-day window at 75 %), with
`isUsingOverage: false`: upgrade-1-hard-aggressive lost its player after turn 1 and the bot's heuristics played the
rest (metal full 70 % of the game, one plant). The guard now stops only on overage or a status other than `allowed`
and `allowed_warning`. A game whose `bot.log` says "Player session ended; heuristics carry on alone" in the first
minute is void: check the `stopped` line in `strategist-0.jsonl` for the reason before reading the game.

