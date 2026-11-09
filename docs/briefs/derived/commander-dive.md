<!--meta
id: D-commander-dive
layer: derived
premises: [D-reach-nests, D-raid-answer, D-surprise-weapons, D-lead-convert-it, F-fight-big-weapons, F-fight-range, F-fight-death, F-fight-consequences, F-barb-how-it-decides, F-unit-armcom, F-unit-corcom, F-unit-legcom, F-unit-corthud, F-unit-armham, F-unit-corraid, F-unit-armstump, F-unit-armrock, F-unit-corstorm, F-unit-corgator, F-unit-armflash, F-unit-armpw, F-unit-armkam, F-unit-armsnipe, F-unit-armliche, F-unit-corllt, M-other-gecko-people, M-pool-fights, M-format-1v1, M-format-team-small, M-format-8v8]
lessons: [L-snipe-commander]
sources: {people: [the user], dates: [2026-11-08], relay: the user after watching the replay of the 1v2 against two Apex of 2026-11-08 (run/matches/1794187275-apex_2v1), on my reading that the body circled his commander at hold position ("The formation hold position is not a problem -- trying to dive on the commander is. Let's also write a brief derived article on what it takes to dive and kill a defensively microed commander"); the pool read by run/replays/commander_dives.py; the claims K-army-a-commander-dies-to-what-out-reaches-its-laser, K-army-raiders-kill-a-commander-at-twenty-five-and-rarely-at-ten and K-player-ten-incisors-were-sent-onto-his-commander in docs/knowledge/army.md}
status: live, unchecked in a game of ours (written 2026-11-08 from one game of ours and the commander deaths of the replayed public games, 88 in duels and 1,769 in 8v8s; the sums in it are sums from the unit files, not engine measurements: the duel harness has no commander that fires its D-gun)
-->
### His commander: what a kill takes

**The heuristic.** The user, after the 1v2 against two Apex: the fault was not how the body stood, "trying to dive on
the commander is". The player had sent ten Incisors onto a Cortex commander building extractors at E6. Three reached
it first and seven half a minute later; for thirty seconds the nearest stood inside its laser and outside their own
guns and took nothing off it; then they closed and eight died in seven seconds, for a quarter of its health and
three of his buildings. The plain line: a commander that fires its D-gun is not a target for raiders. It dies to
what out-reaches its laser, to aircraft, or to a weight that all arrives in the same second and is far above what
the sums ask. Otherwise go round it and kill what it builds.

**The quantity.** Three things decide it before a unit moves. Which side of 300 our guns reach from: its laser
reaches 300, its D-gun 250 (Cortex 262). How far it must walk to cover, since it walks at 37.5 and nearly everything
we own is faster: the walk in elmos times our damage a second from beyond 300 has to pass 140,000 (3,700 health
times 37.5). And what its death buys: in a duel the game; with several seats on his side, nothing but the unit.

**The mechanism.**

- *What it is.* 3,700 health, mending 5 a second; speed 37.5; sight 450. A laser of 187 a second at 300 (Legion's
  219). The D-gun at 250 or 262: one shot every 0.9 seconds for 500 energy, which kills what it touches and
  everything else on its line. It cannot be stunned, captured, reclaimed or carried off by a transport, and a
  nuclear missile on it does 2,500 to 3,000, which a whole one lives through. A constructor mends it at about 5 a
  second, so nothing heals it in a fight; what saves it is the ground between it and its towers.
- *The raider's sum.* A raider fires from 180 to 230, inside the D-gun. If N arrive in the same second and it
  kills one a shot, they deal 0.9 seconds of N minus 1, then N minus 2, and so on: N times N minus 1 has to pass
  8,200 over one raider's damage a second. Ten Incisors, eleven Pawns or Blitzes, and the last one alive fires the
  last shot. That is the floor with no laser, no tower beside it, nobody dead on the way in and never two on one
  line; a shot down a file takes three to seven (F-barb-how-it-decides). It costs him about 4,500 energy.
- *What people's raiders do.* The replayed games' raider parties at a commander, counting those that killed it
  with a raider's shot and those that lost three or more to it and did not. Duels: nine or fewer killed it 0 times
  of 15, ten to fourteen 3 of 11, fifteen to twenty-four 3 of 8, twenty-five and over 6 of 6; the smallest that did
  it was twelve. 8v8s: nine or fewer 4 of 268, ten to fourteen 1 of 55, fifteen to twenty-four 11 of 43,
  twenty-five and over 23 of 43. The parties that killed one in a duel had brought 23 units and 2,250 metal at the
  median and lost 1,230 of it to the commander in its last thirty seconds. Our ten Incisors were the size that
  fails three times in four against people in a duel and nearly always in a team game.
- *What kills it.* Of 42 duel commanders killed by a unit, the last hit came from a gun that reaches past 300 in
  25 and from a raider's reach in 16. In the 8v8s it is 959 of 1,110 from past 300, the Thug first (90), then the
  Razorback, the Sheldon, the Mace, the Tzar, the Sharpshooter; raiders 8 percent, aircraft 8 percent. A Thug or a
  Mace fires from 380 and walks at 45, a Brute or a Stout from 350 at 72 to 75, a Rocketeer or an Aggravator from
  475 at 48 to 51: all out-reach the laser and outrun the commander, so it can neither hit them nor leave. Ten
  Thugs are 600 a second: six seconds of fire for 1,400 metal and no D-gun shot fired.
- *The walk to cover.* A commander that is being played walks to its towers, its plant and its army, and a light
  tower out-reaches a Thug (430 to 380). At 37.5 it needs 27 seconds to cover 1,000. Five Thugs or four Brutes
  kill it if cover is 500 away, twelve Thugs if it is 200; double that for shots that miss a walking target, which
  nothing here has measured. Of the duel commanders killed, 34 of 42 died more than 800 from their start, at a
  median of 2,300: the commander that dies is the one that walked forward. In the duel dives a commander lived
  through it had its own towers within 700 in seven of ten, and it stood alone in one of six.
- *Its energy.* Each shot is 500. A side at zero energy has no D-gun (F-fight-consequences); a commander far from
  its base still draws on the whole store. We cannot see his store. We can count his shots.
- *Its death.* It bursts for 5,000 at the centre, falling to nothing at 350: a Pawn within 320 dies, an Incisor or
  a Blitz within 290, a Thug within 270, a Brute within 210. Raiders that kill it at their own reach die with it.
  Thugs at 380 and Brutes at 350 are not touched.
- *From the air.* Eight Banshees kill it in twelve seconds and it cannot build anti-air under them
  (D-surprise-weapons); the Liche's one bomb is 5,625. In the duels aircraft killed 4 of the 42 and no dive by air
  that cost three aircraft failed; in the 8v8s 93 kills against 11 failures.
- *What the kill buys.* A side loses when it has no commander left. In a duel that is the game, and people pay a
  thousand metal for it. With two or more seats on his side, the seat whose commander dies plays on with
  everything else it owns: the kill removes one D-gun, 300 of build power and a unit that fights like 400 metal of
  soldiers. In the 1v2 the player wrote "removing one of them would change the game"; it would not have removed him.

**Consider, and why it may work.**

1. *Say what the kill buys before ordering it.* The last commander of his side: the game, worth a real price. One
   of several: price it as the unit, and the answer is nearly always no.
2. *Send what reaches past 300.* Cortex: Thugs, Brutes, Aggravators. Armada: Maces, Stouts, Rocketeers. They hold
   at their own reach, outside its laser, and follow it as it walks. Why: it is how seven commanders in eight die
   in people's team games, and the D-gun is never in it.
3. *Count the walk.* Where are his nearest towers and his army, and is our damage a second times that distance
   past 140,000 with room to spare? A commander 1,500 forward building an extractor is the case that pays; one
   beside a Guard is a nest (D-reach-nests), and the Guard dies first or the party leaves.
4. *Raiders only as the whole body, twenty-five or none.* All at one moment, from two sides, never in file, and
   only when the kill is the game. Why: under fifteen, people's raiders killed it 3 times in 26 in duels, and ours
   died at ten.
5. *From the air when he shows no anti-air:* the Banshee flight of D-surprise-weapons, or bombers on a commander
   that stands to build.
6. *Otherwise go round it.* Keep raiders 350 from it, outside the laser, and kill the extractors and the tower it
   is building as they go up: that raid's whole take was two extractors and a Guard, 190 metal, and it had them
   for two Incisors before the body closed on the commander. Mark where it is; a commander forward is his build
   power away from his base.
7. *Write the order so the hands can read it.* "Attack his commander with the whole body" names no reach. Name the
   group of Thugs or Brutes and the party; forbid the raider groups the place by name.

**Where it bends.** (1) BARb's and Apex's commanders fire the D-gun at once and every time; a person sometimes
does not, and a person who is being raided elsewhere may not be looking. (2) A commander whose side is stalling on
energy has a laser and nothing else: five raiders beat a laser. (3) Legion's commander fires at aircraft. (4) The
sea: its laser under water is 200 a shot each second and the D-gun fires there too. (5) Past minute twenty its
killers are tier 2 and tier 3 (the Tzar, the Sharpshooter's two shots from 900, the Razorback), and the sum is
theirs, not the Thug's.

**When to depart from the line.** When it is the last thing he has and the army is already there: finish it with
whatever stands, at any price. When our own commander is the one walking forward: every sentence above is his
plan for us (D-raid-answer).

**How it fails.** *The trickle* (three, then seven a minute later). *The file* (one shot, five dead). *Standing
inside 300 and outside our own reach* (thirty seconds of his laser for nothing). *Ten where twenty-five were
needed.* *The dive at one of two commanders,* for a seat that plays on. *The kill made from inside 240,* which
takes the killers. *A route's last stop set on his start* (four Rovers, one a second).

**The tests.** run/replays/commander_dives.py over every replayed public game with records, each game once: 88
commander deaths and 42 dives lived through in duels, 1,769 and 1,039 in 8v8s, by last hit and by what stood
within 700. It sees a failed dive only when three or more units died to the commander, so the failures are a
floor, and it cannot tell a D-gun kill from a laser kill. Our one game is read from our seat's record. The sums
(the raider's count, the walk, the blast radii) are from the unit files and are not measured in the engine: the
duel harness has no commander that fires its D-gun. Whether the hands hold a body at its own reach against a
commander that walks is not checked; in the one game the Incisors stood outside theirs. No game of ours has been
played on this article.
