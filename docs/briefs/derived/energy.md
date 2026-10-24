<!--meta
id: D-energy-second
layer: derived
premises: [D-metal-is-the-game, D-lab-choice, D-tier2-sprint, F-eco-store, F-eco-energy, F-eco-build, F-eco-conversion, F-eco-consequences, F-map-weather, F-map-catalogue, F-map-consequences, F-unit-armsolar, F-unit-armwin, F-unit-armadvsol, F-unit-armtide, F-unit-armfus, F-unit-armestor, F-unit-armmakr, F-unit-armnanotc, F-unit-armap, M-pool-energy-and-power, M-pool-openers, M-pool-opening, M-gecko-vs-us-comet]
lessons: [L-gecko-three-resources, L-opening-solars, L-opening-solar-cap, L-energy-converters, L-energy-nanos, L-energy-priority, L-sheet-economy, L-sheet-labs, L-gecko-bots-vs-vehicles]
sources: {people: [thebluegecko, Koochie (as heard), Irishstud14, the user], dates: [2026-09-27, 2026-10-23, 2026-10-24], relay: thebluegecko reading metal.md's "Where it bends" aloud (docs/knowledge/_inbox/gecko-brief-review-2026-10-23.md, the first recording), his unit sheet rows, the 3v1 chat lines; the prices are arithmetic on the facts; the habits are the pool's and our own games' (run/metal_killtest.py energy)}
status: draft, checked (run/metal_killtest.py energy, 2026-10-24: the pool's 84 sides and 242 model-played games of ours)
-->
### Energy, the second resource

**The heuristic.** thebluegecko, reading the metal article (L-gecko-three-resources): "Metal is most important. Second
is your e, how much you've got, and how you get it ... You can think of the game as having three resources. Metal, e,
and BP. Metal is the basic resource. Everything grows from it. E is sort of like a luxury resource. It's the resource
that grows out of having more metal and spending it. BP is how fast you can spend ... But BP tends to cost a lot of e."
His sheet on the solar collector (L-sheet-economy): "when e stalling this is generally the answer, but on most maps not
what you want to use to scale e into the mid or late game. lowest build power cost per e." The chat lines of the 3v1s
say the rest: "get to 200+ E per second and add nanos", "con turrets take a lot of e", "if you have extra e make an e
store" (L-energy-nanos, L-energy-converters). Safe to follow as it stands: energy in proportion to what the spending
draws, bought at the map's price, never as an end.

**The quantity.** Energy income against the draw of what is being built, and the price paid for it in metal and in
build time, which differs by map. Not the stall count: a stall is seconds in which the draw ran ahead of the income,
and the winners of our games stall more than the losers because they spend more (the tests). Not the store either: a
full store with metal banked says build power is short, an empty store with metal empty says metal is, and only an
empty store with metal banked says energy.

**The mechanism.**

- *The price list.* A solar is 155 metal and 2,600 of build for 20 a second: 7.75 metal and 130 of build per energy a
  second, with no energy to build it. A turbine is 40 metal, 175 energy and 1,600 of build for about three quarters of
  the map's top wind (F-eco-energy): on Comet Catcher's 1 to 4 that is 3 a second, 13 metal and 530 of build per energy
  a second; on a map whose wind tops at 12 (Carrot Mountains, Flats and Forests) 9 a second, 4.4 metal and 180 of
  build; at a top of 16 (Gecko Isle, All That Glitters, Gasbag Grabens, Boreal Falls) 12 a second, 3.3 metal and 133
  of build; at a top of 20 (Great Divide, Cape Violet, Isidis) 15 a second, 2.7 metal and 107 of build. An advanced
  solar is 4.4 metal and 99 of build per energy a second but costs 5,000 energy, 62 seconds of its own output; a tidal
  generator is 90 metal over the tide (1.2 metal per energy a second on Center Command's 75); a fusion 4.5 metal and 72
  of build; an advanced fusion 3.2 (F-eco-consequences, the unit tables). So the turbine is cheaper in metal wherever
  the wind averages above 5.2, which is every catalogued map but Comet, and the solar is cheaper in build time wherever
  it averages below 12, which is every catalogued map but the five with a top of 19 or 20. Between the two lines, the
  maps with tops of 12 to 17, is his "it comes out to pretty close to a tie", and the tie is broken by the swing: a
  turbine's output follows the wind, which picks a new target every fifteen seconds, and what the store cannot cover
  the solar's steady 20 does. Koochie's preference (as heard) is the build-time line read as tempo: on those maps the
  solar costs 2.3 times the metal and no more time, so where the metal is banked and the commander's 300 of build is
  the constraint the solar field stands sooner and the army with it.
- *The draw.* Energy is spent by builders at the rate of what they build (F-eco-build): a plant alone making Blitzes
  draws 68 a second, three and a half solars; with the commander assisting 203, ten solars; making constructors 72 and
  217; Stouts 93 and 278; a construction vehicle putting up an extractor 25, the commander 83 for six seconds. The bill
  of a thing is its energy per metal: a Blitz 8.2, a Stout 9.3, a Grunt 19.1, a Pawn 16.7, a constructor 14.4, an
  extractor 10, a construction turret 13.9, a solar 0. So a thousand metal of Pawns is 16,700 energy, fourteen minutes
  of one solar, and a thousand of Blitzes 8,200, seven; the energy a side needs follows from what its plants make and
  who assists them, not from how many plants stand, and a second plant with the commander on it is two hundred a
  second more, ten solars or 1,550 metal, before its first unit.
- *Build power costs energy, which is his order.* A construction turret is 230 metal and 3,200 energy for 200 of
  build; a bot lab 500 metal and 950 energy for 150 of build that also makes soldiers; a construction vehicle 135
  metal and 1,950 energy for 90. Two turrets' energy, 6,400, is five and a third minutes of one solar, or 50 metal of
  turbines on a windy map and 250 metal of solars on Comet. That is why the order after metal is energy and then build
  power, and why where energy is dear "rather than building a bunch of con turrets, you tend to build a second
  factory. Maybe one con turret into a factory ... you can build that factory off of your comm after walking to take
  more metal. And it costs less e." The pool on Comet, a solar map, does exactly one turret before the second factory
  (5:24 to 6:31, then 7:12 to 8:56; M-pool-energy-and-power).
- *The stall is a queue, not a brownout.* A builder's step is all or nothing, the oldest builder is served first (the
  commander, then the first plant, then the constructors by birth), and a low-priority builder yields (F-eco-build).
  So in a stall the youngest and dearest steps stand still while the cheap old ones go on; the cure is either income
  or fewer draws, and which depends on the metal store: banked metal with an empty energy store is a generator short,
  both stores empty is metal short and no generator helps (L-opening-solars: the third solar is for the fast
  constructor, not for its own sake; L-opening-solar-cap: more than three early is a mistake).
- *The late forms.* Advanced solars on a solar map once a store stands (L-sheet-economy); the store itself, 170 metal
  for 6,000, as the buffer the wind and the D-gun both need ("if you have extra e make an e store, you will need the e
  later anyway", L-energy-converters; 23 of 80 pool sides built one, the first at 8:51 to 12:27); the fusion as the
  tier-2 bill (D-tier2-sprint: a plant alone on Bulls draws 339 a second; one advanced solar per 80 of new draw, or
  the fusion); and the converter last, 540 metal per metal a second with its solars against 22 for an extractor
  (F-eco-consequences), "on solar maps, eat the solar instead" (L-sheet-economy).
- *His energy is a target.* A solar closes when hit and makes nothing for ten seconds after (F-eco-energy), so a solar
  field under fire is dark; his sheet calls bombers "the lowest cost way to destroy opponents E production after the
  map is split ... most pro players make air lab around 30m/s on many maps" (L-sheet-labs), and the bot player who
  built more energy "may have an advantage in going air first ... bomb the enemy's E earlier, you can spiral that E
  advantage" (L-gecko-bots-vs-vehicles). Against us he had an air plant at 11:29 to 17:06 in the replayed Comet games
  and Stormbringers from 16:00 (M-gecko-vs-us-comet).

**Consider, and why it may work.**

1. *Buy the generator for the next draw, read off the table, not off the store.* Before the second plant, the
   advanced plant or the commander's assist, the energy it will draw is known (200 a second for a plant with the
   commander on it, 342 for an advanced plant under five builders), and so is its price on this map; a side that
   builds it first never sees the stall that every debrief of ours names, and one that reads the stall first has
   already lost the seconds. The store's reading tells which resource is short only together with the metal store.
2. *The map sets the generator.* Solars where the top wind is under 12 and wherever the metal is banked and the build
   time is not; turbines where it tops 19 or more; either between, the turbine for metal and the solar for time and
   steadiness; the tide where it stands about ten above the wind (L-sheet-economy) and 20 of water allows it. On
   Comet that is solars only, as the pool plays it (16 by 8:00, no turbine in 80 sides).
3. *A solar as the tempo buy.* When metal sits banked and energy is what the plants wait for, the solar costs no
   energy and 8.7 seconds of the commander, and the commander, after the plant, is the cheapest build power there is;
   "lowest build power cost per e" is Koochie's whole argument and it holds on every map whose wind averages under 12.
4. *Where energy is dear, a factory before the turrets.* One construction turret, then the second factory, is the
   pool's order on Comet and his rule for solar maps; the factory's 950 energy against the turret's 3,200 is the
   reason, and the factory's units are the rest of it.
5. *A store before a converter on a metal map.* The store buys the swing and the D-gun for 170 metal; the converter
   buys metal at twenty-five times the extractor's price while a spot stands free. Irishstud14's "you won't need
   converters" is Comet's; the sheet's "making too many is a common trap" is every map's.
6. *His energy at 30 metal a second.* His sheet's clock for the air plant; a bomber over a solar field turns off his
   plants for ten seconds a pass, and the energy-heavy opener (bots, Pawns at 16.7 a metal) is the one that hurts.

**Where it bends.** (1) Comet's wind is 1 to 4: the tie is other maps', and here the generator question is only how
many solars and when. (2) The stall is not the enemy: in our 242 model-played games the winners stalled 6% of the
seconds between 2:00 and 12:00 and the losers 3%, and the games stalled a tenth to a quarter of the time were won as
often as any; a stall says the side is spending, and the cure is read off the metal store. (3) Energy per metal of
income runs lower in the winners, not higher: 8.8 against 9.2 at 5:00 and 9.3 against 10.3 at 8:00 in the pool,
8.8 against 10.6 and 9.1 against 10.9 in our Comet games; the losers' energy is not short, their metal is. Energy
second means in proportion. (4) The opener moves the proportion: the pool's bot sides run 11 energy per metal of
income at 5:00 to the vehicle sides' 8.4, three solars more on the same metal, which is the lab article's mechanism
(D-lab-choice). (5) Tier 2 changes the prices and the draw at once: units at 13.7 to 14.5 energy a metal and the
fusion at 3,350 metal (D-tier2-sprint). (6) In a team the energy is the seat's unless shared, while the metal gifts
go to the compounding seat (D-metal-is-the-game); a seat that builds the side's air plant carries the side's energy
bill for it.

**When to depart from the line.** When the metal store is banked and the energy store full: build power, not energy.
When both stores are empty: nothing, the stall is metal's and a generator only delays the next extractor. When his air
plant stands: the solars are his target and the field's anti-air is energy spending too. When a spot stands free:
never a converter. When the wind tops 19 and the store can carry the swing: turbines even for the opening.

**The tests.** `run/metal_killtest.py energy` (2026-10-24). The pool's 84 sides at OS 40+: on Comet no side built a
turbine before 8:00 and the median side had 8 solars at 5:00 and 16 at 8:00; the two Full Metal Plate duels built no
solar and 48 turbines by 5:00, 186 by 8:00, for 1,180 energy a second at 5:00; metal standing in generators over army
metal was level between winners and losers (1.61 against 1.53 at 5:00, 1.52 against 1.64 at 8:00); energy per metal
of income was lower in the winners at every minute (8.8 / 9.2 at 5:00, 9.3 / 10.3 at 8:00, 11.6 / 13.6 at 12:00);
the bot openers ran 11.0 to the vehicle openers' 8.4. Ours, 242 model-played games with both sides' factories known:
energy stall share 2:00-12:00 won 0.02 / 0.06 / 0.10, lost 0.01 / 0.03 / 0.07 (quartiles); by the share, under 2%
won 27 of 79, 2-10% 51 of 113, 10-25% 23 of 46; energy per metal of income won 9.4 against lost 11.1 at 5:00 and 9.9
against 12.0 at 8:00 (Comet, no bonus, fourteen minutes or more: 8.8 against 10.6 and 9.1 against 10.9); energy
income itself higher in the winners (190 against 170 at 5:00, 321 against 260 at 8:00, 570 against 390 at 12:00),
in proportion to the metal. The kill-test as named: had energy been the binding constraint, the winners would show
the higher energy per metal and the fewer stalls; they show the lower ratio and the more stalls in every cut, so the
energy follows the metal and the spending, which is his order. What the tests cannot do: the pool is one solar map
and four sides of one wind map, so the price list's middle band is arithmetic, not habit; and no game of ours yet
carries a turn that named the next draw and bought its generator first, which is the test that matters, when the
episode index exists.
