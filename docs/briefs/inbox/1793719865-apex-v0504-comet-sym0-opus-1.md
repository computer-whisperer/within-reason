# apex-v0504-comet-sym0-opus-1: the player's debrief

Match `run/matches/1793719865-apex-v0504-comet-sym0-opus-1/00`, build 3a54d1379c3a, against Apex v0.5.04 standard on Comet Catcher Remake 1.8: Win after 21.0 min (called by the referee); 110 turns. Debriefed by claude-opus-5 (effort low) in 43 s, in a shape of its own (DECIDED / NEXT PLAYER / WISHED was asked); since snapshot: 7-day +0.0 points, extra usage +0.00.

Debrief for the next player: Comet Catcher Remake 1.8, corner W (home B6, south-west), Armada arm, vs Apex v0.5.04 standard. We won at 21:00 when his commander came into sight at G2 and the whole army went for it. His base is the north-east cluster: bot lab at G1, later an advanced bot lab at G3, air plant and turret nests at F1/F2, a long-range cannon (Gauntlet, reach ~1220) at G1.

What worked, roughly in order.

The opening from the pool was fine and I would repeat it: two extractors, two solars, vehicle plant by ~0:35, third extractor, commander guarding/assisting the plant at low priority. Plant list: constructor, two Rovers, two Blitzes, then a constructor every three to four Blitzes. Apex opens a bot lab with Pawns and has single Pawns in our half by ~2:30, so Blitzes early are right, and the two Rovers across the map paid for themselves (found G1 by 3:18, killed two of his extractors before a Centurion caught one).

Energy is the first thing that breaks. At 1:35 I was making 80 and drawing 160 with the commander assisting; at 2:15 making 120 and drawing 240 with the store empty. Build four solars before the commander goes back to assisting, not two, or the plant crawls for a minute. Right after that it flips: by 2:47 energy was full at 1,450 and metal was at zero, so stop the commander's assist and put builders on extractors instead.

Expansion beat him comfortably: 10 extractors and 25 metal/s at 4:12 with 527 banked, against a pool median of 7. The bank is the signal to add a second vehicle plant from the commander immediately, ahead of a construction turret. From there I was never behind on economy — 6,655 army vs his ~600 at 11:24, ~14,000 by 16:43. Economy was not the problem in this game; converting it was.

Everything I lost, I lost to the same two mistakes, and they are what the next player should spend their turns on.

One: walking into turrets piecemeal. At 8:52 Blitzes fed into his Beamer+Sentry nest at G1 for about 1,300 metal in a minute. At 16:53 only 8 of 36 soldiers were near G1 while the rest streamed in. At 18:48 one group lost ~4,300 in a minute thrashing between the F1 and F2 nests under the G1 Gauntlet. The cure that finally worked: name one target that sits outside the Gauntlet's 1220 (the F2 west nest), take it, hold at a rally point just outside reach, gather everything there, and only then fire a timed signal at the next thing. Shellshockers/Whistlers to outrange Beamers is the right article; walking Stouts into a laser nest is not.

Two: the army shredding into pieces. Repeatedly: groups split into four splinters to chase one Pawn (4:22), a group went home and came back mid-march (9:51), a 27-soldier 4,000-metal group roved scattered over both halves doing nothing (16:43), and three separate bodies held while one advanced alone and read its odds as 1.0:1 (17:38). What fixed it was merging into one named group rather than telling three groups to meet: once they were literally group_L1 the odds and the walk were one body. Keep exactly one main body plus a small set of Blitzes as dedicated raider hunters, and answer a lone Tick or Pawn at an outer spot with a Sentry built there, not with tank chases.

Watch out for two mechanical traps that cost me minutes. A timed go signal attached to a conditional walk can simply not fire — at 20:00 the push stayed put because the walk to F1 was rated 0.30; I had to rewrite the orders so the push did not hang on the conditional. And a group that has arrived can keep reading "advancing to X" forever; rewrite its route to start from the next stop once it is there. Also, the gather sentence kept pulling the body back to the rally after it advanced — delete the gather line when you fire the push.

Late, he had a lone Banshee/Stormbringer picking off outer extractors; two Nettles at the exposed spots handled it. Keep constructors making extractors the whole game, he does not contest the middle or his own south, so take both.

If I played it again: same opening, four solars not two, second plant at 4:00, then one body of Stouts with three or four Shellshockers behind it, one target at a time, each chosen outside the Gauntlet's reach, with the whole group merged under one name. His commander sitting at G2 with construction turrets is the winning condition — if it comes into sight, send everything and say no detachments explicitly.
