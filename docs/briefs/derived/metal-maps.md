<!--meta
id: D-metal-maps
layer: derived
premises: [D-late-economy, D-metal-is-the-game, D-energy-second, F-map-metal, F-map-metal-maps, F-eco-extractors, F-eco-build, F-eco-conversion, F-eco-consequences, F-unit-armmex, F-unit-armwin, F-unit-armnanotc, F-unit-armvp, M-gecko-duels, M-pool-energy-and-power]
lessons: [L-gecko-three-resources, L-sheet-economy]
sources: {people: [thebluegecko], dates: [2026-09-12, 2026-10-30], relay: his two Full Metal Plate 1.7 duels against Artur91 as carded (M-gecko-duels; the replays `2dcaa56a3025256cccfdd7c22622190f` and `51d2a56ad4a0322bfc5192a02066b262`, run/data/maps/full_metal_plate_1.7.sd7 held and not yet read), the pool's energy survey (M-pool-energy-and-power); the rest is arithmetic on the facts}
status: live (2026-11-03, the metal-maps design step 3: in the rework manifest; the two maps read by the shim and played by our hands, F-map-metal-maps; the play still rests on two games of one player, and the article says so)
-->
### Metal maps: the loop cut

**The heuristic.** None of the players' lines is about these maps; the pool's two Full Metal Plate duels are his
(M-gecko-duels) and people play the map "occasionally, for the fun of it" (the user, 2026-10-30). What the two games
say, as a line: on a metal map the extractor is the whole economy, the plant is the build power, and the game is the
army's scale.

**The quantity.** The same two as the late economy's (D-late-economy), with the price list cut to two entries: the
extractor and the generator. Metal is never the short resource after the first minute; energy and build power are the
only prices, and the army is counted by the hundred.

**The mechanism.**

- *Every square is a spot.* Metal lies in the map's metal map, a byte per 16-elmo square, and an extractor draws from
  every square within the map's extractor radius (F-map-metal). Where the bytes are high across the whole field the
  spot list is not a list of places but a tiling: an extractor stands wherever it is built and draws its radius's
  worth. His games: 154 extractors at 10:00, 408 at 20:00, 632 at 30:00 in the won game; 231 at 10:00 in the lost
  one (M-gecko-duels). At 50 metal and 1,800 of build an extractor is the cheapest metal a second in the game by far,
  and nothing else competes for the constructor's time.
- *The late economy's loop is cut at the converter.* D-late-economy's regime is the one in which the spots run out
  and income grows only by conversion; here the spots never run out, so conversion (224 to 540 metal per metal a
  second) never beats the extractor and the loop's third leg, energy into metal, is pointless. What remains of the
  regime is its other two legs: build power to spend the metal, and energy to pay for the build power and the army.
- *The energy price is the only price.* Everything a side builds is priced by its energy per metal (F-eco-build): an
  extractor 10, a constructor 14.4, a turret 13.9, a Blitz 8.2, a Stout 9.3. With metal at cap the generator is bought
  for the draw, not the store (D-energy-second item 1), and on this map the generator is the turbine: the pool's two
  duels built no solar, 48 turbines by 5:00 and 186 by 8:00 for 1,180 energy a second at 5:00 against Comet's 190
  (M-pool-energy-and-power). The map's wind is the first thing to read (F-map-weather), as on every map.
- *Build power is factories.* With metal unlimited the limit on the army is how fast it can be made, and a plant at
  590 metal for 150 of power that also makes the units is the build power to buy: his won game had a vehicle plant at
  0:50 and seven more by 8:48, two advanced plants at 11:09, a gantry at 14:45 and four more by 21:00. Turrets on the
  plants add 200 a piece at 230 metal, the same arithmetic as D-late-economy's block, with the plant in the frame's
  place.
- *The army is the game, at a scale the other maps never reach.* 23 Incisors at 6:00 and 54 at 8:00, 100 Brutes and 52
  Pounders standing at 12:00, 251 Rascals at 30:00, Shivas by the hundred and Tzars at the end; he lost the second
  game at 16:27, with 231 extractors at 10:00 and the plant at 0:15, to 166 Stouts and 34 Bulls. The knob of
  D-late-economy is set by the opponent's army arriving at this scale: an economy that out-expands its army dies to
  the army that did not.

**Consider, and why it may work.**

1. *Extractors with every constructor, everywhere, from the first second.* Why: 50 metal for 2 to 3 a second with
   no walk between spots; nothing else on the list comes close, and the count is the income.
2. *Factories as the build power, turbines as the energy, both bought for the draw.* Why: metal at cap means the
   question is seconds and energy; a plant makes the army and spends the metal at once, a turbine is the map's
   cheapest energy where the wind allows, and the pool's two sides built 186 turbines and no solar.
3. *Converters never.* Why: the loop is cut; a converter's metal is the dearest on the map while an extractor can
   stand anywhere.
4. *Count his army, not his income.* Why: both sides have metal; the game is decided by the army that arrives first
   at scale, as his loss at 16:27 shows against a side that built 200 tanks on fewer extractors.

**Where it bends.** (1) Which maps: Full Metal Plate is the one in the pool, and it is one plate, every square the
same (F-map-metal-maps), so its places are the grid's cells alone; SpeedMetal is 48% metal in one field whose
corners hold most of it and whose highway holds little, so there the fight is for the corners and the ground
between them, and a map with metal in bands rather than everywhere keeps a part of the spot economy. (2) Tier 2: the advanced extractor's four times the spot
is four times a square's worth too, so the upgrade's price per metal a second is the same ratio as elsewhere, but
against a 50-metal extractor on the next square it is never the cheaper buy; his tier 2 was plants and gantries, not
mohos. (3) The late economy's dispersal rules hold with the targets changed: the turbines are the field and the
plants the block. (4) Reclaim is nothing here: a wreck's metal is a square's few seconds.

**When to depart from the line.** When his army is the larger: the plants over the extractors, since the extractors
are already more than the plants can spend. When the wind is low: the solar's steady 20 at the map's price, read as
D-energy-second says.

**The tests.** Two games of one player, both read from cards. The map is read now (F-map-metal-maps): a tier-1
extractor on Full Metal Plate draws 2.3 to 2.4 a second, so his 154 extractors at 10:00 were about 360 metal a second,
and on SpeedMetal our hands' 53 extractors at 19:00 drew 1,624 with the energy store at zero most of the game, the
generators setting the pace as the mechanism says. A game of ours against a person on either map is still to come.
