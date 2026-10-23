# Within Reason documentation

Seven kinds of knowledge with different lifetimes. Put a fact where its lifetime says it belongs.

| Where | What | Changes when |
|---|---|---|
| `knowledge/` | Claims about the game and opponents, each with evidence and a status | every batch can support, weaken or retire one |
| `heuristics.md` | Registry of the brain's rules: ID, code location, the claims it rests on, status | a rule is added, changed or retired |
| `code-decisions.md` | The roster of what code decides in a live game without Jev or the player, with its constants | a code decision is added, changed or removed (the same commit) |
| `experiments.md` | Ledger of arena batches: label, commit, setup, result, what it was testing | every batch |
| `studies/` | Offline studies (simulation, data analysis) with their data; findings enter `knowledge/` as `conjectured` until the arena tests them | a study is added or rerun |
| `design/` | Target designs written before a large change, dated: what is true today, the design, how it will be judged, and a status line kept current while it is built. Decisions move into `DESIGN.md` once built | a multi-part arc starts, or its status changes |
| `briefs/` | What an LLM player is told at the start of a session, distilled from `knowledge/` with each line naming its entries; compiled into the bot (`briefs/player-opus.md` is read beside the brief by an Opus player only: the model's own habits in the seat, not the game's facts; `briefs/commander.md` follows the commander's role prompt) | a claim it cites changes status, or a game shows the model missing or misusing something |
| `harness/` | Engine, AI interface, lobby, arena and tooling facts | rarely; on engine or game bumps |

`../DESIGN.md` is the architecture (shim, protocol, bot process). It records decisions, not findings.

## The loop
1. **Observe.** Run a batch. Compare wins against losses using the per-minute status lines in `bot.log`
   (and, once rule-firing logs exist, which heuristics fired).
2. **Claim.** Write what you think is true as a knowledge entry with status `conjectured`, citing batch, match and frame.
   A claim must be checkable: say what observation would prove it wrong.
3. **Exploit.** Add or change a heuristic. Give it an ID, register it in `heuristics.md` with the claims it rests on.
4. **Verify.** Run a batch that could show the change did nothing. Record it in `experiments.md`.
   Move the claim to `supported`, or to `refuted` with the evidence. 12 matches is noise-level (±14 points); confirm with 24+.
5. **Retire.** When a claim stops holding (new opponent tier, new map, our own play changed the situation), mark it
   `retired` with the reason and retire or rewrite the heuristics that cite it. Never delete entries: why something stopped
   being true is knowledge.

Rules of the road: a heuristic without a registered claim is a guess — register the guess as `conjectured`. A batch without a
ledger line did not happen. Results that contradict a `supported` claim reopen it; say so in the entry rather than quietly
tuning around it.

**Reviewing a game** (step 1 of the loop, one match at a time): the `bar-review` skill (`.claude/skills/bar-review/SKILL.md`,
a subagent given a match directory) reads the record, the pianist's log, the transcript and the truth file through the
`run/` tools, holds the game against the high-OS replay baselines (`.claude/skills/bar-review/baselines.md`) and writes
`review.md` (findings ranked by cost, each with its clock, its evidence and the baseline it falls short of; what a
stronger player would have done; the player, the hands, the harness) and `verdict.json` (`harness/verdicts.md`, the
shape `run/tally_verdicts.py` adds up) into the match directory. The checklist grows with every packet of feedback
from experienced players (the skill's last section; the notes themselves go to `knowledge/_inbox/player-notes.md`).
The study behind it: `studies/2026-09-27-review-skill.md`. A second kind of review tallies the interface rather than
the game: did the harness do what the player ordered, and was the player told what it needed (`harness/impedance.md`:
`run/impedance_sweep.py`, the brief in `studies/data/impedance-2026-10-16/`, `run/impedance_tally.py` to sum the
reviewers' `impedance.json` files; the study `studies/2026-10-16-opus-hands-impedance.md`).

**The live edge** (`design/2026-10-21-brief-rework.md` §11, `harness/arena.md` "The live edge"): the one piece of game
memory that closes without anyone. When a player match ends, `run/live_edge.py close` (run by the arena, and by
`run/human_game.sh` after the bot leaves) writes the game's entry into `run/data/live-edge/<brief>/` (one store a brief, since a player's words written under one brief are not the next brief's episodes): the report in plain numbers
from the record and the truth file, and the player's and the commander's own words with the player's debrief. The next
bot against the same opponent on the same map carries the newest three entries of the last two weeks after its brief.
Nothing moves from there into `briefs/` or `knowledge/` without a person's review; entries age out on their own.

**The unit glossary** (`crates/bot/data/units.json`: numbers generated from the game source by `run/unit_stats.py`, a
gloss and a prose paragraph a unit written by hand) is what the player's `units` tool and every menu line say about a
unit. `run/glossary_sheet.py` lays it out with the repo's other per-unit facts as `glossary/units.csv`, a sheet for
experienced players to annotate (`glossary/README.md`: the columns, what we ask, the way back through `knowledge/_inbox/`).
