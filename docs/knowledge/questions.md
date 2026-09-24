# Questions for the replay survey

What the synthesis agents look for in the game cards (`docs/design/2026-09-23-replay-survey.md`, decision 5). An
agent reads a batch of cards for one map and matchup against this list and files claims in `docs/knowledge/` with the
match ids as evidence and counts ("in 18 of 20 games"); a question answered becomes a claim, a question the cards
cannot answer is said so. Add questions here as play raises them; strike none.

## Per map (a claim per map, `K-map-<map file name>-...`, and `docs/knowledge/maps/<map file name>.md`)

1. The opening by the clock: which spots first from each start, when the first factory stands and which, how many
   solars or winds before it, when the first constructor and the first soldier come out.
2. Where the expansion goes in the first eight minutes from each start, and in what order; which spots are taken
   by the commander and which by constructors; how many extractors and how much income at 4:00, 6:00, 8:00.
3. When the second factory comes and of what kind; when tier 2 comes (the first tier-2 factory and the first
   tier-2 unit) in games that reach it; how many constructors by 5:00 and 10:00.
4. The first contact: when, where (cells), with what; when the first extractor is lost and to what; how raids are
   met (turrets at the spots, units held back, chasing or not).
5. Where the fights are: the cells where most units die by minute; the routes between the bases the cards show
   (cells the army passes through); what decides the game (a raid, a push, an economy gap, a tier-2 timing).
6. How games end: how long they last, how the winner closes (the commander hunted, the base razed, a resign after
   what); what the loser had at the end against the winner.
7. What the higher-OS player does differently from the lower in the same game (the cards carry both sides).

## Across maps (claims in `docs/knowledge/openings.md`, `army.md`, `maps.md`)

8. What a start position's spot geometry decides: how the number of spots within the first minute's walk, the
   distance to the middle, and the passages shape the opening and the raid timing.
9. Unit mixes by minute and what beats what at this level of play: which raiders, when line units replace them,
   when artillery and tier 2 appear, what people build against what.
10. The economy curve of a winner against a loser: the income gap over time, when it opens, whether the winner
    banked or spent.
11. What people do that our brief does not say: standing habits (radar timing, turret pairs, nano counts, when the
    commander leaves home) worth a claim of their own.
12. Wrecks: how much metal the fights leave on the ground and who collects it (constructors on the field, resurrection
    bots, nobody); when the first resurrection bot comes and how many; whether the collectors work the battlefield or
    their own ground. (The card needs: rez bots built and their first clock, income above the extractors' yield by
    minute, collectors within reach of the fight cells.)
