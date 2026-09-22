# The carry loop: games that integrate their own experience

Written 2026-09-22 after the objective games, from the user's direction: "What is stopping us from running many
games wide... we should find a way to scale up attempts and integrate experience"; "letting successive games
auto-integrate somewhat. In human sessions at least nothing should be stopping us from raising the turn limit, ending
with some kind of compaction review, then adding the notes from the previous game to the prompt on the next one. That
won't work as well for parallel arena games, but it seems likely we can find something similar that will."

## What stands today

- A player session is replaced after 40 turns by a fresh one handed its notes (`strategist/mod.rs`
  `turns_per_session`); a 20-minute game peaks near 110k tokens of context, so a whole game fits one session with
  room for the roster (about 10k) and a carry file.
- After a game, `run/debrief.py` asks a fresh session to read the logs and writes a debrief into
  `docs/briefs/inbox/`; the brief (`docs/briefs/player.md`) is edited by hand from those and from the ledger.
- The arena runs matches in parallel (`--parallel N`), each with its own player session and Jev stream; the machine
  allows about ten, the cluster more. A player game costs a fraction of a point of the week's Opus window; the
  five-hour window is the burst limit and has not been measured clean of this session's own use.
- The fundamentals scorecard (`run/floor.py`) reads one match at a time.

## Decisions

1. **One session a game.** The turn cap becomes a game's worth (`turns_per_session` 400 for the player); the restart
   stays as the fallback for a hung session and for an edited prompt.
2. **The game's own session writes the review.** When the game ends (the result is known, the referee or the
   engine), the strategist takes one more turn: the result, the scorecard row and the ledger's shape, and the ask:
   *what worked, what you wished you had known and at which minute, which brief lines were wrong, one or two
   claims with the evidence from this game*. A fixed shape under 400 words. It is written to the match directory as
   `carry-<ai>.md` and appended to `docs/briefs/carry/<label>.md`. `run/debrief.py` retires: a fresh reader of the
   logs is worse than the player that lived them.
3. **The next game reads the carry.** The role text takes `docs/briefs/carry/<series>.md` above the brief, where
   the series is named by the arena (`--carry NAME`; the human game script names its own). For one game after
   another (a human series, a sequential arena series) this is the whole loop.
4. **A batch merges its reviews before the next batch.** For parallel games, `run/carry.py merge <batch>` reduces
   the batch's reviews to one carry file with counts ("raised in 9 of 16 games"), keeps contradictions side by side
   rather than picking one, and drops what a single game raised unless it names evidence. One model call, its
   output checked for size and shape.
5. **The carry is an arm.** A batch runs half its games on the new carry and half on the previous one
   (`--carry NAME@N` names a version), judged by the batch scorecard (decision 7). A carry that does not move the
   medians is not promoted; the loop cannot drift on prose alone.
6. **Carry notes expire; the brief is promoted to.** A note lives three batches unless re-raised; what survives
   with evidence across batches is proposed for the brief's general layer as a claim in `docs/knowledge/_inbox`
   (the observe -> claim -> exploit -> verify -> retire loop of `docs/README.md`). The brief stays hand-edited; the
   carry is the machine's.
7. **Batch scoring.** `run/floor.py --batch <dir>` gives medians and spreads over a batch's matches and a two-arm
   comparison (`--arms A B`), the same columns as the per-match rows. Sixteen games of one configuration first, for
   the baseline spread on Comet.
8. **The layers.** The brief is split into `general.md`, `map/<map>.md` and `opponent/<name>.md`; the role text
   takes general + the map + the opponent + the carry. The map and opponent layers are where a magic packet lives;
   the general layer is what compounds.

## What this does not decide

Self-play (an opponent option that is our bot with its own player; two records and two reviews a game) comes after
the loop above works, since its value is the doubled review. Running on the cluster needs the account's OAuth token
and the Jev key inside the per-game image through a secrets path; nothing else in the arena assumes one machine.

## Steps

1. The sixteen-wide probe on Comet easy, Opus with Jev and Lua, with the five-hour window measured before and after
   and Jev's latency under load read from the logs (no code).
2. `run/floor.py --batch` and `--arms` (7).
3. The end-of-game review turn and the carry file (1, 2, 3); the human game script names its series.
4. `run/carry.py merge` and the carry arm in the arena (4, 5); expiry and the knowledge-inbox proposal (6).
5. The layered brief (8), by hand, once the carry has run for a few batches and shows what is general.
