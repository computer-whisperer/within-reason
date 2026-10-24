<!--meta
id: D-raid-answer
layer: derived
premises: [D-metal-is-the-game, D-lead-convert-it, D-reach-nests, F-eco-extractors, F-eco-build, F-eco-consequences, F-fight-range, F-fight-sight, F-fight-consequences, F-unit-armllt, F-unit-armbeamer, F-unit-armrad, F-unit-armfav, F-unit-armflea, F-unit-armpw, F-unit-armflash, F-unit-armstump, F-unit-corak, F-unit-corfav, F-barb-how-it-decides, M-barb-hard-aggressive-comet, M-pool-pressure, M-pool-energy-and-power, M-gecko-alone-vs-barb, M-u6bkep, F-unit-armmine1, F-unit-armmine2, F-unit-armmine3]
lessons: [L-raid-naked-mexes, L-raid-keep-moving, L-raid-first-handful, L-raid-rovers, L-mex-half-the-map, L-sheet-raids, L-sheet-economy]
sources: {people: [thebluegecko, Irishstud14, the user], dates: [2026-09-27, 2026-09-28, 2026-10-03, 2026-10-23], relay: the lessons' chat lines; the user's review of 2026-09-27 (a Rover cannot repel a Pawn, the fault was the army's stationing) and of player-57 (a body turning for a lone Rover kills nothing); thebluegecko's unit sheet of 2026-10-23 (the Blitz, the Rover, the radar, the mines)}
status: live, checked (run/metal_killtest.py raids, 2026-10-23: the raid ledger of 236 model-played 1v1s, 7,078 extractors lost; correlations across our own games, and the ledger holds only the extractors that died, said so in the tests)
-->
### The raid answer

**The heuristic.** thebluegecko on the Blitz: two "can beat an LLT with flanking damage, making stopping them require
units and enough radar to see them coming. Or many more LLTs, which slows expantion"; on the radar: it "lets you see
oponent's units coming in time to block them, saving your mexes from raids". The user, on a Rover that sat by while a
Pawn burned three extractors: the Rover could not repel the Pawn, the fault was the army's stationing. The plain line:
a raider kills an extractor in three to seven seconds, so what saves it is already in reach when the raider arrives, a
turret within its 430, a picket within three seconds' drive, a radar that saw it twenty seconds out; what comes after
avenges, and the rebuild within the minute is the other half of the answer. The raid is a tax both sides pay, the
pros too; what is ours to set is the rate, the downtime, and what the tax buys on his side.

**The quantity.** Extractors lost a game-minute (the pool's winners lose one, its losers two and a half), the downtime
of each (50 metal plus 2.3 a second until the spot stands again), and what stood in reach when it died.

**The mechanism.**

- *The raider's clock.* An extractor (270 health) dies to a Pawn or a Blitz in 3.0 seconds, a Grunt in 3.6, a Tick in
  6.8, a Rover in 7.7. A Sentry (85 metal, 430 reach) kills a Tick in 0.4, a Rover in 0.7, a Pawn in 2.3, a Blitz in
  4.5, and fires 2.5 seconds before a Blitz can answer, so within its circle a lone raider dies before the second
  extractor; two Blitzes with flanking beat it (his sheet, and the reach article's arithmetic), four lose none. Outside
  the circle nothing arrives in time unless it already stands within about 300: a Blitz 900 away reaches a dead
  extractor.
- *Who catches whom.* A Rover (168) catches everything that raids; a Blitz (101) catches Pawns (87), Grunts (81) and
  anything slower, never a Tick (132) or a Rover; a Stout (75) catches a Mace. A Rover kills a Tick in 1.7 seconds and
  dies to a Pawn in 1.2, so the Rover is the Tick's answer and the Blitz the Pawn's. The chaser's job is mostly not the
  kill: BARb's squad leaves a target whose threat is near its own power (F-barb-how-it-decides; a defended extractor is
  skipped, a bare one dies), and a person's raider leaves when outmatched, which nothing here measures.
- *The warning.* A radar tower (60 metal) sees 2,100: a Blitz is 21 seconds out when it appears, a Pawn 24, a Tick
  16. A picket 600 from the spot arrives in 6 seconds, the body 1,500 away in 20 at Stout speed. The radar buys the
  picket's arrival, not the body's, and it is what the reach article's look before the last step reads.
- *The cost.* A loss is 50 metal, 500 energy and 2.3 a second of downtime: 188 at a one-minute rebuild, 326 at two,
  740 at five, the spot's whole output when it is never rebuilt. Our rebuilds come 52 / 102 / 190 seconds after the
  loss (quartiles), 285 metal a loss at the median, and 42 percent of lost spots are never rebuilt; the ladder's long
  losses lost 50 to 69 extractors each, 15,000 to 20,000 metal of them.
- *The tax, both sides.* Every side of the pool loses extractors all game, 1.5 to 2 a minute, the winners 1.0 and the
  losers 2.5, and re-takes 7.5 spots a game; light turrets by 8:00 are 7 / 10 / 13 for Armada (winners 11) and the
  first radar stands at about 3:00; thebluegecko alone against BARb puts a light turret and a radar at each outer
  spot. BARb's first soldier is in our half at 1:48 to 1:54, a Tick or a Rover, raiders leave in twos and threes on
  hard, and our first extractor dies at 2:46 / 3:15 / 4:27.
- *Our record.* 7,078 extractors lost in 236 games: 0.88 a game-minute in won games and 1.57 in lost, the pool's
  winners' and losers' rates; 9 / 15 / 25 a won game and 22 / 35 / 52 a lost one, the difference being the game's
  length, not the rate. Ten seconds before a loss nothing stood within reach in 24 percent, a turret within 700 in 52,
  soldiers within 900 and no turret in 24; the raiders paid (one dead within two minutes) in 59 percent, 62 with a
  turret near, 60 with soldiers, 50 with nothing: what stood near avenged, it did not save. The hands answered 68
  percent of the losses, seven seconds after first sight (median), and the kind of answer, a hunt by a few or the whole
  group, did not move the pay rate (59 to 66 percent). A third of the losses were to a lone raider, a third to four or
  more, which is an attack and lead.md's fight. We stood 4 / 7 / 10 light turrets by 8:00 against the pool's 7 / 10 /
  13 and had a radar by 4:00 in 49 games of 236. Games with ten or more turrets by 8:00 lost 9 extractors between 8:00
  and 16:00 against 12 with fewer than five, and won 48 percent against 36; games with a radar by 4:00 lost one fewer
  by 8:00 and three fewer by 16:00, and won 49 percent against 43.

**Consider, and why it may work.**

1. *A light turret as the next step after each outer pair, standing within 430 of both, never waiting on the bank.*
   Why: 85 metal against a loss that costs 285 at our median rebuild and the spot's whole output in the 42 percent
   never rebuilt; BARb's squads skip what is defended; the pool's winners stand eleven by 8:00 and we stand seven; and
   our games with ten lost three fewer in the mid-game than those with four. Half our losses had a turret "near": a
   turret 600 from the spot covers nothing of it, and the raider works the far side of its circle.
2. *A radar at the front by 3:00.* 60 metal for twenty seconds' warning, which is the picket's arrival time and the
   reach article's eyes. The pool's first radar is at 3:00, his is at each outer spot, and ours stood by 4:00 in one
   game in five; those games lost three fewer extractors by 16:00.
3. *The picket stands at the cluster and is a catcher.* A Rover or a Blitz within three seconds' drive of the spots it
   keeps, a Rover where Ticks come; a soldier 900 away saved nothing in our record (the raiders paid 60 percent with one
   near, 50 with none). Four or more raiders are not a picket's: that is an attack, and the body's fight.
4. *The body does not turn for a lone raider.* Why: the hands already answer in seven seconds and the raider pays three
   times in five; the body's turn buys the same pay at the price of lead.md's clocks (player-57's Rover; the +20 game's
   dozen detachments). The exception is the raider already inside the body's own reach, which is free.
5. *The rebuild is half the answer.* A constructor at the cluster puts the extractor back in 20 seconds and the loss
   costs 96 instead of 285; the pros re-take seven and a half spots a game; 42 percent of ours stayed empty. A spot lost
   twice (3 / 6 / 11 a game) is item 1 unpaid.
6. *Pay the tax back.* The pool's winners lose one a minute and take two and a half of his, with the first handful
   across the map by 3:00 (M-pool-pressure, L-raid-first-handful); a raid of ours that stops moving gives him the
   minute to answer (L-raid-keep-moving). The raid tax is level between sides that both raid; it is ruinous to the side
   that only receives.

7. *Mines where his raiders drive.* His sheet (L-sheet-raids): the light mine (7 metal) "good vs basic bots", the
   medium (25) "most common choice, many players do not use these because of the AMP it takes. place them where you
   expect oposing light and med tanks to drive", the heavy (50) "good vs T2 and med tanks, too costly to cover the map,
   but good for spots you expect the oponent to move through"; and "Mine use might be a palce where an AI has an edge
   over a human player due to being faster". Why it may work here: a raider's roads into a cluster are few, a medium mine
   is under a third of a Sentry's metal and draws nothing, and what stops people is the attention it takes, which the
   hands have. Unmeasured: no game of ours has laid one.

**Where it bends.** Against a person the raider leaves when outmatched and comes back, and the picket's standing is
the whole of the answer; against BARb its gate does the picket's work once a turret stands. Blitz pairs beat a
Sentry with flanking, so against Blitz raids the post is a Beamer or a Sentry with a picket beside it. On a map with
one pass the picket at the pass covers everything behind it. From the mid-game the raider is a Marauder or a Fatboy
(156 of our extractors died to Marauders) and the turret is nothing to it: that answer is the body's position and our
own tier 2. Air raids are the anti-air's and not this article's.

**When to depart from the line.** When four or more come: the fight, not the picket. When the raider is already inside
the body's reach: kill it, it is free. When the spot is on his side of the middle and uncovered: let it fall and
re-take it with the body, since a turret there is his the moment the body leaves.

**The tests.** `run/metal_killtest.py raids` (2026-10-23): `run/raid_ledger.py` over every model-played 1v1 of fourteen
minutes or more, one row an extractor lost (the party within 900 ten seconds before, what of ours stood within 900 and
700, the hands' first play against that party and its delay, whether a raider died within two minutes, whether and when
the spot was rebuilt), with each game's light turrets standing by 8:00 and its first radar. The numbers are in the
mechanism. Caveats: the ledger holds only the extractors that died, so "what stood near" and "paid" are conditional on
the loss and say nothing of the raids a turret or picket turned away; "within 700" is not "within reach"; "paid" is a
death within two minutes by anyone; and the turret and radar correlations run across games in which everything else
varied too. What a saved extractor looks like needs the raids that failed, which the truth file holds and nothing yet
reads.
