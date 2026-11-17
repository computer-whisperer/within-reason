# The viewer's numbers: a strip of the moment, charts chosen one at a time

2026-11-16. Branch `viewer-stats` (worktree `within-reason.viewer-stats`), merged on the user's word.

## 0. The ask, with his words

"The 3d render helps quite a bit, but there are a lot of other graphs and statistics that are necessary to see how the
game progressed. Given a point in the game, I need to know what income each side has, what the wind speed is, what
units each side has (army value in metal is a decent heuristic), and more. For reference, take a look at what the
statistics menu in the real game engine shows. I think we should re-engineer how the webui presents all of this
information, since it doesn't spend it's screen real-estate very effectively right now."

After my first proposal (a narrower map, a grid of charts in a wider side pane), with his screenshot of how he
watches (a close camera on a fight, the side pane narrow, the bottom charts open): "The main point is that the 3d
scene deserves primary focus, while the various graphs can be individually drilled down to or selected. The numbers
for the given in-game moment at the top right are important, but incomplete in most scenarios (missing the enemy side
in some cases, max metal capacity always)."

On the second proposal: "The top strip looks like a good concept. Let's proceed with step 1."

## 1. What was wrong (measured on 1794780805-human at 1920x937)

- The timeline took 371 px, 40 % of the window: ten lanes of ticks and five charts 32 px tall, ours alone
  ("theirs: no truth file"), no value at the playhead.
- The eight numbers of the moment were ours alone. A game with a record of ours never read its tape's numbers; the
  tape's two-sided table showed only for a game without a record.
- Metal's capacity was in the record (`m[3]`) and never shown.

## 2. The numbers: one model, two sources

`viewer/stats.js` (`WS`) answers one model whatever the source: sides (ally teams), each with its teams (players);
a series a metric, a value a second, for a team or a side (the sum of its teams: every metric is additive).

- **A game with a tape**: the tape's, both sides, worked out in `crates/tape/src/stats.rs` (so `tape stats
  <game.tape>` prints for a script or a review agent the numbers the page shows) and handed to the page by the wasm
  reader (`wr_stats`). From the tables alone, no slice read: the `TEAM` table (stored, capacity, income, spend, pull,
  excess, sent, received a second; the engine's running statistics) and the `UNIT` table with the types (units,
  extractors, build power, count and metal value by class, metal value killed and lost).
- **A game without one**: our record's (stored, capacity, income, spend; extractors, army, builders) and the truth
  file's for the other side (extractors, army) when there is one. What a source does not have is absent, not zero.

Classes in a format 5 tape are an approximation of BAR's (its team stats gadget's `bucketOf`): factories, builders
(commanders and construction turrets with them), economy, defence, utility, air, army. The tape's types do not say
`unitgroup` or the move class, so ships count as army and the strategic buildings as defence until step 2.

## 3. The page

- **The strip** across the top, in place of the title bar and the eight-number block: a line a side, every stored
  amount with its capacity. A click on a side's name unfolds a line a player. A click on a number shows or hides
  that number's chart.
- **The scene** keeps the pane. `s` folds the side pane away.
- **The timeline**: rows chosen one at a time and remembered (`localStorage`): lanes and charts. A chart is about
  84 px tall, both sides as lines, each line's value at the playhead beside it; a side unfolded in the strip is its
  players' lines. One lane of metal destroyed (one side up, the other down, ten seconds a bar) replaces the four
  loss and kill lanes; the hands' lanes start hidden.
- **A Stats tab** in the side pane: the whole table at the playhead (rows the metrics by group, a column a side or
  a player), each row a switch for its chart; and each side's units by type at the playhead.

## 4. Not in step 1 (step 2: tape format 6, every tape made again)

Wind speed over time (not recorded anywhere); BAR's exact classes (`unitgroup`, move class, the long-range flag);
conversion in use and reclaim by team; a summary a second of what lies in the slices (build power in use, idle
builders and plants, the order rate), so a chart does not read the whole tape. Step 3: BAR's marks on the charts
(first tier 2, fusion, a commander lost, a strike).

## 5. Status (2026-11-16, step 1 built on the branch, not yet seen by the user)

Built: `crates/tape/src/stats.rs` with `tape stats` and `wr_stats`; `viewer/stats.js` (the model from a tape or from
a record, the strip, the Stats tab); `viewer/timeline.js` (the rows; the old timeline, its five fixed charts, the
eight-number block and the tape-only side table are deleted from `app.js` and `panels.js`, `WR.totalsAt` with them).

Measured on 1794780805-human in a 1920x937 window: the scene 1436x560 with the trade lane, the two turn lanes and two
charts open (511 before, with five charts 32 px tall); 1904 wide and about 730 tall with the side pane and the charts
folded. `tape stats` on that tape: 0.1 s. Its numbers at 20:00 agree with what the page showed before for our side
(metal +352.3, army worth 45,375) and add the other side's (+291.1, 20,339).

Tested: the crate's unit tests (the unit lives, an ownership change, a kill, the table's rows; the classes);
`viewer/test/scene.js` in Chromium and Firefox on a game with a record and a tape (1794767778-human) and on a tape
alone, with the strip's checks (a line a side with no number absent, a click showing a chart and a second taking it
away, a side unfolding); `viewer/test/browser.js` on a one-seat flagship game without a tape
(1794709644-apex-v0511-ladder-20-1: the strip from the record, the Stats tab, the side pane folding). Not tested: a
game being played (`viewer/test/live.js` needs an arena game).

Found on the way, not from this work (both fail the same on main): `browser.js` on the 8v8
1794724277-vents-barb-8v8-1 stops at "the hands panel shows another gate than the one sought" (the gate of an
eight-seat log is not in before the test looks); `scene.js` failed "as side 1 saw it" on the lobby games because it
required the first side to see some of the other's units halfway through, which is the game's affair: it now checks
the ghosts against the tape's own sight masks.

Seen by the user 2026-11-16 ("Good work. One nit: the top bar thrashes left and right with the changing width of the
various statistic blocks."): every number's cell of the strip now has its width from the start, in digits of its
longest text (a number is five characters at most), so no column moves under the playhead. Measured through 121
moments of 1794780805-human: 68 different layouts before, one after, in Chromium and Firefox; `scene.js` checks it
(one layout through the game, no text wider than its cell). The strip is about 1,520 px wide for two sides.

## 6. Step 2: tape format 6 (on his "Let's fix that before continuing with step 2", 2026-11-16)

What the capture widget writes more (raw version 6), all of it once a second in the `res` row or once a unit type:

- **The wind**: `wind`, the strength of `Spring.GetWind()`, as the game's top bar shows it.
- **A team's row, ten more numbers**: the converters' energy in use and their capacity (the game's `mmUse` and
  `mmCapacity` team rules params); the metal and energy reclaimed so far (the game's `teamStatsReclaimedMetal` and
  `...Energy`, -1 when the game's version does not publish them: the metric is then absent, not zero); the build
  power of the finished builders and how much of it is in use (`buildSpeed * GetUnitCurrentBuildPower`, summed at the
  widget's look at every builder, which it already makes for the audit's `W` rows); the constructors and how many of
  them have no order, the plants and how many have an empty line (the game's own definitions: a constructor is a
  builder that moves and is no commander); the orders given to the team's units that second.
- **A unit type**: `group` (the game's `unitgroup`), `move_class`, `lrpc`, `commander`, `metal_make`.

`tape::stats` then sorts a type into the game's own nine classes (its team stats gadget's `bucketOf`, rule for
rule): army, air, sea, defence, strategic, factories, builders, economy, utility. New metrics: the wind (the game's,
not a team's), conversion in use and capacity, build power in use, metal and energy reclaimed, constructors and
plants with the idle of each, orders a second.

The page: the wind now beside its range in the strip, with a chart; the share of build power in use beside the build
power; sea and strategic columns in the strip of a game that has any; the new metrics in the Stats tab (an Activity
view) and as charts.

Format 6 alone, as format 5 was: the packer reads raw version 6 only, the reader format 6 only; a tape of format 5
is made again from its demo. Until the branch is merged the tapes of main's games stay format 5 for main's page: the
branch's viewer (port 1237) serves copies of the seven lobby games of 2026-11-15 taped again in the worktree's own
`run/matches`.

### Step 2 as built (2026-11-16 evening, on the branch)

Built as section 6 says: `tape_dump.lua` writes raw version 6, the packer and the reader are format 6 alone,
`tape::stats` sorts by the game's own nine classes and has the new metrics, the page shows them (the wind now and its
chart, the share of build power in use, a sea and a strategic column in a game that has any, an Activity view).
The strip dropped its builders and units columns for them (both are in the Stats tab).

The worktree stands on its own engine and data (reflink copies of main's `run/engines` and `run/data`) and has the
seven lobby games of 2026-11-15 as copies in its `run/matches`, each taped again in format 6 and checked the same as
its raw stream. The branch's viewer on port 1237 serves those; main's tapes stay format 5 for main's page until the
merge, after which they are made again there (`run/tape.py <match dir>`, each has its demo).

Read off the new tapes at 20:00: Gecko Isle wind 12.5 (of the map's range), the people's side 8,355 of 14,140 build
power in use, 1 of 52 constructors idle, 2 orders a second against our 30; Ceasefire the people's converters taking
2,093 of 3,840. The game's version of that day counts what is reclaimed (6.7k to 8.8k metal a side by then).
A scout plane is utility, not air: the game's rule puts its `util` group before the question whether it flies.

Tested: the crate's unit tests (the nine classes rule for rule, the new columns, the wind, an absent metric);
`scene.js` in Chromium and Firefox on a format 6 tape with a record and on one alone; `browser.js` on a game
without a tape. Not tested: the arena's own capture of a game as it is played (the same widget and packer; `cargo
check -p arena` passes), a game with ships or a nuke silo on the page (the classes are unit-tested, the two columns
were not seen), `viewer/test/live.js`.
