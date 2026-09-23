# shell-1-hard-aggressive: the player's debrief

Match `run/matches/1790172269-shell-1-hard-aggressive/00`, commit bc46db0, BARb hard_aggressive on Comet Catcher Remake 1.8: Loss after 33.6 min, 141 turns. Debriefed by claude-opus-5-5 (effort low) in 20 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief: shell-1-hard-aggressive, Comet Catcher, arm, corner W vs BARb hard_aggressive (loss, 33.6 min)

## What happened, in one paragraph

The opening and the middle game went well, and the game was lost after that. By 17–20 minutes we had 31–32 extractors against about 6, an army of 8–12k against 2–4k seen, and we had killed their Advanced Bot Lab. We never converted that lead because we never built tier 2. The Advanced Vehicle Plant frames were started four times and abandoned at 0% each time, because two Stout plants took every unit of metal. Their tier 2 and tier 3 units (Hounds, Centurions, Razorback, Fatboy, Vanguard) came online around 22–30 minutes. From there they out-ranged and ground down roughly 75 Stouts in piecemeal fights, and then walked into our base.

## Do this

1. **Open with the players' order. It works.** Commander: mex at 45 and 50, three solars, the plant, two solars, then out to 43/52/38/39 with turrets and a radar. Plant: Rover, two Masons, then Blitzes. We had 11 extractors at 5:00 and 20 at 10:48.
2. **Put a turret on every strip extractor from the start.** Their early raiders are single Pawns and Ticks, and Blitzes cannot catch Ticks. The spot_36 extractor died twice to one Pawn before 4:05, and Ticks were still eating bare extractors at 29 minutes. The constructors' build lists should carry the turret with the extractor.
3. **Tell the Blitzes explicitly to chase lone raiders.** The standing order has to say that any Blitz group near a Tick, Rover or Pawn at our extractors sends one or two after it at once. The vaguer wording left a Blitz standing 860 away.
4. **Start tier 2 by about 14 minutes, and finish it by 18.** When you order the AVP, pause one Stout plant or cap its queue until the frame is up. Watch `labs` in the picture: at 20:02 it read 1, which meant the "duplicate" AVP I cancelled at 19:13 had been the only one. Check labs=2 the turn after you order the plant, and every few turns until it stands. If metal usage is near zero while the bank sits above 1000, something you ordered is not being built.
5. **Turn the lead into a kill before 20 minutes.** Their base sits at G2 (Advanced Bot Lab around 6248, 888), and their commander roams F1–G3. The north-edge route (spot_39 → 13 → 8 → 11 → 6 → 10 → 5) reached it. One 32-Stout push killed the lab, a Rattlesnake and snipers, then died. The 53-Stout push at 21 minutes died at the E3/F2 nest, which had beamers, LLTs, walls, snipers and five Lazarus rezzing. A pure Stout ball cannot break a fortified nest. Bring range: Shellshockers at 710 or Mausers at 820, against beamers at 490 and LLTs at 430. Put a radar or spotter on the nest first.

## Don't do this

- **Don't chase a Razorback with Stouts.** It has 475 range and outruns them, so it kites. Chasing it cost about 20 Stouts at spot_24 and about 25 more into E3. Hold ground with radar and artillery and let it come to you.
- **Don't stand still under unseen fire.** A Fatboy (700 range) killed 9 Stouts standing at spot_26. A Vanguard (1450 range) reaches anywhere in the middle, so retreating does not escape it. Charging the Fatboy did kill it, but it cost about 20 Stouts. Against a Vanguard, kill its spotters first, then go for the gun.
- **Don't feed the army in pieces.** Every loss came from groups of 15–30 arriving separately. When you commit, commit as one body.

## Enemy tells on this map

- Before 10 minutes they raid with lone Pawns, Ticks and Rovers from both a bot lab and a vehicle plant.
- Around 11 minutes a block of about 10 Rocketeers and 5 Centurions (2–3.5k) comes into our half at spot_47. We beat it by holding 9 Stouts at the spot_43 turrets while the main group hit it from behind.
- Their tier 2 appears at 14 minutes (Hounds, a Gunslinger), and tier 3 by 25 minutes (Razorback, then Fatboy, Vanguard, Marauder).
- They wall and turret the E3/F2 approach and rez with Lazarus.
- Late in the game their Marauder and Hounds raid home while the main army is away. Keep a home guard with turrets at spot_43.

## The one lesson

A 4:1 economy lead at 18 minutes is worth nothing if it is all spent on tier 1. Get tier 2 standing before the lead peaks, and never let the frame starve.
