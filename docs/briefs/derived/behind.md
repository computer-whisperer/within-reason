<!--meta
id: D-behind-tempo
layer: derived
premises: [D-metal-is-the-game, D-lead-convert-it, D-raid-answer, D-reach-nests, D-energy-second, F-eco-extractors, F-eco-build, F-eco-reclaim, F-eco-conversion, F-eco-consequences, F-fight-consequences, F-unit-armflash, F-unit-armllt, F-unit-armstump, F-unit-armsolar, F-unit-armmakr, F-unit-armvp, M-pool-closing, M-pool-pressure, M-u6bkep]
lessons: [L-gecko-behind, L-lead-use-it, L-raid-naked-mexes, L-raid-keep-moving, L-energy-converters, L-gecko-one-more-con]
claims: [K-army-a-side-behind-in-income-wins-a-tenth-at-home]
sources: {people: [thebluegecko, the user], dates: [2026-11-14], relay: thebluegecko's spoken review of the 1v1 our seat lost to u6bkep that evening (L-gecko-behind), written on the user's word after the report of it ("Go ahead and write points 1 through 3 for review"); the prices are arithmetic on the facts; the win rates are run/metal_killtest.py behind}
status: in the manifest since 2026-11-15, written 2026-11-14 and read by the user 2026-11-15 and merged on his word ("These look good. Let's leave it at this"; a bare assent); checked (run/metal_killtest.py behind, 2026-11-14: 1,176 sides of our 1v1s and 47 of the pool's duels that were behind in income with an army left)
-->
### Behind in metal: the clock is his

**The heuristic.** thebluegecko, watching our seat fall behind u6bkep by two Blitzes, then by a fight, then by the
game (L-gecko-behind): "sitting back while down metal income will eventually result in a loss. Every second that
passes now puts blue more and more ahead. So red needs to do something desperate"; "the units need to all be engaged";
and on what to buy, "if you're losing, what you need most, more than anything else, is tempo". The plain line: the
side behind in income loses by waiting, so it spends what it has on the present. Every soldier is where it does damage
or where its wreck will be ours, nothing is bought that pays later, and what stands idle is eaten and made into tanks.
Follow it from the minute the gap in income is plain while an army of ours still stands. It is the lead article read
from the other chair: the clocks that run against a surplus that waits are running for him.

**The quantity.** The gap in income and how fast it is widening, set against what of ours is doing nothing: soldiers
standing at home, energy thrown away, a plant with no metal to spend, a free spot inside our own cover, metal lying in
wrecks. In his reading of the one game: an army lead for blue of 15 percent that was 21 a few seconds on, metal
produced 14k against 11k with "nearly 2k" on the ground, and a metal lead of 20 percent that was 58 some six minutes
of the game later.

**The mechanism.**

- *The gap compounds.* Army is metal spent (D-metal-is-the-game): with his income above ours the difference between
  the armies grows every second nobody fights, and each fight he wins takes extractors, which is more income. "U6 was
  up just a couple of Blitzes and then winning meant up a few more and then a few more and on it goes." In the pool
  the side that first holds twice the army ends the game 3.4 minutes later (M-pool-closing): there is no long game to
  wait for.
- *An idle unit counts for him.* "You can sometimes find the advantage you need by causing your opponent's units to
  not be used while yours are used ... If the blue units are chasing red units but not actually firing, the red units
  might be doing damage for free." A soldier at our base fights when he chooses; a Blitz among his far extractors
  kills one in three seconds, 50 metal and 2.3 a second until it stands again (D-raid-answer), and whatever of his
  turns to chase it is not at our spots.
- *His turrets set the raid's price.* "Almost all the enemy mexes are protected by just one LLT. So if you were hoping
  to raid past five LLTs, then assuming you lose one Blitz per LLT, send something like seven or eight Blitz, you can
  go through and raid through all of that and come out pretty strongly positive." Five Blitzes lost are 550 metal;
  five extractors are 250 and 11.5 a second for as long as they are down, so the raid is paid by about half a minute
  of his downtime, and the two or three Blitzes that come through go on.
- *The ground pays.* A fight's dead lie where it was fought, up to six tenths of their metal, for whoever holds the
  ground (F-eco-reclaim, D-lead-convert-it): "try to get the fight on the other side of the map as soon as possible,
  which would also let you pick up the nearly 2k metal laying on the ground", which "would not quite, but almost make
  that even again". The same holds at home when he comes to us divided: "Red is defensive and may be able to bring
  their entire army value to bear. Well, Blue can only bring half of theirs because of the distance ... the metal pile
  will be controlled by Red".
- *Nothing that pays later.* A converter is 1,150 energy to build and 70 energy a second for one metal
  (F-eco-conversion): "If you are winning, you might choose to do that for efficiency reasons, just temporarily. But
  if you're losing, what you need most, more than anything else, is tempo." Tier 2 is the largest thing that pays
  later: "the not transitioning to T2 is correct". And a plant alone on Stouts spends ten metal a second, so "only 20
  metal per second, you can't really run two labs": at that income the second plant is 590 metal standing still.
- *What stands idle is metal.* A live building of ours gives back its full metal when it is eaten, with no energy and
  no wreck (F-eco-reclaim). Energy thrown away is a solar too many: "if you've got E that's wasting, eat a solar.
  Eating a solar wastes build power, but it is fully metal efficient ... drop that metal into more tanks". A solar is
  155 metal for nine seconds of the commander or twenty-nine of a constructor, a Blitz and a half; the second plant is
  590 for nineteen seconds of the commander, five Blitzes. And a free spot inside our own cover is 2.3 a second for
  50: "need to take the mex that is in the main and not taken. There's a constructor right there. It's protected."

**Consider, and why it may work.**

1. *The army goes, all of it, and to his extractors.* Blitzes on the side of the map where his army is not, in parties
   sized by his turrets, a Blitz for each Sentry on the way and two or three over; the plant makes what the raid is
   made of ("I see one medium tank, one blitz. This only works if you're going to raid with the blitz"); and nothing
   that can fight stays in the base ("probably want to get the units that are in the main out"). Why: the mechanism's
   first three clocks, and the record. A side behind in income with an army left won 10 to 13 percent of the time when
   the army stayed home in every population we have, ours and the pool's, and 18 to 39 percent when three tenths of it
   went; the people go in 39 cases of 47, our model in 33 of 109 (the tests).
2. *The fight where the wrecks are ours.* On his side when the ground can be held long enough to collect, with
   constructors or resurrection bots behind the army; at home when he comes divided, and then with everything, in
   front of the base and not through it. Why: the field is the one large piece of metal a side behind can still win, a
   fight that cost both sides 2,000 leaves up to 2,400 lying there, and it goes to the side standing on it. At home
   his army arrives in parts and ours need not ("you've got to try to isolate these armies and kill one of them so you
   can harvest it and then kick the metal back into units").
3. *Buy nothing that pays later.* No converter, no tier 2, no second plant under about twenty metal a second, no solar
   while energy goes unspent. Why: each is metal or build time taken from the next three minutes, which are the only
   ones left; an energy store is the exception, 170 metal for what would otherwise be thrown away ("the E storage was
   great. That's a way to get some use out of it").
4. *Eat what is idle and take what is free.* The solar that feeds nothing, the plant that has no metal, and every free
   spot inside our own cover before any spot outside it. Why: they are the only metal that comes at once and without a
   fight: 155 in nine seconds, 590 in nineteen, 2.3 a second for 50.

**Where it bends.** (1) Going is not landing. The wins are in the raids that land: sides whose army went and whose
opponent's extractors fell by three or more won 7 of 11 in the pool and 10 of 10 in our games; went and did not land,
6 of 28 and 28 of 165, little better than staying (1 of 8 and 104 of 1,001). (2) With an army under four fifths of
his, going won 1 of 24 in our games and 2 of 14 in the pool: there the instrument is the few Blitzes at his far spots
and the eaten buildings, and the body keeps to ground where turrets or the commander stand with it. (3) "Desperate" is
not the body into his nest. At even armies the old code's attacks, walks into BARb's turrets, won 2 of 50 against 43
of 414 for staying home (D-reach-nests); his word is for Blitzes at single-turret extractors and a fight on ground we
hold. (4) Behind in army with more income is not this article: "very slightly slower than your opponent" is a position
he recommends (L-gecko-one-more-con). This is for the side behind in the thing that wins. (5) Against an opponent with
an income bonus the gap is there from the first second and is no sign of a lost fight; the lead article's clocks
describe that game. (6) In a team the side's income is what is behind or not, and a seat that is behind beside one
that is ahead sends its army where the side's surplus goes (D-lead-convert-it).

**When to depart from the line.** When there is no army left to send: then only the raid by a few and the eaten
buildings. When his whole army stands at our door: fight it there, together, and collect. When the gap is a handicap's
and not the game's. Never because the base wants its soldiers back: "It'll be tempting for Red to think, well, I see
blue tanks now below my base, that's why I had tanks down there. But it doesn't work."

**The tests.** `run/metal_killtest.py behind` (2026-11-14). A side counts at the first minute between 6:00 and 12:00
at which its metal income is at most 0.85 of the other side's while its army is at least half of the other's and 500
metal; "went" is three tenths of that army in the other's half at some minute of the next three, "landed" is the
other's extractors falling by three or more within five minutes. Our 1v1s with a model playing, our side behind (109):
kept home, won 10 of 76; went, 13 of 33; went and landed, 4 of 4. Our 1v1s with the old code playing (1,067): 94 of
925, 25 of 142, 6 of 6. The pool's duels at OS 40 and above, either side (47): 1 of 8, 13 of 39, 7 of 11. The sides
that went had the larger armies in our games (a median of 1.8 times his against 1.05 for the model, 1.2 against 1.0
for the old code) and not in the pool (1.0 against 0.9), so in our games the result is partly "a side goes when it
can": by army, going beat staying at 1.25 times his or more in all three (11 of 26 against 4 of 24, 22 of 69 against
37 of 246, 6 of 16 against 0 of 1), at even armies in the model's games and the pool (2 of 6 against 1 of 29, 5 of 9
against 1 of 3) and not in the old code's (2 of 50 against 43 of 414). These are correlations over games in which
everything else varied, and the pool's cells are small.

One game, read by him (2026-11-14, `run/matches/1794707454-human`, lost to u6bkep at 21:36). Our extractors stood at
twelve at 9:00, nine at 14:00 and four from 18:00, thirty-four lost. The player's plan was "one army body that stands
on his road beside our turrets and meets each wave without chasing": from 3:00 no more than a tenth of our army metal
ever stood in his half, and none from 15:00. A second vehicle plant was begun at 9:32 on an income of 27; ten solars
were begun from 8:36 to 11:41 and four converters from 12:32 to 13:02, in a game that threw away about 17,000 energy,
11,000 of it before 8:00; the extractor spot 247 from the commander's start stood empty from 7:26 to 12:33. At 15:15
our fourteen soldiers, nine Stouts among them, stood east of the solar field and, as he read it, filed through it into
sixteen Stouts and six Janus: 2,650 metal lost for 705. No game has been played on this article.