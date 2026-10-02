# The scouting glance (2026-10-01)

Agreed with the user in outline on 2026-10-01 ("If we don't have an equivalent of a human glancing at the map with weak
memory of where we explicitly scouted, then that's the first thing to fix"; "Agreed, build it once the games end").
Information for the player and a clause on the hands' arms; no control rule, nothing decided by code.

## Why

player-33-hard, 5:52: Opus sent the main army (18 soldiers, 1,585 metal) on "the raid on his southern strip" (spot_79,
75, 70, 77, 72, 65), the south-east corner. His base was at G1/H1, his eleven extractors in rows 1 to 3, nothing of his
south of row 3 all game. The group reached H8 at 7:54 having killed one Pawn; five of our extractors died in the
minute before 7:58. Two beliefs were wrong and the report contradicted neither: "his base is at H3" (the Rover had
found his commander at an outpost there; no factory was ever seen; the three spots around his real base were never in
sight), and "he has extractors in the south" (every one of those spots had been in the roving Rover's sight by 3:13,
empty).

What the bot had: `scout.rs` records the last frame each metal spot was within sight of a unit of ours (rovers
included). What the player was shown: the spots never seen (ten names) and "known to hold N". Nothing said "seen, and
empty", nothing carried an age, and a roving group says what it found, never where it looked and found nothing. The
hands' picture carries more (`never_looked`, `looked_long_ago`, each place's "free metal spot when last in sight, N
ago") but the player reads it only by calling `situation`.

## The law

Every turn the player is told, for every cell of the map, how long ago units of ours last saw it, how much of it they
saw, and what of his stood there then.

## The record: ground, not spots

`scout.rs` gains a tile grid over the whole map (tiles 128 elmos square: 64 by 48 on Comet Catcher), each tile with the
last frame its centre was within the sight radius of a unit of ours (`sight_of`, the simulator's table; buildings count,
radar does not). Surveyed twice a second with the spots. The spots' record stays: the rove goals and the hands' picture
read it.

Limits, said in the code and not in the report: sight is a circle, the engine's line of sight is cut by terrain (flat
maps lose nothing); another seat's units are not counted (as for the spots today).

## The roll-up: the named cells

The report speaks in the 8 by 8 cells A1 to H8 (`World::grid`); a cell is 8 by 6 tiles here. Per cell:

- **how much**: the share of its tiles ever seen: "all of it" (90% or more), "most of it" (50%), "part of it" (15%),
  "a corner of it" (under 15%), "never seen" (none).
- **when**: the age of the look, taken as the age of the median seen tile; "in sight now" when that is under ten
  seconds.
- **what of his**: his buildings remembered in the cell (`enemy_buildings`) by internal name with counts, and his
  commander's last sighting when it was there. Soldiers are not listed: they move, and the report's party lines carry
  them.

## The block in the report

A `scouting` block in every turn report, replacing the tail of the `to win` line ("Metal spots never within sight of a
unit of ours: ..."), whose content it carries:

```
scouting (each cell of the map by when units of ours last saw it; an old look says what was built there then, nothing about his army now):
  his: H3, all of it seen 4:25 ago: armcom (his commander), 1 armrad, 1 armllt, 2 armmex
  nothing of his when seen 3 to 6 min ago: G4-G8, H4-H8 (most of each)
  nothing of his, in sight now: A5-A8, B5-B8, C6-C8, D7
  never seen: G1 (spot_5, spot_10), H1, H2 (spot_12), E1-F2 ...
  no factory of his has been seen: it stands where we have not looked; the cells of his box seen least: G1, H1 (never), H2 (a corner of it, 3:02 ago)
```

- One line per cell that holds something of his; the other cells grouped by age band (in sight now, under 1 min, 1 to 3,
  3 to 6, over 6, never) and by how much, each group written as runs down a column ("G4-G8").
- Never-seen and long-unseen cells name their metal spots, since spots are what the player orders by.
- The factory line only while no factory of his stands in the record.

`Score::never_looked` and its report words go (the block says it); `enemy_spots` ("known to hold N") stays in the score
line.

## The hands' side

A group's arm that goes to a spot holding nothing of his in the record says so in its words, with the age: "nothing of
his was there when last seen, 3:10 ago", or "never seen". The arms that go to a spot with his buildings already say
"seen N ago".

## Checks

- Unit tests: tiles marked by a unit's sight, the cell roll-up, the grouping words.
- The recorded 5:52 turn of player-33 asked again with the block added (built from the record), against the turn as it
  was: does the raid order change. This needs a way to re-ask a recorded turn; if that is not cheap, the next game is
  the check and the ledger says so.
