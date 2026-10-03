# The brief as game experience, and the commander's pack

Status: draft 2, 2026-10-03, in discussion with the user. The map sheet (§3a) is approved to start; nothing else is
built. The old brief stays as it is until this is
ratified.

The user, 2026-10-03: "my main goal is to enrich game experience with what typical games against BARbs and against
humans look like, typical composition hints, etc. We should aim to strip out all of the open-loop tuning messages,
relying on the third seat to handle that." And for the third seat: "equip that model with the information it needs to
call meta-level decisions from scratch rather than relying on typical build planning. It needs to be able to reason
through why and when to go t2, what units are worth using, and what balance to strike between economy and units."

## 1. What is measured

- The player's start context is 27,500 words: the prompt (`crates/bot/src/strategist/player.md`, 6,900: tools, packets,
  turns) and the brief (`docs/briefs/player.md`, 20,600 words under one heading, all four map sections whatever the
  map; the Comet section alone is 450 lines).
- The brief is case-heavy (377 lines cite a game or a clock) but most cases are our own mistakes written as
  correctives under an imperative headline; "never" stands 94 times. The balance of economy and army is nowhere stated
  as numbers: it emerges from passages pulling against each other, each sized to how far Opus had drifted.
- Sonnet 5.5 in the player seat on the same seed and code as player-52 (Opus medium, won 25:26): low lost at 20:05,
  medium at 27:20, in opposite directions at 8:00 (21 extractors with one turret queued and no scout; 9 extractors,
  one plant, four constructors). Turns 3-4 s at the median against 10.7 s. Packets to Jev 540-620 characters against
  2,650; notes 1 against 42 (`docs/experiments.md`, sonnet-low-1, sonnet-medium-1).
- The data for a reference already exists: `run/pro_baseline.py` gives, for a map, start and faction, the pool's
  quartiles and the winners' median by minute (extractors, constructors, income, army metal, soldiers) and the opening
  milestones (factory 0:34, first soldier in his half 1:40, light turrets by 8:00 median 10, second factory 8:11; 41
  Armada sides of 30 Comet games at OS 40+). 317 carded games in the manifest; 47 of our games with people; the truth
  file of every arena game holds BARb's real curve.

## 2. The three seats

| Seat | Model | Cadence | Reads | Writes |
|---|---|---|---|---|
| commander | Opus, high effort | minutes, and on events; never holds the world | the pack (§4), both sides' curves against the reference, the scouting picture, what its last targets came to | targets in numbers and the plan in a few lines, per seat |
| player (one a seat) | Sonnet | seconds | the brief (§3), the report, the commander's targets | packets, lists, production: as today |
| hands | Jev | every second | the packet, the menus | picks: as today |

The closed loop that the brief's tuning prose stood in for moves to the commander: it sees the player's curve beside
the reference and corrects in numbers ("18 extractors and 9 constructors by 8:00, 5 turrets on the outer spots"), so
how hard a model leans on a sentence stops mattering.

## 3a. General between maps (the user, 2026-10-03)

"We have a lot of experience on comet catcher, but very little elsewhere. We want to extract as much general
knowledge as possible from the comet catcher specific experiences, and one of the best ways to do that is probably to
just keep the information in the brief ... both sonnet and opus can happily crunch through a 100k opening context ...
We do need to make sure the llm agents are fully aware of the map details that inform those decisions though, so when
they see a new map they can react accordingly."

The pool: Comet 68 carded duels (254 listed), Great Divide 4 carded of 40 duels and 20 team games, Full Metal Plate 2,
Gasbag Grabens 1; our games with people about twenty on Comet and a handful each on Quicksilver, Great Divide and Cape
Violet. So the experience is Comet's, and the reader generalises it on a new map from the facts. For that:

1. **The map sheet**: one structure for every map, computed by code from the map, never written by hand. Size; the
   walk between the starts in seconds for each movement class; the spots within 30, 60 and 90 seconds' walk of each
   start, the total and the spot value; the wind range (and tidal); the share of the ground each class can cross;
   the passages and their widths; water. The sheets of the known maps stand side by side in the brief with the
   current map's, so the difference is on the page.
2. **Every table states its derivation beside its number** ("first soldier in his half 1:40: the factory 0:34, the
   first soldier 0:55, 45 s of walking"), so the number scales with the sheet.
3. **Every case carries its map conditions** as a tag from the sheet.
4. **Logistics stay per map**: spot ids and named places for the map being played only.
5. **A scaling rule is called general only when it has been tested**: card the Great Divide duels, predict their
   milestones from Comet's rules and Great Divide's sheet, compare with what the pool did. What fails stays labelled
   Comet.

The size is not the limit (the prompt is cached; the turn time at 100k is to be measured, not assumed); attention is:
at 27,000 words under one heading Sonnet already dropped the conditions round each headline, so the rewrite is
sections, tables and one case format.

## 3. The player's brief: what games look like

All the experience on every map, with the current map's sheet and logistics. Parts:

1. **The reference game**, as tables from the pool: the milestones and the by-minute quartiles with the winners'
   median; the report then carries where we stand against it each turn.
2. **Whole games as timelines**, minute lines of what was built and where the army stood: two or three pool games
   from this start (a win, a loss, one decided early), and for the opponent at hand two of ours (a win, a loss).
3. **What this opponent's games look like.** Against BARb at this tier, from the truth files: his curve by minute, when
   his first raiders arrive and of what, his tier-2 clock and what comes out of it. Against people: the same from the
   games with people and the pool. Description of the opponent, never our target (the ruling of 2026-10-03: pressure
   and pace are learnt from the pool and the players, not from BARb's curves).
4. **Composition**, as data: what the pool's factories made by phase (shares by minute), what BARb makes, and the
   simulator's cost-for-cost outcomes for the pairs that occur (`combatsim`), beside the unit table.
5. **Cases**, kept as observation and outcome with the clock ("comet-4: 22 Blitzes onto four light turrets: lost 19
   for 2"), without the imperative headline.
6. **The players' own words**, quoted and attributed, as what people say about the game.
7. **How the hands work and what the tools do**: operational, stays.

Removed: every sentence whose job is to push harder or softer than the reader would otherwise go, the correctives
aimed at one model's habits, the harness history ("until 2026-09-28 the list's ..."), the other maps' spot lists.

## 4. The commander's pack: deciding from first principles

Not build orders. What it needs to work out why and when:

1. **The economy as arithmetic.** Cost, build time and return of every income source (extractor, advanced extractor,
   solar, advanced solar, converter, fusion), as payback seconds at this map's spot value; what build power costs and
   what it turns a bank into; what an advanced lab and its first constructor cost in metal, energy and seconds, and
   the income at which the pool started theirs.
2. **Units as value.** The simulator's cost-for-cost matrix across tiers (what 1,000 metal of each kills of each),
   ranges and speeds, what out-ranges static defence, what catches raiders.
3. **Timing as distributions.** When the pool started tier 2, at what income and army, and how those games went; the
   same for second factories; BARb's tier-2 clock by profile and what our games looked like after it.
4. **The opponent now.** The tempo model's estimate of his economy and army from what we saw, with its error.
5. **The live sheet.** Both curves against the reference, the territory, the scouting age, the last targets and
   whether they were met.

## 5. Order of work

1. The map sheet (§3a), then the generated reference tables and timelines for Comet 1v1 with their derivations, and
   the Great Divide check (read-only on existing data), shown to the user.
2. The player's brief for Comet 1v1 from them, beside the old brief, chosen by a flag so the ledger stays comparable.
3. The pack's tables (economy arithmetic, the matrix, the timing distributions).
4. The commander seat itself: its own design, after 1-3 are read.

## 6. Rulings (the user, 2026-10-03 evening)

- **The brief is tested on its own before the third seat exists.** "We do still want to build this for
  non-commander games since the concept hasn't been fully proven out, so the general brief improvements should be
  tested independently before we add the third tier." So the rewritten brief is played by one model (player-N games)
  against the old brief on the same seed, and the commander comes after.
- **Tier 2 is a decision, not a plan.** "We honestly don't have the experience for it yet ... that transition really
  needs to be a player or commander decision when to make and what to do with it. The later in the game we go the
  less we can rely on cookie-cutter game plans, which is what drives me towards adding the third slot." The pool
  agrees that it cannot teach it: one Armada side of 41 and three Cortex sides of 39 started an advanced factory
  (duels at OS 40+ on Comet last 12 to 15 minutes at the median). So the brief gives the arithmetic and the cases we
  have, labelled as few, and no timing rule; the user is looking for game modes where tier 2 is the right call.
- **The arena starts where the pool does.** `--starts 1420,3550:7290,3600` (B5 against H5, the pool's commonest
  pair) in place of the game's placement of an AI at an end of its strip, which is a quarter further apart; the
  ledger's player games before player-54 are from the corner start.
- **The players' quoted advice stays**, quoted and attributed, in the brief.
- **BARb's own curves are shown** as the description of this opponent (not as our target: the ruling on pressure and
  pace stands).
- **One brief for every situation, with nearly everything in it always** (2026-10-03 night, during human-15): "I
  lean towards having most of the information there in all cases rather than swapping out cards, since we need to
  maximize how much advantage out of each piece of experience." So the two briefs behind a flag are scaffolding for
  the comparison only. The target is one document holding every map's sheet and reference game, every opponent's
  description, the team material and all the cases, whatever the game; what changes per game is a short opening
  that says which of it is this game's own (the map, the seats, the opponent) and what is carried from elsewhere,
  and the report's live lines. Nothing is swapped out for relevance.

