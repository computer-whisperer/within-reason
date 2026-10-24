<!--meta
id: D-reach-nests
layer: derived
premises: [D-metal-is-the-game, D-lead-convert-it, F-fight-range, F-fight-sight, F-fight-health, F-fight-accuracy, F-fight-big-weapons, F-fight-consequences, F-map-grid, F-unit-armllt, F-unit-armbeamer, F-unit-armhlt, F-unit-armguard, F-unit-armclaw, F-unit-armrad, F-unit-armfav, F-unit-armflash, F-unit-armstump, F-unit-armrock, F-unit-armfboy, M-barb-hard-aggressive-comet, M-pool-closing, M-pool-fights, F-unit-armflea, F-unit-armham]
lessons: [L-lead-together, L-fight-sight, L-raid-rovers, L-sheet-raids, L-sheet-economy, L-sheet-fighting]
sources: {people: [the user, thebluegecko, Irishstud14], dates: [2026-09-27, 2026-09-28, 2026-10-23], relay: the user's reviews of 2026-09-27/28 (never chase into unseen ground; go in as one body or not at all; a radar at the strip's edge by minute three), thebluegecko's unit sheet of 2026-10-23 (the Blitz, the Rover, the radar, the Gauntlet rows), Irishstud14's lines on arriving together; the old brief's odds model ("a turret counts about three times its metal, nine in ten") traced by the annotation study to a 2026-09-19 fit on the heuristic bot's games and re-derived here}
status: live, checked (run/metal_killtest.py reach, 2026-10-23: 5,977 engagements of 236 model-played 1v1s, their sight events and radar clocks; correlations across our own games, said so in the tests)
-->
### Reach, and the nest

**The heuristic.** The user, reviewing game 9 of the 3v1s: Bulls and Beamer nests out-range tier-1 tanks, so go in as
one body or not at all, and never chase into ground nothing of ours has seen. thebluegecko on the radar tower: it
"lets you see oponent's units coming in time to block them"; on the Blitz, two "can beat an LLT with flanking damage,
making stopping them require units and enough radar to see them coming". The plain line: a gun that reaches past ours
fires for free until we close, and the free damage is fixed by the gun and the seconds, not by how many we bring; a
body that arrives together pays it once, a stream pays it per unit, and a body that stands inside it pays it every
second for nothing. What nobody of ours has seen is priced at the body that walks into it.

**The quantity.** Reach as a toll: his reach less ours, over our speed, times his damage a second, per gun, paid on the
approach and again every second a unit stands inside his circle. Then sight: what of his is in our picture before the
body commits (a Stout sees 350, a Rover 635, a radar tower 2,100; the Gauntlet reaches 1,220 and shows itself at 455).
Not "the metal on the spot": the tests below retire that one.

**The mechanism.**

- *The toll, worked.* A Sentry (85 metal, 430 reach, 161 a second) fires 2.5 seconds at a closing Blitz before it can
  answer, 398 damage with the beam's falloff ignored, half the Blitz; a Beamer (190, 490, 266) fires 3.1 seconds, 816,
  more than a Blitz has; a Gauntlet (1,250, 1,220, 106) fires 11.6 seconds at a closing Stout, 1,228, two thirds of it.
  Standing inside the circle costs the same rate for nothing: a Beamer a Stout every seven seconds, a Gauntlet three and
  a half Stouts a minute, a line of ten Sentries 1,600 a second while the energy lasts. The toll is per gun and per
  second, so twenty units closing together pay what one unit pays; twenty arriving one by one pay it twenty times, and
  each arrives alone against the whole post.
- *The turret's weight.* By the square law (F-fight-consequences) a turret's weight is its damage a second times its
  health: a Sentry equals 1.2 Blitzes or 0.8 Stout, 1.6 to 2.2 times its metal; a Beamer 2.4 Blitzes or 1.6 Stouts,
  1.4 to 1.9 times; an Overwatch 0.7 to 1.0 times; a Gauntlet 0.2 to 0.3 times, a toll and not a weight. The old
  "three times its metal" is roughly a Sentry or Beamer against raiders arriving in a stream, and nothing else. On the
  arithmetic alone, with falloff and focus and no one behind the turrets: two Blitzes take a Sentry keeping 37 percent
  of their health (his sheet's claim), four keep 80; four Blitzes on a Beamer lose two and a half of themselves, eight
  lose one and a half; twelve Stouts on a Gauntlet with three Sentries and a Beamer lose two and the post falls in eight
  seconds; twelve on two Gauntlets, an Overwatch, two Beamers and four Sentries lose nine, twenty-four lose four, forty
  lose three. A nest falls to a body that outweighs it and walks in together.
- *What the record says about the spot.* Five seconds before the first death, the metal on the spot predicts nothing:
  in 3,296 fights with 300 or more of soldiers on each side, the side with more soldier metal lost less soldier metal in
  47 percent; a turret counted at its metal lifts that to 54, at three times to 60 (64 beyond two to one), far from
  "nine in ten". The weight moves the exchange, not the result: our share of the soldier metal lost runs from 48 percent
  when we outweigh him twice to 71 when he outweighs us twice. Fights at our base cost us 48 percent of what dies,
  fights in his half 66, at his base 65: the defender's edge is fifteen points, not a wall. Who keeps arriving and who
  stays in reach decide what the spot's metal does not.
- *What the record says about posts.* Our 729 assaults on his posts (his turrets 1,000 or more on the spot, our
  soldiers 500 or more, his side; 145 games) traded 1.4 of his metal for one of ours; without his soldiers behind the
  turrets 1.84, with them 1.30. The post itself lost half its turrets in one assault in five; what died was what the
  post covered, his solars, Sentries, extractors, advanced solars, construction turrets and constructors. The disasters
  (73 assaults, a tenth, a quarter of the metal those assaults cost) had his soldiers behind the turrets in 60 and his
  commander inside the circle in 27; a Gauntlet fired in two. Our killers across every engagement are his soldiers: Pawns,
  Centurions, Stouts, Blitzes and Maces 43 percent of our attributed deaths, Sentries 5, Fatboys 4. A body spread over
  300 or more traded 1.04 against 1.5 for a tighter one.
- *What the record says about sight.* His advanced factory entered our picture in 64 of the 165 games in which he built
  one, 4.7 minutes after its placement; in 57 of 75 won games and 7 of 90 lost. Our first radar stood at 5:18 (median),
  by 3:00 in 20 games of 236 and never in 79; no scout was built in 79. The Gauntlet fires at 1,220 on what his radar
  shows it while its own sight is 455: a body under its shells sees nothing without a Rover or a radar of its own.

**Consider, and why it may work.**

1. *A radar at the front by 3:00 and a Rover with every body.* 60 metal and 31. Why: 2,100 and 635 against the
   Stout's 350; what fires on us from beyond our sight is in the picture before the first shell, and the body's last
   step is onto ground it has seen. Our lost games saw his advanced factory in 8 percent; the pros' radar is in their
   opening (M-pool-energy-and-power); his sheet puts the radar second only to the extractor.
2. *A post is a toll, not a wall; the economy behind it is the target.* Why: the toll is paid once by a body, the
   post's turrets fell in a fifth of our assaults and still the assaults traded 1.4 to one, because what they killed
   was the solars and extractors the turrets stood beside. Walk through the post's circle to what it covers, or round
   it (a Sentry covers 430, a Beamer 490; the Gauntlet's 1,220 is his home, not his strip, on a map 8,192 wide), and
   never sit in front of it: the five minutes on a Beamer post (player-54) and the forty minutes in front of his line
   (the ladder) paid the toll without crossing.
3. *The look before the last step is for his soldiers, not his turrets.* Why: 60 of 73 disasters had his army behind
   the turrets and his commander near; a Gauntlet two. A Rover's look (635) from beyond the Beamer's 490 tells which of
   the two the post is; a post with his army behind it is lead.md's fight and waits for the surplus or is walked round;
   a post alone is item 2.
4. *Gather only before reach.* In the open a stream pays no toll and gathering pays lead.md's three clocks, so the
   standing order is go. Before a nest the stream pays per unit (a Beamer kills each Blitz that arrives alone; a
   Gauntlet takes two thirds of each Stout), so the body gathers in his half, in sight of the post (a Rover) and beyond
   its reach (500 from a Beamer, 1,300 from a Gauntlet), and takes the last step as one. Irishstud14's "less groups ...
   your armies keep not fighting together" and human-23's 33 Incisors dead in six seconds are the stream.
5. *Count the toll, not the turret.* Why: the turret's weight is one to two times its metal and its reach is what hurts;
   a body sent at a post should be able to pay the approach (guns times seconds times damage) and still outweigh what
   stands there, and a body that cannot does not go closer than its own reach. Against reach we cannot pay, reach of
   our own: a Rocketeer's 475 over the Sentry's 430 (his sheet: "good vs LLTs"), a Mauser's 820 over the Beamer and the
   Overwatch, bombers or a Basilica over a Gauntlet, which nothing tier 2 on the ground out-ranges; or his builders and
   his economy, which stand wherever his turrets are not.

6. *The sheet's three answers to a tower (L-sheet-fighting).* The Rocketeer: 475 of reach against the Sentry's 430,
   "good vs LLTs ... It is just the little bit of extra range that makes the rockets [work] with strong micro"; fifteen
   of them (1,800 metal, 620 a second together) stand 45 outside a Sentry's circle and kill it in a second without a
   shot back, and against fifteen Maces (380 of reach, 46 of speed to the Rocketeer's 50.7) keep the distance, "with
   good micro 15 rockets can beat 15 thugs", though "Thug/Mace have better stats per cost". The Gauntlet as ours:
   "very rare unit, good for any place you need to hold, where the oponent is making 3+ ACLTs": 1,250 metal, fifteen
   Sentries' worth, for a circle of 1,220 that outreaches every tier-1 gun, so where a place has to be held the toll of
   this article runs the other way. The Tick: "These can also be used for vision late game. It may also cost less to
   absorbe a high single target damage shot with a tick vs having the same shot hit a stronger unit. These become
   critical to T2 and T3 artillary battles": 21 metal that sees 600, and a shell meant for a Stout (225 metal) spent
   on it.

**Where it bends.** BARb puts a turret beside nearly every extractor (11 at 8:00, 21 at 12:00, 35 at 20:00), so against
it every raid is item 2 in small: two Blitzes per Sentry, four with a loss of none. A person's posts come with his
army behind them more often, and item 3 is the whole question. On a map with one pass (Great Divide) the post cannot be
walked round and the toll is paid by reach of our own or not at all. Under a Fatboy or a Gauntlet, a body that keeps
moving takes less than the arithmetic says (his sheet on the Shellshocker: "the shot takes too long to land so will miss
vs units"); a body that stands takes it all.

**When to depart from the line.** When the kill is one step away and the toll is a unit or two (the pool's closes are
walked through his turrets with the commander and every factory among the kills, M-pool-closing). When his army is
in the open and the post is beside it: the fight is lead.md's, and the turrets are the toll on where it is fought.
When our body cannot see and cannot wait: it goes round, not in.

**The tests.** `run/metal_killtest.py reach` (2026-10-23): every engagement of the 236 model-played 1v1s of fourteen
minutes or more as `run/analyze_match.py` finds them (deaths clustered in place and time; what stood within 900 of the
centre five seconds before the first), the record's sight events for his advanced factory, our radar's clock and the
scouts built. The odds table and the assault numbers are in the mechanism. Caveats: "before" is one sample five
seconds before the first death, so a body that arrived during the fight is counted on neither side, which is part of
why the spot predicts nothing and is itself the point; "lost" counts every unit and building; the assaults' 1.4 is
metal, and his solars are cheap to kill and cheap to replace. The disasters are 73 engagements in 47 games for the
episode index to read one by one; the pass does not say whether a body gathered or streamed, only how spread it stood
five seconds before.
