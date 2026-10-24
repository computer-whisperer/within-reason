<!--meta
id: D-lab-choice
layer: derived
premises: [F-unit-armlab, F-unit-armvp, F-unit-corlab, F-unit-corvp, F-unit-armck, F-unit-armcv, F-unit-armpw, F-unit-corak, F-unit-armflash, F-unit-corgator, F-unit-corlevlr, F-unit-armfav, F-unit-corfav, F-unit-armflea, F-unit-armllt, F-eco-energy, F-eco-build, F-eco-conversion, F-fight-consequences, F-map-metal, F-map-catalogue, F-barb-how-it-decides, M-pool-opening, M-pool-first-units, M-pool-pressure, M-pool-openers, M-barb-hard-aggressive-comet, M-gecko-duels, D-lead-convert-it, D-reach-nests, D-raid-answer]
lessons: [L-gecko-bots-vs-vehicles, L-sheet-labs, L-gecko-one-more-con, L-gecko-three-resources]
sources: {people: [thebluegecko, the user], dates: [2026-10-23], relay: his spoken account of a vehicle opener against a bot opener, recorded by the user and transcribed locally (L-gecko-bots-vs-vehicles), his unit-sheet rows on the labs (L-sheet-labs), his reading of the metal article aloud (L-gecko-one-more-con); the user's request that the derivations stay lean, ideas with their arithmetic and no doctrine}
status: draft, checked (run/metal_killtest.py openers, 2026-10-23: the pool's 80 Comet sides by first factory and our 280 model-played duels with both first factories read; the numbers in the tests)
-->
### The lab choice

**The heuristic.** thebluegecko's sheet (L-sheet-labs): the vehicle plant is the "default lab choice on solar maps";
the bot lab "1 of two most common starting labs, better on low e cost maps", since "bots are generaly weaker than veh in
a straight fight due to e cost implying the veh player will have a higher army value afor the first big fight". His
account of the duel between the two (L-gecko-bots-vs-vehicles): the vehicle player projects power first, with Rovers and
Rascals that kill extractors and constructors and cannot fight bots; the bot player is pinned on his side and cannot
afford towers, but takes the close spots faster and must then cross, "not to do damage, but to force the vehicle player
to build vehicles with weapons rather than more cons"; at five to seven minutes he meets a vehicle army his energy bill
has made larger, and "if the vehicle player takes a good engagement at this point, the game will end in the vehicle
player's favor"; otherwise he hides under towers at the close spots and switches to vehicles or to air. The pool's
Armada plays the vehicle plant in 40 of 41 sides on Comet (M-pool-opening) and so do we; this article is for reading his
lab, and for the maps and seats where ours is a choice.

**The quantity.** What the first five minutes' metal becomes: spots a minute from the constructors, army metal a second
net of its energy, and the minute that army can stand in his half. The lab fixes all three before a unit is built, and
the duel between the two labs is a race between an economy and an army that each lab makes cheaper.

**The mechanism.**

- *The labs and their constructors.* A bot lab is 500 metal and 950 energy, 16.7 s by the commander; a vehicle plant 590
  and 1,550, 19 s: the lab saves 90 metal (an extractor and most of a second) and 600 energy (half a minute of a solar)
  and stands two seconds sooner. A construction bot is 110 metal and 1,600 energy to the vehicle's 135 and 1,950, so two
  bots save another extractor; it builds at 80 to the vehicle's 90 (an extractor in 22.5 s against 20) and walks at 36
  to 54. The walk is where the lab decides the expansion. Comet's 80 spots stand 400 / 476 / 595 elmos from their nearest
  neighbour, no two within 229, and a start has four within 650 (F-map-metal): a constructor's cycle from one spot to
  the next is 13 s of walking and 22.5 of building for the bot, 9 and 20 for the vehicle, 36 s against 29, so the vehicle
  takes a quarter more spots a minute from the first spot on. On a map whose spots lie in triples within 250 of each
  other the walk is 7 s against 5, the cycle 29 against 25, and the bot's cheaper price buys the difference back: his
  "triples favour bots" is this geometry, and Comet has pairs, not triples.
- *The energy bill.* A Pawn is 54 metal and 900 energy, a Grunt 43 and 820: 16.7 and 19.1 energy per metal. A Blitz is
  110 and 900, an Incisor 120 and 1,100: 8.2 and 9.2. The raider bots cost twice the energy per metal of the raider
  tanks; the line units are level (Mace 10.0, Stout 9.3, Thug 8.2, Brute 10.2) and the scouts near it (Rover 11.9, Rascal
  10.4, Tick 14.3). A solar is 155 metal for 20 a second. Spending 1,000 metal on Pawns over five minutes draws 56 energy a
  second, 2.8 solars, 430 metal of them; the same on Blitzes draws 27, 1.4 solars, 210 metal. The bot player buys 220
  metal more of solars for every 1,000 of raider army, or holds that much less army on the same solars: two Blitzes, or
  four extractors, by 5:00. That is the pool's picture at that minute (M-pool-openers): the bot sides stand 10 solars and
  make 251 energy a second to the vehicle sides' 7 and 190 on the same metal income, and hold 0.64 of their army. On a
  wind map the bill shrinks: a turbine is 40 metal for about three quarters of the wind's top (F-eco-energy), 15 a
  second where the wind runs to 20, 2.7 metal per energy a second against the solar's 7.75, so the Pawn army's extra 28
  a second costs 76 metal of turbines instead of 220 of solars, which is "better on low e cost maps" in numbers. Per
  metal of army the bot is the better fighter: a Pawn carries 1.67 damage a second per metal to the Blitz's 0.82 at the
  same health per metal. Per metal of army and solars together it is not, which is his "more efficient per metal if you
  count the e-build".
- *Speed and sight: the pin.* A Rover moves at 168 a second and sees 635, a Rascal 153 and 600; a Pawn 87 and 429, a
  Grunt 81 and 520, a Tick 132 and 600; a Blitz 101 and 350. Nothing of the bot lab's catches a Rover but the Tick, which
  the Rover kills in 1.7 s (60 health at 35 a second) while the Tick needs 2.6 (105 at 40). A Rover kills an extractor
  (270) in 7.7 s and a construction bot (760) in 21.7, and the bot neither flees (36 against 168) nor fights back; a Pawn
  kills a Rover in 1.2 s if it reaches one, which it does not. So the scout car raids the bot player's spots and
  constructors at no risk from his army, and his answer is a tower (a Sentry, 85 metal and 680 energy, kills a Rover in
  0.7 s and a Rascal in 0.6, and its 430 of reach outranges the Rover's 180; D-raid-answer) or a Tick pair at the outer
  spots, each of which is metal not expanding: the pin. His raiders cross later because they are slower, and they meet
  the same tower on our side: a Sentry kills a Pawn in 2.3 s, the Pawn needs 6.9 to kill it and 430 against 180 of reach
  means the first 2.5 s are free. In the pool the bot sides' first soldier crosses at 3:05 to the vehicle sides' 1:49,
  and their first extractor falls earlier (3:33 against 4:15), to a scout car or a raider tank in 14 of 16.
- *The window, 4:00 to 7:00, and how narrow it is.* The vehicle side's army is 1.5 times the bot side's at 5:00 in the
  pool (the bot side at 0.51 / 0.64 / 0.99) and 1.4 at 6:00 and 7:00; by 8:00 the ratio is 0.89 and by 10:00 0.86,
  because the bot side's constructors (6 at 8:00 to 5) and solars have bought 18 extractors to 15, and its second
  factory, a vehicle plant in 11 of 16, started at 6:19. What the surplus is worth is the square law (F-fight-consequences)
  with the Pawn's double damage per metal in it. A thousand metal of Blitzes (nine: 818 damage a second, 6,636 health) against
  667 of Pawns (twelve: 1,111 and 4,568) is 5.4 against 5.1 million of damage times health, level; against 500 of Pawns
  (nine: 833 and 3,426) it is 1.9 to 1; against those 500 of Pawns standing beside three Sentries (255 metal: 482 a
  second and 1,860 health more) it is 0.8 to 1. The same 1,000 against 500 of Grunts (twelve: 860 and 3,256) is 1.9 to
  1. So the window is real and it is narrow: open at two to one, in the open, and shut at his towers. That is his
  "the blitz player gets twice as much army and can't always win with it because there's LLTs that are efficient on the
  other side of the map they have to trade into" (L-gecko-one-more-con), and it is the toll of D-reach-nests at the one
  place the bot player can afford to pay for it.

**Consider, and why it may work.**

1. *Read his lab as the game's clock.* The pool's Rover is within 1,500 of his start at 1:54 (M-pool-pressure); the
   first look at his base (D-reach-nests) tells which race this is. A bot lab: his raiders come at about 3:05 with
   Pawns or Grunts a Rover outruns, the window is 4:00 to 7:00 at two to one in the open, and from 8:00 the economy
   race is his to win unless the window was used. A vehicle plant: his scout cars are at our outer pairs from 1:49, the
   armies run level (1.5 to 1.7 in ours against BARb's vehicle plant, and we won 65% of those), and the first metal
   goes to a tower per outer pair (D-raid-answer). Against BARb his lab is dice (F-barb-how-it-decides), so nothing before
   the look says which.
2. *Against a bot lab, the surplus of minutes 4 to 7 is the lab's whole return, and it is spent in the open.* Why: at
   two to one Blitzes beat Pawns 1.9 to 1 in the open and lose 0.8 to 1 at three Sentries; in the pool's sixteen mixed
   duels the vehicle side that held 1.5 times the bot side's army at 7:00 won 6 of 7 and the one that did not lost 6
   of 9. In ours (M-barb-hard-aggressive-comet's profile on Comet, 91 games of fourteen minutes or more without a bonus)
   we held 2.1 times BARb's bot-lab army from 4:00 to 6:00 with nothing of ours in his half at the median through 8:00,
   and won 32, 35%, against 65% when its dice gave it a vehicle plant: the lead went unconverted into the one opponent
   whose opening is built to outgrow it (its bot-lab self stands 12 extractors at 8:00 and 18 at 12:00 to its
   vehicle-plant self's 10 and 15). D-lead-convert-it's clocks run fastest against this opener, and the cheap test is the
   surplus's whereabouts at 5:00. Where it is spent: his constructors on their way out (a Rover kills one in 22 s, two
   in 11, and it cannot run), his outer extractors, his Pawns in the open before the towers stand; not his cluster.
3. *Five scout cars by 3:00 are the pin and its answer in the same metal.* 57 of the pool's 64 vehicle-plant sides have
   them (M-pool-first-units). A Rover costs 31 metal, a constructor 110 to 135: "raiders cost less than another
   expanding con, and so you can do more damage to your opponent if you build raiders and they don't"
   (L-gecko-one-more-con). Against a bot lab the Rovers are the only units of ours his army cannot touch, and his
   constructors the only units of his that cannot touch back; against a vehicle plant a Rover kills a Rover in 3 s and
   whoever has more of them at the outer pairs holds the sight there (635 each) and the extractors under it.
4. *One constructor more than him, not two.* His rule (L-gecko-one-more-con): "built one more constructor than them ...
   Four cons to six cons, the player with the four cons is probably ahead because the units they build will beat the
   player that has the six cons." The count is read off his lab: a bot lab's constructors are 25 metal cheaper and a
   quarter slower, so his bot-lab side has one more by 8:00 in the pool (6 to 5); against him the sixth of ours is
   220 metal of Blitzes that were not built. And his adage for the whole: "be very slightly slower than your opponent
   or significantly faster", slower meaning slightly more income, and significantly faster, by the arithmetic above,
   two to one in the open. He also says what our earlier versions taught him: "Within Reason's historical versions don't
   harass me really, really early. And so I tend to open with another con or two more than I would when playing against
   a human player"; those constructors are the price of our first handful staying home.
5. *Our own lab, where it is a choice.* The vehicle plant where the spots are spread and the energy is solar (Comet's
   476 elmos between neighbours and wind of 1 to 4; the pool's 40 of 41 Armada sides and his "default on solar maps");
   the bot lab where the spots cluster within 250 and the wind runs high (the cycle 29 against 25 s, the energy bill a
   third), or as the second seat of a team whose first is vehicles. The second factory of either kind is the other kind
   about half the time in the pool (the bot sides a vehicle plant in 11 of 16 at 6:19; the vehicle sides a bot lab in
   15 of 47 at 8:33), so the choice is not made once.

**Where it bends.** (1) The pool's bot openers are Cortex in 15 of 16 and its vehicle openers Armada in 40 of 64:
faction and factory move together there, and a Grunt walks at 81 whatever the lab (M-pool-openers). (2) Comet has no
triples; the constructor arithmetic above favours the vehicle from the first spot, and the bot lab's case is other maps'.
(3) The pool's vehicle sides are not naked: 4 towers by 5:00 and 9 by 8:00, Cortex's first at 0:27 (M-pool-opening).
His account is of the duel in general; the pool's Comet duels are towered on both sides, and the bot sides' towers (2
and 5) are the fewer. (4) BARb's bot lab is not a bot player: its first units cross early (54 to 75 of army metal in
our half at 3:00) and its expansion is scripted; against it the bot lab is the harder opponent for our habit, not for
its strength. (5) In a team the labs are the side's, one of each kind and the sheet's air plant "around 30m/s on many
maps" (L-sheet-labs), and the duel above is between seats, not sides. (6) The bot player's answer in the open is range, not metal: the Grunt reaches 215 to the Blitz's
180, and "if you get Grunts in a concave all running away, the Blitz can take enough shots before the engage that the
Blitz lose ... in really small numbers ... the engage never happens" (L-gecko-bots-vs-vehicles). A Blitz at 101 closes
on a Grunt backing away at 81 by 20 elmos a second, so the 35 elmos of range are 1.75 seconds of free fire per Grunt,
130 damage, 18% of the Blitz's 730; four Grunts in a concave take 70% of the first Blitz before it fires. Per metal
the Grunt is the better unit by the square law (74 x 280 over 43 squared against 90 x 730 over 110 squared, 11.2 to
5.4: a thousand of Blitzes is level with about 700 of Grunts), so the vehicle side's doubled army is what it has to
spend the free fire against, and "Blowdeer [as heard] stays on Grunts for kind of a long time trying to make that
snowball happen". The vehicle side's own switch is the Incisor, 230 of reach, 820 health, 94 a second for 120 metal,
level with the Blitz per metal and 50 longer: a Blitz closing at 16 a second takes 3.1 seconds of fire, 40% of
itself, before it answers, "just a better unit than the Blitz"; and "on some maps, mash Pounder for the defensive
value" (315 of reach, 1,400 health, 106 a second for 220 metal, too slow at 40 to raid). None of this is in the
pool's cut, whose bot sides are Cortex Grunts in 15 of 16; the Grunt's reach may be part of why they win half the
mixed duels.

**When to depart from the line.** The line is the vehicle plant on Comet and the reading of his. Depart from the
plant on a map with triples and wind; from the reading when his second factory changes the race (a bot player's vehicle
plant at 6:19 ends the pin, an air plant on his surplus energy puts the solars at risk, L-gecko-bots-vs-vehicles); and
from the window when he has already hidden, since the surplus that cannot be spent at his cluster is still a surplus at
his spots and his constructors.

**The tests.** `run/metal_killtest.py openers` (2026-10-23), the pool half over the 80 Comet sides at OS 40+ by first
factory (the kind from the card's factory unit) and the second half over our 280 model-played duels with both first
factories read, ours from the record and his from the truth file. The kill-tests as named: had the bot sides lost most
of the mixed duels, the lab would be the result and the article "take vehicles"; they won 7 of 16 (13 of 24 over every
carded duel). Had the two openers' armies been level at 5:00, the energy mechanism would be wrong; the bot sides held
0.51 / 0.64 / 0.99 of the vehicle sides' on the same metal income with three solars more. Had the winner-against-loser
clocks held within an opener as they do across, his skew warning would be wrong; within the vehicle openers the winners
cross 20 s before the losers and within the bot openers at the same clock, against the 40 s of the pooled cut, so
M-pool-pressure's gap is mostly the openers' mix. Ours: BARb's lab is dice at one profile, and on Comet without a bonus
over fourteen minutes or more our vehicle opener won 32 of 91 against its bot lab and 35 of 54 against its vehicle
plant, holding 2.1 times its army from 4:00 to 6:00 in the first case and 1.5 to 1.7 in the second, with our army in
his half at the median 0 through 8:00 against the bot lab and 93 to 220 from 5:00 against the plant; its bot-lab self
stood 12 / 18 extractors at 8:00 / 12:00 to its plant self's 10 / 15 with half the army at 5:00 (378 to 691). Our own
bot-lab games are 80, all September's harness (the pianist, the one-decider and the commander series), 29 won and 0 of
8 on Comet against BARb, and say nothing about the lab. What the tests cannot do: the pool's cut is sixteen bot sides,
Cortex in fifteen; our cut has the opener on one side only; and no game of ours yet carries a turn that named his lab
and spent the window on it, which is the test that matters, when the episode index exists. The constructor adage of consider item 4 (`run/metal_killtest.py energy`, 2026-10-24, both
sides' constructors read): in the pool the winners stand one constructor ahead at 5:00 at the median (13 of 40 sides
ahead by one, 9 by two or more, 10 level, 8 behind) and the losers one behind; in our 242 model-played games the more
ahead the better against BARb (at 6:00, behind won 32 of 109, level 19 of 44, one ahead 18 of 37, two 14 of 22, three
or more 19 of 30), with no sign of his "four beats six", because BARb does not punish the extra constructor the way he
says he does against us (M-gecko-vs-us-comet); against its bot lab we stand behind in constructors in 75 of 95 games
at 6:00 (its 7 to our 5) and won 22 of those, against its vehicle plant ahead in 37 of 53 and won 26 of them.
