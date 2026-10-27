# Tier 2 in the heuristic bots' seats (2026-10-25)

**Status.** Step 1 (the distilled bot's gate) built as arms and measured (dist41-49): the seats tech and earn, the
results do not follow, and duels lose; `gate=field` stays the default in a duel, and a seat with allies takes
`gate=tech` (the user, 2026-10-26: "Let's use tech in games with allies"). Step 2 (the army's
type) reframed by an offline look, not begun. The spending of a tier-2 income found as a further bottleneck, one
attempt measured without effect.

## Why

The user, 2026-10-25: "These bots have been useful as seats in multiplayer games, and the main issue is how they don't
scale into using the t2 units effectively when the game demands it." Agreed the same evening: the distilled bot's gate
first, then the army's type for both bots; the bench a 2v2 against BARb.

What the user's eight-seat game on All That Glitters Extended shows (the seats' stderr in the client's launcher log):

- The two distilled seats never started an advanced plant (`advplant=0`, `field_won=0` every minute). From 16:00 they
  made 130-180 metal a second and 1,100-2,300 energy, and from 24:00 the metal store sat full (7,547 / 7,600, 9,425 /
  9,450, 12,238 / 12,300) while four to six vehicle plants made tier-1 tanks. D-T2-GATE (`gate=field` since
  2026-10-18) waits for the field to be won (D-FIELD), which a seat in a team game never sees.
- The territory seat t5 had the tier-2 economy: the advanced vehicle plant by 10:00, 21-35 advanced constructors, 140
  advanced extractors at 30:00, thirteen fusions. Its army stayed tier 1: 119 Blitzes at 25:00, light laser turrets and
  Pit Bulls, never a tier-2 soldier. T-GUARD makes "the mobile ground unit with the most ground fighting strength per
  metal" among our factories' products, and per metal a Blitz beats every tier-2 tank.
- The opponents' tier 2 did the killing: from 27:00, 1,474 of our four bot seats' units died to ground weapons, mostly Tzars', and 771 to
  aircraft, 23 Wasps and 5 Dragons at least.

What the derived brief says tier 2 is for (`docs/briefs/derived/tier2.md`, `lead.md`, `reach.md`): the economy ("a
game past twenty minutes belongs to the side with mohos": 103-106 a second from the spots at 20:00 against 28-37);
reach and splash, not strength per metal ("in the open at equal metal tier 1 still trades by the square law ... the
cliff is a reach cliff: his base becomes unreachable, not his army unbeatable"); "a bank is a plant paid" (2,600
stored is the plant with nothing forgone, and metal above the store's cap is lost every second); "decide at his lab".

## Step 1: the distilled bot's gate

D-T2-GATE keeps its income figure (22), clock (6:00), energy store (30 %) and one plant, and its last condition
becomes an arm:

- `gate=field` (the old default, still the default in a duel): the field won.
- `gate=bank`: the field won, or the bank holds the plant's bill: its 2,600 metal, or 90 % of a
  store smaller than that. Nothing else was spending that metal.
- `gate=tech` (the default for a seat with allies since 2026-10-26): those, or anything of his of tier 2 seen
  (`techlevel` from the combat tables).
- `gate=income`: the income alone.

The reason the gate opened is logged (`t2 gate:`), as is the first tier-2 thing of his seen (`his tier 2 seen:`), and the
minute's army by tier (`army: tier 1 ..., tier 2 ...`).

The smoke (dist41, `gate=bank`, one game): the gate opened by the bank at 19.7 and 20.4 min on 85-130 a second of
income; until then D-BUILD-POWER turned the bank into more tier-1 plants and turrets, so 2,600 never stood idle. His
first tier-2 building stood at 11.9 min; at 20:00 he had eleven advanced extractors, a Tzar, flak and two Fiends, at
25:00 twenty-four and four Tzars. Lost at 31.9.

Measured on the team bench, sixteen games an arm on the same seeds (dist42-45):

| Arm | Won-lost-cap | Gate opened (seats, median minute) | Tier 2 of the army at 25 / 30 min | Income a seat at 25 / 30 | Our team's income / his at 30:00 |
|---|---|---|---|---|---|
| `field` (dist42) | 3-6-7 | 0 of 32 | 0 / 0 | 84 / 83 | 170 / 261 |
| `bank` (dist43) | 3-7-6 | 20 of 32, 17.1 | 0.04 / 0.16 | 78 / 90 | 177 / 365 |
| `tech` (dist44) | 1-6-9 | 32 of 32, 14.1 | 0.40 / 0.39 | 127 / 192 | 393 / 333 |
| `tech` + advanced plants by income (dist45) | 2-7-7 | 32 of 32, 14.4 | 0.37 / 0.36 | 117 / 178 | 321 / 357 |

- The sighting is what opens the gate in time: his tier 2 is seen at 14.5 min, two thirds of a minute after his
  first tier-2 building stands; the bank alone waits until 17 and in a third of the seats never.
- With it the seats keep pace: the team's income about doubles by 30:00 and stands near his, the army is a third
  tier 2 from 25:00. The results did not move within sixteen games.
- What follows is spending: from 30:00 the bank piles (4,500-8,500, to 17,800 at 35:00, the store nearly full) on
  one advanced plant. A rule for more advanced plants by income (dist45) did not raise the count: the constructors
  take D-PROJECT's help with the fusions before D-BUILD-POWER's rung.

Then the duels and the wait (dist46-49): in 1v1 on Comet against BARb hard_aggressive `gate=field` went 9-3-4 (no
plant, games over at a median 19.7 min) and `gate=tech` 3-7-6: the plant at 12.8 min, the tier-1 plants paused for it,
the army at 15:00 4,506 against 7,927. The derived brief's "if the surplus is in his half ... the sprint waits for the
kill" as a wait while the strike body pushes past the middle: the duel 3-5-8 (the gate still opened in 14 of 16, at
13.8), the 2v2 0-10-6. So teching as such, at any of these moments, buys these seats nothing in results against
BARb within 40 minutes: the income comes, and nothing turns it into a stronger army in the field.

What the seats die to (the death instrument's `died:` lines, our metal lost a game, sixteen `field` games against the
48 of the three `tech` rows): at 10-20 min his tier 1 21,000 / 22,000, his tier 2 5,300 / 4,700, his turrets 6,600 /
3,100; at 20-30 min his tier 1 21,900 / 15,700, his tier 2 39,300 / 44,300, his turrets 29,700 / 18,200, of which our
soldiers 79,900 / 61,500 (17,800 of them tier 2); from 30:00 his tier 2 49,200 / 94,300 (the `tech` games run
longer). In either arm the army is spent from 20:00 into his tier-2 units and his turrets, at about twice what his
tier 1 takes: the reach article's toll (`docs/briefs/derived/reach.md`), paid by our tier 2 as by our tier 1.

## Step 2 (proposed): the army's type by what beats his

Both bots choose their soldier by strength per metal, which never picks tier 2. The proposal: among the soldiers our
factories make, the one that beats the army he has shown for the least metal, by the combat simulator (`combatsim`,
which carries reach, splash and armour classes), and the same frame for anti-air and for artillery against his
nests. To be checked offline first, on the armies of the user's game, before either bot reads it.

A first look (the `combatsim` command line, twelve seeds a fight, his army the two BARb seats' armed mobile ground
units at 25:00 of the dist41 smoke: 34,033 metal, 23 Stouts, 4 Tzars, 4 Mortars, 4 Furies, 3 Tigers, 3 Sumos and the
rest): at equal metal and at 1.5 times it, every single type of ours lost. The margin at 1.5 times was -0.35 for
Blitzes, -0.50 Bulls, -0.59 Incisors, -0.70 Mausers, -0.73 Tigers, -0.80 Stouts, -0.82 Panthers, -0.97 Starlights.
The fewest that won three in four: Tzars at 1.45 times his metal, Bulls at 2.32, Tigers and Banishers at 3.5, no
tier-1 type within 400 units (the simulator stands them eight to a rank, so 400 arrive as a column fifty deep). So
in an open stand-up fight the simulator agrees with the brief: tier 1 trades as well per metal as tier 2, and "what
beats his army per metal" in the open would keep choosing Blitzes. What tier 2 buys that an open fight does not
measure is reach (his nests and his artillery out-ranging ours), splash against a crowd, the rate a factory spends
(a tier-2 plant's 600 build power against 150), and a body that does not arrive as a column. Step 2 needs that
frame before it is built; not decided.

## The bench

`run/distilled_team_bench.sh FIRST TAG ARMS...`: two distilled seats against two BARb hard_aggressive on All That
Glitters v2.2.3, sixteen games an arm on seeds 1-16, 40:00, played out; `run/distilled_t2.py` reads it (outcome; when
the gate opened and why, the plant standing, his tier 2 seen and on the map; at 15, 20, 25 and 30 minutes our army by
tier, the bank, the income, the advanced extractors).
