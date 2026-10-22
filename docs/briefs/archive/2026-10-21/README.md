# The brief set as it stood on 2026-10-21: the coverage map for the rewrite

Frozen copies of every text the models read on 2026-10-21, at commit d44e318 (unchanged through 47272ad), under their
checkout paths: the standing brief, the experience brief's documents, the generated reference tables, the role prompts
and the hands' rules. No bot reads anything here. The reading pass `docs/studies/2026-10-21-brief-annotation/` (700
rows, one per passage, with a layer, a fate and provenance) refers to these files by path and line number; the live
files move on with the rework, these do not, so the study's line numbers stay valid.

The rework (`docs/design/2026-10-21-brief-rework.md` §6) rewrites from the ground up and uses this set as its coverage
map: every passage the readers marked fact, meta, rederive or episode must end up either covered by a passage of the
new documents or discarded for a recorded reason. `run/brief_coverage.py` tallies that:

- A new document's `<!--meta ... -->` block names what it covers: `covers: [docs/briefs/player.md:92-107,
  docs/briefs/experience/cases.md:115-131/2]`, the study's file and line range, with `/N` for one part of a passage
  the readers split (no `/N` covers every part).
- `dispositions.jsonl` beside this file records the passages that get no home: one JSON object a line,
  `{"key": "docs/briefs/player.md:373-373", "disposition": "discard", "reason": "...", "by": "the user", "date": "2026-10-21"}`;
  a `disposition` of `covered` with an `id` records a cover decided outside a document.
- Everything else is open. Tool rows are not counted: the role prompts stay in place.

The texts' sources of the day: the standing brief `docs/briefs/player.md` (what the players read in people's games),
the experience brief (`docs/briefs/experience/`, `docs/briefs/reference/`; the commander read it whole, the player
under `--brief experience`), the role prompts (`crates/bot/src/strategist/`), the hands' rules
(`crates/bot/src/brain/pianist/`).
