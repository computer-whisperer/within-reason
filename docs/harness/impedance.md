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

## The faults, as of 2026-10-16 (nothing fixed yet; each is the user's call)

Code, exact and cheap: the hunt leash measured from the hunters' own centre (a detachment sent at a raider 900
away ends "after 0 s"); a re-sent list's first step "taken as done" on a build ordered but not started; the turret
step skipped while the spot's extractor "was refused lately" though it stands; `signal` not in `orders`'
`batchable()`; `queue` accepting steps the builder cannot make or that need a place. The report: no metal store cap
on the eco line; a dead enemy building kept in a cell in sight; moho upgrades on the extractors-lost line;
`no_site` dropped silently; no builder type on an advanced constructor's entry; the decode's readings shown for
groups only. The tools: `remove` wants handles the picture gives only in the exit-lane line; `transfer` refuses the
whole list for one stale name; a player's `mark` removal does not touch the side's mark of that name; `produce
"new"` gives no name to write the packet with. Jev and the decode: the packet weights the pick and does not command
(K-jev-the-packet-weights-the-pick-and-does-not-command); an ordered constructor paragraph reads as no extractors
and a negated place as a destination (K-jev-the-decode-reads-an-ordered-constructor-paragraph-as-no-extractors);
the forbidden mark acts at 0.70 and on detachments only; the commander's standing rule has no instructions clause.
What no lever carries (the study's §4): where a builder stands, a route's order, a hunt bounded by a place, targets
inside a fight, converging, splitting by unit type, joining in groups, routes that avoid reach, a forced re-ask,
rove with a destination, cross-seat joins.
