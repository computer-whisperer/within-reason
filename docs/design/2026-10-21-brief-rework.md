# The brief rework: derivations over lessons, sources over artifacts, games decoupled from edits

Status: draft 1, 2026-10-21, in discussion with the user. §5 (the snapshot), the assembler (§3, §4), the archive, the
coverage tally, the fact layer's unit tables, the lessons document and the economic model (§6, steps 3, 4 in part
and 4b) are built; no brief, prompt or rule has changed in what a model reads. The
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

## 2. The five layers (the user, 2026-10-21)

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
5. **Lessons** (added by the user later the same day): a dedicated section for verbatim lessons from the very
   experienced players, primarily thebluegecko, with their context. "We are still weak enough that they are going to
   be high quality heuristics, but the hope should be to build our own reasoning and detailed episodes to support or
   replace them over time." A lesson is kept in the player's own words, never paraphrased into a rule, with who said
   it and their rating at the time, when (date, game, clock), what was on the board and what prompted it, and how it
   reached us (the game's chat in the record, or relayed by the user from a conversation). Its block names the
   derivations and episodes that have come to support it or that replace it; until one does, it stands on the
   player's authority alone, and the brief says so.

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
- **Ids** make the rigor checkable. Prefixes by layer (`F-`, `M-`, `D-`, `E-`, `L-` for a lesson). A derivation cites its premises and
  the episodes that test it; an episode cites the derivations it bears on and its `defects` and `lens`; a fact names
  its grounding (`units.json`, the engine, our code, a measurement); a lesson names its speaker, date, game, clock
  and relay, and a derivation that reproduces or explains away a lesson lists it under `lessons:` the way it lists
  its `tests:`, so the checker can list the lessons no derivation has reached yet. A checker resolves every id, lists
  derivations with missing premises, facts no derivation uses, and refuses a context that cites a deprecated id.
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
keeps its `unit` tool for one unit at a time), and `docs/briefs/rework.manifest`, the brief under construction, which no
role read until 2026-10-23, when it became the default (§8 step 8; the viewer's "Player, rework brief" tab). Written 2026-10-22: the economic
model, `docs/briefs/facts/economy.md` (nine F-eco blocks, about 2,200 words: the store and overflow, extractors and
the metal map, generators, build power and the draw with a table of the common builds, decay, reclaim and
resurrection, conversion, the lobby's multipliers), each block citing the engine and game files it is read from and
the K- claims it replaces; the fight model, `docs/briefs/facts/fights.md` (ten F-fight blocks, about 2,100 words:
damage by armour class, health and experience and flanks, range with the height rule and the beam falloff, accuracy,
sight and radar, ground, death blasts, the big weapons); and the map model, `docs/briefs/facts/maps.md` (nine F-map
blocks, about 3,500 words: distances and the grid, ground and terrain types, metal spots and the extractor radius,
where a building may stand, features, sight over coarse ground, wind and tide and the game's standard gravity, where
the sides start, a catalogue of the nineteen maps with recorded games). Each fact document went to a skeptical Opus
reader after it was written (the economy's refuted five statements, the fights' sixteen, each checked by hand before
it was applied; the map model's fifteen, one of which reached back into the fight model: the slope limits are 27, 33 and 54 degrees, the game dividing the angle by 1.5 before the engine multiplies it, which the fight refuter and its check both missed and the engine's own hello shows). The meta catalogue is written (2026-10-22) as five documents under `docs/briefs/meta/`, one per population and one for the
formats, in the rework manifest after the facts: `pool.md` (the Comet duel pool at OS 40+, ten M-pool blocks, about 3,100 words to the model),
`people.md` (the people we have met in lobby games, by person: thebluegecko in duels, alone against BARb, against our
seats on Comet, on water and other maps, and as a team with u6bkep; u6bkep, Irishstud14, Infern8 and Covariance,
thedujin, the lobby 5v5, the Quicksilver people, VAK, the user as the opponent; fourteen blocks, about 4,900
words), `barb.md` (BARb: two code-read F-barb blocks and five observed M-barb sets, about 2,100 words) and `apex.md`
(Apex by version, four blocks, about 900 words) and `formats.md` (what is common across duels, team games of two to
four a side and 8v8 and what each changes, with the first carded 8v8 of eight people against eight SMRTBARb, our eight
against eight BARb, the lobby 5v5, a block on SMRTBARb as the competition in that category and one on the user's own
games against many AIs; six blocks, about 2,000 words). Every block carries `population:`, `dates:` and a `ground:` key naming
what the statements rest on: card (an engine replay, every unit), truth (an arena truth file), sightings (our records,
floors), person (the user's or a player's reading), model (an Opus reader's row of the episode sweep, unchecked by a
person). The games with people have no truth files, so the people's play rests on the eight replayed lobby games'
cards, the Center Command replay, the user's reviews and the sweep's rows; the pool's and the opponents' numbers are
regenerable (`run/reference_set.py`, `run/opponent_set.py` now taking `--ai`, `--version`, `--seats`, `--bonus` and
`--archive`: the BARb hard_aggressive set is 150 games at +0 with the archive, where the 2026-10-17 document had 110 and
a run over `run/matches` alone finds 31). The meta layer adds about 14,300 words after the readers' corrections and the formats document, taking the rework tab
to 46,300: the
per-role manifests (BARb and Apex for the arena, the people for lobby games) and the §10 redaction decision will cut it.
Each meta document went to a skeptical Opus reader after it was written; their findings are applied in the commit after
the first. No derivation or episode document exists yet (the episode sweep's rows are written:
docs/studies/2026-10-22-episode-sweep/).

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
  Apex), each observation with map and date, rationale attached when known. Built as one document per population
  under `docs/briefs/meta/`; each block names its `ground:` (card, truth, sightings, person, model) so a reader knows
  whether a number is every unit of a replay, a truth file, what our seats saw, a person's reading or a model's;
  BARb's code-read rules are `layer: fact` blocks in the same document, its observed sets `layer: meta`.
  **Formats (the user, 2026-10-22):** a section that separates what is common across 1v1, team games (2v2 and up)
  and 8v8 from what each format changes, because 8v8 is its own developed meta and the popular category is eight
  people against eight AIs on a map like All That Glitters: a game our 1v1 proficiencies do not carry into, and the one
  he suspects BARb and Apex are tuned for. Built as `docs/briefs/meta/formats.md`. Its data: the user's client replays
  (copied to `run/data/demos-client/`: two Glitters games of eight people against eight SMRTBARb and one Supreme
  Isthmus game against eight RR03Barb4, community forks of BARb, plus his own games against many BARbs), replayed
  through the engine and carded as the pool's duels are; then the public replay service's team games with bots
  (`preset=team&hasBots=true`, hundreds on Glitters), which need `run/replays/pick.py` to keep games with AIs and
  record the AI kinds, and a team-game aggregate (per side and per player's lane, not per duel side). **Built
  2026-10-22** on the user's word (player games as well, fetch the versions, build it, engine time is fine; watch
  how the seats split responsibility: resource transfers, tech, unit selection): the picker's `--bots`, `--sides`,
  `--any-ending` and `--demo`; `run/replays/startscript.py`; the dump widget's transfers, unit gifts, chat and
  winners; the card's seats, sides, lanes, plant kinds, built lists, transfers and chat; `run/replays/team_set.py`
  (sides by minute, seats by lane, the choreography, what each role built, the chat about roles). The manifest holds
  167 kept 8v8 games on Glitters (81 of people, 45 against BARb, 40 against SMRTBARb, one against RR03Barb4) and the
  user's nine client demos; the batch replays them on eight engines, SMRTBARb first. Found on the way: the two
  SMRTBARb games were played at +65 and +60 resource bonus for the AIs (the start script's handicap), which the
  formats document now says. **The forty SMRTBARb games carded (2026-10-22, later):** the 8v8 and SMRTBARb blocks
  rewritten from them (the roles by lane on both sides, the gifts by hand told from the store's overflow by the
  player list's chat lines, the pressure clocks by lane, the side curves, what each role built, the players' words);
  **The 45 BARb games carded (2026-10-22, evening):** the stock-BARb comparison in the SMRTBARb block and a BARb
  paragraph in the 8v8 block; the people's set follows as the batch lands it (docs/design/2026-09-23-replay-survey.md,
  the 2026-10-22 statuses).
- **Episodes** are an index, not prose: the game, what it showed, its defects, its lens, the derivations it tests.
  Those with a lesson resting on a fixed defect are indexed with the defect and no lesson. The episode sweep of
  2026-10-22 (`docs/studies/2026-10-22-episode-sweep/`: 57 Opus readers, one per game with people, 13.8 M tokens)
  wrote one row a game (`rows/`, `episodes.jsonl`, `index.md`): the root minute, turning points, our mistakes by
  kind (strategy 199, both 187, harness 60), the person's play, the build's defects, the user's in-game comments,
  what lessons and backlog subjects the game bears on, and the reader's confidence (high 10, medium 47). The rows
  are a model's reading until a person checks them; the episode document is written from them in step 7.
- **Lessons** are one document, `docs/briefs/lessons.md`, by subject: each passage one verbatim quotation (or an
  exchange) with its context in a sentence or two, and a block `id: L-<subject>-<n>`, `layer: lesson`, `sources:
  {people, dates, games, clocks, relay}`, `covers:` the archive rows it takes over. The bar for inclusion is the
  user's "very experienced": thebluegecko and Irishstud14 (OS 48 in the duel list) by default; a weaker player's line
  only on the user's word. The corpus: the chat events of every game with people (856 lines in the records, most of
  them ours and the host's), the two transcripts in `docs/knowledge/_inbox/players-chat-2026-09-2{7,8}.md`, the
  relayed words in `docs/knowledge/_inbox/player-notes.md` and in the knowledge base's claims. A chat event carries the
  engine's player id; a lobby game's `script.txt` in the match directory maps ids to names (the archived games of
  2026-09-27 have none, and were attributed by the lines' content, confirmed by the user). The host's seat carries our
  `[WReason]` lines and, without the prefix, the user's own comments (his lobby name is computer_whisperer): those are
  his observations, sources for episodes and derivations, not lessons. The user's rulings of 2026-10-21 on the first
  draft: the game-3 spectator was Irishstud14; the relayed comments he typed from memory were usually thebluegecko's,
  said after the game; the bar (thebluegecko and Irishstud14 first, the lobby-rated 19 to 25 players in a section of
  their own) stands. Every game with people in the records (63 batches) has had its chat read; the rest is greetings,
  banter and the user's comments.
- **Facts** are generated from `units.json` and the map data where possible; the economic model (metal, energy and
  build power as flows, an extractor's payback, the price of a minute), the fight model (range, damage, hit points,
  speed, what wins at equal metal) and the map model (walks, chokes, sight) are written as facts with their grounding,
  since derivations rest on them and the current reference curves are observations, not the model.
- **Size**: ten to fifteen thousand words in a role's context against today's 34,000 to 45,000.

**The people's 81 games carded (2026-10-22, night):** the second 8v8 batch is complete (SMRTBARb 40, BARb 45, people
81, no failures); the formats document's 8v8 block gained "The people against each other (card, 81 games)" and its
closing paragraph the comparison: the same convention with the lanes free to mirror, the early crossing the people
withhold against the AIs and apply against each other, sides level to 8:00 and decided by whose extractors hold from
12:00 and whose seats stay. The 8v8 arc's reading is done; what remains there is tooling (silo and anti-nuke clocks
into card.py, hand-overs without take-overs and pressure by lane into team_set.py) and the user's six coop games,
carded and unread.

**The coverage pass (2026-10-22, night; the user: have some Opus agents skim the previous brief and flag areas of missing coverage in the new system).** Eight Opus readers judged the 492 open passages of the annotation study against the new set by substance (`docs/studies/2026-10-21-brief-annotation/coverage-2026-10-22/`: the README's tallies and themes, `verdicts.jsonl` a row a passage): covered 145, partly 259, missing 27, deferred 36 (episodes awaiting the index), out 25. Of the 286 gap rows 143 are derivations the backlog names and nobody has written (going and the lead beyond lead.md 29, the tier-2 sprint 23, reach and the odds model 16, the nuke 16, matchups 14, the frontier 14, expansion 13, the raid answer 13, water maps 10), 89 are our own games' accounts, 30 facts, 22 meta, one lesson of thebluegecko's. 124 rows note a conflict between the old text and the new, most of them regenerated numbers or arithmetic the new set gets right; four faults inside the new set were checked against the sources and fixed (a3693a6: one builder a unit in the tables, the commander building a construction turret, four winds for three, metal.md's unnamed population). The verdicts are model readings, twelve spot-checked by the main session; the user's read decides what the next derivations are.

## 7. Measurement

The standing brief is the control: the same seeds in the arena and a game with people. (2026-10-23: the control that
exists with the same seeds is the experience brief, the default from 2026-10-03 to the switch; live-edge-series-1 ran on
it with seeds 20-22 at +0 and rework-series-1 the same line on the new set. Both old briefs stay runnable by `--brief`.) Worse-before-better is
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
4b. The lessons document (§6), from the transcripts and the records' chat, for the user's check: he was present for
   most of the conversations and can verify the context. Built 2026-10-21; the user's corrections applied the same
   day.
5. The meta catalogue (written 2026-10-22; see §6's status paragraph).
6. Derivations in backlog order, clusters first, each with its tests and check. **Begun 2026-10-22:** the root
   article, `docs/briefs/derived/metal.md` (D-metal-is-the-game), drafted for the user's read in the agreed shape (the
   heuristic in the players' words, the quantity, the mechanism with premises by id, where it bends, when to depart,
   the tests). **The kill-test run the same evening** (`run/metal_killtest.py`: every recorded game of ours with
   both sides at 12:00, 1,752 games, and the pool's 38 duels past 12:00): the income gap at 12:00 calls the result
   better than the army gap in every population (74% against 67% in the 242 duels the model played, 88% against 79%
   in the 1,495 the old code played, 84% against 79% in the pool's), so the quantity stands as named; the inversions
   are our 38 model-played losses from ahead on income, the unconverted leads of 27 to 34 minutes, and 18 wins on the
   kill from behind. The article's status is checked. **The first branch, 2026-10-22 night:** `docs/briefs/derived/lead.md`
   (D-lead-convert-it), the unconverted lead, with the user's caution built in (the label is a catch-all for harness
   bugs, poor army movement and wrong tempo recognition, so the article defines the state from the curves and names
   the three ways a lead goes unconverted, not seen / not sent / not landed, for the episode index to attach games
   to). Its kill-test ran first (`run/metal_killtest.py windows` and `pool-windows`, the cache now per minute to
   24:00 with each side's army metal in the other's half): the first window of 1.5 times his army and 500 metal
   between 6:00 and 14:00, kept home, wins 33% in the model's 266 duels, 41% in the old code's 1,535 and 41% in the
   pool's duels; sent into his half it wins 71%, 76% and 82%; a window in which his extractor count fell lost no game
   of ours. The test as named (his count's fall calls better than the window's size) came out split, and the article
   narrows the quantity to where the surplus stands, with the fall as the sign it landed. Our 77 model-played losses
   from a window: 40 never sent, 37 not landed. Next: the episode index, since both articles now point at it.
7. The episode index.
8. The role manifests switched to the new set; the standing brief and the experience brief retired to the archive; the
   control kept runnable from the archive for measurement. **Switched 2026-10-23, before step 7, on the user's word
   (the default now while it is built up, to see what the limited content so far does before more is added):**
   `texts::brief` reads the rework manifest for the player and the commander unless `WITHIN_REASON_BRIEF` (the arena's
   `--brief`) names `experience` or `standard`, the two old briefs kept runnable in place as the controls, not yet
   retired to the archive; the role texts' brief passages carry a rework reading; the live edge's store is one a brief
   (`run/data/live-edge/<brief>/`), so the new brief's live edge started empty as §11 says. The player's prompt is
   64,700 words on the rework set against 44,700 on the experience brief, the five unit tables 20,400 of it; which
   domains a role's manifest carries is still the manifest's choice (f9602b1). The first games: rework-series-1
   (docs/experiments.md).
9. Measure (§7); rerun the reading pass.

## 9. Open, for the user

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
- A fifth layer, lessons: the very experienced players' words verbatim with context, high-quality heuristics while we
  are weak, to be supported or replaced by our own reasoning and episodes over time.
- Who writes: the assistant writes most of it; the user is heavily involved as it is built up (small pieces, each
  put in front of him).
- Direct consequences (arithmetic on the facts, no situational judgement) go in the fact documents themselves, as
  the economy document's "What follows directly" (2026-10-22).
- Fine-grained fact blocks may be redacted from the play context once the full brief's size is known (2026-10-22):
  many details (exact placement logic, for one) are too fine for a model in direct play, but the fact reference
  stays for derived reasoning. Nothing is redacted yet; the decision waits on the token count of the whole new brief.
  The mechanism, when wanted: a block status the assembler withholds from every role while the checker, the viewer
  and the derivations' `premises:` still see it (the deprecated status already works this way for the text).
- **The derived section's shape (2026-10-22).** The user's framing: an article is an honest derivation of a surface
  heuristic, in the way "a knight on the rim is dim" derives from the squares a knight attacks; the heuristic is
  reasonable to follow below a certain comprehension level, and the experienced players carry bodies of explanation
  for why they say it. Both too rigorous and too loose expectations fail: false rigour derives what is meta or picks
  premises to reach the lesson it started from, and a proof per rule is too long for play; too loose is the current
  brief, a rule with "because" and one game attached. The agreed shape: the heuristic in the players' words (the line
  the model may follow blindly), the quantity it protects, the mechanism with fact and meta premises named apart, the
  scope (where it bends and when to depart), and the tests with one cheap kill-test and an honest status; an article
  may conclude "not derivable from the facts, stands as meta". The root is thebluegecko's metal-first, with the
  user's dispute inside it (metal spent, not metal; the sign; the horizon; the end; the side's metal in a team).
  **Added 2026-10-22 night (the user):** an article may carry the names of the ways its discipline fails (the lead
  article's not seen / not sent / not landed) for the episode index to attach games to, and the heuristic line need
  not be tied to a player's quote; a quote is used where one exists, the derivation stands on the facts and the meta
  where none does.

## 11. The episode layer: the live edge and worked examples (the user, 2026-10-22 night)

**What the layer is for.** Worked examples and the live edge of information; anything distilled belongs in the fact,
meta or derived sections, so an episode leaves the layer when its content reaches one of them, when its era ends, or
when its diagnosis dies. The layer starts with zero games: the brief's structure and content have changed whole, and
the new bot's first games are its first episodes. The 57 rows of the episode sweep stay a study and a list of
candidates for worked examples.

**The live edge.** The last few games against the same opponent on the same map, shown to the player and the
commander in the snapshot of the next game of the series, in arena games and in games with people alike. Two parts,
side by side, never the words alone: the **report** of each game in plain numbers from the record and the truth file
(result, both sides' extractors, income and army by minute, each side's army in the other's half, extractors lost and
where, his first soldier in our half, his first kill, his advanced factory, air plant, silo and anti-nuke, and ours),
and the **words**: the player's and the commander's own notes and closing sentences from the game, and the player's
debrief for the next player (`run/debrief.py`, kept). No inference by the tool, no categories: the derived articles'
tests run offline on the curves; the live edge shows what happened and what the previous player believed, labelled as
whose it is, so the next player can check a belief against the curves in a glance. The user's reasons: the live edge
is temporary by definition, so a misread has bounded regret and ages out by default, and inside a three-game series a
fast unfiltered response from the player on a live bug or an opponent's habit is worth more than any filtering. The
2026-10-01 rule (no durable memory from the player's in-game notes) holds for the durable layers: nothing moves from
the live edge into a lesson, a worked example or a derivation without naming the defects in force.

**The loop closes without anyone.** When a match ends, the arena (and `run/human_game.sh` after the bot leaves) runs
`run/live_edge.py close <match dir>`: it writes the entry (report and words) into the store `run/data/live-edge/<brief>/` (one a brief since 2026-10-23: a player's words written under one brief are not the next brief's episodes)
(git-ignored working data) and appends a row to its `index.jsonl` (match, date, build, map, opponent key, result).
When a bot session starts, the bot reads the store named by `WITHIN_REASON_LIVE_EDGE`, keys the game by its map and
its enemy seats from the hello (a person's name; an AI's short name, version and profile; the seat's bonus), takes the
entries of the same key within fourteen days, newest first, at most three, and appends them to the player's and the
commander's prompts as a part of their own, after the brief. The key is computed the same way by the tool (from the
record's header and the start script) and by the bot (from the hello). The old debriefs in `docs/briefs/inbox/` are
not loaded; the store starts empty. `arena --no-live-edge` leaves the store unread for an A/B.

**Pruning.** By age (fourteen days) and by count (three) in the bot's selection; the store keeps everything as
evidence. Era boundaries and a dead diagnosis prune the durable layers, not the live edge.

**Built 2026-10-22 night.** `run/live_edge.py` (close | text | key | show), `crates/bot/src/strategist/live_edge.rs`
(the key from the hello, the selection, the part on the player's and the commander's prompts after the brief), the
arena's `--no-live-edge`, the store from `WITHIN_REASON_LIVE_EDGE` or `run/data/live-edge/<brief>`, the post-match step
waited for (`live-edge.log` in the match directory), and `run/human_game.sh`'s close after the bot leaves. Checked:
the tool's key and the bot's agree on a ladder game ("BARb stable hard_aggressive +35%") and on a lobby game
("[gecko]thebluegecko, [gecko]u6bkep"); an entry written from barb-ladder-35-2 with its debrief (48 s, 3,061 words)
into a scratch store was found and logged by the bot of a two-minute hands-only arena match against the same
opponent (live-edge-smoke); the unit tests cover the key and the selection. The real store started empty.

**The check run (live-edge-series-1, 2026-10-22 23:24 to 2026-10-23 01:05; the user: run this in a real game against BARb hard_aggressive).** Three games on the arena target line at +0, one after another in one batch, each bot reading the store the previous close wrote: won 24:26, 20:25 and 22:35 (Armada west, Armada east, Cortex west). Every close ran by itself in about 42 s with the debrief; the entries 3,416 / 3,352 / 2,340 words; game three carried two (6,768 words after its brief). Game two's player cited the last game seven times, in its opening ("the one that won our last game"), at the repeated mistake ("same lesson as the last game: Blitzes lose to Centurions", said after walking into them once more at 11:22), at the Gauntlet ("died to one body at 1.6 to 1"; "the fix for this exact gun was to drive onto it as one body with footwork off") and at a stalled push ("the move alone is what got it moving"), and won four minutes faster with its advanced plant six minutes earlier; game three's, playing Cortex against two Armada entries, cited them twice ("the last two wins were decided by our own tier-2 tanks") and wrote its own lessons. The predict tool's error, army 17% / 30% / 11% and extractors 30% / 17% / 28% by game, moves both ways: the words did not make the predictions worse, and three games cannot show them making them better. The leash stays as built; the entry size is the user's call (docs/experiments.md live-edge-series-1).

**The check.** A three-game series against BARb hard_aggressive on Comet Catcher with the live edge on, the
transcripts of games 2 and 3 read for whether the player cited it, and the predict tool's error on his extractors,
army and advanced factory in games 2 and 3 against game 1. If the words make the predictions worse, the juxtaposition
is not enough and the words need a shorter leash.

**Worked examples** come later, under their derivations: at most two per article, one that went right and one that
went wrong, each under 150 words, none without a person-grade root (the user's review). The defect register (known
operational flaws with an id, a status of open, deferred or fixed in a commit, and the symptom a player sees) guards
the boundary between the live edge and the durable layers; it is not built yet.
