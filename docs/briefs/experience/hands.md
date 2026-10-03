## How your hands work

Your hands (Jev) pick one move at a time for every unit and group from a menu, every second, by weighing each move
against your packet. They cannot count, plan or remember beyond what the picture and your packet say.

- **What a group's moves are aimed at:** `go` and `fight to` at every place your packet, a list or a mark names;
  `attack`, `shell` and `send` at every enemy party in sight; `attack` and `shell` at his buildings wherever the
  picture knows them; `join` and `follow` at every other group, `follow` at every builder.
- **The hands have no destination of their own.** There is no walk home, no falling back to where a group last
  held, no sweep of unseen spots. Where a group goes when it should not fight is a place its paragraph names; where
  it scouts is a place its paragraph names, or `rove`.
- **A way through several places is prose:** "group_A: spot_49, then spot_46, spot_40, spot_55, spot_58, back to
  spot_64, in that order, advancing; his buildings in sight it kills when they are undefended; from a party it does
  not outweigh it walks to spot_64". The group's entry says which named places it has reached (`reached`) and what
  it has met since (`met`). Your orders land about five seconds after the picture you read.
- **A group is a body:** its entry says the front, the tail, who has arrived and who is on the way from the plant.
  The odds are priced on the part in the fight. Every move's words carry the fight the group is in: the odds, what
  it lost in the last 30 s, whether the party can follow.
- **The enemy section** lists his buildings by place with their guards, the parties that left sight with where and
  when, his biggest party known, and the first of each tier-2 or air type of his the moment it is seen.
- **`produce` lists** are how a count is kept: the hands made three constructors from the words "one constructor
  first" (human-7); `armcv:1` in the list made one.
- **Defence is yours to write.** Nothing in the code answers a raider at a building on its own. Told only "engage",
  the hands sent a whole body after one scout car, never caught it, and another raider killed a lab at home
  (realtime-2).
- A scout's sentence that held under the enemy commander's eyes: "it is a scout: it runs from what can catch it and
  otherwise keeps to its route; being seen is its job".

**`rove`.** `lane` with `{"group_R": "rove"}` hands a group of fast units (Rovers, Ticks; name a plant's output into
it with `produce`) to code that runs each of them ten times a second. It drives to look at what we know least (his
start box and base first, then spots nobody has seen, then the stalest), attacks what it finds unguarded (a
constructor, an extractor, a radar with nothing armed in reach), and does not stand inside the reach of anything
that can shoot it. Your hands do not move a roving group. The group's entry says what each rover is doing and what
it has found; `"on"` takes the group back. It is scouting footwork: at a base with Pawns patrolling, a roving Rover
stepped off from each and killed nothing (player-20, player-21). In player-9, before `rove`, three scouts went at
his base under the hands and none arrived; in player-10 three Rovers on `rove` had his lab in the picture at 3:45
and came back alive.

**Several seats.** When the report's `seats:` line says you command more than one seat, every per-seat name carries
its seat's tag: `commander_t1`, `group_A_t2`, `party_3_t2`, `passage_1_t2`; the other seats' starts are `home_t2`
and so on, and `home` in an actor's paragraph is that actor's own seat's start. Constructors and factories are
unique by number; `spot_N` and your marks are the same for every seat. A list, a `produce` entry, a standing
paragraph or a removal goes to the seat that owns the name; a paragraph headed `constructors:` or `commander:`
applies to every seat's. An enemy party has one name for the whole side. Each seat has its own economy and its own
map picture, and sees the other seats' groups under `allies`. `transfer` moves metal and energy between seats (the
receiver's store caps it) and gives units, groups, builders or plants to a seat. A commander can be reclaimed by
another builder for its metal (`remove`); a starting store holds 1,000 of its 2,700. People in the game know a seat
by its lobby colour, which the `sides:` and `seats:` lines give: "our purple seat". The bot puts `[WReason] ` in
front of every line you say.
