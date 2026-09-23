# escalate-6-hard-aggressive: the player's debrief

Match `run/matches/1790129025-escalate-6-hard-aggressive/00`, commit 1251ee7, BARb hard_aggressive on Comet Catcher Remake 1.8: Loss after 23.0 min, 76 turns. Debriefed by claude-opus-5-5 (effort low) in 20 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief: escalate-6 against hard_aggressive on Comet Catcher, arm, corner W. Lost at 23.0 min.

## What hard_aggressive does here

- **Base.** Its bot lab is in the north-east at G1 (6880, 336), found at 3:12. It was in the north in escalate-3 and 4 as well, so expect it there.
- **Raids from 2:45.** It starts with single Ticks and Pawns on mid spots that have no turret. The first losses were spot_45, spot_24, the Rover near its base, and the radar at spot_39.
- **Waves from about 7:00.** Pawns come in groups of 4 to 10 into the north strip (spot_19, 22, 24, 26) and into home. The waves did not stop for the rest of the game. They killed extractors, a half-built second plant, constructors, and later two finished plants to raids of only 3 Pawns.
- **Line block by 13:49.** It fields Maces, Rocketeers and a Centurion, and five of our soldiers alone in the north died to it.
- **Tier-2 bots by about 16:00.** Hounds, Gunslingers and Sharpshooters appear, while we were still on tier 1.
- **Push on home at 20:00.** A block of about 2,000 metal (Maces, Centurions, Welder, Rocketeer, Snipers, 8 Pawns) came down through spot_19 to home.

## What happened

The opening was good. The players' Comet order was taken on turn one. The commander went out to 43/38/39, and we had 13 extractors on income 30 at 4:54, ahead of pace. The game was lost between 5:00 and 12:00, in three ways:

1. **Metal banked instead of becoming build power.** The bank was 851 at 5:17 and 1,750 at 7:46 on one plant. The nanos and the second constructor pair came too late, and the second plant died half-built. escalate-5 lost the same way. A bank above 500 means build power is missing: add constructors and nanos at once, not another plant later.
2. **Pawn waves answered piecemeal.** Blitzes went out one or two at a time and died, while turretless spots died to single Ticks. Extractors fell from a high of 16 at about 6:40 to 10 at 11:17, 6 at 14:29, and 4 at 19:55.
3. **`send_against` and split broke the army into five to seven groups** between 11:00 and 16:00, and they died one at a time. The trade reached 15.7k lost to 4.5k killed. Only at 15:58, with everything forced into one ball (group_Y), did fights start to go our way. By then income was 15 and the enemy had tier 2.

## What to do differently

- **Put a light turret beside every mid extractor.** Order the turret in the same list step as the extractor, before the next spot. Single Ticks took every turretless spot we had.
- **Spend the bank by 5:00.** Four or more constructors and nanos on the plant, with the second plant built under the army's cover rather than out in the strip. Treat any bank above 500 as an emergency on that turn.
- **Keep one army ball that holds the north strip (spot_19/22/26) from 6:00,** with pickets of one soldier at the outer spots (K-army-pickets-across-the-front). Do not split it with `send_against` against Pawn parties. Let the pickets and turrets absorb small raids and bring the ball only to parties that threaten a plant.
- **Protect the plants.** Both were killed by 3-Pawn raids while the army was away. Keep a turret pair at every plant.
- **Plan for tier 2.** hard_aggressive climbs on a clock (K-barb-hard-aggressive-tech-clock). By about 12:00 we need either a tier-2 lab or a Janus-heavy mix against packed bots. Stouts and Janus arrived too late to matter.

## Problems with the hands to know about

- **Constructors skip list steps.** Some steps were never done: the turret at spot_45 at 3:29, and the spot_24 extractor and turret, which were left at 1–3% built at 14:44. Check that a turret you ordered actually stands before you rely on it.
- **Constructors got stuck walking** for 150–250 s at the same distance near yard2 and spot_38. After about 60 s with no progress, relist or reclaim.
- **The plant ignored "two constructors first"** and kept making Stouts. Forcing a single entry (`armcv:2`) worked. At 17:44 a new plant also built from an old allowed list.
- **Groups ignored their posts** and chased at 4:54, and group_Y stayed fighting at spot_19 while ordered back at 21:04.
- **Waking every 5 s changed nothing.** Let orders play out for about 30 s unless something new appears.
