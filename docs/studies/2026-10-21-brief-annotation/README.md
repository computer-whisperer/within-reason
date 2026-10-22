# The brief annotation (2026-10-21)

A reading pass over every text the models read (the player's and commander's briefs, the role prompts, the hands'
standing rules), one row per passage, before the brief is reworked. The rework (the user, 2026-10-21) moves the body
of knowledge from "lessons from experienced players" to "reasoned consequences of the game design with carefully
explained derivations", keeps what people do as observed behaviour rather than doctrine, and keeps games as episodes
whose diagnosis may be wrong. The user expects to throw out much of the current brief; what is kept is the guideline
for where to re-derive doctrine from scratch. So this pass does not decide; it classifies, sources and flags, and
it leaves the deciding to the user.

The texts as each role reads them: `run/brief_view.py` (a page per revision under `run/data/brief-view/`). The
annotation is of the working tree at commit d44e318 (the briefs unchanged since, except an uncommitted rebuild of
`docs/briefs/player-experience.md` that added two map sheets).

## The four layers (the user, 2026-10-21)

1. **Fact**: core factual information grounded in the game's code and in our code. The references (unit definitions,
   engine behaviour, our harness code) are useless in a game, but they are the ground-truth standard behind the
   facts.
2. **Meta**: the meta of the game. Fluid models of general player behaviour, per-player quirks, common behaviour on
   maps. "The body of humans decides to do X", with its population, map and date.
3. **Derived**: consequences derived from the first two, as logically rigorous as we can make them, so that
   situational reactions do not get baked into doctrine. A derivation carries its premises; a lesson is a derivation
   with the premises thrown away.
4. **Episode**: episodic memory of game experiences, recorded with the defects of the bot systems at the time and the
   review lens the match was inspected through. Diagnosing why a game was lost is a very hard bar; a report may be
   inaccurate and reports deprecate over time.

Two kinds outside the brief: **tool** text (how the harness, the tools, the packet and the turn work; it stays in the
role prompt) and **ruling** text (a person's explicit directive, usually "the user, <date>: ...").

## The fate of a passage

Every row says what the passage would become under the rework. The fate is the annotator's suggestion, not a decision.

- `fact`: a fact to ground; say where it would be grounded (unit definitions in `docs/glossary/units.csv` and
  `units.json`, the engine, the map data, our code) and whether you checked a number against the glossary.
- `meta`: an observation of what people do; name the population (the pool at OS 40+, thebluegecko, BARb's profile),
  the map and the date.
- `rederive`: a lesson to re-derive from the game's design. Name in one line the premises the derivation would rest
  on (which facts, which meta), and say whether it looks derivable (`yes`), partly (`partly`: the lesson has a
  derivable core and a situational remainder), or not (`no`: it is meta in disguise, or an artefact of a harness
  defect). The lesson itself becomes a test the derivation must reproduce or explain away.
- `episode`: a game's account kept as evidence; record the harness defects of its time that its lesson depends on, and
  the lens (a person's review, a model's review, the ledger row, the players' chat).
- `tool`: stays in the role prompt, out of the brief.
- `discard`: nothing fits (a lesson against a defect since fixed, a number from an opponent we no longer play, prose
  that repeats another passage).

## The row

One row per passage: a paragraph, a bullet with its sub-bullets, a table, or a heading's introductory text; three to
thirty lines, one subject each. A long bullet that argues several subjects is split into several rows with the same
line range and a `part` number. JSON Lines, one object a line:

```
{"file": "docs/briefs/player.md", "lines": [92, 107], "part": 1, "quote": "Tier 2 is a sprint, and the question",
 "subject": "the tier-2 sprint's timing in a team game",
 "layer": "derived|fact|meta|episode|tool|ruling|mixed",
 "fate": "fact|meta|rederive|episode|tool|discard",
 "premises": "the advanced plant's cost and build time; the feeders' income; the spots left; his army in sight",
 "derivable": "yes|partly|no|n/a",
 "grounding": ["engine", "code", "pool", "person", "our-game", "model-review", "unknown"],
 "sources": {"games": ["player-9", "bluegecko-3v1-10"], "people": ["the user", "thebluegecko"], "claims": ["K-..."], "dates": ["2026-10-18"]},
 "checked": "the 2,600 metal of the advanced plant matches units.csv (armavp 2,600 m)|unchecked|mismatch: ...",
 "era": ["barb-medium", "barb-hard_aggressive", "people", "pool", "apex", "pre-hands-rebuild", "undated"],
 "defects": "the harness defects of the time the lesson depends on, by name and date, or none, or unknown",
 "lens": "person|model|ledger|chat|unknown",
 "doctrine": "none|rule-from-one-case|rule-from-pool|rule-from-ruling",
 "conflicts": ["docs/briefs/experience/cases.md:133-146 says the sprint starts at the first bank; this says now"],
 "duplicates": ["crates/bot/src/strategist/commander.md:19-39"],
 "confidence": "high|medium|low",
 "note": "one or two sentences: what the passage claims and why it got this fate"}
```

`layer` is what the passage is now; `fate` is what it would become. `doctrine` flags a rule ("never", "always",
"must", "the only question is") and where the rule came from: one case, the pool's numbers, or a ruling.
`conflicts` are passages that read the same subject differently; `duplicates` are the same subject stated again
(the sprint paragraph stands in five files). Within your own files, give line ranges; for another reader's files,
give the file and the subject and the merge pass will join them.

## Eras and defects, as hints

From the ledger (`docs/experiments.md`) and the memory of the sessions; check the ledger or `git log` before building
on one. A lesson from an era is not wrong for being old, but the annotation says which era it is from.

- BARb easy and medium were the opponents until about 2026-09-22 (the Comet escalation series); hard_aggressive from
  2026-09-27 (the arena target); hard from 2026-10-20 (the bonus ladder). Apex from 2026-10-13.
- Games with people: the bluegecko 2v1/3v1 series from 2026-09-27; human-N games through October; the lobby games
  human-8v8-1 (2026-10-18), Center Command 2v4 (2026-10-19), the +40/+50/+60 Altored 3v3s (2026-10-19), Gasbag
  Grabens and Project SD-129 (2026-10-20).
- The pro pool: public replays at OS 40+, mined 2026-09-23 on; Comet Catcher Remake 1.8 has the cards, Quicksilver has
  none.
- Hands defects fixed, with the date, so a lesson against one can be marked: the stale under-fire record and three
  list bugs (2026-10-01); the escort order and the waiting list step (2026-10-01); the hands rebuilt with one menu per
  actor and layers (2026-10-02; `pre-hands-rebuild` is anything before it); the human-11 fixes, losses with killers,
  staging lists, help words (2026-10-03); rove as a group task (2026-10-07/08); the extractor in the lab's exit lane
  (2026-10-13); plants and turrets built at low priority (2026-10-13); the chase that did not withdraw at danger, the
  frames looked up by type and distance, the spending line (2026-10-17); the ten impedance fixes and seven levers
  (2026-10-16/17); the standing `transfer` (2026-10-19/20). Jev 1.14.0 pinned 2026-10-16.
- Review lenses: `docs/experiments.md` rows are the main session's reading; `review.md` files in match directories are
  a model's; the players' words (`docs/briefs/experience/players-words.md`, `docs/knowledge/_inbox/player-notes.md`)
  and the user's rulings are a person's. The user's rule (2026-09-27): people find the cause minutes earlier; model
  reviews name the symptom.

## The parts

| part | files | reader |
|---|---|---|
| a | `docs/briefs/player.md`, passages whose first line is 1 to 600 | reader A |
| b | `docs/briefs/player.md`, passages whose first line is 601 to 1204 | reader B |
| c | `docs/briefs/experience/intro.md`, `winning.md`, `cases.md`, `late-game.md` | reader C |
| d | `docs/briefs/experience/units.md`, `hands.md`, `opponent-barb.md`, `players-words.md`, `team-games.md`, `other-maps.md`; `docs/briefs/reference/*.md` at the level of their sections | reader D |
| e | `crates/bot/src/strategist/player.md`, `commander.md`, `player-under-command.md`, `packet-example.md`; `docs/briefs/player-opus.md`; `crates/bot/src/brain/pianist/rules.md`, `rules_gate.md`, `default.md` | reader E |

Each reader writes `part-<x>.jsonl` (the rows) and `part-<x>.md` (a summary: counts by layer and by fate; the
derivation backlog, each subject with the lessons it must reproduce; the meta catalogue entries; the episode index
with defects and lens; the numbers checked against the glossary; the conflicts; the era-bound passages). The merge
pass joins the conflicts and duplicates across parts into `summary.md`.

## Rules for the readers

Read every line of your files; the row count should cover the file. Facts about games come from the text, the
ledger and `docs/knowledge/`, never from memory of other games. Do not edit any brief, prompt or rule; do not run
games, engines or model calls; write only into this directory. A number is checked against `docs/glossary/units.csv`
when it is a unit's cost, range, speed or hit points and the check is one lookup. Where a passage's claim rests on a
model's review of a game, say so; where you cannot tell, say `unknown` rather than guess.
