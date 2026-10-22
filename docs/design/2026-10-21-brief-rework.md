# The brief rework: derivations over lessons, sources over artifacts, games decoupled from edits

Status: draft 1, 2026-10-21, in discussion with the user. §5 (the snapshot), the assembler (§3, §4), the archive, the
coverage tally and the fact layer's unit tables (§6, steps 3 and the generated half of 4) are built; no brief, prompt or
rule has changed in what a model reads. The
reading pass that motivates it is `docs/studies/2026-10-21-brief-annotation/` (700 rows, `summary.md`, `backlog.md`);
the texts as each role reads them are shown by `run/brief_view.py`.

The user, 2026-10-21: the brief "got us to where we are, but ... it still has a lot of anecdotal experience from back
when BARb medium was a fair fight ... a pile of conflicting guidelines that are probably overconstraining the player
and commander models too harshly." The aim: "transitioning the body of our knowledge from 'lessons from experienced
players' to 'reasoned consequences of the game design with carefully explained derivations' ... reduce our reliance
on direct lessons and instead focus more on the why behind them." He expects "to throw out much of the existing
brief, instead using that as guidelines for where to try re-deriving doctrine from scratch, or factoring into meta
explanations of 'the body of humans decide to do x'", and leans to "a complete ground-up rewrite of everything, with
the existing version archived in the repo as a basic coverage map for the rewrite." Worse-before-better is expected
and accepted. Two things fold in: a better approach than the markdown rebuild, which "has backfired repeatedly", with
"a comment syntax that lets us add provenance metadata that doesn't make it into the in-game model context"; and
fixing "the current issue where we can't modify key parts of the codebase while games are ongoing."

## 1. What the reading pass measured

Of 700 passages across the standing brief, the experience brief, the role prompts and the hands' rules: 177 are tool
text, 162 episodes, 148 lessons to re-derive (106 cleanly, 42 partly, none found underivable), 105 meta, 84 facts, 24
discards. 203 rules, 123 of them from a single case. 258 passages name a harness defect of their time; 107 of the 128
games cited carry one. 24 numbers disagree with the glossary, several by unit class. The sprint paragraph stands in
nine files. Two briefs are live: `run/human_game.sh` and the arena without `--brief` give the players the standing
brief while the commander always reads the experience brief, so the seats and their commander read different texts.
The standing-versus-experience comparison (player-54/55) never resolved. Twenty conflict clusters await a ruling
(`summary.md` §b); the derivation backlog by subject is `backlog.md` §a.

## 2. The four layers (the user, 2026-10-21)

1. **Fact**: core factual information grounded in the game's code and in ours. The references (unit definitions,
   engine behaviour, our harness code) are useless in a game but are the ground-truth standard behind the facts.
2. **Meta**: the meta of the game. Fluid models of general player behaviour, per-player quirks, common behaviour on
   maps. "The body of humans decides to do X", with its population, map and date, and a rationale when we have one.
   Some knowledge is only this (the pool techs in 4 of 58 duels) and is never derived.
3. **Derived**: consequences of the first two, as logically rigorous as we can make them, so that situational
   reactions do not get baked into doctrine. A derivation carries its premises; a lesson is a derivation with the
   premises thrown away, which is why lessons conflict. The model runs the last step itself under the premises in
   front of it.
4. **Episode**: episodic memory of games, recorded with the defects of the bot systems at the time and the review
   lens the match was inspected through. Diagnosing a loss is a very hard bar; a report may be wrong and reports
   deprecate over time. Episodes are evidence and tests for derivations, not doctrine.

Outside the brief: **tool** text (how the harness, the tools, the packet and the turn work) stays in the role prompts;
**rulings** (a person's directive) are recorded as sources of facts, meta or derivations, dated, not as free-standing
rules. The hands' standing rules are a role text of the hands and are read per pick; they carry fight doctrine today
(reader E) and are in scope for the same treatment.

## 3. The source format: documents, metadata in comments, ids

Built 2026-10-21: the comment syntax (`texts::strip`: every HTML comment goes; a `<!--meta ... -->` block with
`status: deprecated` takes its passage with it, to `<!--/meta-->` or the next blank line), the checker
`run/brief_check.py` (ids, premises, tests, duplicates, deprecated citations; no document carries a block yet). Not
built: front matter, generation of the fact layer.

Markdown documents stay the source of truth. Provenance and structure live in HTML comments, which Markdown renderers
and GitHub hide and which the assembler strips, so nothing of it reaches the model. Today only the player's role
prompt carries comments (seven markers), and the reader handles exactly those kinds; any other comment reaches the
model verbatim, so the assembler must strip every comment.

```
<!--meta
id: D-sprint-timing
layer: derived
premises: [F-armavp-cost, F-moho-payback, M-pool-tech-rate]
tests: [E-sd129-2v3, E-center-command-2v4]
sources: {people: [the user], dates: [2026-10-18, 2026-10-21]}
status: live
-->
```

- One block before each passage; `key: value` lines, lists in brackets, a map in braces. A document may open with a
  front-matter block (`---` lines) for document-level keys (layer, owner, last review); the assembler strips it.
- **Ids** make the rigor checkable. Prefixes by layer (`F-`, `M-`, `D-`, `E-`). A derivation cites its premises and
  the episodes that test it; an episode cites the derivations it bears on and its `defects` and `lens`; a fact names
  its grounding (`units.json`, the engine, our code, a measurement). A checker resolves every id, lists derivations
  with missing premises, facts no derivation uses, and refuses a context that cites a deprecated id.
- **Status** deprecates in place: `status: deprecated 2026-10-21 (reason)`. The source keeps the passage; the
  assembler drops it. Episodes deprecate as their diagnosis is overturned.
- The claim ids, game labels and dates now in the prose (104 `K-` claims, 102 `player-N` labels in the context) move
  into the blocks. The model reads prose without them.
- The fact layer is generated where it can be: unit facts from `units.json` by a generator, never hand-copied (24
  hand-copied numbers were wrong). Generated documents carry the generator's name in their front matter.

## 4. The assembler: manifests, no artifact, one implementation

Built 2026-10-21: `docs/briefs/standard.manifest` and `experience.manifest`, `texts::assemble_parts`, the artifact
`player-experience.md` and `run/build_brief.py` deleted (the map sheets became the generated document
`docs/briefs/reference/map-sheets.md`, `run/map_sheets.py`; the other generators are run by
`run/regenerate_reference.py`), `bot --print-prompt ROLE [--sources]`, and `run/brief_view.py` printing from the
binary. The context the models read was checked byte for byte against the artifact and the previous composition.
The brief markers in the role prompt and the compiled copies of the role texts stay until the standing brief retires.

The rebuild's failures share a cause: an intermediate artifact (`docs/briefs/player-experience.md`) that is generated
and committed and compiled, so parts, artifact and compiled copy disagree, a person must remember the step, and the
tool writes on any invocation. The artifact goes.

- A **manifest per role** (the player, the player under command, the commander, the hands) lists the documents in
  order. The bot assembles at session start from the manifest, reading the documents themselves. Generated tables are
  documents like any other, written by their own generators. Nothing assembled is committed.
- The bot gets **`--print-prompt ROLE`**, so the viewer, the tests and the game share one implementation of
  composition. `run/brief_view.py` re-implements it in Python today, which is the same bug waiting to recur; it will
  call the binary instead (for a revision, the binary of now over the texts of then, said on the page).
- The role prompt's `<!--brief:standard-->`/`<!--brief:experience-->` passages go with the standing brief. One source
  set serves every role; a role's manifest picks.
- The compiled copies in `texts.rs` (`include_str!`) stay only if a game must run away from a checkout. Otherwise they
  are a second source of truth and go. (Open: §9.)
- `run/build_brief.py` is deleted, not fixed; `--regenerate` becomes each generator's own job.

## 5. Games decoupled from edits: the snapshot

Built 2026-10-21 (`bot --snapshot DIR` in `crates/bot/src/texts.rs`; the arena once a batch, `run/human_game.sh` once a
game; `run/brief_view.py --match <batch>`; the smoke `snapshot-smoke` ran from its snapshot). The reload signal and the
compiled copies (§9) are untouched.

Today an edit to a text read at run time reaches a running game: the player's session restarts the turn the prompt on
disk changes (`strategist/mod.rs`, the digest compared every turn, by the user's 2026-09-22 ruling that text updates
need no rebuild), the hands read their rules from disk at every pick, and the arena spawns `target/release/bot` fresh
for every match while `run/install_ai.sh` rebuilds it at every arena start, so a `cargo build` or a second batch
changes the binary the first batch's later matches run. The shim alone is safe: installed by rename, a running engine
keeps the library it mapped.

- At launch, the launcher (the arena per batch, `run/human_game.sh` per game) writes **`<batch>/snapshot/`**: the bot
  binary, every text the manifests name and the hands' texts, and the code state (`git rev-parse HEAD`, `git status`,
  `git diff` of the working tree). It sets `WITHIN_REASON_TEXTS` to the snapshot (the bot's `texts::root()` already
  honours it) and runs the snapshot's binary.
- The restart-on-edit rule is then moot for a running game and the 2026-09-22 ruling is kept for the next one. If live
  edits are wanted on purpose, a deliberate signal (a file touched in the snapshot) can reload; by default nothing
  changes under a game.
- The record then carries the texts a game was played with. That is the fourth layer's "defects at the time" made
  concrete: a review diffs the game's snapshot against HEAD, `run/brief_view.py --match <dir>` shows the brief as
  read, and a ledger row's commit column becomes exact.
- The arena's rebuild at every start stays (it is how the shim reaches the engine) and becomes harmless to running
  batches.

## 6. The rewrite

Built 2026-10-21: the archive `docs/briefs/archive/2026-10-21/` (the 27 annotated texts under their checkout paths, a
README, `dispositions.jsonl`), the coverage tally `run/brief_coverage.py` (523 rows counted, every one open at the
start; `--by backlog` groups the open rows by the merge pass's subjects), the fact layer's unit tables generated from
`crates/bot/data/units.json` by `run/fact_units.py` (five documents under `docs/briefs/facts/`, 586 citable ids
`F-unit-<name>`, 19,600 words in all; which domains a role's manifest carries is the manifest's choice, and the player
keeps its `unit` tool for one unit at a time), and `docs/briefs/rework.manifest`, the brief under construction that no
role reads yet (the viewer's "Rework" tab; `bot --print-prompt manifest:PATH`). Not written: the economic, fight and
map models; no derivation, meta or episode document exists yet.

Ground-up, not edited in place: editing preserves hidden assumptions (the repository's own delete-then-rebuild rule),
and generation is near deterministic, so a rewrite that starts from the old text reproduces it while one that starts
from premises produces different text.

- **The archive as coverage map.** A snapshot of the current briefs, prompts and rules goes under
  `docs/briefs/archive/2026-10-21/` so the study's line numbers stay valid; the live files stay where the bot reads
  them until the replacement lands. A script tallies coverage from the study's JSONL: every fact, meta or rederive row
  either has a home in the new documents (an id) or a recorded discard reason.
- **Derivations** are written subject by subject in the backlog's order, the twenty conflict clusters first since they
  are the contested subjects. Each names its premises by id, the old lessons it must reproduce or explain away as
  tests, and a check beyond the lessons where one exists (the combat simulator, a unit duel, a pool number, a recorded
  game). The user rules only where a derivation cannot settle a cluster (taste, or meta). A derivation's hit rate is
  about half until checked; an unchecked derivation says so in its block.
- **Meta** is catalogued from `backlog.md` §b by population (the pool at OS 40+, named people, BARb by profile,
  Apex), each observation with map and date, rationale attached when known.
- **Episodes** are an index, not prose: the game, what it showed, its defects, its lens, the derivations it tests.
  Those with a lesson resting on a fixed defect are indexed with the defect and no lesson.
- **Facts** are generated from `units.json` and the map data where possible; the economic model (metal, energy and
  build power as flows, an extractor's payback, the price of a minute), the fight model (range, damage, hit points,
  speed, what wins at equal metal) and the map model (walks, chokes, sight) are written as facts with their grounding,
  since derivations rest on them and the current reference curves are observations, not the model.
- **Size**: ten to fifteen thousand words in a role's context against today's 34,000 to 45,000.

## 7. Measurement

The standing brief is the control: the same seeds in the arena and a game with people. Worse-before-better is
expected (the user, 2026-10-21) and is not a reason to stop; the measure is whether the derivations' predictions hold
in games (the predict tool, the floor's columns) and whether the players' reasoning in their notes cites premises
rather than cases. The reading pass is rerun on the new set when it stands, with the same rubric, and the two censuses
compared.

## 8. Order of work

1. The snapshot (§5): launcher, env, `--match` in the viewer. Small, and it unblocks the rest, since the rework is
   weeks of edits while games run.
2. The source format and the assembler (§3, §4): the comment syntax, ids, the checker, the manifests,
   `--print-prompt`, the viewer calling the binary; `build_brief.py` deleted. The new documents are born in the
   format.
3. The archive and the coverage script (§6).
4. The fact layer, generated where it can be; the three models written.
5. The meta catalogue.
6. Derivations in backlog order, clusters first, each with its tests and check.
7. The episode index.
8. The role manifests switched to the new set; the standing brief and the experience brief retired to the archive; the
   control kept runnable from the archive for measurement.
9. Measure (§7); rerun the reading pass.

## 9. Open, for the user

- Who writes the derivations: agents subject by subject with the user reviewing each, or the user with an agent per
  subject.
- Whether live text edits mid-game remain possible by a deliberate signal, or go entirely.
- Whether the compiled copies of the texts stay (a game away from a checkout) or go.
- The id scheme's prefixes and whether the knowledge base's `K-` claims map onto them or retire.
- Whether the hands' rules join the same source format now or after the brief.
- The archive's place (`docs/briefs/archive/<date>/` is the proposal).

## 10. Rulings so far (the user, 2026-10-21)

- The four layers as in §2, in his words.
- Lessons give way to derivations with explained derivations; the existing brief is the coverage map, not the source.
- Worse-before-better is expected.
- The rebuild system is replaced, not repaired; provenance goes in comments the model never sees.
- Edits to key parts of the codebase must not touch a running game.
