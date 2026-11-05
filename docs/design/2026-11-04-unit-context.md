# Unit context: what the player is told about units, and when

Status: ratified to build 2026-11-04 (the user: "Write up the design doc, then build it"). Built the same day; see §7.

## 1. The problem

The player's system prompt carries five generated unit tables, every unit of all three factions, 571 rows, 47,850
tokens: 24% of the prompt (203,000 tokens after the trim of 2026-11-04). The first report of each session adds
the roster, one glossary line per unit of our faction, 198 lines and 16,700 tokens, re-sent at every 40-turn
session reset. The prose paragraphs (32,000 to 42,700 tokens a faction) sit behind the `units` tool, which the
player called 0 times in 2,065 turns over 32 sessions with people (it called `situation` 3 times and `map`,
`overview`, `plan`, `search` never): nothing is fetched on demand, so what the prompt omits is gone for the game.

What a game needs is much smaller. Over sixteen recent games (lobby, Apex, BARb ladder) a game touches a median of
45 distinct unit types: about 20 of ours built, 30 of his seen. Fifteen of those first appear after 10:00 and none
after 20:00 at the median; the 3v10 lobby game touched 173. The hand-written articles name 155 of the 571 units
(77 Armada, 57 Cortex, 21 Legion). The report names his units by internal name only ("armflash 1"), so a table row
is what lets the player read a sighting.

The user's rulings (2026-11-04): a faction nobody in the game can reach is content for no game and goes; the gloss
should be merged into the rows rather than sent as a separate roster; a system should decide per unit whether to
show it, with "a pretty generous filter to start", and early sessions should not describe the bulk of the enemy's
late-game units; whether the prose must stand ahead of the derived articles is a later question, easier once the
base system controls what context is introduced.

## 2. The two speeds

The player's context has two places where unit words can enter, and both exist already:

1. **The session prompt**, assembled at every session start: the first, and every 40 turns, and whenever the role
   text on disk changes (`strategist/mod.rs` `drive`, `TURNS_PER_SESSION`). The prompt function is re-run every
   turn to detect an edit by digest, so what it generates from game state must be frozen for the session: a
   snapshot is taken when a session spawns and the prompt reads the snapshot, never the live state.
2. **The report**, which works by deltas: a line the first time something is true. A unit type that becomes
   relevant between session starts (his, first seen; ours, first buildable) gets its gloss line in the report that
   first shows it, once, and is in the next session's tables.

The bot keeps the set of types *introduced*, by either route, so a line is never given twice and nothing the
player has met is ever dropped from a later prompt.

## 3. The level of each unit

A function of (unit, game state) to a level:

| Level | What the prompt carries |
|---|---|
| omitted | nothing; the report introduces it at first sight |
| row | its row in the faction's table, with the gloss clause as the last column |

A third level, prose (the row and its paragraph in a glossary section after the tables), is the hook for the "prose
ahead of the articles" question; it is added to the enum when a rule needs it, not before.

The state is what the bot holds at assembly time: each seat's faction from the hello (ours, allies', enemies'; a
seat on Random counts as all three until its first unit says), the map's share of cells under water, the game
minute, what our standing builders and factories can build now, the types of his seen so far, and the introduced set.

The rules, in order, first match wins (`unit_context.rs` `level`):

1. Faction reachable by no seat in the game: omitted.
2. Introduced already (seen of his, built or buildable of ours at any report): row.
3. Our faction (any seat of ours): row, every tier. The player writes build lists for the future, and a tier-2 plan
   at 6:00 needs the tier-2 names before any trigger fires.
4. Sea domain on a map with under 3% of its cells under water: omitted (ships cannot be built; his amphibious and
   hover units are land-domain rows and stay).
5. Enemy or ally faction, tier 1, land, economy or structures: row.
6. Enemy or ally faction, air domain: row from minute 8, or once an aircraft plant of either side stands or is
   seen; omitted before.
7. Enemy or ally faction, tier 2: row from minute 12, or once a tier-2 factory of his is seen (the briefing's
   `enemy_factories`), or any tier-2 unit of his has been seen; omitted before.
8. Enemy or ally faction, tier 3: row from minute 20 or once a gantry is seen; omitted before.

The minutes are generous by the pool's clocks (tier 2 at 11:48 median for the 22 of 80 pool sides that reached an
air plant; advanced factories in 9 of 29 BARb games at 19:32 or later) and by the record (none of the 45 types
first appears after 20:00 at the median). They are the knob, in one table, and the measurement in §6 says
whether they are too tight.

A flag `WITHIN_REASON_UNIT_REFERENCED=1` adds a ninth rule ahead of 4 to 8: a unit the brief's prose names (scanned
from the assembled brief text at assembly, display or internal name) is a row regardless of tier and minute. Off
by default, so the plain filter is measured first; it is the switch for the "articles read without context"
question.

## 4. What the prompt carries

In place of the five documents `docs/briefs/facts/units-*.md`, the manifest line

    generated:units <- docs/briefs/facts/units-economy.md docs/briefs/facts/units-land.md ...

names a generated part and the documents it stands for (the brief scripts read the sources for their meta blocks,
the assembler calls the generator). The generator writes the same five tables from the glossary (`units.json`), the
same columns as `run/fact_units.py` plus the gloss column, one section per faction present and domain, each
heading saying how many of the faction's units in that domain are shown ("Cortex: land (31 of 52 units)"). A
short preamble says what is held back and how it arrives: the enemy's tier-2, air and sea rows come when the game
reaches them, a type first seen or first buildable comes in the report with one line, and `units` has the prose
for any name.

The commander reads the same manifest and so the same part.

`bot --print-prompt player` with no hello shows the pre-game state of a Random seat against unknown sides: all three
factions at the tier-1 level, for measuring.

## 5. What the report carries

The roster goes. In its place, every report (not only the session's first) carries

    new units this report (one line each, the first time a type is seen of his or buildable of ours; its row is in your next session's tables): ...

for the types not yet introduced, each line the glossary's `line()` (name, internal name, made by, tier, metal,
gloss, air words). The session's first report keeps a short `can build now:` line of internal names, since the
[now] mark of the old roster was the one thing the tables cannot say.

The seen types of his come from the briefing: `Field::enemy_types_seen`, the definitions of `enemy_buildings`,
`enemy_soldiers` and this tick's enemies in sight, merged across our seats. Allies' units are not sightings and are
not introduced; their faction's tier-1 rows stand from the start (rule 5).

## 6. Measurement

- Unit tokens per session: the generated part's size is logged at every spawn (`[strategist] unit context: N
  rows of F factions, M chars`), and the transcript's first-report size falls by the roster.
- Raw names: a count, after the game, of internal names the player met in a report before their line or row
  (`run/unit_context_audit.py`, to write when the first game is read): should be 0.
- Too tight for our side: `queue` or `produce` steps naming a unit the prompt omitted. Rule 3 makes this
  impossible for our own faction; the audit checks it for allied names.
- The first game under it: a Comet duel against Apex, compared with rove2-apex-1 on the first report's size and the
  cost per turn.

## 7. Built 2026-11-04

`crates/bot/src/strategist/unit_context.rs` (the state, the rules, the generated part, the report lines),
`texts.rs assemble_parts` (the `generated:` line), `Launch::spawn` (the snapshot), `report.rs` (the roster
replaced), `briefing.rs`/`shared.rs`/`seats.rs` (`enemy_types_seen`), `glossary.rs` (`entries()`, the faction),
`run/brief_check.py` and `run/brief_coverage.py` (the manifest line), the role text and the `units` tool
description, H-PLAYER-UNIT-CONTEXT in `docs/heuristics.md`, K-player-nothing-is-fetched-on-demand in
`docs/knowledge/jev.md`. Not installed to the client; not yet measured in a game.

Measured on the assembled part (`WITHIN_REASON_UNIT_SIDES=ours:his bot --print-prompt player`, the knob that
stands in for a hello when measuring; Opus token counts):

| State | Rows | Tokens | Before |
|---|---|---|---|
| no hello (three factions his, minute 0) | 229 | 28,800 | 47,850 tables |
| Armada against Cortex, minute 0 | 271 | 33,900 | 47,850 tables + 16,700 roster in the first session |
| Armada against Random, minute 0 | 347 | | |

The three-faction duel's first session reads 34,000 tokens of unit content with the gloss in place of 64,550, and
later sessions about the same as before (the rows the game reaches, with the gloss) instead of the full tables plus
the roster again.
