# The tier-1 duel tables of 2026-09-19

The first set: every pairing of 23 tier-1 land units at equal metal (1,200), no lane, both sides attack-moving
(docs/harness/duels.md), tight (ranks 56 apart, eight duels a pairing, batch `duel-t1-matrix-2`) and wide (100
apart, four a pairing, `duel-t1-matrix-wide` with the scouts' pairings from `scouts-wide`), on the duel harness of
0a83e87+ (`docs/experiments.md`). The batches are gone and recorded neither game nor map; from the day's files: the
game was `Beyond All Reason test-31357-b06bb1a` (fetched into the arena's pool at 13:07 that day and the newest
there until the next afternoon; the arena then played the newest fetched), the engine 2026.07.04, the map the harness's default, Quicksilver Remake 1.24, where
the site rectangle of the time (1,440 x 640 whatever the army) left the rear ranks of large armies off the checked
ground.

Written before `run/matchup_table.py` existed. Flown again on 2026-11-19 (`docs/data/duels-2026-11-19`) with a
control arm on this set's game and engine, which reproduced it: correlation 0.992 tight and 0.994 wide over the 252
pairings of two units, no pairing changing sides (`docs/knowledge/units.md`, "Duel results").
