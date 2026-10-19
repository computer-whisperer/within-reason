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
- Energy by price: an advanced fusion 9,700 metal for 3,000 a second (3.2 metal per energy a second), a fusion 3,350
  for 750 (4.5; Cortex 3,600 for 850, 4.2), an advanced solar 350 for 80 (4.4), a basic solar 155 for 20 (7.75), wind
  40 metal over the map's average wind (about 3 on Quicksilver), a geothermal 560 for 300 (1.9) and the advanced one
  1,600 for 1,250 (1.3), on a steam vent only. The fusion is not cheaper energy than advanced solar; it is dense, steady
  and one frame instead of nine. The advanced fusion is the cheapest energy off a vent, by a quarter, and the engine of
  the converter economy below (K-t2-fusion-numbers, K-t2-post-moho-scaling).
- Converters: 70 energy a second into 1 metal, for 1 metal and 1,150 energy; the advanced one 600 into 10.3 for 370
  to 380 metal and 21,000 energy, a sixth better per energy and mostly a saving of ground. The converter's own price is
  small; the energy is the cost (1,940 metal of advanced fusion for the 600 a second one advanced converter eats). On
  Comet Catcher the experienced players build none and store the energy, the duel being over before the spots run out
  (K-mech-converter-rates, K-t2-advanced-converter, the players' words).
- A tier-2 army is an energy purchase: its beam and lightning weapons charge energy a shot, and the recommended
  mixes cost 390 to 500 energy a second at +30 metal and 580 to 740 at +45, over the 20 for each tier-2 extractor;
  below about 150 spare the fights fall off a cliff (K-t2-army-energy).
- A wreck is 60 % of the unit's metal and reclaiming it costs no energy; after an even trade of forty units the field
  holds 2,000 to 3,000 for whoever stands on it (K-mech-reclaim-values).
- Storage grows 600 metal with every tier-2 extractor, so "the store half full" is a larger number the richer you are
  (K-t2-the-late-spend-ceiling-in-a-bot).
- A stockpile takes no build power and fills itself: a silo's missile is 1,000 metal and 125,000 energy over 120 s
  (1,042 energy a second, 8 metal), an anti-nuke's interceptor 150 and 7,500 over 90 s, and nothing a builder does
  shortens either (measured: 142 s to the first missile with and without two turrets on the silo). Nobody orders a
  round: the game queues ten the moment a silo stands and keeps it at ten, so a standing silo draws 1,042 energy a
  second for twenty minutes, and again after every launch, unless its kept count is lowered; an anti-nuke the same,
  its first interceptor 90 s after it stands. The silo's frame (8,100 / 90,000 / build time 178,500: at 100 metal a
  second 2,200 power and 81 s; one advanced constructor alone, ten to fourteen minutes) is where a crew pays
  (K-rules-a-stockpile-takes-no-build-power, K-rules-a-stockpile-fills-itself-to-its-limit).

**Scaling past the mohos** (the arithmetic; K-t2-post-moho-scaling). A tier-2 extractor is the cheapest metal in the
game, +6 a second for 620 on an ordinary spot and repaid in 104 s, so every held spot is upgraded before anything
below is begun; a seat with fourteen spots stands at about 110 metal a second when all are mohos, drawing 280 energy a
second in upkeep, which wind and solars do not carry for long. From there the income grows only by turning energy
into metal, and the chain that does it at the best price is: one or two fusions (750 to 850 a second each; 39 s with
two advanced constructors and four construction turrets on the frame) for the mohos' upkeep and the next frame's
energy bill; one advanced fusion (9,700 metal and 69,000 energy, 48,000 for Cortex; 3,000 a second at 3.2 metal per
energy a second against the fusion's 4.5); then advanced converters by the handful, 380 metal each turning 600 energy
a second into 10.3 metal. One advanced fusion feeds five: 11,600 metal for +51.7 metal a second, repaid in 3.7
minutes, against 4.9 minutes for a fusion with its converter and a quarter of the frames; the next engine is paid from
the last one's output, and a seat at 110 a second adds one every two minutes, each adding half again to its income.
The advanced fusion's frame is the catch: build time 312,500, nine minutes for two advanced constructors alone, 143 s
with eight construction turrets beside them drawing 68 metal and 480 energy a second, so the fusions stand first, the
site is where the turrets can stand round it, and it stands alone under anti-air, since it levels everything near it
when it dies and bombers come for it. Each engine brings its own store (a fusion 2,500 energy, the advanced fusion
9,000, a moho 600 metal) and the converters live on the energy above three quarters of it, so an energy storage (170
metal for 6,000) beside the first fusion is where the conversion begins to run smoothly, and a seat "banking it
unused" has the store full, not the economy served (K-eco-energy-storage-is-the-cheap-buffer). Our games have not
reached this ladder: the +50 Altored 3v3 of 2026-10-18 built
one, two and three fusions a seat from 16:20 and no advanced fusion, and one seat banked 2,000 energy a second unused
from 16:30 to the end with a single converter while its metal store sat at 150 to 300 and the report read STALLING
(the user, 2026-10-18: "one or two fusions is reasonable as a transition to advanced fusion, and once there heavy
investment in advanced converters is what lets players scale further").

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
- **The first game with a silo** (nuke-1-medium, Comet Catcher, BARb medium, the objective set to the nuke; undecided
  at the 40-minute cap). The advanced plant begun 8:38 after a fourteen-Ak raid at 8:23 took its first frame, three
  constructors and eight extractors; fusion 14:26; the silo listed with one id for an advanced constructor, the
  commander and four constructors, begun 14:29, stood 17:31 (182 s against 174 by the power on it); the first
  missile ready 20:00, fired 21:07 at his lab: everything of his within 640 gone except his commander. Eight
  missiles by 37:37, one every two minutes; two fell on points where nothing stood (a lab the picture still listed
  four minutes after the third missile had killed it out of sight), two on his commander where it stood at the
  order, and it lived both times (22 s from order to impact; a commander walks). Extractors 23 at 16:00 fell to 10
  at 24:00 under his raiders while nine soldiers held one post and the silo was paid; then 28 and income 218 by
  36:00 with 17,000 banked at the cap while an advanced fusion and a second silo were built. He had two extractors,
  two turrets, five raiders and his commander left against our 239 soldiers at 40:00; the body reached his base at
  38:31 (K-nuke-the-first-flagship-game).
- **The second game with a silo** (nuke-2-hard, Comet Catcher, BARb hard_aggressive, the same objective; lost at
  32:22). The advanced plant stood 10:01, the fusion 14:29, the silo begun 13:19 and stood 16:32 with about 1,000
  build power on it and the store empty a quarter of the time; the first missile was ready at 18:38, two by 20:38,
  three by 22:38. His anti-nuke was begun at 21:40 and stood 28 seconds later; its first interceptor was ready by
  23:38. The player asked for a firing twenty seconds before the first round (refused) and then not again until
  23:39: it believed the silo held nothing ("energy-limited"), and it sent the army north to find his base so the
  missile would have a target, a base whose place it had already named by its coordinates and never saw. The three
  missiles then sent at his base were all intercepted, over his vehicle plant, a fusion, a gantry, a Shiva, three
  advanced extractors and fifteen construction turrets that two missiles at 20:38 would have taken. The four sent at
  his forward army landed: a Banisher, two Tzars, two Erasers, about five forward constructors, extractors and
  turrets, and two Stumpies of ours standing inside one blast. His tier-2 army then took twenty of our twenty-eight
  extractors between 24:00 and 32:00 while a second silo and an advanced fusion were paid from a bank at the cap, and
  the commander, sent to rebuild extractors at the front, was killed in nine seconds by two Sokolovs it never saw
  (K-nuke-the-second-flagship-game).

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
anti-nuke. In the engine the missile goes nine seconds after the order and lands thirteen seconds later, and an
anti-nuke with an interceptor ready within 2,000 of the target takes it (the check of 2026-10-09). Two games of ours
have built one (above). Against medium the missile wiped his base at 21:30 and the game was undecided at 40:00, his
commander having walked out of two blasts aimed at where it stood and two missiles having gone to remembered buildings
that were already dead. Against hard_aggressive the silo stood at 16:32 and two missiles were ready a minute before his
anti-nuke was begun, and they sat unfired for five minutes while the army went looking for a base whose place was known;
every missile sent at his base after the anti-nuke stood was taken, every one sent at his army in the field landed. A
missile needs no sight of its point, only its coordinates: his start is on the map sheet and his plant is where his
first units came from. The first round comes 120 s after the silo stands when energy holds; the report's `missiles`
line says every turn how many each silo holds, and a round coming ready wakes you; against hard_aggressive his base is the target only before minute 21, and
after that the missile is for what stands in the field, two thousand from any anti-nuke of his. The army, not the
missile, ends a game. The `fire` tool sends a ready missile at a place in coordinates; the kept count is the game's ten for
now (a silo at its limit is a 1,042 energy a second draw until it holds ten), and a silo or anti-nuke listed without a
place goes to the back field at home with the generators.

**What players said** on the late game is under "What players said", on tier 2.
