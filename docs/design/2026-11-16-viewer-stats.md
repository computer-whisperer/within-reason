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
