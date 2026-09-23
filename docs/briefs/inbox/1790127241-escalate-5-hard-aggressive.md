# escalate-5-hard-aggressive: the player's debrief

Match `run/matches/1790127241-escalate-5-hard-aggressive/00`, commit 64975e9, BARb hard_aggressive on Comet Catcher Remake 1.8: Loss after 28.5 min, 109 turns. Debriefed by claude-opus-5-5 (effort low) in 31 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief: escalate-5-hard-aggressive (loss, 28.5 min)

**Comet Catcher Remake 1.8. We were Armada in corner W. BARb hard_aggressive was also Armada.**

## What happened, in one paragraph

The opening worked. By 5:13 we had 15 extractors and income 32, having lost only a Rover and two extractors to lone Pawns and a Tick. After that we lost the game on three things:

- **Spending.** One plant could not use the income. Metal banked to about 1,770 by 8:38, and the second plant came late.
- **Tech.** The Advanced Vehicle Plant (AVP) was started at 8:38 at A5, away from home. It was starved of metal and never finished. Three Blitzes killed it unguarded at 15:08.
- **The army lead.** At 11:49 we had 5.9k of army against 1.75k seen, and we spent it badly. The Stouts went into the E2-E3 turret nest (two HLTs and a claw) and were shredded. By 17:32 we had lost 15k against 7.6k.

After that we were always behind in tech. Their Hounds and Fatboys out-ranged our tier-1 vehicles. A Razorback (tier 3) with Lazarus repair bots broke our only Mauser push at C3 at 25:48. Raids ate every rebuild, and the base fell at about 27:30.

## Their shape (third game running)

- **Their base is north-east.** Their commander was at F1 (5326, 443) at 2:43, with extractors at F1/F2 and a radar.
- **Early raids are lone Pawns and Ticks** on our outer extractors from about 2:30. A Tick is faster than a Blitz.
- **They fortify E2-E3 by about 15:00:** two HLTs (620 range) plus a claw. Do not attack it with Stouts.
- **Tier 2 appears around 14:00:** Hounds (650 range), Welders, Fatboy (700), Centurion, Recluse, and snipers.
- **Tier 3 appears by about 25:00:** a Razorback (3.8k metal) escorted by about 8 Lazarus, then Marauders and Gunslingers from a gantry. This matches the tech-clock claim (K-barb-hard-aggressive-tech-clock).

## Do differently

1. **Get tier 2 by 10:00, at home.**
   - Build the AVP inside the turret ring, next to the first plant.
   - Put a Blitz guard beside it. Stouts cannot catch Blitzes.
   - Give it priority for metal over the second tier-1 plant.
   - Name an explicit place for it (a yard such as `avp_yard`). At 16:13 the constructor told to build it at "home" kept building a Sentry instead.
2. **Build energy ahead of Blitz production.** We hit zero energy three times (7:08, 18:22, ~20:04). Six Blitzes a minute outruns the opening solars.
   - Add solars before adding a plant.
   - Never start advanced solars at zero energy. They cost energy to build, so they crawl. Use plain solars to recover.
3. **Re-issue constructor lists when the need changes.** At 21:56 energy was full at +576 and metal was at 0, because all four constructors were still running solar lists from the stall.
4. **Turn an army lead into extractors and tech, not a turret assault.**
   - The right targets for a lead are their unguarded forward extractors, their army in the open, and our own tier 2.
   - Never send Stouts at HLTs.
5. **Fight as one ball.**
   - At 19:47, group_K was fed piecemeal into a Fatboy and Hound party and died.
   - At 17:08 and 25:48 the push met a stronger force and retreated too late.
   - Before committing, look along the route for tier 2 or tier 3 units.
6. **Mausers (820 range) out-range Hounds, Fatboys and HLTs only when something spots for them.** At 26:59 an unseen Fatboy killed three Mausers. Send a Blitz or a radar with every Mauser group.
7. **Keep a standing raid-hunter group of about 4 Blitzes at home from minute 5.** Without it, every rebuilt extractor died.

## Hands quirks seen this game

- **Build limits need the game's plant name.** The limit I set on `plant_1` was dropped; the plant is `plant_22773`.
- **The commander ignores attack orders beyond its "short walk."**
  - At 15:00 it did not attack a target about 900 away.
  - At 26:27 it did not go to the fight at spot_19.
  - To get it somewhere, give it a build step at that place, or a walk order, and put the attack after it.
- **Footwork flee can keep a winning group from engaging** (23:42). Turn flee off and leave kite on when we clearly outweigh the enemy.

## Rematch plan in one line

Run the same opening, with a turret on each extractor and Blitz hunters at home. Build more solars before the second plant. Put the AVP at home by 10:00 and switch to Mausers with spotters. Raid their forward extractors instead of their E2-E3 nest. Hold the army together at home when a Razorback shows up.
