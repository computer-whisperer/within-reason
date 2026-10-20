# The unit glossary as a sheet

`units.csv` is the unit glossary laid out for a spreadsheet: one row per unit of the three factions that a
commander's build tree reaches (571 on game source 54199a0d, the checkout `run/unit_stats.py` reads), with everything
the repo knows about the unit in the first thirty columns and five empty columns for experienced players to fill.
`run/glossary_sheet.py` writes it; regenerate after the glossary changes and re-import, or upload once and let the
sheet drift from the file (the import matches rows on the `unit` column, so added rows and reordering do not matter).

## Where the columns come from

| Columns | Source |
|---|---|
| `unit` (the game's internal name, the key; **do not edit**), `name`, `faction`, `tier`, `class`, `made by`, the numbers through `flags` | `crates/bot/data/units.json`, generated from the game's unit files by `run/unit_stats.py` (`docs/design/2026-09-22-full-roster.md`). `range` and `dps` are over the weapons that can hit the ground; `against aircraft` says what the unit does to aircraft. |
| `death blast`, `self-destruct blast` | the engine's own numbers, read from the newest match records' headers under `run/matches` (`docs/harness/record-format.md`, `unit_defs`). Radius in elmos, damage per unit within it. Blank for a unit that dies quietly. |
| `on the builders' usual menu` | `crates/bot/src/brain/roster.rs` `USUAL_*`: the buildings Jev is offered by default when a builder is idle; the player can widen it. A blank here is one answer to "why does the bot never build X". |
| `sim duels: clearly beats / loses to` | `crates/bot/data/matchups.csv`, the combat simulator's equal-metal duels of 2026-09-19 (23 tier-1 units; the margin in brackets, +1 wiped them untouched). One situation only: two blobs of one type charging each other. |
| `pros' body (typical largest / largest seen)` | `docs/studies/2026-09-28-body-sizes.md`: over 40 high-OS Comet Catcher replays, the median game's largest body of this type and the largest body in the pool. |
| `our gloss`, `our prose` | the hand-written words in `units.json`. The gloss is the clause on a menu line; the prose is what the Opus player gets from its `units` tool. Written by model agents from the unit files and the wiki, corrected by play; **this is the text we want opinions on.** |

## What we ask the players

The five columns at the right, per unit, in their own words:

- **verdict on our text** — `ok`, `wrong` (say what in the next columns), or `missing` (true but leaves out the thing
  that matters).
- **what it is for, when to build it, when not** — the role in a real game, the minute or the trigger, the map or
  matchup where it shines or is a waste.
- **beats / loses to** — at the numbers people actually field, not one on one.
- **tips, micro, mistakes** — spacing, stance, fire-at-will or hold, what new players get wrong with it.
- **who** — a handle, so that a claim can be followed up and so human-sourced lessons rank above model findings
  in the brief (`docs/knowledge/README.md`).

Half a sentence is fine. A blank row is "no opinion". Rows a player disagrees with another on are the valuable ones:
add a second line in the cell rather than overwriting.

## Putting it on Google Sheets

File > Import > Upload `units.csv`, "Replace spreadsheet", separator "Comma", no conversion of text to numbers needed
either way. Then View > Freeze > 1 row, select the two prose columns and the five player columns and set
Format > Wrapping > Wrap, and Data > Create a filter so that players can filter by `faction` and `tier`. Share with
edit rights; the `unit` column is the key the import reads, so protect it (Data > Protect sheets and ranges) if the
sheet is open to many hands.

## Back into the repo

Download the sheet as CSV into `docs/knowledge/_inbox/` with the date in its name. The players' columns are unreviewed
claims from people, the kind `docs/knowledge/_inbox/player-notes.md` holds: each becomes a `reported` claim or a
correction to the entry's prose, and the prose is edited in `units.json` by hand (the generator keeps every
hand-written field). An import that carries the players' words into the glossary as a field of their own, for the
`units` tool to serve beside our prose, is the next step once the first sheet comes back; it needs the user's word,
as every change to what the player is told does.
