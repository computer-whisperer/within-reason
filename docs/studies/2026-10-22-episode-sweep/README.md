# The episode sweep: every game with people, read for the brief's episode layer

Started 2026-10-22 (the user: "start the background run for the episodic memory section we will add later"). The
brief rework (`docs/design/2026-10-21-brief-rework.md`, §2 layer 4, §6 "Episodes") wants episodic memory as an index,
not prose: for each game, what it showed, the harness defects of its time, the lens it was read through, and what
in the brief it bears on. Most games with people never had a formal review (17 of 57 have a `review.md`), so this
sweep reads every one of them with the review skill's instruments (Opus 5.5 readers, the user's call: background fleets run
on Opus) and writes one row per game.

## Inputs

- `games.jsonl`: one line per game, `batch`, `dir` (absolute; some are in the archive on /ceph), `map`, `minutes`,
  `people` (name and lobby rating as the record's header has them; the games before 2026-09-27 evening recorded no
  person, a shim bug, so `people` is empty there and the chat says who played), `our_seats`, `records`, `review`
  (an existing review.md, or null).
- `chat.jsonl`: every chat line of every game, attributed: `speaker` is a person's lobby name, "our player" (the
  `[WReason]` lines our player typed), or `computer_whisperer`, who is the user, hosting and watching; his lines are
  his comments on the game as it ran and mark the minute he saw it go wrong. Spectators are marked.
- The ledger `docs/experiments.md` (grep the batch's label, without the timestamp prefix) for the main session's
  reading of the game and the build it ran on; `docs/briefs/lessons.md` for the players' lessons (ids `L-...`);
  `docs/studies/2026-10-21-brief-annotation/backlog.md` §a for the derivation backlog's subjects;
  `docs/studies/2026-10-21-brief-annotation/README.md` "Eras and defects, as hints" for the harness defects by date.
- The review skill `.claude/skills/bar-review/SKILL.md` (read it; do not invoke it) and its `baselines.md`: the
  instruments (`run/minutes.py <dir>`, `run/analyze_match.py`, `run/pro_baseline.py` on Comet only, `run/floor.py`,
  `run/raid_ledger.py`, `run/commander_turns.py`, `run/wake_read.py`, `run/hands_window.py`, `run/impedance_sweep.py`)
  and the pro baselines. A tool may fail on an old record (format drift); say so in the row and go on.

## The row (`rows/<batch>.json`)

One JSON object per game, the fields below. Facts come from the tools and the records, never from memory of other
games; a claim the tools cannot support is marked as the reader's reading. Cause before symptom (the user's rule):
walk back from every visible failure to the earliest minute it was still avoidable, and rank by root.

```
batch, label (the ledger's label or the batch name), map, date (YYYY-MM-DD), shape ("3v1" etc.), people (names and
ratings), our_side (seats, factions, player model and effort, commander if any, hands version: from the banner lines
in chat.jsonl and the ledger), build (commit from the banner), result ({outcome: won|lost|aborted|unknown, minute}),
summary (3 to 6 sentences: how the game went, both sides),
root ({clock, what, evidence}: the earliest minute the game left the winning path, with the tool line that shows it),
turning_points ([{clock, what, evidence}]),
our_mistakes ([{clock, what, evidence, kind: strategy|harness|both}]: what we did badly and what a stronger player
  would have done),
their_play (what the person did that decided it: timings of tier 2, raids, unit choices, where they fought),
defects ([{defect, fixed: date or "open", evidence}]: harness defects of that build seen in this game, named against
  the eras list; a loss that rests on a since-fixed defect says so),
user_comments ([{clock, text}]: computer_whisperer's lines from chat.jsonl, verbatim),
bears_on ([{id, relation: supports|contradicts|illustrates, note}]: lessons by L- id and backlog subjects by name),
lens ({reviewer, date, tools: [the tools run], existing_review: path or null, confidence: high|medium|low, caveats}),
open_questions ([strings]).
```

## Rules for the readers

Read only; write only your own `rows/<batch>.json`. Do not run engines, replays, cargo or model calls; do not kill
processes; do not edit anything else in the checkout, which the main session is working in. The archive directories
on /ceph are read-only to you. Quote tool output as evidence, with the clock. Prefer what a person said (the chat,
the ledger's quotes of the user) over your own reading, and say which is which.
