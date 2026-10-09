## The late game: what it takes

The pool of duels between experienced people ends at 12 to 15 minutes at the median and 4 of 80 sides ever started an
advanced factory, so the late game has no reference game. What there is: the arithmetic of the unit files and the
engine's rules; the build-order simulator's frontier (the next part: the fastest order to each milestone on three
maps, with no enemy); and our own long games, most of them lost. Set beside each other they say what the late
game is made of and where ours went wrong. Nothing here is an order.

**The late game is build power on a few frames.** A tier-2 economy is a handful of dear frames (an advanced plant
2,600 metal, a fusion 3,350, a tier-2 extractor 620 each, an advanced fusion 9,700, a silo 8,100) and the power on
each sets how long it stands empty. The power a frame can use is set by what you can spend on it: its build time,
times the metal a second you give it, over its metal; more only stalls (the builders table, under units and costs).
Construction turrets are the cheapest power (1.15 metal a point, reach 400) and stand still, so they pay where the
frames stand together; constructors walk to the frame; air constructors huddle at one without jostling and die to
anything. Massed power on the one frame that matters, then the next, is what the experienced players mean by "stop
building other units while doing t2 transition" (the players' words, on tier 2).

**The arithmetic** (Armada; Cortex's numbers differ by a tenth or so except where given; `docs/knowledge/tier2.md`,
`mechanics.md`, `game-rules.md`).

- A tier-2 extractor: 620 metal and 7,700 energy, four times the spot, 20 energy a second to run. On a Comet Catcher
  spot it adds 6.9 metal a second over the extractor it replaces, so 90 seconds of its own output repays it; on a
  poorer spot (the map sheet's spot value) proportionally longer. One advanced constructor alone takes about 70 s
  an upgrade (K-t2-moho-first, K-t2-moho-energy).
- Energy by price: a fusion 3,350 metal for 750 a second (4.5 metal per energy a second), an advanced solar 350 for
  80 (4.4), a basic solar 155 for 20 (7.75), wind 40 metal over the map's average wind (about 3 on Quicksilver). The
  fusion is not cheaper energy; it is dense, steady and one frame instead of nine (K-t2-fusion-numbers).
- Converters: 70 energy a second into 1 metal, for 1 metal and 1,150 energy; the advanced one 600 into 10.3 for 370
  metal and 21,000 energy, a sixth better per energy and mostly a saving of ground. On Comet Catcher the experienced
  players build none and store the energy (K-mech-converter-rates, K-t2-advanced-converter, the players' words).
- A tier-2 army is an energy purchase: its beam and lightning weapons charge energy a shot, and the recommended
  mixes cost 390 to 500 energy a second at +30 metal and 580 to 740 at +45, over the 20 for each tier-2 extractor;
  below about 150 spare the fights fall off a cliff (K-t2-army-energy).
- A wreck is 60 % of the unit's metal and reclaiming it costs no energy; after an even trade of forty units the field
  holds 2,000 to 3,000 for whoever stands on it (K-mech-reclaim-values).
- Storage grows 600 metal with every tier-2 extractor, so "the store half full" is a larger number the richer you are
  (K-t2-the-late-spend-ceiling-in-a-bot).
- A stockpile takes no build power: a silo's missile is 1,000 metal and 125,000 energy over 120 s (1,042 energy a
  second, 8 metal), an anti-nuke's interceptor 150 and 7,500 over 90 s, and nothing a builder does shortens either.
  The silo's frame (8,100 / 90,000 / build time 178,500: at 100 metal a second 2,200 power and 81 s; one advanced
  constructor alone, ten to fourteen minutes) is where a crew pays (K-rules-a-stockpile-takes-no-build-power).

**What our late games looked like** (the records; the mechanism as we read it).

- **The plant came late.** Our advanced plant stood at 21 to 27 minutes and the first tier-2 extractor at 25 to 33;
  his at 10:41 and 15:12 at the median (player-16, 30, 31, 45, Comet Catcher). Nothing in the game made ours late;
  it was not begun.
- **The frames waited on the store, not the builders.** Over 73 tier-2 frames in five games, a frame with a full
  store finished in 1.0 to 1.2 times its build time over the power standing on it; with the store empty it took
  longer in proportion: player-32's two fusions 274 and 276 s against 186 by the power on them, the energy store
  empty in three samples of four (`run/t2_frames.py`). Our energy was at zero through the whole plant build in
  player-10, 11 and 32.
- **One builder on a frame.** bluegecko-3v1-10 (three seats against a person): one seat's advanced plant at 0 % from
  15:40 until it was abandoned at 19:15, one constructor on it, that seat banking 1,432; nothing sent between seats.
- **The bank was the frames not begun.** human-18 at 17:54: "Energy STALLING with 6.2k metal banked and ~20
  constructors idling"; player-50 held five to ten times his army and thousands in the store for twenty minutes
  (called undecided). The frontier's orders waste 7 metal in thirty minutes.
- **Income peaked at twelve minutes and fell.** player-31: 64.8 metal a second at 12:00, 20.7 at 26:00; player-30,
  31 and 32 held 25 to 39 extractors at 18:00 and half that by 26:00. The outer spots fell to his raids and were not
  retaken, and the tier 2 that would have made the held spots worth four came at 29 to 39 minutes.
- **The transition under his pressure, measured in a bot of ours** (`bots/distilled`, the same economy rules in code,
  not your hands). Against BARb hard_aggressive on Comet Catcher it started the plant at the community's gates
  (income 22 or 30) and won 1 game in 16 and 0 in 8 against 3 in 16 without tier 2: the plant stood at 9:00 to 13:30
  on schedule and the chain ran (constructors within a minute, a fusion by 14:00 to 18:30, tier-2 extractors to 30),
  while the extractors went 16 / 19 / 14 / 8 at 10 / 15 / 20 / 25 minutes against 19 / 33 / 37 / 39 without it and
  the army 12 / 8 / 17 / 21 soldiers against 11 / 32 / 68 / 63. Pausing the tier-1 plants was not the cost (0 of 8
  without the pause); the plant's 2,600 and the builders' minutes out of the army at the minutes his pressure
  peaks were, and the income the chain bought (150 to 245 a second at 25 to 30 minutes) came after the army was
  gone. On Great Divide against medium, 11 spots a side and saturated at 10:00, the bot with tier 2 won 2 of 16 and
  on tier 1 with converters 7 of 8 (K-t2-the-transition-under-pressure-loses-at-either-gate). Behind an army that
  holds his pressure is the case nobody has measured; the frontier's tax stands in for it.
- **Past income 150 the same bot banked at the storage cap** (19,173 of 19,450 at 30:00): generators crowded the
  yard until the site search behind a plant found nothing, an advanced fusion ran at 830 power against a budget of
  ten thousand and took seven minutes, and the energy rule built 40 to 56 solars for a draw that never came. The
  rules were the bot's; the crowded yard and the storage cap are the game's
  (K-t2-the-late-spend-ceiling-in-a-bot).
- **His side**: BARb hard_aggressive's advanced lab at 10:41 at the median, his army flat from 10:40 to 20:00 and
  doubling every five minutes after (the cases, "Tier 2: his and ours").

**What the frontier adds** (the next part, drawn 2026-10-09). With no enemy the fastest orders put the advanced lab
at 6 to 7 minutes behind three or four constructors, the first tier-2 extractor at 6 to 9, the first fusion at 7 to
12, income 100 a second at 10 minutes on forty spots a side and 14 to 18 on eleven, and the first missile at 15:30 to
16:50 under the standard tax. Our games put the plant at 21 to 27. The missile's orders on all three maps share one
shape: a lab, three or four constructors, the advanced lab at 6 to 7 minutes, two advanced constructors, tier-2
extractors on the held spots (on Quicksilver advanced solars and converters instead, the extractors after), one or two
fusions, two or three turrets, help on the silo, and 1,000 to 2,000 energy a second when the round draws its 1,042. On eleven spots a side income 100 is the late ceiling and 200
is never reached; on forty, 200 comes at 19:33 with no tax and 27:33 under the heavy one.

**The nuke.** Armada's silo is 8,100 metal, 90,000 energy and 178,500 of build time (ten to fourteen minutes for one
advanced constructor, three for a thousand power); a missile 1,000 metal and 125,000 energy, stockpiled in two
minutes; the warhead does 12,500 at its centre, falling to nothing 640 away (every tier-1 and tier-2 unit and building
in the game dies at the centre), with 13 s of flight and the whole map in range; a silo holds ten. Cortex: 7,700 / 82,000 for the silo, 1,500 / 187,500 / three minutes a missile. The counter
is an anti-nuke at 1,500 metal and 38,000 energy with a 2,000 coverage, whose first interceptor takes 90 s. BARb
hard_aggressive on Comet Catcher had one in 27 of the 33 long player games, standing at minute 21 to 25 (median 22);
medium in one long game of six, at 25; his own silo stood in 14 of 848 long games, at 25 to 38
(K-barb-anti-nuke-by-minute). So against hard_aggressive a missile must be stockpiled before about 21:00, and the
frontier's 15:30 to 16:50 under the standard tax says the order exists, on paper, on every map we drew; against
medium and against people the window is far longer, and with several seats the question is which seat has no
anti-nuke. No game of ours has built a silo. **Your hands have today no order for a silo's stockpile and none to
fire it**: a silo built now is 8,100 metal that does nothing until those exist.

**What players said** on the late game is under "What players said", on tier 2.
