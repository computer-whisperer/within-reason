# Can Jev follow a stateful route written in prose?

**Question.** The station list (`standing` `station: "spot_49 spot_46 ..."`) is a mechanic of the hands: code marks
each stop reached, code picks the next, and Jev sees only a per-second yes or no on the default. The user's
assumption was the other design: Jev reads the route as prose and uses the group's current orders as the state. Before
replacing the mechanic, the kill test: at the recorded moments where a route was being walked, does Jev name the next
stop, and does the bot's two-stage ask then play it?

**Material.** `run/matches/1790513490-player-9-posing/00` (the game lost 36:32 for never finding the base): the five
arrivals of the 6:00 raid loop (6:25, 6:30, 6:36, 6:46, 6:59; six stops, twelve Blitzes), the scout Rover's arrival at
spot_42 at 3:53 on its eight-stop route to G1, and its fork at 3:58 with the enemy commander in sight, where the
recorded hands picked "falls back to our base" at 0.62 with four stops left. The recorded gate calls (the state with
the packet and rules in force re-inserted, the group's questions) are put to Jev again; the worlds choice is rebuilt
from the flagged answers the way `plan::compose` does. Four repeats per moment. `run/route_ab.py`; the data
`docs/studies/data/route-ab-2026-09-28.jsonl` (140 rows); the cost about 300 Jev calls.

**Variants.**

- A, as recorded: the route is the station list; the next leg is inside "nothing changes" (the base world).
- B: the route in the packet's prose ("one raid loop through spot_49, then spot_46, ..., and back to spot_64, in that
  order, advancing ..."; for the scout "scouts spot_72, then ... to find the enemy base. It is a scout: it runs from
  what can catch it and otherwise keeps to its route; being seen is its job"), no station rule, the menu offering an
  advance to every remaining stop, and `recent` carrying one line per stop reached ("6:24 group_A reached spot_49").
- C: as B without the `recent` lines.
- D: as B, and the group's own `doing` says "reached spot_49 at 6:24 and holds there, its stop done; the instructions'
  next stop is spot_46, not yet ordered".
- E: as B, and the "nothing changes" world's line carries the cost, the composer's convention for an idle lab:
  "group_A holds at spot_49, a stop of its route it has reached, with the route's next stop spot_46 not ordered."

**Results** (four repeats; "continued" is the pick that starts the next leg, or for A the default playing; at 3:58 the
group was mid-walk, so "continued" there is any pick but the retreat).

| variant | next stop is Jev's top-rated move | next stop at or above the 0.5 flag | route continued at the six arrivals | the 3:58 fork kept the route |
|---|---|---|---|---|
| A recorded | never asked | never asked | 24/24 by construction | 2/4 |
| B prose + reached lines | 24/24 | 21/24 | 2/24 | 3/4 |
| C prose only | 21/24 | 0/24 | 0/24 | 1/4 |
| D as B + "stop done" in the entry | 24/24 | 4/24 | 1/24 | 4/4 |
| E as B + the cost in world 1 | 24/24 | 23/24 | 24/24 | 4/4 |

The next stop's noul: B 0.49-0.69 against 0.3 for the other walks; C 0.29-0.45; D 0.36-0.49 with the "should it
change" noul up to 0.6-0.7; E 0.52-0.66. E's picks of the next leg: 0.56-0.75; its 3:58 pick against the retreat:
0.82-0.90.

**Findings.**

1. Jev tracks the state of a prose route from the picture. With a line per stop reached it names the next stop as
   the best move at every arrival; without the lines it still finds it 21 of 24 times but rates it under the flag.
   The state it needs is "which stops are done", as words.
2. What stopped the leg in B and D was the machinery, not the judgement: the flag at 0.5 sits where the leg's noul
   sits, and the worlds choice reads a group "holding for 1 s since it arrived" as a course, so "nothing changes"
   won 22 of 24 even when the leg was flagged. Saying "stop done, next is X" in the group's entry raised "should it
   change" and lowered the leg's own noul: the sentence "is this what group_A should do now, rather than <holding,
   next stop not yet ordered>" reads to Jev as already answered.
3. One sentence in the nothing-changes line, the cost of standing at a reached stop, turned every arrival and the
   fork. The convention exists for labs (K-jev-hold-words-carry-the-cost); it was missing for a group at a stop.
4. The scout's prose role ("runs from what can catch it, otherwise keeps to its route, being seen is its job") is
   what held the 3:58 fork: 3-4 of 4 in B, D and E against 2 of 4 for the recorded mechanic and 1 of 4 in C.

**Caveats.** Seven moments of one game, four repeats, one Jev model (jev-1.13.0). E forced the leg into the worlds
once at 6:30 when the gate had not flagged it. The replay rebuilds the worlds from the group's questions alone; the
real call carries the other actors' questions too, and the world count and their lines differ. The arrival as an
event asking at once was not tested here (the recorded rows are the asks the bot made).

**What it feeds.** `docs/design/2026-09-28-routes-in-prose.md`; the claim K-jev-follows-a-prose-route-from-the-picture
in `docs/knowledge/jev.md`.
