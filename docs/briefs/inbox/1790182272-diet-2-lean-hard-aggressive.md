# diet-2-lean-hard-aggressive: the player's debrief

Match `run/matches/1790182272-diet-2-lean-hard-aggressive/00`, commit 3212dbe, BARb hard_aggressive on Comet Catcher Remake 1.8: Win after 15.5 min, 52 turns. Debriefed by claude-opus-5-5 (effort low) in 15 s; /home/christian/.claude  [?]
  HTTP 429 (retry-after 0 s): rate limited, or the token is stale — run that account's CLI once to refresh it.

# Debrief: Comet Catcher, arm from the W corner, against BARb hard_aggressive. Won at 15.5 min.

## What won
Two things won this game. The first was one large Stout ball of about 24 Stouts and 5.3k metal. The second was the **north-edge route**: spot_13 → spot_8 → spot_3 → spot_11 → spot_6 → spot_10 → spot_5. It came in behind their defences, and on the way it killed a Gauntlet, LLTs, a radar and extractors at D1/D2 for two Stouts. Over three minutes the trade went from 1.7k lost / 1.0k killed to 4.5k killed / 2.4k lost. Their commander turned up at **G2 (6365, 860)**, beside its Advanced Bot Lab, and an `attack_unit` with every soldier ended the game. In every game so far their base has been north (G1/G2), so plan the approach from there.

## Opening (keep it)
Use the players' order: mex 28, 30, three solars, the vehicle plant at ~0:55, two more solars, then the commander goes out to 36/43/38/39/26 building turrets and a radar. On the factory, set armfav:1, armcv:3 and armflash. By 5:00 we had 12 extractors and income 26 with 5 constructors, which is on the players' pace.

## Mistakes to avoid
- **Energy stalled twice.** The first stall was at 3:58, with 5 solars against the players' 7 by 4:00. Put two more solars in the opening lists. The second stall was at 8:55 with 2000 metal banked. Each Stout costs 2100 energy, and one plant with three nanos needs more than ~20 solars. Add solars in step with every nano and every second plant; don't wait for the store to hit zero.
- **Home raiders at 3:00–4:30.** A Tick and several Pawns killed spot_28 and spot_38 before the Blitzes arrived. Station the first Blitz at spot_38/spot_33 as a home hunter. Don't send it out to spot_19, which is 1,500 away from the middle.
- **Don't push through E3/F3.** It is a turret nest with a Beamer and LLTs. At 12:19 we lost 5 Stouts chasing Pawns into it, and their commander wasn't there. Their commander's "last seen" point at E3 was bait. Go around by the north edge.
- **Keep our commander at spot_33, never E4.** At 8:55 it walked into a Centurion+Rocketeer party and dropped to 23%. Later the hands kept picking the walk to spot_13 even after I cancelled its list. Say "stays at home beside the plants" outright.
- **The Rover scouted into their commander and died.** It had already found the commander by then, so that trade was fine. Just expect it.

## Their timeline (hard_aggressive)
- Tick/Pawn raids on home spots from 3:00.
- A Centurion+Rocketeer party in the middle around 9–11 min. Killing it cleared the way.
- Tier-2 bots (Hound, Gunslinger) at 11:39, and a Welder by 14:00.
- A Hound's 650 range out-reaches the Stout's 350. A group told only to hold will stand and take the shells, so order it to charge any Hound that is shooting it.

## Standing groups that worked
- **group_R, home guard at spot_33:** it killed the Welder raid.
- **group_H, south guard at spot_61:** nothing goes east of spot_59/spot_47.
- **New Stouts:** they join the main ball along the north route, never through E3/F3.

We started the AVP at 11:39, but it contributed nothing before the end. If the game runs longer, start it earlier, around 10 min, once income is above 45.
