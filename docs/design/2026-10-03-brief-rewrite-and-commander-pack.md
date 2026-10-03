# The brief as game experience, and the commander's pack

Status: draft 1, 2026-10-03, in discussion with the user. Nothing built. The old brief stays as it is until this is
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

## 3. The player's brief: what games look like

Generated per game for this map, this start, these opponents; no other map's section. Parts:

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
aimed at one model's habits, the harness history ("until 2026-09-28 the list's ..."), the other maps.

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

1. The generated reference tables and timelines for Comet 1v1 (read-only on existing data), shown to the user.
2. The player's brief for Comet 1v1 from them, beside the old brief, chosen by a flag so the ledger stays comparable.
3. The pack's tables (economy arithmetic, the matrix, the timing distributions).
4. The commander seat itself: its own design, after 1-3 are read.

## 6. Open, the user's

- Whether the stripped brief is played before the commander exists (it will likely play worse with one model: the
  tuning is load-bearing today), or only with it.
- Whether the players' advice stays as quoted words in the player's brief, moves to the commander, or both.
- Whether the first version is Comet 1v1 only.
- Whether BARb's own curves may be shown to the player as the opponent's description.
