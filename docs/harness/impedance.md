# The player-to-hands interface: instruments and the known faults

How to measure whether the harness did what the player ordered and whether the player was told what it needed, and
the faults the first tally found (`docs/studies/2026-10-16-opus-hands-impedance.md`, 23 games, 2026-10-16).

## Instruments

- `run/impedance_sweep.py MATCH [--seat N | --commander] [--all] [--from m:ss --to m:ss] [--counts]`: every turn's
  orders in compact form (instruct, queue, produce, direct, signal, mark, remove, fire, transfer, lane, wait) beside
  the refused tool results and, from the next report, the hands' lines that say an order did not happen as written
  (the engine refused a site, a blocked site, a skipped step, a hunt ended on its leash or with no hunters, a frame
  abandoned, a follow ended, a step waiting). `--counts` prints the denominators for every seat and the commander:
  turns, orders by tool, refusals by tool, flagged lines by kind, session replacements, landing delays. `--all`
  prints every result line and every hands' line of the window. Transcripts only; it never reads the hands' log.
- `run/impedance_tally.py MATCH... [--items] [--cost ...] [--category ...] [--no-lever] [--json OUT]`: sums the
  `impedance.json` files reviewers wrote into match directories (the brief's shape: categories A1-A6, B1-B6, C;
  items with clock, seat, wanted, happened, evidence, cost, confidence, fix class; the no-lever asks), one row a
  game, the totals, the items by cost, the no-lever list.
- The reviewers' brief and the categories: `docs/studies/data/impedance-2026-10-16/BRIEF.md`. A review of a new game
  runs the same brief on one Opus agent and drops its two files into the match directory; the tally then reads it
  beside the others.

## The faults, as of 2026-10-16

**Fixed on the user's word the same day** (the cheap code faults and the report lines): the hunt leash is measured
for each hunter from where it stood when the hunt began (`Hunt::anchors`); a re-sent list whose first step is the
build the builder was ordered and has not begun keeps the step at the front until the frame is up
(`Pianist::list_caught_up`, "the step waits for the frame"); a turret step at a spot whose extractor was refused
lately goes on when an extractor of ours is there; `signal` batches in `orders`; `queue` refuses a building the
builder cannot make and a turret, radar or jammer with no place (`Field::builders`, the words each builder can take);
the eco line reads `metal 1794/2100` with `FULL: what comes in beyond the store is wasted` at 98%; a remembered
enemy building whose place a unit of ours sees (within four fifths of its sight) with nothing there is forgotten
and said ("his corlab at E4 is gone: the place is in sight and nothing stands there"); an extractor of ours
replaced by our own advanced extractor is not a loss; a build with no site is said ("no site could be found for a
armavp where it was asked, so the order is dropped: name another place or a spot for it"); a constructor's entry
carries its type (`constructor_54 (armacv): ...`). Not yet measured in a game.

**Also fixed 2026-10-16, after the thirty-paragraph measurement** (the study §3.2b): the extractor move is never
fused on the decode's reading (every other build type still is; the players name those units and the readings hold),
and the player's report shows each builder's extractor reading (`reads: extractors 0.23 (how the packet reads for
it; this one shuts no move)`) beside the groups' place readings.

**Built 2026-10-16 night, the first batch of levers for the asks nothing carried** (the study §4; the user: "That
first batch you outlined sounds good, proceed"): `produce` groups a factory's output per unit type (`"group":
"by_type"`) and has its newcomers walk to their group so many together (`"join_in": N`, waiting beside the plant
out of its lane); the whole-group attack returns to the menu when the decode reads every detachment as forbidden;
the whole-group attack and the shelling of a party of aircraft carry the forbidden question (the decode's `Air`
reading); a `reclaim` list step takes a grid cell or a place (the wreck field there) and runs until the wrecks are
gone; an id step naming a building the builder cannot make is its help on that building, waiting for its starter;
`remove` takes `"<type> at <place or cell>"` for a reclaim. Each has unit tests; none has been in a game: the next
games' sweeps and reviews measure them (`run/impedance_sweep.py`, the brief's §4 asks).

Still open, each the user's call: the decode's other readings (places, joins, follows, detachments) stay fuses at
their bars. The tools: `transfer` refuses the whole list for one stale name; a player's `mark` removal does not
touch the side's mark of that name; `produce "new"` gives no name to write the packet with. Jev and the decode: the
packet weights the pick and does not command (K-jev-the-packet-weights-the-pick-and-does-not-command); an ordered
constructor paragraph reads as no extractors and a negated place as a destination
(K-jev-the-decode-reads-a-constructor-paragraph-by-its-words-not-its-meaning); the forbidden mark acts at 0.70; the
commander's standing rule has no instructions clause. What no lever carries yet (the study's §4): where a builder
stands (the exit lane), a route's order, a hunt bounded by a place, targets inside a fight, converging, routes that
avoid reach, a forced re-ask or a direct order, rove with a destination, cross-seat joins, a contact named, a
repeating `produce` entry.
