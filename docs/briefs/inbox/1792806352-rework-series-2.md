# rework-series-2: the player's debrief

Match `run/matches/1792806352-rework-series-2/02`, commit 9ab415b, BARb hard_aggressive on Comet Catcher Remake 1.8: Win after 21.7 min, 110 turns. Debriefed by claude-opus-5 (effort low) in 46 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief: rework-series-2 — Comet Catcher Remake 1.8, Cortex west (B8) vs BARb hard_aggressive

Won at 21.7 minutes. Read that number with suspicion: we lost the metal exchange for the whole middle game (7.9k lost against 3.5k killed at 14:23) and won because the economy outran the losses. The win is the economy's, not the army's.

## The map as it played

We start at B8, south-west. **His base was G1–H3, the far north-east corner**, and his commander was seen at H2 (7426,1313) at 2:43 building a light turret. Assume that corner until you see otherwise; the diagonal is long, so his first raiders reach you before you reach him, and your first push walks a long way.

The contested ground is a line F3–F4–E4 at the mouth of his half. He fortified F3 early and kept it fortified: a Twin Guard, a Guard, a Maw, a Lasher, with Pounders and Brutes living under it. **That nest beat four separate attempts** (push3 at 11:05, the F3 attempt at 13:43, another at 15:03) and it did so each time the same way — our tanks arrived in ones and twos and died in ones and twos.

He also had standing fire we couldn't see: an Agitator at E1 did **5.9k damage to the main army from out of range** before 17:39, and a Wolverine at guns_22 (F5) did ~3.7k into our middle extractors. Both were cheap for him and expensive for us. Look for the thing shooting you from outside your sight before you accept a damage report as "attrition".

## The opening (keep it)

The Cortex vehicle line from rework-series-1 worked: extractor, extractor, solar, solar, vehicle plant, third extractor; commander then assists the plant at low priority. Plant: constructor, two Rascals roving his half, constructor, then Incisors with a constructor after every three.

Pace: 4 extractors / 11 metal at 3:00 (behind the pool median of 6 / 14), recovered to 6 / 16 by 3:42 and 9 / 22 by 4:20, 11 / 27 at 4:40 against a median of 8 / 18. The dip at 3:00 is because the plant and the commander's assist both draw at once. **The fix at 3:00 is extractors, not solars** — metal was empty with energy full.

The two Rascals paid for themselves: one found his commander at 2:43 and killed an extractor at G1 before dying. Send them, expect to lose them.

## What cost us the middle game

**1. Brutes into Pounders.** At 8:45 we lost 6 Incisors to two Pounders at G4; the riot tank shreds light tanks, and it went on shredding Brutes for six more minutes. Pounders outrange nothing — the point is that they win the close fight at any reasonable odds. **Lashers (575) and Wolverines (710) outrange the Pounder (315).** We only shifted the plants to them at 14:23. Shift earlier: the moment you see a Pounder, the answer is range, not more Brutes.

**2. Arriving in ones and twos.** Every failed push had the same shape in the notes: a group waits at a gather spot and is picked apart while waiting, or its body is 2.6k behind its front. Twice I pulled back to gather and then gave the push early anyway because waiting was bleeding us (10:55, 15:03). If the gather spot is inside anyone's reach it is not a gather spot — spot_35 was not; spot_32 (E4) was.

**3. Energy, all game.** The pattern repeated at 10:05, 10:45, 11:05, 18:18, 19:07: **metal banked at 1.5–2.1k while energy stalled**, which stops tier 2 and slows every build. Three T1 plants plus an advanced plant draw more energy than a solar field supports. At 19:07 we were seven seconds from empty at +1370/−1778 and had to put the commander and three constructors onto *plain* solars because we had no energy to build anything that costs energy. Build the energy ahead of the plant that will eat it; a banked 2k of metal is a mistake you already made two minutes ago.

## The hands

Three times the standing rules fought the packet, and you should expect all three:

- **"A group fights any enemy party within 600 that it outweighs"** turned the main army west off its eastward march twice (19:27, 19:47) and cost about a minute each time. A sentence telling it to ignore raiders behind it is not enough; set the footwork and name the target explicitly.
- **Automatic step-back from turret fire** had six home-guard groups backing out of the F5 light turret's reach, leaving four Brutes to go in alone (20:46). Switch it off when you want a mass to commit.
- **Split groups wander.** group_V (3 Brutes) was sent to follow the home guard 3.9k away (16:50). Fold splits back by name.

Also: at 4:40 a whole tool call was refused because a radar step had no place written after it, and the commander's second plant and two constructor lists went with it. **A malformed step loses the entire call** — one turn gone.

## What ended it

At 14:33 his Pounder party died and left 5.8k of wrecks at F3/F4; three constructors reclaimed it while metal was stalled at 195. From there the army compounded: 23 soldiers / 5.3k at 14:33, 65 / 16.7k at 19:37, 79 / 21k at 21:06, with 48 Wolverines in it. The last five minutes were a walk: Agitator at E1 killed ~19:00, **his advanced vehicle plant found at G2 (6504,997) at 20:56 and dead at 21:04** — it had been building Banishers and we never saw it until a Tiger raided our middle extractors at 19:57. His commander was caught at G1 (6833,654) among a Warden and Twin Guard at 21:16 and shelled dead at 21:35.

**His tier-2 plant existed for a long time before we saw it.** The first Tiger at our E5 extractors was the only warning. Scout his corner again around 15:00, not only at 3:00.

## If you sit here again

1. Keep the opening and the two Rascals; fix the 3:00 dip with extractors.
2. Build energy one step ahead of every new plant. Don't let metal bank past ~800.
3. Stop making Brutes the moment a Pounder appears; Lashers and Wolverines, and shell from range.
4. Do not touch F3 until you have ~35 tanks gathered **outside** reach with shelling behind them. Three early attempts gave him 4k of free kills. The fourth worked because the army was finally one body.
5. Name the thing shooting you from out of sight and kill it first — the E1 Agitator alone was worth more than any of our failed pushes.
