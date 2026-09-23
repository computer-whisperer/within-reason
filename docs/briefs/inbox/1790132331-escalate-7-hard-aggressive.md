# escalate-7-hard-aggressive: the player's debrief

Match `run/matches/1790132331-escalate-7-hard-aggressive/00`, commit 8cec316+, BARb hard_aggressive on Comet Catcher Remake 1.8: Win after 16.0 min, 66 turns. Debriefed by claude-opus-5-5 (effort low) in 23 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief: escalate-7, Comet Catcher, arm corner W vs BARb hard_aggressive. Win at 16.0 min.

This is the first win at the top tier, after three losses. The game ended when our attack army found and killed their commander at G1. Most of what follows is what made that possible and what nearly threw it away.

## The opponent (the same shape as escalates 3–7)
- **They open with a bot lab.** The first contact is a lone Pawn or Tick around C4 near 2:40. Expect single raiders on any extractor without a turret from 3:00; a Tick killed spot_36 beside home at 3:30.
- **Their block arrives around 11:00–11:30 at D4.** It is Rocketeers, Maces and Centurions, about 2k metal. A large Blitz ball broke it cleanly: 2.5k lost for 1.0k.
- **Tier 2 is up by about 12:45.** Gunslingers and Hounds appear, then a Welder, and a Sharpshooter (cloaked sniper) by 15:45.
- **Their base is north-east, G1–G2.** The commander sat at G1 near (6390, 750) beside spot_6, with a moho and a Rattlesnake. The factories are near (6860, 340).
- **A turret nest sits at E2/E3.** It has a beamer, an LLT, walls, and later a Hound and a Gunslinger. A Gauntlet (range 1,220) with LLTs and a radar sits at E1 (spot_8). Neither can be seen until you are under fire.

## What won
1. **Spend the bank on build power early.** At 5:13 the bank was 565 and rising, so I added 2 constructors and 2 nanos. At 6:53 it was 1,480, so I added a second plant at yard_north (B2), more nanos and solars. In escalates 5 and 6 we lost sitting on 1,600–1,900. Here the bank drained, and the army went from 4 Blitzes at 6:53 to about 2.2k at 9:30 and 5.9k at 10:47.
2. **Mass one ball before committing.** Around 9:30–11:00 the army stood at spot_39/spot_24, with pickets north and south. It took the block at midfield instead of chasing it.
3. **Attack along the far north edge,** spot_24 → 0 → 1 → 8 → 3 → 11 → 16/6 → G1. This avoids the E2/E3 nest. It still meets the Gauntlet at E1, which is worth killing.
4. **Keep a real home guard.** We kept a separate group of 2–7k at spot_33/spot_39, not at yard_north. It killed the Hound/Gunslinger raid at spot_24 while the attack was out.
5. **Go all-in on the commander.** When their commander showed, I put `attack_unit` on it with every soldier. It died in about 15 s along with the armalab, the Gauntlet and the beamer: 16.9k destroyed in 3 min.

## What cost us (avoid)
- **Plant names.** The plant is named `plant_<id>` (here `plant_5440`), not `plant_1`. A limit on the wrong name silently does nothing, and we got 3 Rovers. Read the name from the packet after the plant stands.
- **The Blitz raid on F2 (spot_11) at 6:50.** It lost 5 Blitzes to Pawns plus 2 LLTs. Don't raid their outer extractors with a handful of Blitzes, because the hands keep the walk even after "abandon."
- **Energy stalled from 6:20 to 6:45.** Six constructors and a plant were running on about 7 solars. Add solars with every build-power step.
- **Chasing a party into the E2/E3 nest.** At 13:00 we lost about 1.5k chasing. If a small party retreats east from D2/E2, let it go.
- **Blitzes die to turrets.** The E1 fight cost 12 Blitzes to LLTs. Once the attack is pushing into structures, switch the plants to mostly Stouts; I did that at 14:48 and should have done it at 12:45.
- **Home guard placement.** Guard parked at spot_22 or yard_north loses the middle extractors (spot_31 fell to one Blitz). Station it at spot_33, covering spots 24, 31 and 39.

## Harness problems still open
- **`shelling` place.** The hands kept walking toward the vanishing `shelling` place under an unseen beamer: 386 hits in 20 s at spot_23. We need either a place that persists until the shooter dies or a way to forbid a place. Reroute by hand if you see it.
- **Constructor queues.** Queue lists given to constructors mid-walk (8:19–8:42) weren't taken; they kept their first order.
- **Plant queues.** Plants kept building `armcv` after the produce limits changed, because the engine factory queue isn't cleared.
- **Wavering.** The army went back and forth between spot_3 and spot_11 for about 30 s. Marking cleared spots as "done, don't return" in the packet fixed it.

## Timeline to aim for
| Time | Target |
|---|---|
| 1:30 | Plant up |
| 5:00 | 12 extractors, income about 28, bank being spent |
| 7:00 | 17 extractors, second plant |
| 9:30 | 20 extractors, army over 2k |
| 10:45 | Army about 6k, commit north |
| 11:30 | Break the block at D4 |
| 13:30 | Merged ball of about 7.7k on the north edge |
| 15:45 | Commander found at G1, all-in |
