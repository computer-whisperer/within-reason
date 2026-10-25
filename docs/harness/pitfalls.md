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
- **A game runs from its snapshot, not the checkout** (since 2026-10-21; `crates/bot/src/texts.rs`): `bot --snapshot`
  copies the bot binary, every text and the reference tables to `run/matches/<batch>/snapshot/`, and the arena and
  `run/human_game.sh` run `snapshot/bot` with `WITHIN_REASON_TEXTS` pointing there. Edits and rebuilds in the checkout
  during a game reach the next game, not this one; before 2026-10-21 an edited brief reached the hands within a
  second and the player at its next turn as a fresh session. The shim is the one part still shared: installed by
  rename, a running engine keeps the library it mapped, but a later match of a batch loads whatever is installed then.
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
export hung the engine at the first spawn (watchdog stack trace through `Mutex::lock_contended`). The shim queued
spawns and issued them after unlocking (`engine::Spawner`), and still does, though since 2026-10-24 the spawn is the
synced `/give` (next entry but one), which does not re-enter. Ordinary unit orders travel through the network layer
and do not re-enter. Any future synchronous engine call (other cheats, Lua calls) needs the same treatment.

## Hold fire keeps the target a weapon already has; units heal when left alone

`Command::FireState` 0 (hold fire) only stops a weapon choosing a new target (`CWeapon::AllowWeaponAutoTarget`); a
target it took in the frames after the unit appeared stays, and the unit keeps shooting it. Follow the hold with a
`Stop`, which drops every weapon's target (`CCommandAI::ExecuteStop`). And a hurt unit left alone heals: a scenario's
Stouts hurt at the start of a 90 s preparation were whole again when the fight began (K-engine-hold-fire-keeps-a-target,
K-engine-a-unit-left-alone-heals).

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
  that night `UnitDefInfo` gained `footprint` and the blasts, `OwnUnit` `facing`, `BuildSite` `keep_out`, and `Command::ReclaimUnit`, on 2026-09-23 `TeamInfo.controller` (who plays each seat), and on 2026-09-30 `BuildSite.keep_out` became `placements` (a facing with its keep-out, tried in order)): the arena builds both
  sides, but GUI play needs `run/install_to_bar.sh` re-run so the installed shim matches the bot.
- The shim says Hello at `init` since 2026-10-01 (the opening turn, `docs/design/2026-09-30-opening-turn.md`): a shim
  installed before that connects at the first frame, the opening turn then starts as the game does, and the hands wait
  for it with the game running. Re-run `run/install_to_bar.sh` before a game with people. The bot must be listening
  before the game loads, as before.

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
  the bot of a batch are the tree's (the bot as snapshotted at that start; a build during the batch does not reach it), but
  the arena process that launched the batch is the binary that existed before the build: a new arena flag or default (diet-1:
  `--hands-effort`, default lean) is not in effect until the next arena start after a build. After changing `crates/arena`,
  run `cargo build --release` (or one batch) before the batch that needs it.

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


## The replay manifest is rewritten whole by the fetcher and the runner

`run/replays/fetch.py` and `run/replays/replay.py` read `run/data/replays/manifest.jsonl` at their start and write it
back whole when they amend a line. A `pick.py` run that appended lines while a replay batch ran lost them at the
batch's next write (2026-09-24, thebluegecko's four duels, re-added). The runner now merges its `match`/`card` fields
into the file as it is at write time; the fetcher still writes its start-time copy, so do not run `pick.py` while
`fetch.py` runs.

## Replay records have no damage (2026-09-24)
The records written from the replay pipeline (`run/replays/replay.py`, the dump widget) carry an empty `dmg` on every
sample: the widget does not hook UnitDamaged. Damage in those records is the drop in a unit's health between
consecutive samples (`run/replays/shapes.py` does this for both sides), never the field. Shots, friendly fire and the
exchange tables (`shots`, `ff`, `xo`, `xi`, `xf`) exist only in the bot's own records.

## A stray sweep bomb can wipe the next duel (2026-09-25)
`form-stout-on-nofocus` rep 3: the second army was wiped 4.9 s into a fight with 27,240 damage taken and 0 dealt by
either side, on the site's second duel. A crawling bomb of the sweep before, outside the wreckage box (so not counted
as standing) and set off late, went up among the new units. Rare (1 of ~200 duels this day); `run/duel_ab.py` drops a
wipe under 15 s in which neither side dealt damage, and a study should say when a row was dropped.

## The lane's rules are gated by `enabled` only since 2026-09-25
Before that, `WITHIN_REASON_DISABLE=H-MICRO-FOCUS` (or -FLEE, -KITE, -FAN) switched nothing off: only H-MICRO-LANE
as a whole was checked. An ablation of one lane rule from before that date ran the whole lane; the duel director
honours the variable for the lane it runs too.

## An arena's ports run to `base_port + 2 x matches` (2026-09-25)
Each match takes two ports from `--base-port` upward by its index, not by its parallel slot: 24 matches span 48
ports. Three arenas started 30 apart (9600, 9630, 9660) collided at indices 15-23 of the second with 0-8 of the
third, and two matches aborted with "Address already in use" (lane-ab-old). Space concurrent arenas by
`2 x matches`, and keep the last index inside the range you were given.

## The flagship plays only with `--pianist` (refused without it since 2026-10-05)

From 2026-09-25, when the heuristic brain was deleted, to 2026-10-05, `bot` with neither flag (the arena's default)
built a brain with no pianist whose first `think` panicked (`expect("pianist mode")`); the session died, the shim
reconnected a second later, and the seat stood idle for the whole game (shim-refactor-smoke: 350 connections in
three minutes). A flagship row in the ledger from those days without `--pianist` or `--player` is a void game. Now
`bot` does not start without `--pianist`, and the arena stops at its arguments when the flagship is seated without
it (`--packet PATH` or `--player` gives the hands something to play; `--our-ai` seats a bot of `bots/` instead).

## Two libraries of ours in one game both write `balance` lines

Every library built on `crates/ai-host` prints the arena's `heartbeat`, `census` and `balance` lines under the same
`[wreason ai=N]` prefix, each from its own seat's side. With a bot of `bots/` as the `--opponent` of the flagship,
the referee's settled-game call reads whichever `balance` line is newest, ours or theirs: pass `--play-out` (no
settled call) for such a game until the lines carry the side.

## A user widget may write only under `LuaUI/Config/` from game test-31677 on (2026-10-19)

`run/replay_match.py` dumps a demo's records through a LuaUI widget (`run/replay-template/LuaUI/Widgets/replay_dump.lua`).
From the game build the lobby ran on 2026-10-19 (test-31677), `luaui/barwidgets.lua` sandboxes user widgets' file
writes to `luaui/config/` and `luaui/widgets/` (`userWritableFolders`): every `io.open` elsewhere fails with
`record-0.jsonl: is outside the write paths for user widgets` in infolog.txt, the engine replays the whole demo,
and the match directory ends with no records (the first Center Command replay, `1792466245-replay-center-command-2v4`).
The widget now writes under `LuaUI/Config/within-reason/` and the runner moves the files up when the engine exits.
Watch infolog.txt for that message when the game moves the folders again.

## A batch of long replays with one flat timeout recycles held ports (2026-10-22)

The first 8v8 batch (`run/replays/replay.py --parallel 8`) gave every demo an hour and left a timed-out engine
running, as `run/replay_match.py` does for a replay by hand. Eight 8v8 engines at once run at about 0.6 of game
speed, so three games over 40 minutes timed out with their engines still on their ports; the three ports went
back into the pool, every later engine on them died in two seconds (the port taken), and the three fast-failing
slots ate the remaining 23 demos of the stage in a minute while the five good slots were still replaying. The
runner's log is block-buffered when written to a file, so the stage looked healthy until it ended. Now: the
timeout is four times the game's length plus ten minutes (at least an hour), the batch passes
`--kill-on-timeout`, a port that is still held after a failure is retired from the pool and said, a failure under
a minute prints the runner's last lines, and the log is line-buffered. When a batch shows failures in seconds,
look for an engine still holding the port before anything else.

## Some archive files on the ceph mount hang in an uninterruptible read (2026-10-22)

A reader that opens `/ceph/christian/within-reason/run/matches-archive/<batch>/results.jsonl` (or a record or truth file
under it) can block in the kernel (state `D`, `folio_wait_bit_common`) and never return; a second reader of the same
file blocks behind it, `kill -9` is deferred until the read completes, and every other file of the archive reads in
milliseconds. On 2026-10-22 nine `results.jsonl`, 35 records and 31 truth files of 734 batches did this while
`run/metal_killtest.py extract` ran. Read archive files through `timeout` (the tool's `read_text`, `timeout 60 head -c N
<file>`), skip and count a file that does not answer, and never loop over the archive with plain `open()` in a process
you need back.

## A `claude -p` from the checkout without its own system prompt answers as Claude Code (2026-10-23)

`run/debrief.py` asked the player's debrief with `claude -p --tools ""` from the checkout under the default system
prompt, so the model read both CLAUDE.md files and the repository as its working directory and took itself for a coding
agent with no tools: five of nineteen entries in the rework live-edge store carried a tool call written as text ("I'll
look at how previous debriefs in this repo are written ... `<invoke name="Bash">`", or a bare `ls` in a code fence,
done in 5 to 7 s) and eleven a long document in a shape of its own, not the DECIDED / NEXT PLAYER / WISHED shape
asked for; only three were in shape. The text passed as non-empty output into the inbox and the entry, and the next
player of the series read it. Every model call that is not a coding session takes the launch docs/harness/claude-p.md
verifies: `--setting-sources ""`, `--system-prompt` (or `--system-prompt-file`), `--tools ""`, `--strict-mcp-config`,
and an empty working directory; and its answer is checked for a coding agent's leavings before anything downstream
reads it. The shape asked for is another matter: under the player's prompt every answer was a debrief and none took
the DECIDED / NEXT PLAYER / WISHED shape (ten of ten, a retry asking for it included), so the tool reports the shape
in its header line and keeps the debrief.

## The checkout is a worktree of a bare repository, and the arena embeds its path at compile time (2026-10-24)

Since 2026-10-24 the repository is `~/workspace/within-reason/within-reason.git` (bare) and the tree is its worktree
`within-reason.main`, like the user's other projects; a side worktree goes beside it as
`within-reason.<branch>` (`git -C ~/workspace/within-reason/within-reason.git worktree add ../within-reason.<branch> -b <branch> main`).
The git-ignored working data (`upstream/`, `run/engines/`, `run/data/`, `run/matches/`, `target/`, 33 GB in all) lives
only in the main worktree; a fresh worktree has none of it and needs its own `run/data` (install_ai.sh) and engine.
`crates/arena/src/harness.rs` fixes `REPO` from `CARGO_MANIFEST_DIR` when it is compiled, so a binary built in one
place looks for the briefs, the scripts and the matches directory there for ever, and cargo does not count the
directory's name as a change (the move of 2026-10-24 recompiled six crates and left the arena with the old path):
after a move or in a new worktree, `touch crates/arena/src/harness.rs && cargo build --release` before any arena
run, or the arena quietly reads the old tree. Claude Code keys the session transcripts by the working directory (`within-reason.main`) and the auto-memory by the repository's git
directory (`within-reason.git`), under `~/sync/claude/projects/`, which both accounts share.

## The engine's resource cheat gives metal and never energy (2026-10-24)

`COMMAND_CHEATS_GIVE_ME_RESOURCE` (`Command::GiveResource` in our protocol) returns -2 and gives nothing for energy:
in `rts/ExternalAI/SSkirmishAICallbackImpl.cpp` the `break` after the metal branch sits outside its braces, so the
energy branch is never reached (2026.07.04 and the 2026.09 pre-release source alike). Measured in the gather drill's
first baselines: metal topped up as asked, the energy store at 0 from 30 s on, the lab at half pace. Energy for a
harness team comes from `--economy fed` (a free fusion beside the commander); the duel's gather drill is fed by
default. The host now logs a refused command that names no unit (`[wreason ai=N] GiveResource {..} refused: -2`),
which was silent before.

## A weaponless target for the bench is neither a generator nor a wall (2026-10-24)

The `into` drill's first target was the advanced fusion (8,300 health, no weapon): a bomb. Under the mixed column the
Blitzes alone had it down before the Janus arrived and its death blast (10,600 damage over 1,280) wiped the twelve
units that killed it, so the drill read as a side with no weapon dealing 66,727 damage and every duel timed out. The
fortification wall (8,900 health, a blast of nothing) is never fired at: a fight order ignores walls
(K-mech-a-fight-order-ignores-walls). The target is the gantry (`armshltx`, 16,000 health, a blast of 1,330 over
420), which outlasts the ramp and is fired at. The death blasts are in `upstream/Beyond-All-Reason/weapons/Unit_Explosions.lua`.

## A demo of a cheat-spawned duel does not replay (2026-10-24)

The duel harness (and every scenario and drill in it) spawns its armies through the AI cheat interface
(`Command::GiveUnit`, the engine's `COMMAND_CHEATS_GIVE_ME_NEW_UNIT`). `CAICheats::CreateUnit` calls
`unitLoader->LoadUnit` on the host's simulation directly (`rts/ExternalAI/AICheats.cpp`): no network message, so the
demo never carries the spawn. A replay re-simulates the game from the recorded net stream, in which only the start
script's commanders and the AI's orders exist, so the client shows two commanders standing, no other units, and
checksum and desync warnings every few seconds (the recorded sync checksums include the units the replay lacks).
Seen the night the drill demos were copied into the user's client. What does replay: a synced console action, such
as `/give`, which the engine sends as a `CommandMessage` over the net (`Game.cpp` `ActionPressed`), recorded and
re-executed. Built the same night, and simpler than a widget: the engine takes an AI's text that starts with a slash
as a console command (`CAICallback::SendTextMsg` calls `CGame::ProcessCommandText`), so `engine::Spawner::give`
sends `/give 1 <unit> <team> @x,0,z` as the AI's text and the shim sends `/cheat 1` from its first update. Two things
found building it: the unsynced give executor checks the cheat flag locally (`UnsyncedActionExecutor.h`), and the
flag set by `/cheat 1` comes back from the server a frame later, so a `/give` in the same frame as the `/cheat` is
refused without a log line (the first smoke's `spawn_failed`); the shim held spawns two frames (`lib.rs`
`CHEAT_LAG`). **And the game can swallow the `/cheat 1` altogether** (2026-10-24 night, two engines started
together: the infolog shows both AIs' `/cheat 1` lines at frame 0 and no "Cheating is enabled!" ever, then every
`/give` refused with "cannot be executed (cheats required)" and the duel `spawn_failed`; the match beside it, started
the same second, was fine), so the shim now sends `/cheat 1` again eight frames after the first and holds spawns
fifteen frames (`CHEAT_RESEND`, `CHEAT_LAG`): half a second a match. Whether the first command is lost in the server's
start or in the client's is not known; the second sending has not been seen to fail. Checked with `run/replay_match.py` on a drill's demo: the twelve Blitzes appear at frame 120 and walk
to the goal, no desync in the replay's infolog; the baselines' numbers did not move. The gather's metal top-up is
still the AI's resource cheat, so a gather's demo desyncs on its resources; everything else in a duel replays.

## The synced give sets thirty units down over seconds, and the sweep judged its bombs gone before they appeared (2026-10-24)

The user, watching the demos of the group bench: "in at least one of the games the area-clearing bedbugs survived until
the subsequent test round and affected the AI unit control." The field-clearing sweep between duels (`director.rs`
`Sweep::Bombs`) gives a lattice of crawling bombs (`corroach`, the Bedbug) over the wreckage, sets them off, and
judged a wave gone when no bomb stood one second after the give. That held while spawns were the AI cheat's
`CreateUnit`, which sets a unit down the same frame. The synced `/give` (the entry above) goes through the console,
the server and back, and thirty of them take the engine two to three seconds to set down: `run/replay_match.py` on
`regress-fboy5400-on`'s first match shows the first wave given at about 43 s and its thirty Bedbugs first present at
46 s, so the wave was judged gone at 44 s and the next two waves given at once; the first wave's bombs were set off by
the third wave's check when they finally stood (gone at 47 s), one of the later waves' thirty appeared at 48 s after
the sweep had rested and finished, and the next duel's 25 Stouts spawned at 51 s among thirty standing Bedbugs of
their own team, which died slowly through the next two duels (30 to 13 by 145 s). BAR's self-destruct gadget
(`game_selfd_resign.lua`) is not it: a Bedbug's countdown is 0 and the gadget cancels only countdowns in progress.
Every fight match since the synced spawner (31dc99e) with wrecks between its duels, so with a sweep, carried such
leftovers into its second to fourth duels: the regression arms of the group bench (ledger row regress-*) among them;
drills with no deaths (the walks, the reverse, the gather, the gantry) have no sweep and are clean. Fixed the same
night: a wave is gone only when every bomb given has appeared (within `SWEEP_APPEAR_ALLOWANCE`, 10 s, after which
the missing are presumed refused and said so) and none stands anywhere on the team, not only within the wreckage box;
the rest sets off any bomb still standing and starts again; `WITHIN_REASON_SWEEP_DEBUG=1` prints each give, set-off
and wave's end. The fight arms re-run after it (ledger row regress2-*): on those wreck boxes nine bombs a wave,
set down 3 to 12 frames after the give (once in two batches, 2 then 7: the engine sets a few units down a frame),
every wave gone in a second with every bomb seen, none standing at any rest; thirty a wave took two to three seconds.
