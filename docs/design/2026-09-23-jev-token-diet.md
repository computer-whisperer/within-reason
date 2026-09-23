# The Jev token diet

Written 2026-09-23 after shell-1 and started-1 (hard_aggressive, 30-34 min, 1,350-1,370 calls, 24-25M input tokens,
about a dollar a game, two calls past the 64k window). The user: "review for ways to improve our jev token usage ...
look for wasted tokens", then "let's work on those improvements. My own thought is to also look for anywhere we start
asking the same question multiple times in a row with no input data change."

## Measured (started-1; shell-1 within two points)

| block | share of the characters sent | note |
|---|---|---|
| group `where` + `where_scout` | 23 % | two lists of all 55 places (3.1k chars each) with every group question; `where_scout` used twice a game, `where` in 8 % of group plays |
| state.places | 21 % | 54 entries at 168 chars, every call; about one a call named in the instructions |
| builder questions | 18 % | 24 options of 200-950 chars; the economy phrases repeated in each option are 21 % of it |
| state.actors | 15 % | 12 points of it for actors not asked in the call |
| group `do`, `whom`, `how_many` | 9 % | the decision |
| state.enemy | 4.5 % | |
| rules | 3.3 % | 1,469 chars fixed |
| quiet reviews | 17 % of tokens | 223 calls asking only holding groups with nothing near |
| unchanged inputs | 4-6 % of tokens | 262-295 asks (mostly the plant) whose question, entry and instructions differ from the previous only in numbers; the answer was the same 90-98 % of the time |

## Decisions

1. **`where_scout` goes.** When `scout` is chosen the hands pick the place: the enemy's marked or remembered base if
   not seen for three minutes, else the nearest never-looked spot inside the enemy's box, else the longest-unseen spot.
2. **`where` lists the places in play**, not the map: home, the marks, the passages, the enemy places, `shelling`,
   every place named in the instructions, the group's own destination, spots with enemies at our extractors, and the
   nearest spots within `WHERE_REACH` (2,000 elmos) of the group, at most `WHERE_MOST` (14) in all.
3. **`state.places` is the union of the asked actors' places in play** plus every non-spot place and every spot named
   in the actors' entries, the enemy block or the instructions. The entries' words are unchanged.
4. **`state.actors` carries full entries for the asked actors** and for actors named in the instructions; the rest are
   one line each (`at`, `doing`, `units`), enough for the join options and the picture of the whole.
5. **Builder options say the economy once.** The `.do` question's `instructions` carry "Our metal now ... Our energy
   now ..." once; each building option keeps the unit's line, where it goes, what it draws and for how long, and the
   facts that bear on it (generators standing, factories, under way). The per-option economy sentences go.
6. **A quiet holding group is reviewed every 15 s**, not 5: holding, no enemy within twice the alarm reach, not hit
   since the last call. Events still bring it forward at once (H-HANDS-SCHEDULE).
7. **An unchanged question is answered from its last answer.** For a factory: when the question text, the actor's
   entry and the instructions differ from the previous ask only in numbers, Jev is not asked; the choice is drawn
   from the previous answer's probabilities (so a two-to-one mix stays a mix) for up to `REPLAY_FRAMES` (60 s), then
   a real ask. Groups and builders are not replayed: their numbers (a distance, a share) are the change.

Rules text, the enemy block and the global questions stay as they are.

## Levels (the user, 2026-09-23: "build this to be able to take multiple effort levels. Bulk training games are where
the costs are going to start adding up ... high-level matches against humans may warrant escalation")

One setting, `WITHIN_REASON_HANDS_EFFORT` = `lean` | `normal` | `full`, read into `Diet` at start; the arena's
`--hands-effort` (default `lean`: the bulk games) and `run/human_game.sh`'s third argument (default `normal`) set it.
Decisions 1 and 5 hold at every level (pure waste). The rest by level:

| knob | lean | normal | full |
|---|---|---|---|
| `where` places: reach / most | 1,500 / 10 | 2,500 / 14 | every place |
| `state.places` | in play only | in play + every spot we hold | every place |
| `state.actors` not asked | one line | one line | full entries |
| quiet holding group review | 15 s | 10 s | 5 s |
| factory replay on unchanged inputs | 60 s | 30 s | off |
| question budget (characters) | 60k | 80k | 80k |

The record's header and the jev log's header carry the level, so a game's cost is read against it.

## Expected

About 55-60 % of the characters: a 30-minute hard_aggressive game from about $1.05 to under $0.50, and no call near
the window. Checked by `scratchpad/jev_waste.py` on the next hard_aggressive game against started-1.

## Status

Built 2026-09-23 13:00 (`pianist/diet.rs` and the sites it names); the check game at lean follows the Lua series.

## Checked (2026-09-23)

| game | level | result | calls | tokens a call (median) | $ a minute |
|---|---|---|---|---|---|
| started-1 | (before) | Loss 30.6 | 1,370 | 15.6k | 0.035 |
| diet-1 | normal | Win 18.9 | 637 | 10.3k | 0.016 |
| diet-2 | lean | Win 15.5 | 530 | 9.25k | 0.015 |

The ask latency after a loss stayed at 1.2-1.3 s. Left at lean: builder questions 34 % (24 options of 300-460
characters), the places block 22 % (36 places a call), group questions 14 % (`where` lists 31 places, the
instructions naming many). Next cuts, if wanted: the places block from the asked *groups'* reach and the spots the
instructions and the asked entries name (not the enemy block's); a shorter unit line per builder option.
