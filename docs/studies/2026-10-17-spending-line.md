# What governed the early economy, and what the player could see (2026-10-17)

The user, watching chase-player-1: "I notice some floating metal and energy in the early game ... what currently
governs the build order overall, and how many constructors etc to build?", then "Do we know when our reference curves
build construction turrets or more plants?", then "the BP shortage should have been completely predictable from just
the scaling income and the units we were trying to build. What was the player aware of at that point in the game?"

Games: chase-player-1 (`run/matches/1792275658-chase-player-1`, timeout) and chase-player-2
(`run/matches/1792286854-chase-player-2`, won 22:26), both on the arena target line. Read from the records' samples,
the hands' decision rows, the players' transcripts (every tool call and turn text before 7:00) and the 41 pro cards.

## 1. What governs the build order before 6:00

Nothing in code. The hands' default standing instructions apply only with no player connected; every builder and
plant decision before 5:07 of chase-player-1 came from a list the player wrote (`source: list`), the only picks two
"help the plant" moves for builders whose lists had run out. The player copied the sheet's most-played opening
(extractor, extractor, solar, solar, plant: 22 of 41 sides) and wrote the plant's `produce` list with four counted
constructors, then two more at 4:48; the uncounted tail (Blitzes) runs until a new list. Constructors stood 2 / 2 /
3 / 4 / 5 / 6 at minutes 2 to 7 against the sheet's median 3 at 4:00 and 4 to 7 made by 8:00. The build-order
simulator (`plan`) was never called in either game.

## 2. The floats

chase-player-1: energy at its cap 2:40-5:00 (five solars by 3:23 against the pros' two, after the first constructor
under the commander's help emptied the store at 1:20) while metal was 0-50; then metal 250 at 4:00, 882 at 5:00,
1,441 at 5:40 (one plant against 13 extractors; the second plant ordered 4:22, begun 5:16 after a Pawn pulled the
commander home) while energy fell to zero at 5:28 (a Pawn killed three solars); nine solars ordered, energy full
again at 7:00 with metal 1,605.

chase-player-2: energy empty 1:20-2:10, full 3:00-4:00, metal 603 / 1,382 / 1,895 at minutes 5 / 6 / 7 against the
pros' medians 260 / 435 / 537 and upper quartiles 486 / 838 / 1,223; energy at zero 5:30-6:30 and 8:30-10:00; the
second plant stood 6:23, the lab 8:09, a third plant 8:31, the first construction turret 8:51; 8 constructors by 8:00.

## 3. What the player saw at 5:08 of chase-player-2

The eco line: metal 708 of 1,650 at +27.2 in and 24.1 out; energy 1,110 of 1,550 with a net of -89 (twelve seconds
from empty). The reference line at 5:00: extractors 11 against 8 / 9 / 11, constructors 4 against 2 / 3 / 4, metal
income 27 against 18 / 20 / 27, energy income 170 against 150 / 180 / 200. The curves line: income 27 now, 7 three
minutes ago. The plant's entry: its queue and allowance. No line gave the plant's or a builder's draw at full speed,
the build power, or the seconds to empty; the picture's per-actor draw line reached the hands and was dropped from
the player's actor entries (0 of 111 turns carried "draws about"), the build power was printed only in the
commander's report, and the brief's builders table has build power but no build times, so the rates could not be
derived from the prompt.

| turn | it read | it ordered |
|---|---|---|
| 4:40 | 8 extractors, 20 a second, "ahead of the pros" | nothing on the economy |
| 4:58 | 11 extractors, 27 a second | a constructor's list with a nano in third place |
| 5:08 | "metal 700 banking at income 27" | the second plant |
| 5:18 | energy 345 "falling by about 70 a second, and the new plant will draw more" | two solars and an advanced solar |
| 5:28 | energy 88, 873 metal unspent | two advanced solars |
| 5:59 | energy empty, 1,374 banked | nine basic solars, "lesson: start solars at about 4:00" |

The sinks at full speed from the unit numbers: the plant alone 8.2 metal and 68 energy a second on Blitzes, a
constructor on an extractor 2.5 and 25, the commander on one 8.3 and 83; at 5:08 about 27 metal and 250 energy
against income 27 and 170, the income rising about 5 a minute. A second plant adds 8 metal a second of sink and 68
energy of draw; a construction turret on the one plant 11 metal for 230 metal at 90 energy.

## 4. The pros' sinks

From the 41 cards (the sheet's new rows): the first construction turret begun at 5:24 / 5:56 / 6:31 (30 of 41;
winners 5:45, losers 6:25) with 9 / 11 / 13 extractors, 20 / 25 / 28 income, 3 / 3 / 4 constructors, 196 / 358 / 572
stored and one factory; the second factory at 7:12 / 8:11 / 8:56 (30 of 41) with 14 / 16 / 19 extractors, 32 / 37 /
43 income, 446 / 898 / 1,502 stored and 0 / 1 / 2 construction turrets; 23 of 24 put the turret first; the third
factory 10:04 / 11:55 / 14:51 (14 of 41). Their store by the minute (medians, upper quartile): 79 (171) at 4:00, 260
(486) at 5:00, 435 (838) at 6:00, 537 (1,223) at 7:00, 837 (1,431) at 8:00, 356 (952) at 10:00.

## 5. Built (the user: "Sounds good, let's build it")

The sheet's rows, the reference line's three cells, the eco line's flow words for both stores, the `spending` line,
the factory entry's draw line with the marginal, the prompt's sentence (docs/design/2026-10-17-spending-line.md).
One inconsistency found and fixed in the prompt: its prose said the pros had "about five constructors by 5:00" where the
sheet says 3 at 5:00 (4 at the upper quartile) and 4 / 5 / 7 made by 8:00; the prose now says three (the player had quoted the sheet).
