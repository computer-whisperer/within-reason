# The waste audit

A review of a game reads its fights. The audit reads its ledgers: every second of every unit of both sides, from
the engine's own state, so that what a player sees without thinking (metal sitting in the store, a builder walking
across the map, an army strung out along its road) is a number with a clock and a place beside it. It was written
2026-11-06 for the distilled bot, whose games leave no match record (`bots/README.md`), after review sweeps of the
bot's games kept missing its largest faults; it reads any game the engine can replay, the flagship's and people's
included, and prints every table for both sides so that the opponent is the yardstick.

## Making the dump

    run/replay_match.py <match dir>/demos/<the demo>.sdfz --label <name> --audit
    run/audit.py run/matches/<time>-replay-<name>

The replay runs the demo in the headless engine at about twice game speed (a 20-minute game: ten minutes of wall).
Beside the match records for every team (`record-format.md`) the widget `run/replay-template/LuaUI/Widgets/audit_dump.lua`
writes `audit.jsonl`, about 2 MB a game minute. The unit ids and frames are the game's: a bot's own log lines in the
arena match's `engine.log` (`[distilled ai=N] f=...`) line up with the dump by frame.

## `audit.jsonl`

JSON Lines; `t` names the row.

| `t` | When | Fields |
|---|---|---|
| `header` | first | `map` {name, width, height}, `seats` [{team, ally, side, name, ai}] (the names read "UNKNOWN" in a replay; the reader takes them from the match's `_script.txt`), `spots` [[x, z, metal]] as the game's spot finder publishes them, `sample_frames` |
| `def` | before a type's first row | `d`: the unit type: `id`, `name`, `human`, `metal`, `energy`, `power` (the engine's: metal + energy / 60 unless the game says otherwise), `build_time`, `build_speed`, `build_distance`, `speed`, `health`, `reach` (its longest weapon), `weapons`, `building`, `factory`, `air`, `extracts`, `energy_make`, `energy_upkeep` (negative for a solar), `wind`, `tidal`, `converter`, `metal_store`, `energy_store`, `can_assist`, `can_reclaim`, `tech`, `sight`, `radar`, `xsize`, `zsize`, `options` (what it builds) |
| `s` | every 30 frames | `sides` [{`team`, `m` and `e`: [current, storage, pull, income, expense, excess, sent, received] (the engine's `GetTeamResources`: pull is what the builders asked for, expense what they got, excess what overflowed a full store that second), `st`: the team's cumulative statistics (metal/energy produced, used, excess; damage dealt and received; units produced, died, killed; updated every 16 s)}]; `u`: every unit, [id, def, team, x, z, health %, build progress %, queue length, the head order's command id (negative: a build of that type), its place x, z (0, 0 when it has none), its target id (0 when none), the unit it is building (0), the share of its build power in use % (the engine's nano-piece tally: the share of the last frames it actually added build power, so a stall shows), a factory's build queue length, seen by the other side (1 line of sight, 2 radar), experience]; `hits`: [[victim id, damage taken since the last sample]] |
| `ev` | as they happen | `k` `created` (+`by` builder), `finished`, `destroyed` (+`by`, `by_d`, `by_team`, `progress` %), `given` |
| `cmd` | every order given | `u`, `team`, `c` (command id), `x`, `z` or `target`, `shift` |
| `wrecks` | every 450 frames | `w`: [[feature id, name, x, z, metal]] for every feature holding metal |
| `over` | the game's end | `winners` (ally teams) |

Damage dealt is not in the engine's call-ins for a user widget (the game's widget handler passes `UnitDamaged`
without the attacker, and a user widget may not replace that call-in), so it is read from each unit's experience:
the engine adds 0.1 x the game's `experienceMult` x the share of the victim's health x the victim's power over the
attacker's. The reader scales the result so that the sum of what everybody dealt is the sum of what everybody took
(the raw sum was 0.58 of it on the first game): a unit's `dealt` is its share of the enemy's losses in metal-worth.

## A team game: `run/audit_side.py <dir>/audit.jsonl [our teams]`

`run/audit.py` reads one seat against one opponent. For a game of many seats `run/audit_side.py` prints the side-level
truth from the same dump (written 2026-11-10 for the lobby 10v8 on Erebos Lakes): every ally team's armed mobile metal
by minute and by thirds of the map with each seat's beside it, our seats' metal ledger with the engine's `excess`,
`sent` and `received` (a full store's overflow goes to the side: t16's 260,600 `sent` there was its overflow, 14,300
`excess` what was lost), orders by side, the people's losses by killer team, what the silos killed, units given between
teams, the last minutes' unit counts, the engine's team statistics and the winners. The thirds are of x.

When the people call lag, read the demo itself before the audit: `run/demo_net.py <demo.sdfz>` (no replay needed).
It prints the seats with their countries and the AIs each client hosts. It prints the stream every client receives,
KB/s by minute, with each AI host's orders apart. And it prints each client's distance behind the server (the
server's ping) beside its own sim load. A client falling behind on an idle sim is starved by its link
(K-engine-a-far-client-falls-behind-when-the-stream-outgrows-its-link): the 10v8's far clients fell behind from about
30 KB/s.

## The tables (`run/audit.py <dir> [--sections a,b] [--top N] [--team N]`)

| Section | What it answers |
|---|---|
| `summary` | one row a side: metal made, overflowed, the mean bank, energy thrown away and stalled, the share of the standing build power that was used, builder time idle, waiting and walking, plant time not building, army made and lost, damage dealt, soldiers that died having dealt under a tenth of their cost, losses outside the army |
| `economy` | by minute: the store, income, pull, spend and overflow of both resources, seconds the metal store was full or the energy store empty |
| `floats` | the spans the metal store stood over half (and over 400), and what every class of builder did meanwhile |
| `power` | every builder-second by role: building (and at what share of its power), starved, walking, waiting with an order, guarding idle, a plant blocked, idle; build power delivered against standing by five-minute blocks |
| `builders` | each of our constructors: its time by state, the path it walked, its jobs, the path per job, and the longest idle, waiting and walking runs with their clocks and places |
| `plants` | each plant: its share building, blocked and idle, what it made minute by minute, its longest stands with the bank at their start |
| `frames` | frame time with nobody building, frames lost unfinished and the metal in them, frames left alone longest |
| `army` | by minute: armed metal of both sides, his turrets, the ratio with and without them, where ours stood by thirds of the way to him, the shares standing, moving and fighting, the share that has never fought; the first minute of a 2x and a 3x lead against the game's end |
| `front` | armed metal by tenths of the way from our start to his, every 30 s: a strung-out army is a row of small numbers |
| `cohesion` | the army outside our third every 30 s: how many, how many with the foremost unit, his armed metal at that unit; how alone our soldiers died |
| `types` | per unit type of both sides: made, lost, the damage it dealt over its cost, deaths for under a tenth of the cost, mean life |
| `trips` | the time and path from the plant to a soldier's first shot, soldiers that never fought, reversals along the axis, trips out and back with no shot |
| `fights` | deaths of both sides clustered in time and place, ranked by our net loss, with the armed metal of both sides at the place ten seconds before |
| `raids` | everything of ours lost outside the army: per killer what it took, how long it had been in our sight and out of his third first, whether it lived; one by one with the nearest soldier and turret of ours |
| `spots` | spots held by minute, and for each spot of our half when it was first taken, how often lost, how long it stood free |
| `wrecks` | metal lying on the ground by thirds, every two minutes |
| `orders` | orders by kind for both sides, the most-ordered units, build orders that never became a frame |
| `sight` | the share of each side's units the other had in sight or on radar by minute; the first building of his seen |

Drill-downs: `--unit ID` (a unit's life as runs of one state, with its order, target, what it built and at what
power), `--window M:SS M:SS [X Z R]` (events and the units of both sides in a span and a place), `--map M:SS`
(both sides on a character map).

## Reading it

Start at `summary`, then the table behind the largest number. A finding is a row with a clock: take it to the
bot's own log at that frame (or the flagship's record) for the cause, and to `--unit` for what one unit did. The
tables are complete where a checklist is not: a fault nobody thought to look for still has to sit in some
builder's seconds, some soldier's path or some store's level. Limits: one sample a second (a unit that dies inside
its first second has no row); the damage dealt is experience, so a kill by a death blast or a unit lost the same
second it fired is not credited; "walking" is an order with movement, which a builder dodging or queued behind
another also is.
