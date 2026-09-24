# Comet Catcher Remake 1.8

Map script "Comet Catcher Remake 1.8", file `comet_catcher_remake_1.8`. Facts from the replay survey's 29 duel cards
(2026-09-23; claims in `../_inbox/replays-comet-catcher-2026-09-23.md`, the games in `../replays.md`) and the existing
claims in `../maps.md` and `../openings.md` (K-maps-comet-*, K-open-comet-*).

## Facts

- Size 8192 x 6144 elmos (map units 16 x 12). The cards' grid is 8x8: columns A-H of 1024, rows 1-8 of 768.
- 80 metal spots in the game's own list (the cards' numbers); the engine's list, which the bot reads, has 75. **Spot
  numbers on this page are the cards'**; they map to the bot's by position (nearest within 130).
- Starts seen in 29 duels (58 sides): west A4 17 (about x 820-960, z 2490-2600), B5 12 (x 1250-1430, z 3540-3600);
  east H5 19 (x 7240-7370, z 3580-3760), G4 8 (x 6760-6850, z 2570-2610), H1 2 (x 7290, z 750-760). The map is
  point-symmetric: A4 mirrors H5 and B5 mirrors G4 under a half-turn; H1's mirror (A8) was never a start
  (K-open-start-geometry-mirror-pairs).
- No side built a wind generator in 58; energy is solars (K-open-comet-radar-nano-solars-habits).

Spots by the cell the cards give most often:

| row | A | B | C | D | E | F | G | H |
|---|---|---|---|---|---|---|---|---|
| 1 | 7 | 2, 9 | 0, 4 | 1 | 8 | 3 | 5, 6, 10 | |
| 2 | 14 | | | 13 | 15 | 11, 16 | | 12 |
| 3 | | 19, 22 | 24, 26 | 21 | 20, 23 | 18 | | 17, 25 |
| 4 | 28, 30 | 36 | 38, 39 | 31, 33 | 32, 37 | 27 | 29, 34, 35 | |
| 5 | | 43, 45, 50 | 52 | 44, 47 | 46, 49 | 40, 41 | 42 | 48, 51 |
| 6 | 54, 62 | | 61 | 56, 59 | 58 | 53, 55 | 57, 60 | |
| 7 | | 67 | 63 | 64 | 66 | | | 65 |
| 8 | 68 | 73, 74 | 69, 76 | 71 | 78 | 75, 79 | 70, 77 | 72 |

28 of the 80 spots sit on a cell boundary and appear in two cells on some cards (spot_62 A6 or A7, spot_12 H2 or H1).

## 1. The opening by the clock

Two openings: the plant at 0:34-0:39 on two extractors and two solars (22 sides, mostly Armada), or two extractors, a
solar, a light turret at 0:25-0:28, the third home extractor and the factory at 0:58-1:10 (19 sides, mostly Cortex); the
later plant won 18 of 29 and 12 of 17 head to head, mostly with the higher OS
(K-map-comet-catcher-remake-1-8-two-openings, K-map-comet-catcher-remake-1-8-turret-at-0-27). A vehicle plant in 45 of 58;
Armada's first units are scout cars and Blitzes, Cortex's a constructor then Rascals
(K-map-comet-catcher-remake-1-8-first-factory-kind).

## 2. Expansion in the first eight minutes

The commander takes the three home spots and often the next pair toward the middle; constructors take the flank strips
(K-map-comet-catcher-remake-1-8-home-spots-then-flanks, K-open-comet-constructors-take-the-flanks). Orders per start:
A4 (K-map-comet-catcher-remake-1-8-a4-expansion), H5 (K-map-comet-catcher-remake-1-8-h5-winners-go-south), B5
(K-map-comet-catcher-remake-1-8-b5-winners-go-south-east), G4 and H1 (K-map-comet-catcher-remake-1-8-g4-and-h1-expansion).
Extractors at 4:00/6:00/8:00: winners 8/12/18, losers 6/11/15 (K-map-comet-catcher-remake-1-8-extractor-curve).

## 3. Second factory and tier 2

A second factory by 8:30 (median, 44 of 58); by 10:00 in 18 of 22 winners and 13 of 22 losers in games that long
(K-map-comet-catcher-remake-1-8-second-factory). Tier 2 in 4 of 58 sides, never before 12:51
(K-map-comet-catcher-remake-1-8-tier-2-is-rare). Constructors 4 at 5:00 and 7 at 10:00 for winners
(K-map-comet-catcher-remake-1-8-extractor-curve).

## 4. First contact and raids

Not on the cards (the first enemy seen is the commander at 0:00). Every side loses an extractor, the first at 4:18
(median), most often to scout cars (K-map-comet-catcher-remake-1-8-first-extractor-lost); raids are met by turrets
(winners 9 by 8:00, losers 6) and the raiding runs both ways in minutes 0-5
(K-map-comet-catcher-remake-1-8-raids-met-by-turrets).

## 5. Where the fights are

Rows 4-5 hold 68% of the deaths, in the home columns B and G and the middle column D; the fighting moves into the
loser's half from minute 6 (K-map-comet-catcher-remake-1-8-fights-on-rows-4-5). The game is decided by the extractor
gap opening between 8:00 and 12:00 (winners +11 extractors, losers +1)
(K-map-comet-catcher-remake-1-8-decided-by-the-8-to-12-gap, K-eco-winner-pulls-away-8-to-12).

## 6. How games end

13:39 median (6:57-21:31); the loser's base and commander go in the last minute, in or beside its start cell, with the
winner holding a median 29 extractors to 8 and 2.3 times the army (K-map-comet-catcher-remake-1-8-endings).

## 7. The higher-OS player

Won 22 of 29; builds its second factory earlier, more solars and more energy banked, not an earlier plant or more scout
cars (K-map-comet-catcher-remake-1-8-higher-os-differences).

## What our bot should copy

1. The turret opening: extractor, extractor, solar, light turret by 0:28, the third home extractor, the factory by 1:05
   (K-map-comet-catcher-remake-1-8-two-openings, K-map-comet-catcher-remake-1-8-turret-at-0-27).
2. The commander takes the three home spots and the next pair toward the middle; constructors take the flanks
   (K-map-comet-catcher-remake-1-8-home-spots-then-flanks).
3. From A4: spot_19 (B3) by 3:00, then spot_22 (B3), spot_14 (A2), spot_7 (A1), spot_2 (B1), spot_9 (B1), spot_4 (C1)
   by 6:15 (K-map-comet-catcher-remake-1-8-a4-expansion). From H5 the mirror: spot_60 (G6) by 2:10, spot_65 (H7),
   spot_72 (H8), spot_57 (G6), spot_77 (G8) by 5:00, before H3 (K-map-comet-catcher-remake-1-8-h5-winners-go-south).
4. Keep adding extractors from 8:00 to 12:00: 18 at 8:00, 23 at 10:00, 28 at 12:00, with 7 constructors by 10:00
   (K-map-comet-catcher-remake-1-8-decided-by-the-8-to-12-gap, K-map-comet-catcher-remake-1-8-extractor-curve).
5. A second factory by about 8:30 and before 10:00, a vehicle plant (K-map-comet-catcher-remake-1-8-second-factory).
6. Spend the bank through 8:00-10:00 (winners under 500 at 10:00 in 15 of 22) (K-eco-winner-pulls-away-8-to-12).
7. Meet raids with light turrets at the extractors (about 9 by 8:00), not the army; expect the first extractor lost at
   about 4:00 and rebuild it (K-map-comet-catcher-remake-1-8-raids-met-by-turrets,
   K-map-comet-catcher-remake-1-8-first-extractor-lost).
8. Tier-1 raiders through 12:00 and no tier 2 before then (K-army-comet-raiders-to-12,
   K-map-comet-catcher-remake-1-8-tier-2-is-rare).
