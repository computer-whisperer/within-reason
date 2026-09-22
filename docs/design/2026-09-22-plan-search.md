# Plan and search: the player explores build orders in a simulator, not in matches

Written 2026-09-22 evening, from the user: "the solution is coming up with a method for allowing you and opus
agents to explore the space of build orders rather than relying on piecemeal human recommendations or match-scale
trial and error." Every player game opens with "the comet-4 proven build order" from the brief; the Advanced Solar
Collector, on the roster and Jev's menu with a gloss that calls it as metal-efficient as fusion, was built in one
player game out of thirty.

## What stands

- `crates/buildorder`: a fixed-step economy simulator (resources, storage, wind, extractors, generators, converters,
  builders' walks, factories, nanos, stalls) and a simulated-annealing search over build orders, calibrated on twelve
  quiet openings to about 10 % (`docs/studies/build-order.md`). Its palette is the tier-1 opening: the cheapest
  extractor, wind, steady generator, converter, storage, radar, one factory type, nano, one turret; factory queues
  draw from that one factory's soldiers. Objectives: income, army, mix, tempo, expect.
- `crates/bot/src/brain/planner.rs`: the rolling planner turns the live game into a simulator `State` (standing
  economy, builders with their jobs, resources) and re-plans for the heuristic modes; it ranks builders by the Kit
  (commander, the Kit's lab, the Kit's constructor) and drops every other factory or constructor.
- Nothing of this reaches the player: the `queue` tool takes lists, the picture reports curves after the fact.

## Decisions

1. **The palette is the roster.** `Palette::roster(units, commander)`: every building the commander or any
   constructor reachable from it can build that makes, stores, converts or extracts, every factory and nano, the
   turrets on offer, and `assist`; the factory list is the union of every reachable factory's soldiers and mobile
   builders (a step a factory cannot build is passed over, as today). Tier 2 is in the space by construction:
   advanced solar, fusion, advanced extractors, tier-2 plants and their units.
2. **An advanced extractor is an upgrade.** A step for an extractor that extracts more than a standing extractor of
   ours goes on that spot (the named one, else the nearest) and replaces it when finished; it takes no free spot.
3. **Builders by role, not by Kit.** The snapshot ranks the commander, then every factory, then every mobile
   builder, so a plan's queues follow whatever stands: air plants, hover platforms, construction vehicles, a captured
   constructor of the other faction.
4. **A `target` objective.** `target UNIT [by M:SS]`: the metal of finished units of that type at the horizon, plus
   the mix objective's income term, minus a stall penalty, plus an earliness bonus for the first one finished before
   the asked minute. It is the question the player asks: "Bulls at full speed by minute nine".
5. **The `plan` tool.** The player hands queue lists in the `queue` tool's vocabulary (`extractor spot_N`, `assist`,
   any internal name with an optional place), keyed by the actor names it knows (`commander`, `lab_N`, `plant_N`,
   `constructor_N`) and `next_factory_1..` / `next_constructor_1..` for what is not standing yet, with a horizon in
   minutes. It returns the curves by minute (extractors, metal and energy income, stall share, army metal, build
   power), the minute each named unit type first finished, and the effective queues with what was passed over and
   why. It simulates from the game's latest published state, on the tool's own thread.
6. **The `search` tool.** An objective in words (`income`, `army`, `mix`, `target UNIT by M:SS`), a horizon and a
   time budget in seconds (at most twenty: the game holds during a turn). The annealer runs from the live state with
   the roster palette, warm-started from the player's current queues when it gives them, and returns the best plan
   in the same vocabulary with its curves, and the score's parts. The player writes the packet; nothing is ordered.
7. **The same on the command line.** `buildorder optimize|simulate --palette roster --objective target:armbull@540`
   and plan text in full internal names, so the exploration can run offline against a record's header.
8. **The published state.** The brain publishes a `PlanContext` (the game's unit table and scenario once, the state
   and the builders' names every ten seconds) to the strategist's shared board; the tools never touch the brain.
9. **Calibration before trust.** `buildorder calibrate` over today's tier-2 records (the Bulls games, the A/B
   batches) before the tools are offered in anger; the error goes in the study document beside the tier-1 numbers.
10. **The bot's own planner is unchanged**: the heuristic modes keep the Kit palette until the roster palette has
    been read against them (H-OPEN-SEARCH).

## What this does not decide

Whether the search should ever run without being asked (an "advisor" line in the picture): no, for now. The carry
loop is where a plan that keeps winning becomes a claim in the brief.

## Steps

1. `buildorder`: the roster palette, the upgrade rule, the target objective, the CLI flags; the study's palette
   note.
2. `planner.rs`: builders by role; `PlanContext` published to `Shared`.
3. `mcp.rs`: `plan` and `search`; the vocabulary mapping both ways; the role text.
4. Calibration on today's records; the study document.
5. One player game with the tools on the Bulls objective (the energy question in the same game), read by the
   scorecard and the transcript's tool calls.

## Status (2026-09-22 night)

Steps 1-4 built (e5f8280, 793b206, e3bca72). Step 5, plan-1-bulldogs: WON with 19 Bulls and seven Advanced Solar
Collectors; the player searched once on its first turn, read the answer as a ceiling ("no army at all") and planned
around it. Left open: the simulator's army curve runs 30-50 % low and its income 15-25 % high on the player's
openings (the study document); the search's answers favour many constructors and no army, which the player must
weigh; the `queues_note` says the order must be given by `queue` and `produce`, and the player queued a building to
the commander that only a constructor builds (the tool's queue names carry the right builder; the words could say
"only a constructor builds this"). Two hands findings from the game are in the ledger row for the next floor pass.
