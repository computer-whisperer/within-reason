# fixes-1-hard-aggressive: the player's debrief

Match `run/matches/1790137350-fixes-1-hard-aggressive/00`, commit cac6eca, BARb hard_aggressive on Comet Catcher Remake 1.8: Win after 17.0 min, 53 turns. Debriefed by claude-opus-5-5 (effort low) in 17 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief: fixes-1-hard-aggressive (Comet Catcher, BARb hard_aggressive, arm, corner W)

**Result:** Win at 17.0 minutes, 53 turns. My notes don't record what ended the game. At 16:50 group_T had pulled back from G1 and our commander was walking home at 25%. The win came between that turn and the next, so don't read the last push as the winning move without checking the record.

## Where the enemy is
- **Their base is at the north end of their strip, G1.** The bot lab is at (6880, 336) and their commander was seen at (6405, 648). The advanced bot lab is at (6440, 392), beside spot_6. This matches the earlier escalate games. Send the Rover early and mark `enemy_lab` as soon as it's seen.
- **E3 in the middle holds a beamer and light-turret nest.** Go around it, not through it.
- **G1's defence is strong:** a heavy laser tower (HLT), a beamer, their commander and Centurions. A lone Stout group of about 20 died on it.

## What worked
- **The players' opening.** Extractors at spot_28 and spot_30, three solars, the plant, a solar and spot_36. Then the commander goes out to spot_43, 38, 39 and 26 with a turret beside each pair and a radar at spot_39. We had 14 extractors by 4:50 and 19 by 6:25, well ahead of pace.
- **Stouts as the main line.** The 12:22 middle fight showed Blitzes melt against turrets.
- **The north-edge route:** spot_13 → 8 → 3 → 6 → enemy_lab. group_T (about 24 Stouts, 5.8k metal) cleared E1's light turrets and radar and killed the advanced lab, a Rattlesnake, a Sharpshooter and a moho.

## What went wrong; do these differently
1. **Metal banked again.** It reached 1.7k at 6:25 and 2.1k at 7:23, the same failure as games 5 and 6. Start the second plant and nanos by about 5:00, and watch energy. At 7:05 energy, not metal, was the real limit, so queue solars alongside.
2. **The hands started three advanced vehicle plants at once** (spot_28, spot_30, west_yard) with metal stalled. Name exactly one site. Put a turret at the yard before a big frame: a Tick killed an unfinished plant worth 2,600 at home.
3. **Blitzes chased into the E3 nest** and we lost about 18. Keep flee footwork on, and name a stop line.
4. **Don't name a shelling place in the packet.** group_T stood at spot_8 under fire for two minutes because the hands kept choosing a vanishing shelling marker. It moved the turn I dropped the word.
5. **Extractors stayed flat at 21–23 from about 7:30 to 15:50** while five constructors helped the plants, and raids killed about four every three minutes. Keep at least two constructors expanding and re-taking spots with turrets the whole game.
6. **Expansion list steps only take spots the picture currently names.** Spots it doesn't name are skipped as "not free", so check that each spot is listed before writing the list.
7. **Tool shapes.** Cancelling a list and marking a place were both refused once for bad format. Place names must be the top-level keys in a mark call.
8. **Protect the commander.** It wandered toward spot_53 and spot_31 into enemy groups. Give it explicit "stay at X" orders once the fighting starts.

## If you're here at the end state
Hit G1 with one combined push rather than a single group: both Stout groups, fresh Stouts and Janus for the HLT, with a guard left at spot_26. Kill the roaming raid parties (Welder and Mace) with the home group first.
