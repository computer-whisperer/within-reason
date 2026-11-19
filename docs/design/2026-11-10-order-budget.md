# An order budget per seat (2026-11-10)

## Why

The people's lag in our lobby games is the server's stream outgrowing the far clients' links
(K-engine-a-far-client-falls-behind-when-the-stream-outgrows-its-link). Every unit order an AI gives is its own message
to every client, about 33 bytes for a move. In the 10v8 on Erebos Lakes (`1794293029-human`) the clients in HK, AU,
GR and RO fell behind from 21:00, once the stream passed about 30 KB/s. The rise was our two seats' moves: 5 KB/s at
19:00 and 21 at 27:00, while Apex went from 13 to 19. Our two seats executed the most orders of the eighteen. From
20:00, 56 % of our moves put the point under 16 elmos from that unit's last one, and 60 % came under half a second
after it (K-group-the-march-re-sent-every-member-every-tick).

## The standard (the user, 2026-11-10)

"Apex uses much of the same engine as BARb, so our standard should be to be no worse than that. I would aim for some
kind of per-seat regulation, since the burden went up with unit count ... a game with relatively fewer units has
more to gain from careful unit control. I also think skipping near-sends is a reasonable thing regardless within a
certain tolerance." On my proposal: "Agreed, go with the Apex average and moves only".

- **The Apex average.** In the 10v8 the six Apex seats sent 3.1-3.7 KB/s each late on, about 100 messages a second a
  seat. Eight AI seats at that rate are 26-28 KB/s, the far clients' ceiling, so the number is a cap, not a target.
- **Per seat.** One budget a seat whatever its unit count. Fifty units can have two orders a second each; six hundred
  get about ten a unit-minute, under BARb's fifteen.
- **Moves only.** Only a move order is ever held back; every other order goes out when given.

## Rule 1: a slot order is not re-sent for a point that moved less than the tolerance (crates/micro)

`take_slot` (the lane's march) sends a member its point again when the old order no longer serves. It now also
needs the point to have moved more than the formation's **near tolerance** since the last order: a quarter of the
company's pitch across, at least 16 (`form::near_tolerance`). That is 16 for the first tier (pitch 64), 21 for Bulls
and Starlights (84), 27 for Marauders (108) and 39 for Thors (156): under a quarter of the gap between two slots,
which nobody watching a formation sees.

Kept: a member the engine has stopped short, standing with no order a second after its last and further than the
tolerance from its point, is sent again even to a point that has not moved (K-engine-a-blocked-move-ends-short). A
member in a relaxed pass is sent once, as before. The first order of a claim goes out as before.

What it does to the rate: a member standing at its point is sent again each time the point has walked a tolerance on,
so its rate is the pace over the tolerance (48 a second over 16 is three a second). The rule removes the re-sends
under that, the jitter of a point that moves a few elmos a tick. A seat of hundreds of marching members can still be
over the Apex average: the budget, rule 2, takes it from there.

## Rule 2: each seat's orders are held to the Apex average (crates/ai-host)

Every unit order of a seat, the flagship's and every bot's, goes through `Instance::apply` (`crates/ai-host/src/lib.rs`).
There an **order budget** (`crates/ai-host/src/budget.rs`) keeps a bucket of 100 orders a second (`RATE`), at most a
second's worth deep (`DEPTH`).

- **Not a move:** every unit order goes out when given and takes one from the bucket (the bucket may go below empty;
  from 2026-11-18 by no more than it holds, below).
- **A first-class move** (`Command::MoveNow`, from 2026-11-18, below): out the frame it is given.
- **A move that replaces the unit's orders** (`Move { queue: false }`): it goes out at once when the bucket has one
  and nothing is waiting. Otherwise it waits, one per unit: a newer move for the same unit replaces the one waiting,
  which then never goes out, and the unit keeps its place in the wait.
- **Each frame** the waiting moves go out as the bucket refills, the largest first. The weight is the distance from
  the last move sent to that unit, times one plus the seconds waited. A unit with no move sent yet goes first. So a
  big change goes before a nudge, and a nudge that has waited long enough goes too.
- **Another order for a unit with a move waiting:** a queued one (`queue: true`) sends the waiting move first, so the
  queue keeps its order. An order that replaces the unit's queue (a non-queued build, fight, attack, reclaim, repair,
  resurrect, or a tend, guard, stop, self-destruct, D-gun, fire-at) drops the waiting move, as the engine would have
  replaced it. An order that sets a state (move or fire state, repeat, target, stockpile, priority) leaves it.
- **A unit that dies, or is given or captured,** loses its waiting move.

This decides when a move goes out, never what it says: an execution rule like the micro engine, shared with the bots.
The arm `WITHIN_REASON_ORDER_CAP` sets the rate in orders a second (0 turns the budget off). Once a game minute the
host logs a line `orders f=… minute: …`: orders sent and moves among them, moves held, replaced while waiting,
dropped by a later order, still waiting, the longest wait; from 2026-11-18 the first-class moves sent (and those
past the class's share) and the orders that went out past the bucket; from 2026-11-19 the orders whose place was
off the map and was pulled onto it (H-HOST-ON-THE-MAP, not the budget's: the one place the host changes what an
order says, since the engine drops an order to a place off the map).

### The debt's limit and the first class (2026-11-18)

**What happened** (`air-patrol-3-apex50`, the flagship against Apex +50; `docs/design/2026-11-16-air-intercept.md`
8.5): from 12:08.28 two roving Rovers were each sent a way of 31 to 35 queued legs again every six frames, 2,067
moves in eight seconds (205 to 393 orders a second; the rove planned its way round a bomber crossing the map, which
covered a leg of every new way the next tick: mended in `crates/micro/src/rove.rs` the same day). A queued move is
not held and takes one from the bucket, and the bucket had no floor: it stood some 1,300 in debt. Three fighters
ordered to meet that bomber were issued 12 to 14 moves each by the bot and the engine received none: the minute's
longest wait was 20.7 s. The bomber came in unmet.

**The user, in the workshop after it:** "A priority class for micro decisions that have a significant influence on
the game would be good to add in general, but if we can make it more efficient that would also be beneficial"; on
the proposal (the route, a limit to the debt, the class, the wing's own orders cut): "Agreed, proceed with 1 through
3."

- **The bucket owes no more than it holds** (`depth`, a second's worth). An order that goes out with the bucket at
  that limit is counted (`past the bucket` in the minute's line) and not owed. So the moves that wait behind a flood
  wait about a second after it, not until it is paid: the average over a stretch with a flood in it is above the
  rate by what the flood was, which went out anyway. A flood shows in the log now instead of in the waiting.
- **A first class of moves** (`bot_protocol::Command::MoveNow { unit, to }`): a move that replaces the unit's orders
  and is a decision, not the refinement of one. It goes out the frame it is given whatever waits and whatever the
  bucket owes, takes one from the bucket like any order, and drops the unit's waiting move (the older decision). To
  the engine it is a move. **Its own share:** 20 a second (`FIRST_RATE`, a fifth of the rate; of a lower
  `WITHIN_REASON_ORDER_CAP`, no more than that cap), two seconds of it at once (forty fighters leaving their loop in
  one frame). Past the share it is offered as a plain move and waits its turn, so a lane that marked every move
  first-class would flood nothing: the door has a width. The record writes it as a move with a fifth field.
- **Who sends it today:** the wing's conversion alone (H-MICRO-INTERCEPT: the move out to meet a raid, the turn
  in), three or four a fighter a scramble. **Who may:** a lane, for a move whose lateness by seconds decides
  something (a flee's first step, a dodge, a split under a blast). Each new sender is a decision to be named, with
  its number a second.

Not done here: a queued chain is still never held (a way of thirty legs is thirty messages; the rove sends one a
way, and a way stands a second at least now).

## How it is judged

1. **Rule 1 on the drill bench**, the binary before and after, four duels a row: orders a unit-minute from the drill
   logs' `sent`; arrival, formed share and the fights within a duel's noise. The rows are the walk of the first tier
   and the mixed company of 24 on Full Metal Plate, the yard and the wall's door, and the lap of Quicksilver with a feed.
2. **Rule 2 in an arena game of many seats with Apex,** with bots and no model: messages a minute per seat from the
   demo (`run/demo_net.py`, a column per AI team) against the Apex seats of the same game; the drill rows again with
   the budget on.
3. **A lobby game** (the user's to arrange), read with `run/demo_net.py`: the far clients keep up.

## What the bench said (2026-11-10; ledger near-*, cap-*, budget-*)

- **Rule 1:** the mixed company of 24 at 49 orders a unit-minute against 153, arrival and formed share the same; the
  Marauders' lap of Quicksilver 34 against 47; Pawns, Thors, the yard and a fight level.
- **Rule 2 at 100 a second:** a sound march is held only in bursts. A body of 96 had 136 moves held under 0.9 s in
  its first minute; a 4v4 of WRDistilled against Apex had its busiest seat-minute at 69 a second, 506 held under
  0.6 s.
- **Found:** two of four walks of 120 in five companies kept asking 150-209 moves a second at the goal with every
  member formed, with the budget off as on (K-group-five-companies-told-to-one-goal-do-not-settle). The budget holds
  that seat at 100 a second; the lane's end is the open item it already was.
- **At 30 a second** (the stress arm) the 96 waited up to 8 s for a move and stood formed 0.50-0.54 against 0.67-0.71.

## Not in this design

The army that marches for minutes without fighting (the review's string of companies) is the reason there were six
hundred units to order. The budget holds the stream whatever the army does; it does not make the army fight.
