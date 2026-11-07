<!--meta
id: D-defence-hold
layer: derived
premises: [D-metal-is-the-game, D-lead-convert-it, D-reach-nests, D-raid-answer, D-energy-second, D-tier2-sprint, F-fight-range, F-fight-sight, F-fight-health, F-fight-big-weapons, F-fight-consequences, F-eco-energy, F-unit-armllt, F-unit-corllt, F-unit-leglht, F-unit-armbeamer, F-unit-corhllt, F-unit-corexp, F-unit-armhlt, F-unit-corhlt, F-unit-armclaw, F-unit-cormaw, F-unit-armpb, F-unit-corvipe, F-unit-legapopupdef, F-unit-armanni, F-unit-cordoom, F-unit-legbastion, F-unit-armguard, F-unit-corpun, F-unit-armamb, F-unit-cortoast, F-unit-armrl, F-unit-corrl, F-unit-armflak, F-unit-leglraa, F-unit-armamd, F-unit-corfmd, F-unit-legabm, F-unit-armsilo, F-unit-corgate, F-unit-armdrag, F-unit-armrock, F-unit-corstorm, F-unit-armflash, F-unit-corgator, F-unit-armstump, M-pool-pressure, M-pool-fights, M-format-8v8, M-barb-bonus, M-barb-team, M-lobby-5v5, M-gecko-team]
lessons: [L-other-infern8-defences, L-other-u6bkep-nests-and-air, L-gecko-nuke-surprise, L-sheet-fighting]
sources: {people: [Infern8, apexearth, thebluegecko, u6bkep, the user], dates: [2026-09-27, 2026-10-18, 2026-10-23, 2026-11-06], relay: Infern8 watching our raiders die on a Beamer (L-other-infern8-defences); apexearth in the 5v5 ("mass bastions does hold off enemies pretty well", 49:59); the user's observation that opened the survey (2026-11-06: we use stationary attacking structures far less than our opponents; people hold off far more BARb seats than they have by damage efficiency while they out-scale), his framing of the three cases (the duel's perimeter, the 8v8's cross-section with no way round, the one seat against two); the survey docs/studies/2026-11-06-static-defence-survey.md and its claims in docs/knowledge/defence.md}
status: in the manifest since 2026-11-06 (the user, on the draft: "Good. Let's enable this article, then run a BARb 1v1, then an 8v8"); the numbers are the survey's (run/replays/defence_duels.py, run/defence_pool.py, run/defence_census.py on 2026-11-06) and the unit files on game source 54199a0d
-->
### The hold: what a tower is for, and where it earns

**The heuristic.** Infern8, watching our raiders run into a Beamer (L-other-infern8-defences): "defensive structures
(like the beamer in the north) are generally stronger per metal than mobile units ... attacking them effectively
requires either long-range units or overwhelming firepower". apexearth, in the lobby 5v5, on the player who held the
north-east for thirty minutes (M-lobby-5v5): "mass bastions does hold off enemies pretty well". The user, opening
the survey: the people who hold off and then roll over far more BARb seats than they have do it with structures,
by damage efficiency, while their economy and tier catch up. The plain line: a tower is a gun that never leaves the
place it is bought for, so it is worth exactly what comes inside its reach there; people keep a sixth of their
soldiers' metal in towers from the third minute on and put them where the enemy comes, and theirs earn their metal
back; ours stand beside extractors by rule and earn half. Reach.md says how to pay a nest's toll; this says how to
collect one.

**The quantity.** Two numbers a tower has and a soldier has not: it does not retreat, and it does not go where the
fight is not. So its worth is the metal that walks inside its reach where it stands, over the game, against its
price and its energy. Counted by last hit in the record (a floor: it under-credits what the tower softened and
something else finished): the people's defence kills 0.9 to 1.1 of its own metal in our replayed games, 0.6 in the
high-OS duels, 2.8 for the eight people who held eight BARbs at +100 for 49 minutes; ours 0.3 in duels and 0.7 in
team games. Not "stronger per metal": damage dealt per metal by units that died runs 7.4 for ground defence against
6.2 to 6.5 for soldiers, a seventh more, and a Sentry is a Pawn's damage per metal on a Stout's health per metal.
The multiple is in the reach and the standing still, cashed only by what comes to it.

**The mechanism.**

- *What a tower is, in numbers.* The Sentry and the Guard (85 and 90 metal): 161 damage a second at 430, 620 health,
  20 energy a shot every 0.47 s (43 a second firing). The Beamer and Twin Guard (190 and 195): 266 and 321 a second at
  490, 1,430 and 1,670 health, 60 energy a second firing. The Overwatch and Warden (440, 480): 215 a second at 620,
  2,600 health, 42 energy a second. The Exploiter (Cortex, 240): an extractor with a Guard's gun. The pop-ups: Dragon's
  Claw and Maw at tier 1 (340, 290; 430 and 410 of reach; the Maw's flame hits everything in a cone), Pit Bull, Scorpion
  and Chimera at tier 2 (680 to 750; 730 of reach; 412 to 465 a second; cloaked and closed until they fire, taking no
  damage closed). The plasma towers: Gauntlet and Agitator at tier 1 (1,250; 1,220 of reach; 106 a second, area),
  Rattlesnake and Persecutor at tier 2 (2,500; 1,380). The fortresses: Pulsar (3,500; 1,400; 10,800 a shot every 9.9 s
  for 1,500 energy), Bulwark (3,000; 950; 1,513 a second), Bastion (4,200; 1,100 of reach; a 3 s sweep of up to 3,875;
  12,000 health). Walls: Dragon's Teeth 8 metal for 2,800 health, the Fortification Wall 40 for 8,900, no gun; a
  tier-1 unit neither paths nor shoots low through them. Anti-air: Nettle and Thistle (80; 765), Ferret and SAM
  (330; 950), Chainsaw and Eradicator (750; 1,200), flak (820; 850), the Xyston (1,600; 2,000). Against soldiers for
  the same two numbers: Pawn 167 damage a second per 100 metal and 6.9 health a metal at 180 of reach, Grunt 172 and
  6.5 at 215, Blitz 82 and 6.6, Stout 36 and 8.0 at 350, Rocketeer and Aggravator 34 to 38 at 475. The Sentry is
  189 and 7.3 at 430.
- *The edge is reach, and reach is a bet.* A Sentry fires 2.5 s at a closing Blitz before it can answer and the
  Blitz pays 398 of its 730 (reach.md's toll); a Beamer 816, more than a Blitz has. Everything that reaches 435 or
  more fires on a Sentry for free: the Rocketeer and Aggravator at 475, the Whistler and Lasher at 550 and 575,
  every artillery; the Beamer's 490 is under the skirmishers too; the Overwatch's 620 is over every tier-1 ground
  gun but artillery. So a tower is a bet that what comes is what it out-reaches, and in a duel the bet is good:
  by metal, what the high-OS duellists' towers kill is Incisors and Blitzes first (0.42 million each over 527
  games), then Brutes, Grunts, Stouts, Pounders and Pawns, and 5:00 to 20:00 is when they kill it, the same minutes
  they are built in. What kills a tower is the same raiders in numbers, the line tanks, and shots from out of sight
  (artillery and beams: half of 15,000 tower deaths name no killer).
- *The tower fires on radar, and the big ones bring their own.* A gun may fire at anything the side holds in sight
  or on radar (F-fight-sight), and the tier-2 towers carry radar past their reach: the Bastion sees 780, has radar
  to 1,200 and reaches 1,100; the Pulsar 780, 1,500 and 1,400. A Bastion line kills what walks within 1,100 whether
  or not anything of his sees it; the 5v5's Legion seat lost 62,000 of metal to attackers it never saw and 16,000 to
  towers in sight. A body under such a line is not blind because his towers are, and nothing of ours that reaches
  less than 1,100 lives there.
- *The energy.* A tower pays per shot and holds the shot when the store cannot: ten Sentries firing are 430 energy a
  second, ten Beamers 600, a Pulsar 150 a second on average and 1,500 at once, a Bastion 167, a Xyston 1,800 a rail.
  A tower line is an energy bill that comes due exactly when the line is needed, and a stalled store is a silent
  line (energy.md). The Cacophony, the pop-ups and the plasma towers fire for nothing.
- *Where people put them.* Of 23,700 ground defences in 527 high-OS duels, 3,840 stood within 1,500 of the start,
  15,800 in the own half beyond that, at the extractors and the lanes, and 4,085 nearer his start than theirs: two
  in three are outpost towers, one in six is forward. Per side: 14.9 Sentries and 3.3 Beamers (Armada); 14.1 Guards,
  5.1 Exploiters and 3.7 Twin Guards (Cortex: the armed extractor is the outpost tower in 61 percent of Cortex sides,
  five where it is); 28 Pharos (Legion); 0.4 Pit Bulls, 0.5 Scorpions; the tier-1 plasma tower in 16 sides of 1,054,
  tier-2 defence in 110. The first tower goes up at 3:20 (median), 255 metal of towers stand by 5:00 and 1,295 by
  10:00; the share holds at 8 percent of everything standing from 12:00 to 30:00, winners and losers within a
  point. Winners lose 550 of tower metal over a game, losers 2,100: the share is not what wins, losing the towers
  is what loses.
- *The team game is different in kind.* Each seat holds a cross-section of the map and the fight is over the same
  ground for forty minutes (M-format-8v8); early harassment cannot be leveraged because there is no way round the
  contended ground (the user, 2026-11-06). There the tower's bet is certain: his trickle comes to the same place
  every minute, so a gun bought for that place fires all game. The people in the public 8v8 against eight BARbs at
  +100 held 16 percent of their soldiers' metal in towers and got 2.8 back; four seats of the eight built 16 to 28
  Pulsars each from 14:00 to 24:00, standing 2,000 to 2,800 forward of their starts on the lane, after a Beamer
  line at 2:00 to 4:00 at the base's edge, and those four seats' towers returned 1.4 to 6.8 of their metal; the
  best seat (OS 38) built 4,240 of towers and 729,000 of army. The BARbs' towers all stood at home and died to the
  Pulsars' reach. In the people's team games in our own replays (OS 35+, 132 sides) the share is 15 percent and the
  yield 1.1; ours 9 percent and 0.7.
- *The asymmetric game.* One seat against two BARbs or two weak people: the two arrive piecemeal and from known
  directions, which is the tower's bet made certain again, and the seat's economy and tier run while the towers
  hold (the user's reading; in the public record a dozen such wins in 1,200 games, unread). The duel has none of
  this: its perimeter is wide and his units choose where to come, so the duellists' tower share is the same sixth
  and its yield lower (0.6), the towers earning where the outer pairs are raided and nowhere else.
- *What ends a turtle.* A line of tier-2 and tier-3 towers under shields and anti-nukes is not broken by units
  that reach less than it does; the 5v5's north-east (13 and more Bastions from 27:00, Soteria shields, an Astraeus
  at 3,100, eight anti-nukes) fell to 23 allied silos. An anti-nuke stocks one interceptor per 90 s and covers
  2,000 (F-fight-big-weapons); two silos fired together into one circle pass one anti-nuke with one interceptor
  stocked, and a silo stocks one missile per 120 to 180 s. A shield stops cannon shells and nothing else: Thugs,
  Stouts, Fatboys, Mausers and artillery die on it, rockets, lasers, heat rays, bombs and the nuke pass, and the
  generator (3,550 health, 273 of sight) is the target, not the bubble.
- *Our record.* The flagship's 13,000 towers over 609 sides: 11,550 Sentries, 1,348 Guards, 835 Nettles, 167 Beamers,
  122 Twin Guards, no tier-2 defence; 9 to 10 percent of its soldiers' metal, placed beside extractors (the raid
  article's item 1, which is right as far as it goes), yielding 0.3 in duels and 0.7 in team games. The same towers
  as the people's, half the share, and at the places the raiders do not come.

**Consider, and why it may work.**

1. *A sixth, from the third minute, and never more than that while the fight is open.* Why: it is what every pool
   does at every level and in every size of game, and a tower bought while his army is in the open is metal that
   does not walk to the fight (lead.md). The share is the budget, not the plan; the plan is item 2.
2. *Buy the tower for the place the enemy comes to, not for the thing it stands beside.* The hold point in front of
   the base, the lane's choke, the outer pair raided last game, the ground his trickle crossed this minute: a tower
   there fires all game; a tower beside an extractor nobody raids earns nothing and dies to the first Rocketeer.
   Why: the yield gap between the people's towers and ours is placement, not count or type. The raid article's
   Sentry at each outer pair stays, as the raid answer; the hold's towers are these.
3. *One Beamer or Twin Guard per two or three light towers, where his line units have shown.* Why: it is the
   people's ratio; the heavy laser out-reaches every tier-1 unit but the skirmishers and takes three Blitzes to
   kill, and its 60 energy a second is one solar. Cortex puts the gun on the extractor (the Exploiter): an outpost
   that is its own tower at 240.
4. *Past the first tier-2 bank in a lane game: the long gun forward.* Pulsars (or Bulwarks, Bastions) 2,000 to
   2,800 out on the lane, after the base's Beamer line and not instead of it, with anti-air beside them and energy
   for them (1,500 a shot). Why: his trickle arrives at the same ground every minute and nothing tier 1 or tier 2
   on the ground out-reaches 1,400; the pilot's four seats held eight +100 BARbs with it for forty minutes while the
   best seat spent on army. The tower fires on radar: a radar tower beside the line is what gives it its reach.
5. *The pop-up at tier 2 where a raid or a push arrives, not at home.* Why: it takes no damage closed, costs no
   energy to fire, and 730 of reach at 412 a second is the Beamer's job done to tier-2 raiders.
6. *Energy before towers.* A line is bought with the generators that will feed it: count the firing bill (item
   "The energy") against the income before the third tower, and never a line on a store that stalls (energy.md).
7. *Walls where the lane is narrow.* Dragon's Teeth at 8 metal for 2,800 health funnel tier-1 units and stop low
   shots; a funnel that puts the trickle inside the towers' reach for longer is the cheapest reach in the game.
   Unmeasured in our record (the knowledge file says so); the people's walls are for the replay batch to read.
8. *Against a turtle, silos in numbers and the rockets that pass shields, not the army.* Why: an anti-nuke holds one
   interceptor per 90 s; the 5v5 ended by 23 silos against eight anti-nukes and by nothing else in thirty minutes;
   a shielded nest is shelled with rockets, lasers and bombs, and the generator is the target.
9. *Count a tower at its reach, not at three times its metal.* Why: the three is reach.md's fight-on-the-spot
   number for a Sentry against a stream of raiders and nothing else; a tower that out-reaches the body is a toll
   until something that out-reaches it arrives, and a tower the body out-reaches is a wall of 620 health.

**Where it bends.** (1) The duel: the perimeter is wide and he chooses the place, so the towers earn at the outer
pairs only and item 4 does not apply before the game is decided; the duellists' towers get 0.6 back and still every
one of them builds them (the raid answer, not the hold). (2) BARb's own towers: it puts one beside nearly every
extractor and, with a bonus, a static line and Pulsars past 25:00 (M-barb-bonus); against it item 2 is reach.md's
walk-round and the economy behind the line, and item 8 is ours to do first. (3) Energy-poor maps: a Beamer line
on wind is a line that fires in gusts; the Cacophony and the pop-ups fire for nothing. (4) The lobby options: a
lobby that runs `experimentalshields` makes every shot bounce off a shield (the start script says); then only the
generator is the answer. (5) Air: no ground tower answers bombers; the hold's anti-air is the air article's.

**When to depart from the line.** When his army is in the open and ours outweighs it: the fight is lead.md's and
every tower bought that minute is a soldier not there. When the hold point moves (a lane taken, an ally fallen):
the towers do not move with it, so the budget goes to the new place and the old towers are written off, not
defended. When his reach has passed ours (a Pulsar or an Astraeus seen): the towers under it are gone already, and
the answer is item 8 or the air.

**How it fails.** *Beside the extractor* (ours: half the people's share, half the yield, every game). *The line
without the energy* (a Beamer line on a stalling store). *The line without the radar* (a Pulsar that sees 780).
*Bought in the open fight* (the Glitters 8v8s: towers at home while his 2.6 times the army stood in our half,
M-barb-team). *Walked into* (reach.md's toll: 94 tanks on 15 Bulls, 6,800 of Blitzes in a Beamer camp).

**The tests.** The survey's pools (docs/studies/2026-11-06-static-defence-survey.md; the claims K-def-* in
docs/knowledge/defence.md): 1,054 sides of public duels above OS 40 with the engine's own defence value every half
minute and every kill's last hitter; 2,313 sides of people in 355 games replayed through our pipeline, split by OS
and by game size; the flagship's 609 sides and the heuristic bots' 1,541 games; one public 8-against-8 at +100 and
the lobby 5v5 read two-sidedly. The yield is by last hit, a floor; the people's team games are the lobby games we
played in, so the people's towers there faced us and Apex, not BARb; the forward-Pulsar hold is one game's reading
until the replay batch (twenty public 8v8s against bonused BARb) confirms it; walls are unmeasured. The mechanics
(radar-fire, the towers' own radar, the shield's interception bit, the anti-nuke's stock) are read from the engine
and the unit files and cited there.
