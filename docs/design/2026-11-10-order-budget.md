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

- **Not a move:** every unit order goes out when given and takes one from the bucket (the bucket may go below empty).
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
dropped by a later order, still waiting, the longest wait.

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
