# The spending line and the pros' sinks (2026-10-17)

Decisions, not findings; the findings are `docs/studies/2026-10-17-spending-line.md` and the claims
K-player-the-draws-reached-the-hands-not-the-player and K-pro-comet-first-construction-turret-before-the-second-factory.

## The user's rulings (2026-10-17)

1. Watching chase-player-1's early floats: "the BP shortage should have been completely predictable from just the
   scaling income and the units we were trying to build." A line copying the pros' timing would say when, not why.
2. "Let's figure out a clean way to give the player both things: the commentary on the pros' construction turrets and
   second factory is cheap, and the basic numbers needed to project balance expectations forward."
3. Both halves existed for other readers (the hands' draws, the commander's build power, the json table's store
   quartiles, the cards' clocks) and the player's report dropped them: route what exists from the same sources.

## What is built

- **The sheet** (`run/pro_baseline.py`, `run/reference_set.py`): a "first construction turret begun" milestone row;
  under "More factories and tier 2", what the side had when it began its first construction turret and its second
  factory, and how many put the turret first; the by-minute table and the json carry `factories` and
  `construction_turrets` begun by the minute, counted from the cards' clocks (the curves carry none).
- **The reference line** (`strategist/reference.rs`): three more cells when the table has the keys, metal stored,
  factories begun, construction turrets begun; ours counted with their frames (`Counts::factories_begun`,
  `construction_turrets_begun`). The line's caveat "what they did, not a target" stands.
- **The eco line** (`strategist/report.rs`): both stores with income and usage and the hands' `flow_words`
  (stalling, running down with the seconds to empty, banking, in balance), one source with the picture.
- **The spending line** (H-PLAYER-SPENDING-LINE): the picture's `production_draws` (now `ProductionDraw`, with the
  actor's own power and the power helping a factory: builders guarding it, construction turrets told to guard it)
  summed at full speed, each factory alone and with its helpers, the builders grouped by what they build, those
  helping and those walking or idle, and the seat's build power by kind. A factory's entry in the report keeps its
  draw line, which now ends "alone; each 100 build power helping it adds E and M; P is helping it now".
- **The prompt**: one sentence beside "in the first minutes the plant's metal is the limit": from about the fourth
  minute the income outgrows what one plant and the builders can take, the `spending` line says by how much, and
  the three build powers (turret 200, constructor 90, commander 300). The report's line list names `spending` and
  `reference`.

## Left out on purpose

- The hands' "short by 80, which is 4 solars" conversion stays off the player's line; the brief gives the
  generators' output and that division is the player's.
- The hands' energy budget still sums the actors' own draws (no helpers): Jev's inputs are unchanged by this.
- No doctrine on what to build when; the pros' rows and the numbers are observations.

## How it is judged

`run/floor.py` `e0%` and `mfull%` (chase-player-2: 4.1 and 5.6), the bank against the reference line's quartiles by
minute, and whether the transcripts quote the spending line when the bank grows.
