<!--meta
id: D-vents-take-them
layer: derived
premises: [D-energy-second, D-tier2-sprint, D-metal-is-the-game, D-raid-answer, F-eco-energy, F-eco-build, F-eco-conversion, F-eco-consequences, F-map-build, F-map-features, F-unit-armgeo, F-unit-armageo, F-unit-armgmm, F-unit-corgeo, F-unit-corageo, F-unit-leggeo, F-unit-legageo, F-unit-armsolar, F-unit-armadvsol, F-unit-armfus, F-unit-armmmkr, M-pool-energy-and-power]
lessons: []
claims: [K-eco-people-take-the-vent, K-mech-a-geothermal-plant-stands-within-36-of-its-vent, K-mech-a-plant-over-a-plant-refunds-the-one-beneath]
sources: {people: [the user], dates: [2026-11-15], relay: the user's observation ("geothermal power spots. We never build them ... Most maps have them ... and some maps that puts us significantly behind"), written on his word after the research report ("Go ahead"); the practice is counted from the carded public team games and the cached public duels above OS 40 (K-eco-people-take-the-vent); the prices are arithmetic on the unit files; the rules of the vent are read from the engine and the game and measured in the bots' games}
status: draft, written 2026-11-15 for the user's read and not yet read by him; in no manifest; no player game has been played on it
-->
### A vent is the cheapest energy on the map

**The heuristic.** Where the map has a geothermal vent on our side, take it: the tier-1 plant once the energy income
carries its bill, and the advanced plant over it as tier 2's first energy, before the first fusion. People do: on All
That Glitters 311 of 332 sides of the public 8v8s built the tier-1 plant, in duels above OS 40 on maps with vents
85 of the 114 sides whose game ran ten minutes (K-eco-people-take-the-vent). Before 2026-11-15 no seat of ours had
built one. Most of the maps the lobby plays have vents, two to nine each; Comet Catcher, Quicksilver and Great Divide
have none, and there this article says nothing.

**The quantity.** Metal for each unit of energy a second, beside the builder's time. The tier-1 plant is 560 metal
for 300 a second, 1.9; a solar is 7.75, an advanced solar 4.4, a fusion 4.5, a turbine 2.1 on a map whose wind
averages 19 and 13 on Comet Catcher's. The advanced plant is 1,600 for 1,250, 1.3. One vent is fifteen solars, or
with the advanced plant on it more than a fusion and a half. What it costs beyond metal is energy and a builder: the
tier-1 plant is 13,000 energy and 13,100 of build, two and a half minutes of one construction vehicle drawing about 90
energy a second, and a walk to wherever the vent is.

**The mechanism.**

- *Only on a vent, and by a constructor.* A geothermal plant stands on a vent and nowhere else (F-map-build); the
  commander builds none. The `map` tool lists this map's vents as `vent_N` and the score line of every report says
  what stands on each.
- *The bill is energy.* 13,000 energy is a minute of an income of 220. Begun in a stall it starves the plant beside
  it for as long as it takes; begun with the store above half it is paid from the store (D-energy-second).
- *The upgrade is in place.* The advanced plant may be built on a bare vent or over our own tier-1 plant; when it is
  finished the game takes the tier-1 plant away and gives its 560 metal back. So the tier-1 plant costs nothing
  that the advanced one does not return, and a vent taken early is not a vent spent.
- *Energy is metal past the first converter.* 1,250 a second through two advanced converters is about 21 metal a
  second: 2,360 metal for it, about 110 for each metal a second, against 90 for an advanced extractor and about 290
  for a fusion with its converters (F-eco-consequences; ours, derived).
- *It dies loudly, and it is a building far from home.* The tier-1 plant's blast is 1,420 over 520 across, the
  advanced one's a fusion's (10,600). A vent is where the map put it: beside a start on some maps, in a corner
  both sides reach on others (Onyx Cauldron's two lie as far from one start as from the other).
- *One a side in a team.* All That Glitters has two vents for sixteen seats: one seat of a side holds the side's.
  A plant built over an ally's plant becomes the ally's.

**Consider, and why it may work.**

1. *The tier-1 plant on our vent when the energy income is about 200.* People begin it at 6:03 / 7:37 / 9:45 in
   duels (the quartiles) and at 3:26 / 4:55 / 6:24 in the 8v8s, where the seat beside the vent does. It replaces the
   next fifteen solars and the commander's time on them.
2. *The advanced plant over it as soon as an advanced constructor stands and the metal is there.* It is less than
   half a fusion's metal for two thirds more energy, and it brings the tier-1 plant's metal back. In duels the first
   tier-2 vent building is begun at 12:15 / 14:24 / 18:01.
3. *A turret and a radar's warning at a vent that is not inside the base,* as for an outer pair of extractors
   (D-raid-answer): 560 metal and two and a half minutes of a constructor stand there.
4. *A second vent of ours the same way,* before more solars or a fusion, while one is free on our side.

**Where it bends.** (1) No vent on the map, or none on our side: nothing. (2) A vent he can reach as soon as we can
is an outpost, not a generator: the plant is his target and its builder walks into his raiders; take it with the
army's cover or leave it. (3) In an energy stall a solar comes first: it stands in nine seconds of the commander and
costs no energy. (4) A game that will be over in ten minutes does not pay the walk back (the 8v8 sides with no plant
played 10:55 at the median). (5) Behind in metal the plant is one of the things that pay later (D-behind-tempo).
(6) An ally's plant on the side's one vent is the side's energy already.

**When to depart from the line.** When the vent's ground is under his guns, when the builder is the only one we have
and free spots stand open beside it, and in the opening before the plant makes units.

**The tests.** K-eco-people-take-the-vent: the counts above, from the 168 carded 8v8s on All That Glitters and the
72 cached duels on maps with vents (small per map: Gasbag Grabens 20 of 20 sides, Cape Violet 11 of 12, Canis River
18 of 38). Counts of what people do, not win rates: which side built a plant is tangled with which side lived
longer. The rules of the vent are measured (the plant stands 5 to 9 elmos from the vent; the advanced plant over our
tier-1 one takes it away within a minute or two of standing). Our code-played bots with the rule on and off
(2026-11-15): WRDistilled against BARb on seven maps with vents 26-11-19 against 21-17-18 over 56 games a side, the
plant begun at four to five and a half minutes; its economy alone on Onyx Cauldron 347 metal a second at 20:00
against 312; WRTerritory's on All That Glitters 1,067 against 1,003. Small samples, all leaning one way. No
flagship game has been played with vents on the menu.
