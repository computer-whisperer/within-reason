<!--meta
id: D-metal-is-the-game
layer: derived
premises: [F-map-metal, F-eco-extractors, F-eco-energy, F-eco-conversion, F-eco-build, F-eco-reclaim, F-eco-consequences, M-pool-pressure, M-pool-expansion, M-pool-closing, M-format-8v8, M-barb-bonus]
lessons: [L-metal-first]
sources: {people: [thebluegecko, the user], dates: [2026-10-22], relay: the user's paraphrase of thebluegecko in the brief-rework discussion; the user's framing of the section as an honest derivation of a surface heuristic}
status: live, checked (the kill-test run 2026-10-22 with run/metal_killtest.py over 1,752 games of ours and the pool's 38 duels past 12:00: the income gap at 12:00 calls the result better than the army gap in every population; the inversions are our unconverted leads and the kills)
-->
## The derived layer

What follows are the reasons behind the disciplines. Each article names the heuristic as the players say it, the
quantity it protects, the mechanism in the game's rules and the pool's habits, where it stops holding, and what would
show it wrong. The heuristic line is what to do with no time or no information; the rest is for the situation that
does not fit it.

### Metal is the game

**The heuristic.** thebluegecko: the main thing in BAR is metal, the top thing to optimise for; last, eventually,
killing the enemy; "get mexes" and "build units to stand out on the map and defend territory" derive from that, and a
decision should always be aligned with what gets more metal. The lobbies say it shorter: "this game is 95% about
metal". Safe to follow as it stands until one of the conditions at the end applies.

**The quantity.** Metal income over the game, what the side extracts, reclaims and converts a second, and its
integral: what has been spent on things that compound. A bank is not income; a thousand in the store is an extractor
and a constructor not yet standing.

**The mechanism.** The map fixes the metal: a spot's worth is the sum of its squares and no building adds to it; an
extractor draws the squares within its radius at one depth, an advanced one at four times that. Energy is a
purchased good, bought with metal at a known price (a solar costs 7.75 metal per energy a second, an advanced fusion
3.2), and metal is bought back with energy at a terrible one: a tier-1 converter with the solars that feed it costs
about 540 metal per metal a second, against 22 for an extractor on a Comet Catcher spot and 90 for the advanced
extractor. While a free spot stands, taking it is twenty-five times cheaper than converting. So every energy
decision is a metal decision, and the question of every building is metal per metal a second, and when. When
matters as much as how much: an extractor placed a minute late forgoes 138 metal, three times its price, and the
extractor that metal would have built is late in turn. Army is metal turned into an instrument, and its jobs are in
metal: taking spots (every side of the pool sends a soldier into the other half, three quarters of them by 3:00, the
winners forty seconds earlier than the losers), denying his (the pool's first extractor dies at about 4:00; a side
loses 1.5 to 2 a game-minute, the winners one, the losers two and a half), and protecting ours (a light turret's 85
metal against an extractor's 50 and its 138 a minute). A fight is a metal exchange whose wrecks, up to six tenths of
what died, go to whoever holds the ground afterwards. From these, "get mexes", "build units to stand out on the map"
and "defend territory" follow as the three ways an army moves the quantity, and so does the pool's shape: the winner
adds ten extractors between 8:00 and 12:00 where the loser adds five, holds twice the army at minute 7, and ends it
three and a half minutes later.

**Where it bends.** Four places, each a dispute with the heuristic as stated, and a fifth for team games.
(1) Not metal but metal spent: the second thing in thebluegecko's order, which the relay lost, is probably build
power, since income nobody turns into anything is the bank. (2) The sign: denying his metal counts as much as gaining
ours, and the measure is the ratio of the incomes, not ours alone; the AIs' lane seats cutting the people's income
from 2:05 in 8v8 is metal-first play with the sign flipped. (3) The horizon: an advanced extractor repays in four
times an extractor's time, a tier-2 sprint is metal now for metal later, and "what gets more metal" says nothing
until the horizon is named, which is the game's expected length (a duel closes at 12:39 median, an 8v8 against stock
BARb at about 31 minutes). (4) The end: once his income is gone the remaining game is the kill, and metal stops being
the objective; the nuke is the 8v8 form of that. (5) In a team the quantity is the side's, not the seat's ("metal in
labs dont make more metal"), which is why the gifts go to the one seat that compounds them and why the won sides
give five times what the lost sides give.

**When to depart from the line.** When his raiders are about to cut more income than the next extractor adds (the
raid answer); when a lead in army metal is being held while his spots stand, since army depreciates and income
compounds (the unconverted lead); when his income is gone (go and kill); and in a team, when the seat's metal
compounds faster in a neighbour's hands.

**The tests.** The pool: income and extractors at 8:00 to 12:00 separate winners from losers before the army does.
The 8v8 lobbies against stock BARb: the people's won and lost sides had the same army at 10:00 (10,132 against
9,762) and different income (271 against 193), and the won sides' extractors held at 71 while the lost sides' fell to
50. Our own games: every loss on the BARb bonus ladder was a lead in army held to about 20:00 while his spots stood.
The kill-test, run 2026-10-22 (`run/metal_killtest.py`): if the army gap at 12:00 predicted the result better than
the income gap, the quantity would be wrong as named and the root would be army. It does not, in any population. Over
every recorded game of ours with both sides at 12:00, the sign of the income gap calls the result in 74% of the 242
duels the model played against the army gap's 67% (where the two disagree, 51 games, income is right in 34), in 88%
against 79% of the 1,495 duels the old code played (322 disagreements, income right in 231), in 93% against 80% of the
15 team games, and in 84% against 79% of the pool's 38 duels that lasted past 12:00 (7 of 11 disagreements); the
ratio ranks the won games above the lost ones at 0.84 against 0.81 (the model's), 0.95 against 0.92 (the code's) and
0.89 against 0.83 (the pool's). Earlier the game is less decided: at 8:00 the pool's ratios rank level (0.75 against
0.76) and 10:00 is the one minute where the army calls more often (71% against 63% over 49 duels). What the inversions
are: of our 134 model-played losses, 38 were ahead on income at 12:00 and 35 ahead on both, and they are the long
games, 27 / 31 / 34 minutes against 16 / 21 / 28 for the losses from behind: the unconverted lead of the bend above,
his tier 2 arriving at about 25:00 into an army that never went to his base (rove-task-1, levers-player-1,
barb-ladder-35-2, player-30 to player-40). Of our 108 wins, 18 were behind on income at 12:00 and won on the kill
(barb-hard-20-1, his commander at 21:10). Our income in the test is the extractor income, each standing extractor's
spot worth times its rate times the side's bonus, which runs at 0.96 of our seat's exact income; the pool's is exact.
