# Baselines for the review: what the pros' games look like, by the numbers

Reference tables for `.claude/skills/bar-review/SKILL.md`. Every number names its source; recompute with the command
given rather than trusting a stale table. Written 2026-09-27 by Claude (Fable 5.1) from the carded replays and our
recorded games (`docs/studies/2026-09-27-review-skill.md` is the study behind it).

## The pool

The public replays carded by the replay survey (`docs/design/2026-09-23-replay-survey.md`; index
`docs/knowledge/replays.md`; manifest `run/data/replays/manifest.jsonl`): 68 Comet Catcher Remake 1.8 duels with
cards, 40 of them with both players at OS 40 and above. **The 40 are the baseline** (the players' advice, 2026-09-24:
25 is too low to learn from; 40 and above is where the lessons are); the 29 at OS 25+ carded first are the source of the
`K-map-comet-catcher-remake-1-8-*` claims in `docs/knowledge/maps.md` and are quoted where the 40+ pool has not been
recounted. `run/pro_baseline.py` selects the pool per game: the record's map, our start cell and its half-turn mirror
(A4 with H5, B5 with G4), our faction, `--floor 40`.

Spot numbers: the bot's records and the cards number the same 80 spots (checked 2026-09-27: every spot of
onepass-player-fable-1's header within 2 elmos of a replay record's, same order; `docs/design/2026-09-23-replay-survey.md`,
2026-09-24 note). The warning in the 14:45 note of that design, and in `docs/briefs/player.md`, that the cards' numbers
are not the bot's, is stale.

## Opening milestones (Armada, pool at OS 40+; `run/pro_baseline.py <match>`)

| milestone | B5/G4 pool p25 / median / p75 (18 sides) | B5/G4 winners | A4/H5 pool (20 sides) | A4/H5 winners |
|---|---|---|---|---|
| first factory started | 0:34 / 0:34 / 0:34 | 0:34 | 0:41 / 0:58 / 0:59 | 0:58 |
| first factory standing | 0:53 | 0:53 | 0:59 / 1:17 / 1:17 | 1:17 |
| its first unit (a Rover) | 0:53 | 0:53 | 1:00 / 1:17 / 1:18 | 1:17 |
| first soldier | 0:53 | 0:53 | 1:12 / 1:17 / 1:22 | 1:18 |
| first constructor | 1:17 / 1:34 / 1:39 | 1:29 | 1:10 / 1:25 / 1:33 | 1:29 |
| second factory started | 7:13 / 7:20 / 8:33 | 7:55 | 7:20 / 8:13 / 9:54 | 8:13 |
| first extractor lost | 2:26 / 3:26 / 4:40 | 3:41 | 3:09 / 4:48 / 6:07 | 5:13 |
| solars before the plant | 2 / 2 / 2 | 2 | 2 / 2 / 2 | 2 |
| factory units by 4:00 | 17 / 18.5 / 20 | 19 | 17 / 21.5 / 25 | 21 |
| constructors made by 8:00 | 4 / 4.5 / 6 | 6 | 5 / 6 / 7 | 6 |
| light turrets by 8:00 | 7.5 / 9.5 / 13 | 13 | 7 / 10 / 11 | 11 |
| solars by 8:00 | 14 / 16 / 18 | 16 | 10.5 / 13 / 15 | 13 |
| radars by 8:00 | 1 / 2 / 2 | 2 | 2 / 2 / 3 | 2 |
| nano turrets by 8:00 | 1 / 2 / 2 | 2 | 0 / 1 / 2 | 1 |

Pressure (the first army in his half; `card.py` `pressure`, computed 2026-10-03 over 80 sides of 40 Comet duels at OS 40+, all
starts and factions; winners / losers, medians): first soldier in his half 1:48 / 2:26, within 1,500 of his start 2:06 / 2:57,
first building of his killed 4:02 / 3:41, first extractor of his killed 4:22 / 3:58, first extractor lost 3:58 / 4:22; army
metal in his half at 3:00 78 / 43 (of 280 / 334), at 5:00 142 / 171 (of 844 / 876), at 8:00 486 / 440. Every side was in his
half and all but one killed a building: the behaviour is the floor of strong play, not what separates winners (they are
40 s earlier). Quartiles, all 80 sides: in his half 1:38 / 2:06 / 2:56, first building killed 2:25 / 3:49 / 5:08. By
faction: Armada (41 sides) in his half 1:22 / 1:40 / 1:49, within 1,500 1:36 / 1:54 / 2:08, first building killed
2:18 / 2:48 / 4:14, first extractor killed 2:19 / 3:04 / 4:42; Cortex (39) in his half 2:16 / 2:48 / 3:24, first building
killed 3:33 / 4:35 / 5:35.

The A4 Armada pool at 40+ is thebluegecko's line (three solars, the plant at 0:58; `K-map-comet-catcher-remake-1-8-two-solars-before-the-plant`):
both openings are in the pool, the count of solars is not what separates winners from losers, and nobody builds a
fourth before the plant. The commander guards the plant 50 s of the first 4:00 (median; thebluegecko 140 s), and the
metal store sits below 20 in 9% of the seconds from 1:00 to 4:00 (`K-map-comet-catcher-remake-1-8-commander-assists-the-plant`;
`run/replays/assist.py --floors 40 --ours <match>` gives the same numbers for one of our games: `fac4`, `stall4`,
`assist4` on `run/floor.py`'s row).

## Curves by minute (Armada from B5/G4, pool at OS 40+, p25 / median / p75, winners' median in brackets)

Sides still playing at the minute: 18 to 8:00, 11 at 10:00, 9 at 12:00, 3 at 16:00 (the pros' games end at 12:39, so the
late columns are the long games).

| min | extractors | constructors | metal income | army metal | soldiers | metal stored |
|---|---|---|---|---|---|---|
| 2 | 3 / 4 / 4 (3) | 1 / 1 / 2 (1) | 9 / 9 / 11 (9) | 128 / 289 / 326 (265) | 3 / 5 / 6 (4) | 72 / 128 / 156 (158) |
| 4 | 6 / 7 / 9 (8) | 2 / 3 / 3 (3) | 14 / 18 / 20 (18) | 414 / 533 / 654 (564) | 7 / 8 / 10 (8) | 17 / 82 / 120 (111) |
| 6 | 10 / 13 / 15 (14) | 2 / 3 / 4 (4) | 25 / 30 / 36 (32) | 714 / 1083 / 1677 (1176) | 9 / 12 / 17 (15) | 362 / 551 / 881 (798) |
| 8 | 12 / 17 / 19 (18) | 3 / 5 / 6 (5) | 31 / 36 / 41 (41) | 1430 / 1523 / 1766 (1743) | 13 / 17 / 20 (18) | 305 / 964 / 1497 (1497) |
| 10 | 12 / 16 / 20 (20) | 4 / 6 / 7 (7) | 33 / 43 / 53 (53) | 1840 / 2654 / 3008 (3008) | 16 / 23 / 30 (30) | 126 / 356 / 668 (654) |
| 12 | 17 / 20 / 26 (26) | 5 / 8 / 8 (8) | 39 / 48 / 62 (62) | 2862 / 4163 / 5601 (5601) | 25 / 37 / 40 (40) | 25 / 221 / 964 (149) |
| 16 | 24 / 25 / 32 (38) | 8 / 8 / 14 (21) | 92 / 117 / 126 (117) | 3914 / 5263 / 5956 (6650) | 27 / 38 / 44 (38) | 78 / 140 / 782 (16) |

The same from A4/H5 (20 sides): extractors 7 / 11 / 15 / 17 / 22 / 30 at 4 / 6 / 8 / 10 / 12 / 16; income 16 / 25 / 36 /
36 / 48 / 75; army metal 616 / 1054 / 1475 / 2310 / 3254 / 6375. The 29-duel survey (OS 25+, both factions):
extractors at 4:00 / 6:00 / 8:00 / 10:00 / 12:00 winners 8 / 12 / 18 / 23 / 28, losers 6 / 11 / 15 / 15.5 / 14.5;
from 8:00 to 12:00 the winners added 11 extractors and the losers 1
(`K-map-comet-catcher-remake-1-8-extractor-curve`, `-decided-by-the-8-to-12-gap`).

Income per extractor: the pool's 16:00 column gives 117 metal a second from 25 extractors (4.7 each) where a tier-1
extractor here yields about 2.6; the difference is mohos (tier 2 in 4 of 58 sides only, so mostly not), wrecks
reclaimed and resurrection bots (24 of 58 sides built them, `K-eco-comet-pros-rez-bots`). Ours at 16:00: 59-62 from
30-32 extractors (2.0 each) in onepass-player-6 and -8.

## Raids (`run/raid_ledger.py <match>`; the pros from the cards)

- Every pro side loses extractors: 19.5 per side per game (median; p75 36), 1.85 a game-minute, the first at 3:26
  (B5/G4 Armada 40+; the 29-duel survey 4:18, before 3:00 in 18 of 58 sides), most often to scout cars, Incisors and
  Blitzes. A side re-takes 7.5 spots a game (a spot taken again after it was lost; the cards' `spots_taken`).
- Raids are met by light turrets at the extractors (winners 9-13 by 8:00) and by the scouting swarm, not by holding the
  army at home (`K-map-comet-catcher-remake-1-8-raids-met-by-turrets`); a Tick (140 speed) is caught by a Rover (168),
  never by a Blitz (101) (`docs/briefs/player.md`, the Comet section).
- Ours (2026-09-27, `run/raid_ledger.py` over the fourteen games of the study): extractors lost a game-minute 0.6-1.1 in
  the wins (escalate-7 0.6, held-1 1.1, onepass-player-6 0.8), 1.9-2.3 in the hard_aggressive losses (onepass-player-5
  2.0, -7 1.9, -8 2.2, fable-1 2.1, opus55-low 2.3); undefended (no soldier of ours within 900 and no turret within
  700 ten seconds before): wins 2 of 9, 6 of 12, 4 of 15; losses 14 of 19 (fable-1), 19 of 47 (player-7), 14 of 44
  (player-5). Rebuilt within the game: wins 7 of 9, 8 of 15; losses 6 of 19 (3 stood) and 19 of 47; the rebuild's delay
  100-160 s (median) in every game with ten or more rebuilds.

### Packet 2026-09-27 (the user's review of onepass-player-8; `run/raid_ledger.py`, the record's `ev` lines)

| | player-8 (loss 32:32) | player-6 (win 20:18) | escalate-7 (win 16:02) |
|---|---|---|---|
| extractors lost | 70 | 15 | 9 |
| undefended (no soldier within 900, no turret within 700) | 10 | 4 | 2 |
| answered within 60 s of first sight (median delay) | 57 of 59 (7 s) | 13 of 13 (7 s) | 8 of 8 (11 s) |
| parties that paid | 41 | 10 | 6 |
| spots lost twice or more | 23 | 1 | 2 |
| construction turrets finished by 12:00 (lost) | 4 (2 at 7:07-7:11, none 7:11-10:48) | 2 (0) | 5 (0) |
| store at 12:00 | 2,736 of 3,050 | | |

The answer's delay is not what separates the games (7 s in both); what does is who answers (the commander after a Pawn
for 42 s at 6:29-7:11, one Blitz after a Tick), whether the order holds from one second to the next (7:13-7:38, the
rule's whole group against the plan's four), and whether the same raider is remembered when it comes back (2:24-2:30,
4:41-5:01).

## Fights (`docs/studies/2026-09-24-pro-fight-shapes.md`, `run/fire.py`, `run/queued.py`, `run/analyze_match.py`)

- The pros fight in bodies of 6 (median at contact); ours 13. A friend stands on the line of fire of 22% of their units
  in reach at contact, 43% of ours (Stouts 24% against 53%); nearest-friend spacing 67-70 elmos against our 25-40.
  Seven in ten of their group orders give each unit its own point on a line (spacing 64); three in four of ours go to
  one point. Mixed raider-and-line bodies: 10% of theirs, 61% of ours. Short-range units fight at 0.94 of reach (ours
  0.86). A line at contact trades 5 points better than a blob (interval +1 to +10).
- `run/fire.py` on onepass-player-8: 12% of soldier-seconds in reach muzzled (43% of them a friend on the line), 9% of
  damage dealt landed on our own units (Stout on Stout 20,733). worth-1 (2026-09-24): 8% muzzled, 5.4% friendly fire.
- Where the metal is lost separates our wins from our losses (`run/analyze_match.py`, "metal lost by place"):
  escalate-7 lost 7,692 in their half and 2,677 in ours; onepass-player-6 19,170 in theirs and 3,511 in ours; onepass-player-7
  lost 27,799 in ours and 7,551 in theirs; opus55-low 54,335 in ours, 12,525 in theirs. The pros: from minute 6 the deaths
  are in the loser's half (479 of 697 in minutes 6-9, 1,390 of 1,645 from 14) and the loser loses two to three and a half
  times what the winner does (`K-map-comet-catcher-remake-1-8-fights-on-rows-4-5`).

## Closing (pool at OS 40+, 40 games; computed 2026-09-27 from the cards' `curves`)

- Games last 12:39 (median; p25-p75 9:17-16:07). The eventual winner first holds twice the loser's army metal at
  minute 7 (median over the 27 games where it happens at all; p25-p75 6.5-13) and the game ends 3.4 minutes later
  (median; p25-p75 1.2-5.4; over 6 minutes in 5 of 27). At the end the winner holds 27 extractors to 14.5 and 1.8 times
  the army; the loser's commander and factories die in or beside its start cell in 28 of 29 surveyed games
  (`K-map-comet-catcher-remake-1-8-endings`). Tier 2 in 4 of 58 sides, never before 12:51: the games are decided on tier 1.
- Ours (`run/analyze_match.py` curves, army value ours/theirs): onepass-player-8 held twice the truth's army metal from
  13:00 (7,035 against 1,388; six times at 14:00 and 16:00) to 23:00 and lost at 32:31; onepass-player-7 from 13:00
  (6,484 against 729) and lost at 25:42 with the base push at 15:25-16:34 failing on a beamer and unseen shelling;
  onepass-player-6 from 6:00 (1,052 against 519; three times at 10:00), pushed at 11:32 on 7.8k against 1.8k seen and won
  at 20:18 after eight minutes of sweeping for a commander first seen at 19:08; escalate-7 from 11:00 (6,418 against
  2,150) and won at 16:00; held-1 never (1.7x at 10:00) and won at 11:11 because the enemy commander walked out to D2.
  The pros' 3.4 minutes from 2x to the end is the number to hold ours against.

## Scouting

The cards cannot give it (a replay sees everything). Ours: `run/floor.py`'s `look_min` (the minute one of our units first
stood within 1,500 of the enemy commander's true start): 16.0 in onepass-player-8, 14.3 in onepass-player-6, 15.9 in
escalate-7, 3.6 in held-1; `known%` (the player's enemy-army metal as a share of the truth, median over its turns past
minute five) 17-49% in every game. The pros make five or more scout cars by 3:00 in 38 of 45 vehicle-plant sides
(`K-map-comet-catcher-remake-1-8-raids-met-by-turrets`); onepass-player-6's commander was found by a Rover sweep of the
box's unseen spots (its notes, 15:53-19:08). Against BARb the AI is placed at an end of its strip, not the mirror point
(`docs/briefs/player.md`, the Comet section).
