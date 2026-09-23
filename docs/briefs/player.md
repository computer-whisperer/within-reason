

# What we know (the player's brief, written 2026-09-21 evening)

This is the project's accumulated knowledge, from some hundred recorded games against this opponent's medium setting,
unit duels, the opponent's own configuration, and seven games of your hands playing alone from a default packet against
its easy setting. Each line names its source entry in `docs/knowledge/`. It is evidence, not orders: say in a `note`
when the game in front of you contradicts it.

**Units (tier-1 bots; metal / health / speed / range).** Armada: Tick `armflea` 21/60/132/140 scout; Pawn `armpw`
54/370/87/180 raider; Rocketeer `armrock` 120/720/51/475; Mace `armham` 130/1000/46/380 line unit; Centurion `armwar`
270/1590/45/325 brawler; Crossbow `armjeth` anti-air ONLY, it cannot hit ground units; Lazarus `armrectr` 130, repairs,
reclaims and resurrects, cannot fight. Cortex: Grunt `corak` 43/280/81/215 raider; Aggravator `corstorm` 110/740/48/475;
Thug `corthud` 140/1100/45/380 line unit; Trasher `corcrash` anti-air only; Graverobber `cornecro` resurrects.
[K-units-t1-bot-roster, K-units-anti-air-is-air-only]
- At equal metal, head on: Mace and Centurion (Cortex: Thug) win most tier-1 matchups, including against everything this
  opponent fields and against light turrets. Pawn beats only Grunt and loses badly to Thug. Rocketeer loses to every
  mobile unit it cannot outrange, and is for turrets and buildings: it outranges the light laser tower (475 against
  430). [K-units-duel-line-bots-beat-raiders, K-units-duel-range-vs-turrets]
- Raiders are for what cannot shoot back: extractors, constructors, lone turrets under construction. A Pawn or two
  standing on an approach also stops a lone enemy raider; chasing raiders with the army does not. [K-units-dont-chase-raiders]
- Defence is the player's: nothing in the code answers a raider at a structure on its own, and told only "engage", the
  hands send the whole ball after one scout car and never catch it while another kills a lab at home (realtime-2, won
  anyway on hard). A single Tick or scout car at an extractor is met by one soldier from the nearest group (`send_against`
  with `how_many` 1; two or four for a small party) or by a group left where the raids pass; the packet says which,
  before the ball leaves. The raids are standard and expected (Ticks from 5:00 on Comet Catcher, outermost spots
  first), so the answer stands across the front before they come: pickets of one or two Blitzes at each outer spot
  cluster from the first Blitzes out of the plant, told in words to attack on sight, with a light turret beside each
  pair of extractors; a home guard that walks out after the extractor is gone is not defence (the user, watching
  escalate-4: 33 extractors lost, most to single Ticks at spots no soldier stood near). [K-hands-ball-chases-lone-raiders,
  K-army-pickets-across-the-front]
- Fights are decided by value on the spot: the side with more metal of soldiers there, a turret counting about three
  times its metal, loses less nine times in ten; beyond 2:1 almost always. Our units move at different speeds and a
  group sent across the map arrives strung out unless it is sent with `fight_to`, which marches it together.
  [K-army-combat-prediction, K-army-waves-die-to-static-defence, K-army-piecemeal-midmap]

**This opponent (BARb).** It has no resource or vision cheats. [K-barb-no-resource-cheats]
- It raids from the first minutes: raiders gather at its base and leave as a squad of about four, then every new raider
  joins the raid at once. They go for buildings where the threat is low, builders and extractors first, outermost
  first, and skip a target whose defence is worth more than about three quarters of the squad: one turret and two
  soldiers on an extractor is usually enough. [K-barb-raid-timing, K-barb-raid-threat-gate]
- It builds a turret beside nearly every extractor, and forward turret nests in our half later on. Its base has turrets
  on a clock (minutes 2, 4, ...). Its commander builds at home and is the win condition; in one easy game it walked into
  our half with a party and died to a group of ours that outweighed it. [K-barb-medium-observed-build,
  K-barb-base-defence-clock]
- It hoards its army at home as one block and sends it when it judges itself stronger than what it has seen of ours;
  the block then walks to our nearest weakly defended cluster. On medium the block is about 500 metal at minute 4,
  1,250 at 6, 2,000 at 8, 2,800 at 10, 3,800 at 12 (games vary by a third either way); easy is softer and slower, and
  its column has come at minutes 8 to 16 in your hands' games. After a wave of ours dies in its half, it resurrects the
  wrecks and they fight for it. [K-barb-medium-army-curve, K-barb-attack-gate, K-army-dead-waves-are-resurrected]
- It counter-builds against what it has seen: riot units against raiders, assault units against static defence; its
  second factory is a tier-2 lab by minute 15-20 at the latest. The game should be decided, or well in hand, by then.
  [K-barb-response, K-barb-tier2, K-eco-t1-ceiling]

**Economy.** [K-eco-production-is-the-bottleneck, K-eco-expansion-before-conversion, K-open-sim-energy-ratio]
- Metal in the bank is army we do not have: when metal is banking up, a second lab or a construction turret, not
  converters. Extractors beat converters while free spots remain. Energy only needs to keep storage from emptying;
  laser turrets stop firing in an energy stall, and the picture says STALLING when it happens.
  [K-units-laser-towers-need-energy]
- Constructors die in the field walking alone to far spots. Expand in steps the army has already covered.
  [K-army-verdicts-v18]

**Which map.** Two maps have sections here, Quicksilver Remake and Comet Catcher Remake. The `map` tool names the
one you are on; read that map's section and its opening, and skip the other's.

**This map, Quicksilver Remake, from the north-west corner start (the games so far: `--corner nw`, mirrored).**
[K-maps-terrain-not-straight-lines, the terrain picture in the `map` tool]
- 38 of the 44 spots can be walked to; six on islets cannot (the map's `walk_from_home` is null for them). The
  nearest spots are on and just below our plateau; the middle rows are the contested ones and change hands all game.
  The opponent starts on the mirror plateau across the open middle, about 3,900 on foot. The map's `passages` list
  names the narrow ways; the picture names the three narrowest `passage_1` to `passage_3`.
- Energy on this map, from the people who play it (human-3): the wind averages 12 and is good; strong players build one
  solar first, then wind for the rest. Building a lab draws about 80 energy a second and a wind generator costs
  energy to build, so a lab before any generator empties the store for a minute and a half and the lab then builds
  at a crawl (human-3: the store at zero from 0:46 to 1:54, the first constructor 43 s in the making, no soldier
  before 3:01 and the lab dead at 3:03). Solar first when the store is empty, since building solar costs no energy.
  About twenty energy for every metal spent is the rule of thumb they gave; "two constructors is very greedy on this
  map" (human-6), so one constructor and the commander helping the lab is the early shape.
- The fight, once the opening is right (human-8: Matt's opening minute for minute, ten Grunts by 2:43, and lost
  anyway): Grunts trade one for one with Pawns and lose in ones and twos away from the commander (eight lost by
  3:14, all 380 to 1,231 from it). The user's reading: the Pawns drifted too far out, and the commander was not used
  to improve the exchange rate. So the raiders hold at home beside the commander until they are a handful, the
  commander attacks a raider party at a building of ours or one our soldiers are fighting within its short walk (the
  `attack` row offers exactly that), and no group of one or two goes anywhere alone. Their raid comes at 2:00; the
  first Grunts are the guard, not the raid.

**The opening to copy on Quicksilver: Matt's, an experienced player's replay on that map (the user, 2026-09-22: "you need a
different opening build order, take a look at the Matt game").** [K-open-early-pawn-pressure-is-standard,
K-open-matt-order-timings] His first three minutes, from the replay's record, to be written as your first packet:
- commander: an extractor on each of the two spots beside the start (his at 0:03 and 0:10), then three wind
  generators (0:16, 0:22, 0:28), then the lab (0:33, finished 0:50; energy was at 490 and rising when it started).
  The moment the lab stood (0:50) it helped the lab; at 0:53 it ordered seven wind generators and a solar as one
  batch, built them back to back by 1:30 (one every six seconds, no walking between), and from 1:36 it helped the
  lab for good, leaving it only for a wreck at 2:02. Ten generators in all. The store never emptied: it dipped to
  15 while the lab made its first constructor and was full again by 1:35.
- lab: one constructor first (0:50), then raiders and nothing else (his first Pawn at 1:12, one every eleven seconds
  while the lab worked alone, then every four to fourteen seconds with the commander helping: the rate is then set
  by metal, not build power, and the bank was spent to 8 by 2:00 and kept under 20). A second constructor only at
  1:45, after six Pawns. The numbers: a Pawn is 1650 build time, the lab builds at 150 (eleven seconds a Pawn), the
  commander at 300 on top of it (under four seconds a Pawn, 54 metal each: the commander helping turns the bank
  into Pawns as fast as it can, so it helps the lab whenever it has no generator to build).
- constructors: extractors on the free spots outward from home, a light turret beside the far ones (his first far
  extractor at 1:45, its turret at 2:32); a converter when energy banks up (2:55).
- army: the raiders gather at home into one group until they are a handful, then go for the enemy's extractors and
  builders, from about 2:30; his commander never left the base.
So the packet's first lines are: two extractors, three wind, the lab, then generators until energy banks up high and
then help the lab, and help the lab whenever nothing is due; the lab on one constructor and then raiders only, a
second constructor after the sixth raider, said with `produce` (`{"all": ["corck:1", "corak"]}` on Cortex,
`["armck:1", "armpw"]` on Armada), since the hands cannot count and made three constructors from the words alone
(human-7). Not four constructors, not a second constructor before the first
raider (human-5: a second constructor at 1:15 put the first Pawn at 1:40, thirty seconds behind Matt), not the
lab before the generators, not solar because the wind looked weak: it is not. Where we still lag him (human-5): our
commander built its generators one pick at a time, eight and a half seconds each against his six, and began
helping the lab at 2:24 against his 1:36; write "help the lab" as the commander's standing state and generators as
the exception while energy is low.

**This map, Comet Catcher Remake (the series from 2026-09-23: `--corner nw` is the west strip, Armada, mirrored,
against this opponent from easy up to hard_aggressive).** [K-maps-factory-by-terrain, K-units-vehicles-vs-bots,
K-maps-comet-barb-opens-bots, the tempo model]
- 8192 by 6144, flat and open, no water, no passages: a vehicles map. Played west against east in full-height strips a
  fifth of the map wide. 80 spots: 14 in our strip (spot_2, 7, 14, 19, 28, 30, 36, 45, 50, 54, 62, 67, 68, 74, north
  to south, at x 540 to 1370), 14 in theirs (spot_5, 10, 12, 17, 25, 29, 34, 42, 48, 51, 60, 65, 72, 77), 52 in the
  open middle. Our start is placed within reach of two spots. The opponent's strip is 5,500 to 6,500 elmos east; the
  game puts the AI at an end of its strip (the north-east (6899, 681) or the south-west (1286, 5421), diagonal from
  ours, not straight across), so scout the strip, not the mirror point. Each extractor gives about 2.6 metal a
  second here.
- Wind is dead: 1 to 4. Solar collectors only (155 metal, a steady 20 energy a second each, no energy to build). A
  factory draws about 80 energy a second while it is being built, and vehicles cost eight to fourteen energy per
  metal (Blitz 900 for 110, Stout 2100 for 225, Mason 1950 for 135), so a plant running steadily wants about eight
  solars behind it; build them beside the commander before and while the plant goes up.
- This opponent opened with a bot lab in 91 of 104 recorded games on this map (a vehicle plant in 13), whatever the
  terrain says: expect Pawns and Ticks early and Maces or Rocketeers later, from the far end of its strip.
- Tier-1 vehicles (metal / health / speed / range), Armada with the Cortex twin in brackets: Rascal `armfav`
  31/105/168/180 scout car (Tumbleweed `corfav` 26/90/153/180); Blitz `armflash` 110/730/101/180 raider tank
  (Instigator `corgator` 120/820/85/230); Stout `armstump` 225/1800/75/350 tank, the line unit (Bulldog `corraid`
  235/2000/72/350); Janus `armjanus` 240/1030/54/380 rocket burst (Leveler `corlevlr` 220/1400/40/315 assault);
  Whistler `armsam` 150/820/55/700 rockets (Slasher `cormist` 155/860/52/700); Shellshocker `armart` 135/620/54/710
  artillery (Wolverine `corwolv` 170/750/48/710); Pincer `armpincer` 200/1340/63/305 amphibious tank (Coyote
  `corgarp`); Mason `armcv` 135/1380/54 constructor vehicle (Rover `corcv` 145/1430/51); Beaver `armbeaver` 150
  amphibious constructor. The plant `armvp` is 590 metal (`corvp` 570) against the lab's 500; `plant_N` is its name
  in the picture and in `produce`. Against bots: a Blitz is two Pawns' metal with twice a Pawn's health and speed
  and kills constructors and extractors the same way; a Stout beats any tier-1 bot head on and takes a light turret
  with a few friends but does not outrange it (350 against 430); Whistlers and Shellshockers do.
- **The opening two experienced players used on this map** (a 1v1 of 2026-09-09, both rank 5, Armada against
  Armada, the west player from (954, 2520), fifty elmos from where the arena places you;
  `run/matches/1790122851-replay-vak-vs-artur-comet`, run/view_match.py opens it) [K-open-comet-pro-order,
  K-open-comet-commander-expands, K-open-comet-rascals-lose-to-the-commander, K-open-comet-flash-raids-take-the-resign].
  The commander: extractors 0:03, 0:10, 0:24 (the three spots beside the start: spot_30, spot_28, spot_36 from the A4 start at (904, 2457), where the arena puts you too), solars 0:31, 0:40, 0:49, the plant
  at 0:58 (standing 1:16), one more solar, then out: from 2:00 it walks toward the middle and builds an extractor
  every twenty to sixty seconds at the middle spots nearest home (spot_43 at 2:33, spot_38 and spot_39 by 3:36,
  spot_26 and spot_24 by 4:46, spot_33 and spot_31 by 6:15, at x 1,900 to 3,700), a light turret beside each pair
  (2:45, 3:42, 4:53, 6:22), a solar or two and a radar at each outpost (4:09, 5:40), and it stood 2,600 elmos east of
  its start at 6:30 at full health; the other commander did the same from its side. The plant: two Rascals, a
  constructor, a second constructor at 1:41, two Rascals, then Flashes from 2:12 one every ten to fifteen seconds
  (thirteen by 5:00, twenty-four by 6:50), a constructor at 3:09, 4:30 and 5:46 (five in all), a Rascal now and then
  (seven). The constructors take the strip's spots behind the commander (spot_45, spot_50, spot_52, spot_61, spot_63,
  spot_64, spot_59, spot_54) with a turret at the far ones, and from 5:00 build nano turrets on the plant (three by
  6:37) and an energy storage: one plant all game, made faster, not a second plant. Metal spent as it came (the store
  under 30 through minutes 2-4), then 300-500 banked from 5:00 with income 23 at 4:00, 32 at 5:00 and 43 at 6:52 (9,
  13, 19 extractors); solars 5 by 2:20, 7 by 4:00, 17 by 6:46, the energy store dipping to 22 while the plant went up
  and to 114 in minute 5, never zero. His Flashes were never one ball: in twos and threes across the whole map from
  4:00, at the other side's extractors, constructors and solars (a start extractor at 4:44, four solars beside the
  plant at 5:46-5:51, the north end's extractors and two constructors 6:08-6:29), losing seven, and the other player
  resigned at 6:45 with fifteen extractors to his nineteen and five Flashes to his fourteen. The loser's mistake, to
  avoid: sixteen Rascals in six minutes sent at the enemy base in twos and threes; the commander, one light turret at
  the plant (2:03) and the first Flashes killed fifteen of them, and he had no army at 4:00. So on this map: the plant
  by 1:00 after three solars, two Rascals for sight and then Flashes without a gap, a light turret at the plant by
  2:00 for the enemy's scout cars, the commander out to the middle spots with a turret and a radar at each outpost
  while the constructors fill the strip, nanos on the plant at 5:00, and Flashes spent on their economy in small
  groups from 4:00. Against this opponent rather than a person: its Ticks and Pawns come at the outer extractors from
  4:00 (comet-3), so the turret beside each pair matters more, and its commander D-guns a small group (comet-1), so the
  raids go at extractors and constructors, never at the commander. Say the plan you chose in a `note` on turn one and,
  at the end, what you would change; the next game's brief carries it.
- comet-1 (easy, WON at 17.5 min): the plan was two extractors, three solars, the plant by 0:40, `armcv:1` then
  Blitzes. The hands built the extractors at 0:03 and 0:11, solars at 0:18 and 0:28, then two more extractors and a
  solar, and the plant only at 1:40 with 890 metal banked: the plant's menu words warned that the store would empty,
  and the hands did not count the solars (both words are fixed: the store's fate over the build is computed, and
  the generator options carry the count standing). Blitzes from 2:43; 233 of them, five plants, 27 solars and 38
  extractors by the end. The fight: fourteen Blitzes went east at 6:20 with no radar and nothing scouted, found the
  commander at 8:58 at the north-east end of its strip and fed nine Blitzes into its D-gun (1,540 metal lost to
  660); the game was won by a gathered wave of 19 and then 32 onto its base at 11:57. The user, watching the replay:
  "radars would have helped there". No radar was built all game: have a constructor build one at the front edge of
  our strip by minute three (`armrad` at a spot or a mark; it sees 2,000), and another mid-map when the tanks go
  out. The base was packed (18 solars and 5 plants within 800 of home) and tanks path badly through buildings:
  groups of 8 and of 48 stalled at home for 35 to 67 s trying to walk out. Put the solars on named spots away from
  the plants (a constructor's job) and the plants apart. Energy sat at 0 at 11:57 with five plants: about eight
  solars behind each plant. This opponent opened with a plant on easy here, against its bot labs in the older games.
- comet-2 (medium, LOST at 8.6 min): the same opening was written in words and the hands did the same again: four
  extractors and three solars, the plant at 1:45, then three plants and five constructors from "one plant, one
  constructor" (every `produce` naming a Blitz or a Mason was refused that game: the tool knew the bot lab's units
  only; fixed). The first Blitz came at 4:52; its Rascals and Blitzes ate thirteen extractors from 4:10 against no
  army, and the commander chased a scout car it cannot catch. So: the opening goes in a `queue` list per builder,
  which the bot does step by step (commander: extractor spot_45, extractor spot_50, armsolar, armsolar, armsolar,
  armvp, armsolar, armsolar, assist), `produce` caps the constructors (`armcv:1`), the instructions say the plant is
  the only factory and the commander never chases; and a few Blitzes stay home as the guard from the first one,
  since medium raids from 4:00 with scout cars and Blitzes. Their plant came first on medium too.
- comet-3 (medium, LOST at 17.0 min): the `queue` list ran to the second (extractors 0:03 and 0:11, three solars,
  the plant at 0:51, three more solars, one constructor, then helping the plant), and the game was lost on metal.
  Six solars are 930 metal, eight Blitzes' worth, spent while income was 6; the store sat at zero from minute two
  to minute thirteen and the plant, which with the commander helping can turn 25 metal a second into Blitzes, made
  28 in seventeen minutes (the first at 3:03). Metal is the bound here, never energy: two solars before the plant,
  then the commander takes the strip spots nearest home (spot_36, spot_54, then spot_28, spot_30) before it helps
  the plant, and a solar only when the energy words say stalling. The one constructor, told "expand along our
  strip", took the middle spots the menu lists first by walking time (spot_43, spot_52, spot_38, on open ground
  toward the enemy) and each died within 27 to 70 s to Ticks and Pawns; the strip's own spots were untouched until
  5:38. So the moment `constructor_N` appears in the picture, give it a `queue` list of the strip's spots by name
  with a turret after each pair (extractor spot_36, extractor spot_54, armllt spot_54, extractor spot_28, ...), and
  allow the second constructor by minute three: the strip has fourteen spots and one Mason never reaches them. The
  raids: 24 extractors lost, to Ticks (21 metal, faster than a Blitz), Pawns and Blitzes; Blitzes cannot catch
  Ticks, a light turret at the spot can. The fight: thirteen Blitzes died to one Rocketeer and one Centurion at
  11:57; this opponent opened with a bot lab and had Centurions by minute twelve; switch the plant to Stouts the
  first time a line bot is seen, and keep Blitzes for what cannot shoot back.
- comet-4 (medium, LOST at 16.9 min, the commander killed at home): the opening the bot played through comet-5 and opus55-1/3 (superseded by the players' order above: three solars before the plant, not two, and the commander out; its spot names are from an older start, escalate-1's note: "nearest spots here are 28 and 30, not 45/50").
  Commander list: extractor spot_45, extractor spot_50, armsolar, armsolar, armvp, extractor spot_36, extractor
  spot_43, armsolar, assist; `produce {"all": ["armcv:2", "armflash"]}`; each constructor given a list of the strip's
  spots with a turret after each pair as it appeared. That gave the plant at 0:41, two constructors by 1:30, the
  first Blitz at 2:08, thirteen by 4:07, six extractors at 2:30, thirteen at minute 8 and nineteen at minute 10 with
  income 41: ahead of this opponent all game on the board. It was lost on three things, all after the opening.
  (1) Metal banked from minute 8 (800, then 2,100 for six minutes) with one plant: at income 30 the second plant is
  due, at 40 the third; a plant is 590 metal and pays for itself in twenty seconds of production. Spend the bank on
  plants and constructors the turn it appears. (2) 22 Blitzes were sent onto their base at 10:18 and died to four
  light turrets and a beamer for 2,245 of theirs: Blitzes never go at turrets; Stouts take a light turret with a few
  friends, and Shellshockers (710 range) outrange every tier-1 turret. (3) The whole army went east at 14:12 to 16:12
  while their block of Maces, Centurions and Rocketeers walked into our base and killed the commander at 16:49; the
  recall came at 25% health. This opponent's block arrives in our half around minute 10 and comes back every few
  minutes: a Stout group stays at home on the turrets whenever the ball is east, and the commander goes to the far
  end of the strip at the first sight of a line bot near home, not at 25%. Their commander stood at the north end
  (7383, 609) this game, the base at H1-H2. Blitz raids took 17 extractors (8 to Blitzes): turrets at the outer
  spots held only where they stood two together.
- comet-5 (medium, LOST at 27.7 min): the same opening (plant 0:40, first Blitz 2:05, twelve extractors and income
  27 at 5:16), and this time the opponent opened with a vehicle plant: its Blitzes took 41 of our extractors over the
  game (25 to Blitzes) while ours sat in five small guard groups, then a block of nine Janus (380 range) killed
  eleven Stouts (350) at 17:07, and 24 Shellshockers without a screen were eaten at 23:06. Against its plant: one
  block, never five guards; Stouts trade even with Blitzes, so the turrets do the guarding (31 were built, from 5:07,
  too late for the first raids: the first two go up with the first outer extractors); the Janus is the counter to
  Stouts, and Shellshockers behind a Stout screen are the counter to Janus, never alone. Energy: the plant's units
  cost eight to fourteen energy per metal and Stouts at two plants stalled the store at 17:07; then 46 solars were
  built with the metal at zero and energy full ("solars were pure waste", 24:06): count the plants and hold solars at
  about eight per running plant. `queue` takes an empty list or null to cancel a list (the strings sent at 24:11 are
  accepted now too).
- escalate-1 (easy, Opus 5.5 at medium effort, WON at 14.6 min, their commander killed at H2 at 14:36): the first
  game on the players' order. Commander list mex 28, mex 30, two solars, plant (standing 1:00), mex 36, mex 19, then
  out at 3:05 to spot_38, spot_26, spot_39 with turrets and a radar; `produce armcv:2, armflash`. Energy hit zero at
  2:00 with two solars while the plant made constructors: three before the plant, as the players do, and two more by
  2:00. Twenty-five extractors, none lost; income 30 at 5:00 and 45 at 10:00; metal banked 1,030 at 5:00 with one
  plant, then two more plants at 5:10 and 5:15 and 0-60 banked from 7:00 (the players nano one plant instead; either
  spends it, banking does not). The hands put the two "home" plants where their builders stood (the strip's north end
  and the commander's outpost): a factory step may name its place now (`armvp spot_39`), and a cancelled list does not
  stop a building already started. Nothing was scouted until a solar came into sight at 8:00: the players have a
  Rascal out by 1:20 and a radar at the front by 4:00. Sixty Blitzes lost, about twenty of them fed piecemeal into the
  commander's D-gun, light turrets and a beamer at H2 from 9:20 to 11:00; the win came when 57 soldiers gathered at
  spot_35 and went in together at 13:46. Its own lessons: gather first and commit everything in one push; turn the
  footwork off for a retreat or a run through turrets (sidestepping held a group under a beamer for 49 hits at
  11:05-11:40); give a raid one far destination, not a chain of middle spots to go "through".
- escalate-2 (medium, Opus 5.5 at medium effort, WON at 13.5 min, their plant killed at H2 at 13:21 and the
  commander at G1 after): the same adapted comet-4 list, and energy hit zero at 2:00 on two solars a second time
  (three before the plant, then three more by 2:00: the plant's Blitzes cost 900 energy each). The home guard was
  stationed at spot_36, beside the plant, and five of twelve Blitzes stood jammed in the plant's exit lane from 3:30
  (the picture's `stuck` line said so; the station moved to spot_38 at 4:12 and footwork went off to let them out):
  never station a group on the spot beside a factory; spot_38 or farther. The commander stayed home helping the
  plant all game and the constructors took the middle; nothing was seen until 5:25. This opponent went vehicles
  (Blitzes, then Janus at 9:00) and raided the middle spots in parties of five to seven from 6:00 (spot_39, spot_52,
  spot_62); four-Blitz detachments arrived one at a time and lost eight extractors and six turrets, the whole home
  group did not. Three plants by 8:26 kept the bank at 0-450 with income 36-46. The ball of 41 committed at 9:55 with
  footwork on and chased single Blitzes to spot_59 instead of going east; at the H2 plant it lost 26 Blitzes, twelve
  to the commander's D-gun. Its own lessons: footwork off and a spot route for a commit; Stouts in front before
  closing on a plant with the commander at it; each raid group its own single target.
- escalate-3 (HARD, Opus 5.5 at medium effort, WON at 17.3 min, their commander killed at G1 (6291, 747) by 25
  Stouts): the players' order taken on turn one and on schedule (two extractors, three solars, the plant at 0:49
  standing 1:08, `armfav:1, armcv:2, armflash`, the commander out at 2:00 to spot_43, 38, 39 with turrets and a radar,
  standing at (2891, 2951) by 3:00 and never hurt). Energy still hit zero at 4:00 on four solars with metal banking:
  the players had seven by 4:00; budget a solar per Blitz queued and put them on a constructor before the drop, and
  again when a plant is added (a second stall at 9:51 with four plants making Stouts at 2,100 energy each). Hard
  opened a bot lab and fought differently: a fortress at E3/F3 (a Gauntlet at 1,220 range, a heavy laser tower at
  620, a beamer, walls, an artillery turret at F2 by the end) with Maces, Rocketeers and resurrection bots in front,
  and Pawn raids that took extractors from 20 to 16 from 13:00 and killed a plant. The trade was even all game
  (11,591 destroyed against 11,442 lost at 16:50); the win was volume (18 extractors and 3,500 of army against 225 seen
  at 8:00) and the route: after two pushes into the fortress, the ball went along the north edge (spot_3, 6, 10, 5,
  12, 17, all over 1,300 from F3), found the advanced bot lab at G2 (6248, 856) and the commander beside it. Their base
  was north (G1-G2) every time it was looked for; four Rovers found nothing in the south-east. Its own lessons: go
  round by the north edge from the first commit; keep a Blitz group at home for Pawns (Stouts cannot catch them) with
  turrets at spot_36, 52, 61, 22; cap the factories in words (the hands built four plants: the commander cannot build
  a nano turret, it is not on its roster, and it made a plant instead); the hands keep a group's old walk after the
  packet changes unless the packet names that walk wrong and abandoned ("group_B's walk to spot_13 is abandoned;
  advance east to spot_6" took effect in one turn); "advance to X first" with X behind the group sends it home; give
  artillery its own named group or it splits off and dies alone; footwork off for a group you are committing.
- escalate-4 (HARD_AGGRESSIVE, Opus 5.5 at medium effort, LOST at 28.4 min, the commander killed at home by a
  Razorback): the opening held (the players' order from the B5 start: 10 extractors and income 20 at 4:00, nothing
  lost, their commander seen at H3 by the Rover at 2:56), and the game was lost between 6:00 and 10:00: extractors
  stalled at 14-17 from 7:00 to the end (the players have 19 at 6:52 and climb), the bank went to 1,273 at 8:00 with
  three constructors, and four plants went up on income 29 (energy 0 at 9:17, metal 0 at 10:44, a nano took 100 s).
  This opponent went tier 2 while we stayed tier 1: Centurions and a Gunslinger at 12:00 ground down a 17-Blitz ball
  (4.5k lost for 1.6k), a Fatboy and Sharpshooters at 19:00 broke the spot_38/39 line as our push left, a Pulsar
  (1,400 range) at D2/E2 and unseen snipers killed two pushes into their half at 24:00, and a Razorback (tier 3,
  3.8k metal) with Hounds, Welders and a Fatboy killed fifty Stouts in the middle at 26:15 (13k lost in three
  minutes). Ticks took every extractor without a turret from 5:00 (33 extractors lost); their base reached west to a
  moho at D1. `remove` worked in play: a plant whose exit another plant blocked was reclaimed by the commander
  (asked 10:44, gone 11:22; the first ask went to a constructor's list and waited for its step). Its own plan for the
  rematch: the same opening with a turret on every extractor and the guards told in words to attack on sight; no
  more than two plants plus nanos, the bank into three or four constructors taking the north strip (19, 22, 14, 7),
  the south-middle (47, 63, 64, 69) and the middle (24, 33), 22 extractors by 10:00; a heavy laser tower line at
  38/39/26 and the Advanced Vehicle Plant (armavp) started by 12:00, Stouts and Janus holding meanwhile; convert an
  army lead within two minutes of having it (4:1 at 17:00 was the window), with tier 2 in front, a scout or radar
  ahead of the ball, and the commander's list cancelled before any fight it is needed in (a builder on a list is off
  your hands' menu, and the commander never engaged the Razorback because "help the plant" was still running).

**What your hands did alone (pianist-smoke-1 to -6, pianist-easy-1, pianist-audit-1: four wins, four losses and a
timeout against easy from the default packet, `docs/experiments.md`).** [docs/knowledge/jev.md]
- The economy from the default packet is strong: 8 extractors by 4 minutes, 14 by 6, income 30 by 7, several labs and
  a hundred soldiers by 15. What lost games was the army: it holds where it stands unless told where to stand, and it
  engages only when the menu's odds words say it outweighs the party; it never went for the enemy base from the default
  packet (the base reads "not found" until a scout has stood there, and one scout at a time goes there and mostly
  dies on the way). A timeout at 40 minutes with 206 soldiers at home is the shape of a packet with no attack plan.
- The commander died twice walking to far spots for extractors beside the enemy; it retreats when the picture says a
  party outweighs it alone, and the packet should keep it near home after the opening.
- Words the hands act on: counts as words beside the option ("far too many constructors"), stalls as STALLING, the
  odds as "we outweigh it"; numbers alone change nothing. One kind of action per option; the place in a parallel
  question, which is why instructions name places. [K-jev-words-not-numbers, K-jev-split-vote,
  K-jev-where-needs-its-premise]

**People (human-1, the first game against a person, 2026-09-22).** A human on this map rushed with Pawns at 2:10,
straight at the field constructors, and never stopped; against zero army the game was over by 3:00 (all builders
dead by 2:55, the lab by 4:46). Soldiers before the fourth constructor, and no constructor beyond spot_12 without a
soldier near it before 3:00. The people in chat gave real advice ("30 pawns before mace"; "moving the commander
around costs build power"; "about 20 energy for every metal, and more than one or two constructors makes defending
hard") and answered a question in a minute: ask early, take what fits, say what you are doing. Both games the lab
made three constructors while three windmills ran and energy sat at zero from 1:00; the wind here gives energy, less
than that spends. The commander can `attack` a raider party beside it that it outweighs: two Pawns at home are its
job, not a reason to walk away. [experiments ledger: human-1, human-2]

**What earlier players did, right and wrong.** [experiments ledger: pianist-player-1]
- pianist-player-1 (2026-09-21, Opus at low effort, lost at 14 minutes): the opening packet was sound and the hands
  played it (five extractors by 2:00). At 2:13 it wrote "commander: go home now and build a lab at home"; the hands
  read the packet afresh every ten seconds and alternated going home with starting the lab, abandoning the frame each
  time, and the report of the day called the decayed frames losses to something unseen. The player read that as an
  air raider, spent the next four minutes on Crossbows (anti-air only, six of them) and generators that were abandoned
  in turn, and met the first real raid (ten Flashes at 7:48) with three Maces. Both harness faults are fixed
  (H-HANDS-STARTED; abandoned frames are reported as abandoned). Its lesson: write the packet as states that hold
  ("the commander stays at home and builds the lab there"), never as a sequence of commands to be done once.
- pianist-player-2 (2026-09-21, Opus at low effort, lost at 27 minutes): the best economy and army of the series (14
  extractors by 6:00, a hundred soldiers by 18:00) and a loss all the same. Three waves went at enemy_base one after
  another and were ground down piecemeal; the whole ball of 82 stood short of the base for five minutes under
  artillery while the hands answered "continue": the hands of that day would not walk a group into a turret's reach
  (fixed: `fight_to` now fights its way through turrets, so it is the attack, and the odds words on the option say
  what it faces). Meanwhile the raids took the middle spots behind the attack (16 extractors to 8) and every field
  constructor died: an attack needs a home guard on the passage the raids use, and constructors told to rebuild
  behind it. The ball also chased single Flashes when told to engage parties it outweighs: name a small raider-hunting
  group for that and keep the ball's instruction to holding and advancing.
- pianist-player-3 (2026-09-21, Opus at low effort, WON at 11 minutes, the series' first win): the same opening,
  and the packet rewritten four times against what the hands did (the commander looping generators into an energy
  stall, the commander walking 2,700 to a far spot, the ball chasing a lone Flash 2,000 away, lone Maces sent as
  scouts and lost); at 8:51, with the army at 3,200 against 870 seen and 17 extractors, "committing the whole block to
  attack via spot_36 toward the presumed enemy base", and the ball of 25 Maces walked through fourteen artillery
  pieces and seven towers to the commander. The pattern: expand hard, one ball, read the army lead off the score line,
  and one `fight_to` with everything.
- pianist-player-4 (2026-09-21, Opus at low effort, WON at 13 minutes): the same shape as game 3 (10 extractors by 4:13,
  15 by 5:43, the ball committed at 8:27 with the army at 3,250 against 370 seen), and the first game in which the report
  named the hands' own failures: the player read "held back by their own footwork" at 11:33 and kept the ball moving
  through named spots inside their half instead of holding at enemy_base, where a holding group steps out of turret reach.
  Its other lessons: a scouting instruction on the ball peels one soldier per ask until it names a small group for the
  job; energy needs a named builder of its own or it stalls twice; the enemy commander leaves its base to raid our spots
  once its factories are dead, and the ball must come home to hunt it, gathering first so stragglers are not eaten.
- pianist-player-5 (2026-09-21, Opus at low effort, the first game against MEDIUM, lost at 20 minutes): the same
  economy as the wins (15 extractors by 6:38, a 4:1 army lead at 8:03) and the attack went nowhere: the enemy base
  guess had been pulled to the middle of the map by a forward extractor of theirs, the ball stood on the empty guess,
  and the southern spots the player then named were never on the menu (both fixed: a guess found empty goes back to
  the start, and every spot named in the packet is a place). Medium's raids are twelve Flashes by minute 10, and they
  took the extractors from 15 to 5 while the ball hunted; its block came at 16 with Janus rocket trucks and Stumpy
  tanks behind two artillery pieces. Its lessons: on medium, turrets and a home guard on the raids' passage come before
  the hunt; labs and the commander never forward of the army; a ball never stands under unseen artillery: advance
  onto it or leave; and a place our units have stood on and seen nothing has no base: the opponent is where the
  evidence says, never where a point on the map says.
- pianist-player-6 (2026-09-21, Opus at low effort, WON at 29 minutes, the first win against MEDIUM): a win with the
  economy destroyed. Ten extractors at 6:00, then twelve Flashes held us at 0 to 3 for fifteen minutes; turrets on
  every spot and a ball of Maces parked at the passage both failed, because Maces cannot catch Flashes and the raids
  went round the ball. The ball gutted the enemy base from 11:01, but the enemy commander had left it to raid our half,
  and the game was won at 29:42 when it died at C3 in our half with its army still alive. Named far spots and marks
  (`mark`) were followed by the hands. Its lessons: against medium's raids, raiders of our own (Pawns) on the raids'
  passages, not turrets alone and not Maces; the enemy commander leaves its base once pressed and dies to whatever
  stands at home; a ball under unseen artillery advances onto it or leaves, never stands.
- Since pianist-player-6 the player has `produce` (what each lab may build: the sure way to get raiders built
  against raiders), `mark` (a place of its own) and `lane` (the footwork rules per group).
- pianist-player-7 (2026-09-22, Opus at low effort, WON at 33 minutes against MEDIUM, called by the referee): the
  first game with `produce`, used from the first turn, and the labs built what was listed (89 Pawns among 300 units).
  The raids still took the extractors from 14 to 3 while the ball was away at 13:03; what won was clearing our half
  with the whole army, rebuilding to 24 extractors behind two home groups and turrets, and only then hunting. Their
  commander rebuilt in the far south-east corner on the shore beside a shipyard, where bots cannot walk: a mark on
  water stops the group at the nearest ground, and an advance that has not got nearer for 90 s is now given up by
  the hands and reported. Its lessons: the ball does not leave until the home guard is standing; a lab of Pawns
  from the start against medium; when the report says a group gave up its advance, the place is unreachable: name
  the reachable spots around it instead.
- pianist-player-8 (2026-09-22, Opus at low effort, WON at 22 minutes against MEDIUM, their commander killed in
  our half): the third medium win in a row, and the third time the extractors fell to one while the ball was in
  their half. The pattern that wins: expand to 13 by minute 5, commit at a 5:1 army lead, kill their factories, and
  hold the whole army at home when their commander comes raiding. The pattern that still costs: nothing stops
  twelve Flashes behind the ball except the ball.
- pianist-player-9 (2026-09-22, Opus at low effort, WON at 11 minutes against HARD, their commander killed): hard's
  commander comes into our half early (5:45) and eats a small ball piecemeal; massing first and committing at a 20:1
  seen lead at 7:59 won it in four minutes. Hard's raids are the same Flashes, plus Shellshock artillery of range
  710 that shells from out of sight: the picture's `shelling` place is where it stands, and advancing onto it is
  how the ball answered it.
- pianist-player-11 (2026-09-22, Opus at low effort, WON at 10 minutes against HARD, their commander killed): the
  same shape as game 9 and faster: mass to a real ball while their commander wanders into our half, commit south at
  a clear lead, mark their commander when it is seen, send the whole ball onto the mark.
- pianist-player-12 (2026-09-22, Opus at low effort, WON at 11 minutes against HARD): the same shape again. The
  `shelling` place moves with each hit and vanishes when the hits stop; send the ball onto it once, not as a
  standing destination, and name the next spot beyond it.
- pianist-player-13 (2026-09-22, Opus at low effort, WON at 16 minutes against HARD_AGGRESSIVE, their commander
  killed): the top tier falls to the same shape, and its commander hides: never seen for 15 minutes, rebuilt in the
  far south-east corner with a hover plant. Sweep the far corners spot by spot with marks once their first base is
  empty, and keep the home guard on the raids, not chasing. (An earlier lesson here read "wind is unreliable": it is
  not; the store empties when the lab or the lab's units draw more than the generators standing make, see the energy
  line of the map paragraph.)
