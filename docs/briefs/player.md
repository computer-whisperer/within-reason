

# What we know (the player's brief, written 2026-09-21 evening)

This is the project's accumulated knowledge, from some hundred recorded games against this opponent's medium setting,
unit duels, the opponent's own configuration, and seven games of your hands playing alone from a default packet against
its easy setting. Each line names its source entry in `docs/knowledge/`. It is evidence, not orders: say in a `note`
when the game in front of you contradicts it.

**The decisions, as cases from every map (read these whatever map you are on; the user, 2026-09-28: we are giving a
player that has never played a game of BAR years of experience, so the lessons are situations, not rules, and a
situation from another map still teaches).** Each case: the situation as your report would show it, what was done,
what came of it, and why, with who said so (a person's word outranks our own reading of a record). Where two cases
have the same shape and opposite answers, the difference between them is the thing to learn.

- **A lead is spent within two minutes or it is gone.** The pros end a Comet Catcher duel 3.4 minutes (median) after
  first holding twice the other's army. Cases: (1) player-9, Comet duel vs BARb hard_aggressive, lost 36:32: at 5:00
  the report read army 801 against 270 seen, 14 extractors to 10, his lab unseen; the order was a raid loop through
  six of our own spots and `armcv:2`; by 8:00 he had the bigger army and the base was never seen in 36 minutes
  (the user: "look even earlier -- 6:00": the loss was there, not at the 12:00 lead the review opened on). (2)
  player-10, the same matchup, lost 29:06: rovers had his lab in sight at 3:45 with our army at 4.1x, and the next
  packet made the Blitzes a home guard that "never goes east of spot_59"; the lead was spent at 8:27 by a push after his
  commander into a turret line at F3 (18 of 20 Blitzes dead by 10:46). (3) escalate-7, the same matchup, WON 16:00:
  one ball at spot_39/24 with pickets, his block broken at D4 at 11:33, then the far-north route past the E2/E3
  nest to G1 and everything onto the commander the moment it was seen. (4) pianist-player-3, Quicksilver vs medium,
  WON 11:00: at 8:51 with the army at 3,200 against 870 seen and 17 extractors, "committing the whole block via
  spot_36", and 25 Maces walked through fourteen artillery pieces and seven towers to the commander. (5) pianist-player-2,
  Quicksilver, lost 27:00 with a hundred soldiers by 18:00: three waves went at his base one after another and the
  ball of 82 stood five minutes under artillery. (6) The 3v1s of 2026-09-27 (thebluegecko, spectating): "up by
  almost 400% army value, time to go fight and win; if you dont use that army advantage this will soon go bad". The
  difference between the wins and the losses is not the size of the lead (player-9's was 2.4:1 at 6:00, player-10's
  4.1:1 at 4:00): it is whether the ball was in his half, as one body, on a route a scout had proven, within two
  minutes of the report saying it. **And the push goes where the scouts found his base, not where they found his
  extractors**: player-18 launched five times from 10:56 with 2 to 3x his army, three of them at "his southern
  strip" (spot_79, 75, 70, 77, 72, 65, 60, 57: the spots the Rovers passed on their way in, the empty end of his
  strip), while his base at G1-G2 had been in the picture since 5:00; the raid "found little (killed an LLT and an
  extractor)" and each launch was turned back within two minutes by his party in our middle (11:34, 12:50, 18:27),
  the body strung out (13:29: 12 of 37 in the fight, the tail 3,700 back). The route ends at his base's outer
  extractors and turrets, from the side the scout proved; a party in our middle is answered by the home groups and
  the turrets, or by the whole body only when it is his army (his spending line says what his army is worth: a
  1,200-metal party is a fifth of it at 11:00). **And a lead is never spent as more of the same unit past a dozen.** The
  experienced players' largest body of one type in a typical game is a dozen raiders or line tanks and eight Stouts
  (seven of a long-range type, four Hammers or Levelers; `docs/studies/2026-09-28-body-sizes.md`): their bodies over
  thirty are late and always two or three types together. Ours was 24 Stouts in one body at 17:13 of player-15, and
  the bench says what that buys: at equal metal six Stouts beat one Fatboy (+0.12) and twenty-five lose to four
  (-0.54), eight Stouts lose to two Bulls by 0.18 and twenty-one to five by 0.31; a Stout ball loses nothing to a
  Stout ball at any size. So when a body of a type reaches its dozen (Stouts eight), the next metal is a second group
  somewhere else (the other flank, his expansions, a raid picket), and after 12:00 it is tier 2, never the thirteenth
  of the same (the user, 2026-09-28). Two bodies at two places also make him choose; one big one lets him bring his
  splash to it.

- **His tier 2 is our window; ours is a scenario call.** Cases: (1) player-9: his advanced lab started at 10:32 and
  his army fell from 2,736 to 2,220 by 12:00 while ours rose to 6,565; our mass stood at D5-D6 in our half and his
  factory was never in the picture; the lab was never seen and its Fatboys, Sharpshooters and a Razorback ended the
  game. (2) player-10 and player-11: his lab at 11:42 and 12:10, and the answer both times was to wait for our own
  (standing 14:15, energy 0 at 11:37 for it). (3) bluegecko-3v1-comet-catcher-9, lost 20:16 after leading three to
  one: we stayed tier 1 and 94 tier-1 tanks died to 15 Bulls. (4) bluegecko-3v1-comet-catcher-10, lost 26:10: our
  Lasher mass let him tech ("you made many lashers, so I was safe to go T2"). (5) Great Divide, the plan: tier 2
  and mohos on both seats by 10:00, because the pass camp is broken by Pillagers at 1,300, not by the ball. (6)
  escalate-4 and -5 vs BARb hard_aggressive: it techs on a clock (Hounds 14:00, Fatboy 19:00, Razorback 25:48)
  whatever we do, and both games waited. (7) With several seats the tier 2 is one seat's, paid by all (the user,
  2026-09-27, again 2026-09-28: "Two seats can gift resources to one seat, that seat can build t2, then share t2
  constructors to let advanced mexes go up"): the other seats send metal with `transfer` the moment the advanced
  plant is placed, that seat's own plants pause, its first tier-2 unit is an advanced constructor for each seat,
  given away with `transfer`, and the mohos go up on every seat at once. Game 10: the top seat's advanced plant sat
  at 0% from 15:40 until it was abandoned at 19:15 while that seat's store rose to 1,432 and no `transfer` was made
  all game (none in game 8 either); game 12 made three, a dying seat's units and 700 metal to the live one at
  11:41-12:13. The pattern applies on land and water alike, as the water paragraph's one hover platform does. And the metal is
  nothing without build power on the plant (the user, 2026-09-28: "The top player had only one constructor applying
  build power to it, and a second constructor eventually came along and tried building a second t2 vehicle lab before
  eventually assisting the first one. Two players gifting to one doesn't help if that player doesn't have the build
  power to push it through"): in game 10 one constructor built the frame from 15:40 and a second was given a list
  naming the plant at 18:07 and started a second frame 224 away (abandoned 19:15; the hands now help a frame of the
  asked type already standing instead). When the plant is placed, every constructor of that seat and a construction
  turret or two beside it go on `assist` of the plant, and the seat's other plants pause; the received metal is then
  a plant standing in two minutes, not a store. The calculus (the user, 2026-09-28): in a close duel the tech is a fatal
  weak point, so his tech is the moment to find and kill his base, and matching him is the mistake (the pros tech in
  4 of 58 duel sides); past about 25:00, with more than two players, or against an opponent that techs on a clock,
  the window is covered and tier 2 on our side is right, with that seat's other plants paused ("Top should stop
  building other units while doing t2 transition", thebluegecko, game 10). Say which scenario you are in before you
  choose.

- **Income past the spot count is tier-2 extractors and advanced solars: build a lot more of both** (the user,
  2026-10-02, from human-10, player-40 and player-42: "we need to be building a lot more t2 mexes and advanced
  solar"). A side that holds its half of the spots has reached its tier-1 income: player-42 held 27 to 35
  extractors from 15:00 and never passed 128 a second, human-10's three seats had 35 extractors and 70 a second at
  12:00 while the opponent said "not a unit choice problem, a not enough mex problem". The tier-2 extractor
  (`armmoho`/`cormoho`, the role word `advanced_extractor`: `advanced_extractor spot_N` over our extractor there)
  gives four times the metal of the spot it replaces for 620 metal and 20 energy a second, and the advanced solar
  (`armadvsol`/`coradvsol`, 80 energy a second for 350 metal) is the energy for them and for the tier-2 plant: one
  per four mohos, and more for the plant. So from the moment an advanced constructor stands, every held spot is a
  moho in turn, the nearest home first and the ones under turrets as they are, with an advanced solar beside home
  for every four of them, and the `produce` list of the advanced plant keeps an advanced constructor on it until
  every seat has one (several seats: one plant, its constructors given away, as the bullet above says). Two things
  the games showed: an advanced solar costs 5,000 energy to build, so it goes up with the store above a third and
  never in an empty store (the note below, a game whose advanced solars crawled at 18:22 in an empty store: plain
  solars then); and a seat that reaches tier 2 and builds no moho has spent the plant for nothing. Say in the packet how many mohos stand and how many are next, the way you count
  extractors.

- **The unit mix is a message to him, and his to you.** Cases: (1) game 10: many Lashers said "no pressure on your
  spots", he teched, and Tzars and Tigers beat the Lashers; "brutes would have given you the ability to apply more
  pressures" (thebluegecko). (2) games 3-5: from about 7:30 a person switches to Pounders against a light-tank
  ball; the answer is medium tanks (Stouts, Brutes) with three to five Lashers or Janus in all, never more ("too
  many lashers lose to med tanks"; "3 med tanks per pounder"). (3) comet-3 vs BARb medium: thirteen Blitzes died to
  one Rocketeer and one Centurion at 11:57; the plant switches to Stouts the first time a line bot is seen. (4)
  comet-5: nine Janus (380) killed eleven Stouts (350) at 17:07, and 24 Shellshockers without a screen were eaten at
  23:06: Janus counters Stouts, artillery behind a Stout screen counters Janus, never alone. (5) bluegecko-3v1-2:
  Janus and Stouts ate packed Blitzes (eleven lost at 12:09): from the first Janus seen every plant makes Stouts and
  Janus, Blitzes only as pickets. (6) human-8, Quicksilver: Grunts traded one for one with Pawns and lost in ones
  and twos away from the commander; the commander was not used to improve the exchange. (7) game 11, SailAway 2
  (water; three seats, resigned 24:33): the hover army was thirteen Mangonels (artillery hovers, 700) to four line
  hovers on one seat, and his Riptide frigates and Buccaneers beat it; thebluegecko after: "mongonals are better
  than t1 arty because they outrange things. They can't get in a straight fight, so you want 1-3 of them with other
  units in front. Same with t1 artillery -- it shouldn't be the primary unit body" (comet-5 said the same of 24
  Shellshockers eaten without a screen). Artillery is one to three behind a body, never the body. Read his army every turn
  for what our mix has made safe for him, and change before he does; a mix that threatens his spots keeps him
  honest, a mix that only holds does not.

- **Raids are answered before they come, by the unit that catches them.** Cases: (1) escalate-4: 33 extractors to
  single Ticks at spots no soldier stood near, with a home guard that walked out after the extractor was gone (the
  user: that is not defence). (2) bluegecko-3v1-1: the north seat lost 17 extractors from 2:06 and answered none,
  its Blitzes in one ball far from the spots. (3) bluegecko-3v1-4: Incisor swarms of ten to fifteen slipped past the
  Brute ball and took our extractors from 33 to 9 while every fight we had we won; a ball of medium tanks never
  catches Incisors. (4) pianist-player-6, Quicksilver vs medium: twelve Flashes held us at 0-3 extractors for fifteen
  minutes; turrets on every spot and a ball of Maces at the passage both failed, since Maces cannot catch Flashes;
  Pawns on the raids' passages did. (5) comet-5: 31 turrets from 5:07, too late for the first raids. (6) escalate-7,
  WON: pickets north and south, turrets beside the outer pairs, nine extractors lost all game against 32-45 in the
  three losses before it. The answer stands across the front before the raid: a Rascal (168) catches a Tick (132),
  a Blitz (101) never does; a light turret beside each outer pair; one soldier from the nearest group against a lone
  raider, never the ball.

- **A bank is build power missing.** Cases: (1) escalate-5: 1,600 to 1,850 metal from 6:00 to 10:00 with one plant
  and three constructors, lost 28:30. (2) escalate-6: 1,357 at 6:00, 1,898 at 7:00, named in a note as "the key" at
  7:46 and not spent until the raids had taken the income; lost 23:00. (3) comet-4: 2,100 for six minutes with one
  plant; lost 16:49. (4) escalate-7, WON: the plant made constructors 8 asks of 13, six by 6:20, and the bank drained
  from 9:00. (5) The pros: five constructors by 5:00, three nanos on the one plant by 6:37, never past 700 banked;
  "you dont have enough build power" and "get that reclaim asap" (thebluegecko, Irishstud14). The bank goes into
  constructors and nanos the turn it appears; a second plant only after that.

- **The opening is metal, the lab, and the commander on it.** Cases: (1) human-3, Quicksilver: the lab before any
  generator emptied the store from 0:46 to 1:54, the first constructor took 43 s, no soldier before 3:01; on that map
  one solar then wind, and the lab at 0:33 (Matt's replay), the commander helping it from 0:50. (2) comet-3, Comet:
  six solars before anything else were eight Blitzes' worth spent while income was 6; the store sat at zero from
  minute two to thirteen and the plant made 28 Blitzes in seventeen minutes. (3) The Comet pros: three solars (two if
  the plant is up by 0:45), the plant at 0:43-0:58, and the commander at the plant for its first units, out only
  for one solar or one extractor at a time (thebluegecko, game 10: the commander should be assisting the lab when
  it first comes up); his own plant makes a Blitz, five Rovers and only then the constructor (the recommended line
  in the Comet section). (4) human-5, Quicksilver: a second constructor at 1:15 put the first Pawn at 1:40,
  thirty seconds behind Matt. (5) Until 2026-09-28 the list's `assist 25` after the plant ended within a second (a
  harness fault, fixed): write `assist 40` after the plant and check at 2:00 that the commander is still there and
  the store reads about 150. (6) player-12, Comet vs BARb hard_aggressive, lost 26:28 with the root at 1:45: the
  first game in which the assist held, and `produce` written as `armcv:2, armfav:3` put two constructors out back to
  back under it. A constructor is 1,950 energy over a 4,050 build time: the plant alone draws 48 a second, the plant
  with the commander on it about 220, and three solars with the commander make 90. The start's 1,240 of energy is one
  assisted constructor's worth and no more; the second emptied the store at 1:30 (drain 223 against 90) and it stayed
  at 0 to 1:46, when that constructor rolled out. The answer given at 1:45 was four solars (620 metal at an income of
  6 to 8), so the metal store sat at 0 from 2:04 to 3:10 while the energy store was full again from 2:15; three
  extractors stood at 3:00 (six the game before, on the same seed), two at 4:00 after single raiders ate four at home
  with no turret affordable, nine at 8:00 against his fifteen, and his army led ours from 8:00 to the end. The rule
  that follows: Rovers under the assist (370 energy each: the assisted plant makes one every 3 s), the constructor
  after five of them and the second when the stream runs, as thebluegecko's line has it; an energy store at 0
  behind an assisted plant that is making units is the plant working and is never a reason for a solar; when the metal store reads 0 in the first three minutes the
  commander's own build is the thing to drop, and a solar with the energy line not STALLING is three extractors
  spent on nothing. The energy source is the map's (wind on Quicksilver, solar on Comet, solar with wind on
  Cape Violet); the bound in the first four minutes is metal, never build power.

- **Static defence is out-ranged or bypassed, never walked into.** Cases: (1) comet-4: 22 Blitzes onto four light
  turrets and a beamer for 2,245 of theirs. (2) player-8 and player-9: the E3 nest (a Gauntlet at 1,220) took 9k and
  11k of tier-1 tanks over two games; the Pillager (1,300, from the advanced plant, standing from 15:17 in
  player-9) was never ordered. (3) bluegecko-3v1-9: Bulls (460) and beamer nests (490) outrange every tier-1 tank;
  94 tanks died to 15 Bulls, 73 more to shooters out of sight. (4) Cape Violet: 6,800 of Blitzes died in the B3
  beamer camp at 8:45. (5) Great Divide, the plan: the ball holds at the pass's north mouth as the screen, and
  Shellshockers then Pillagers kill the camp for nothing. (6) escalate-3, WON: after two pushes into the fortress
  the ball went round by the north edge, over 1,300 from it, and found the lab. Reach decides: read the
  `enemies_near` line's "turrets covering it reach N"; if nothing of ours reaches back, it is artillery from a
  screen, the whole ball at once onto a nest it outweighs, or a route round.

- **One body at one place, or the push waits.** Cases: (1) escalate-1: about twenty Blitzes fed piecemeal into the
  commander's D-gun and the H2 turrets from 9:20; the win came when 57 gathered at spot_35 and went in together at
  13:46. (2) bluegecko-3v1-3: the groups reached spot_24, spot_37 and spot_60 one at a time all game and each was worn
  down alone; (3) bluegecko-3v1-2: the seats' groups broke his second wave when they fought together on our turrets at
  13:19, and every loss came when one group met him alone. (4) player-10: the route to his lab at 16:09 walked as
  a thin line, its tail the plant stream 5k behind, and reached no stop. (5) thebluegecko: "make sure your units are
  more of a straight line when attacking, less groups"; "your armies keep not fighting together". One gathering
  place, named; the groups arrive together or the push waits; new units join at the gathering place, not at the front.

- **Nothing is known until something of ours has looked.** Cases: (1) comet-1: fourteen Blitzes went east at 6:20
  with no radar and nothing scouted and fed nine into the commander's D-gun at 8:58 (the user: "radars would have
  helped there"). (2) escalate-3: four Rovers found nothing in the south-east; the base was north at G1-G2, as it was
  every time it was looked for. (3) player-9: three scouts sent at his base and none arrived (one turned home by the
  hands, one into two Pawns, one onto a hunt); the base was never seen. (4) player-10: three rovers on `rove` had his
  lab in the picture at 3:45 and came back alive. (5) bluegecko-3v1-1: his commander and factories were never seen by
  us all game; a person scouts and reads. (6) Great Divide: a scout plane before planning against the camp, since
  "known to hold" is only what our units have seen. A rover group on `rove` from the first Rascals, a radar at the
  front by 3:00, and no push onto a place nothing of ours has looked at.

- **The commander is a fighter at home and a builder at the plant, never a walker.** Cases: (1) comet-4: the
  commander was killed at home at 16:49 while the whole army was east; the recall came at 25% health. (2)
  bluegecko-3v1-2: a commander told to stay home died at home when the block arrived (15:51, 19:13); it goes to the
  neighbouring seat before the block is within 1,500. (3) human-1, Quicksilver: two Pawns at home are the
  commander's job (`attack`), not a reason to walk away. (4) human-8: the exchange rate of Grunts against Pawns
  turned on whether the commander stood with them. (5) escalate-4: the commander never engaged the Razorback because
  "help the plant" was still running on its list: cancel the list before a fight it is needed in. (6) The hands'
  own games: the commander died twice walking to far spots beside the enemy. It stands at the plant, fights what
  comes within its short walk, and leaves for the neighbouring seat, not the map's edge, when a block is coming.

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
  anyway on hard). A single Tick or scout car at an extractor is met by one soldier from the nearest group (`send`:
  one; two or four for a small party) or by a group left where the raids pass; the packet says which,
  before the ball leaves. The raids are standard and expected (Ticks from 5:00 on Comet Catcher, outermost spots
  first), so the answer stands across the front before they come: pickets of one or two Blitzes at each outer spot
  cluster from the first Blitzes out of the plant, told in words to attack on sight, with a light turret beside each
  pair of extractors; a home guard that walks out after the extractor is gone is not defence (the user, watching
  escalate-4: 33 extractors lost, most to single Ticks at spots no soldier stood near). [K-hands-ball-chases-lone-raiders,
  K-army-pickets-across-the-front]
- An enemy Aircraft Plant in sight (the report names factory types) means gunships in about ninety seconds: a Nettle
  beside each pair of extractors and Whistlers in the plant before the first one comes. Tier-1 tanks and bots do a
  fifth of their damage to aircraft and fire up only when nothing on the ground is in range; every unit's roster
  line says what it does to aircraft. (human-9: the plant was seen at 2:16 and reported as a bare grid, the first
  Banshee came at 3:54, six of them killed eight Blitzes at 8:22.) [K-army-air-plant-seen-means-aa,
  K-mech-air-damage-per-armour-class]
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
- Wrecks are income nobody is collecting. In the 29 Comet Catcher duels of the replay survey, 24 of 58 sides built
  resurrection bots (11 winners, 13 losers), a median of ten a side, the first between minutes 4 and 10, and they worked
  wrecks on their own ground rather than the field a fight just left (a bot within 400 of a death in the next 90 s in
  22 of 2,730 deaths); a constructor stood within 250 of a fresh death within 60 s after only 15 % of deaths; the winners'
  income ran above what their extractors give in 12 % of minutes against the losers' 7 %. What you have: the report's
  `wrecks` line and a wake when 500 metal of wrecks lies on ground we hold; `produce` armrectr (they work the richest
  held field on their own, raising soldiers worth 100 metal or more while energy is above half, and wait at home
  between fields); a constructor's `reclaim` option when a field is within 1,800 of it; `remove` for buildings of
  ours. Send constructors to wrecks on ground we hold, never onto a field the fight is still on.
  [K-eco-comet-pros-rez-bots, K-army-dead-waves-are-resurrected]

**Several seats.** The people in the game know a seat by its lobby colour, which the `sides:` and `seats:` lines
give beside each seat: in chat say "our purple seat", never `t2`, and never say a seat's faction or where it
started (a person in game 1: "your opponent will know what faction you are playing and where you spawned"). The
bot puts `[WReason] ` in front of every line
you say and cuts nothing (the game shows 127 characters a line; a longer text goes out as several lines): never
write the prefix yourself.
When the report's `seats:` line says you command more than one seat, every per-seat name carries
its seat's tag: `commander_t1` and `commander_t2`, `group_A_t2`, `party_3_t2`, `passage_1_t2`; the other seats' starts
are `home_t2` and so on, while `home` in an actor's paragraph is always that actor's own seat's start. Constructors
and factories are unique by number as ever, and `spot_N` and your marks are the same for every seat. A list, a
`produce` entry, a standing paragraph or a removal goes to the seat that owns the name; a paragraph headed
`constructors:` or `commander:` applies to every seat's constructors or commanders, a `commander_t2:` paragraph to
that seat's over it. An enemy party has one name for the whole side (`party_N`, the same in every seat's picture). Each seat's picture carries the other seats' groups under `allies` (theirs to order,
but their odds count with ours when they stand within reach of the same party), and every seat has its own economy
and its own map picture: a `_t2` actor sent to `passage_1_t2` goes to its own seat's first passage. Write the opening
for each seat's commander by its tagged name, and give each seat its own spots, written in full (`spot_10, spot_5`;
the hands also read a run like `spot_10, 5, 6`, but not past a word). The `transfer` tool moves metal and energy
between seats (the receiver's store caps it) and gives units, groups, builders or plants to a seat: the team's ways
are one army under the seat nearest the front, one seat's advanced plant fed metal by the others from the moment it
is placed (not from a clock), its first tier-2 units an advanced constructor for each seat, given away, so the mohos
go up everywhere at once (the case in the tier-2 decision above), and a dead seat's plants and constructors given to a live one
before the game's ending rules take them; a commander can be reclaimed by another builder for its metal (`remove`),
but a starting store holds 1,000 of its 2,700, so spend as it comes. Say in chat which seat is which by colour.

**What your hands can do (rebuilt 2026-10-02; `docs/design/2026-10-01-hands-rebuild.md`).** Every unit has one menu
and makes one move at a time; the role text lists the verbs. What matters for writing a packet:
- A group's moves are aimed at what you name and at what the picture sees: `go` and `fight to` at every place your
  packet, a list or a mark names; `attack`, `shell` and `send` at every enemy party in sight; `attack` and `shell`
  at his buildings wherever the picture knows them; `join` and `follow` at every other group, `follow` at every
  builder. Nothing is cut to "the nearest few" any more, and nothing is offered because code thought it fit.
- The hands have no destination of their own. There is no fall back to where a group last held, no walk home, no
  pull-out place, no sweep of unseen spots, no stand at the next extractor on a raider's heading: each of those was
  a place code chose. Where a group goes when it should not fight is a place its paragraph names; where it scouts
  is a place its paragraph names, or `rove`.
- A way through several places is prose: "group_A: spot_49, then spot_46, spot_40, spot_55, spot_58, back to spot_64,
  in that order, advancing; his buildings in sight it kills when they are undefended; from a party it does not
  outweigh it walks to spot_64". The group's entry says which named places it has reached (`reached`) and what it
  has met since the last (`met`); an arrival asks the hands at once, and a group standing with nothing ordered is
  told which move Jev itself rated best for it. Name the whole way and its exits in one turn, since your orders land
  about five seconds after the picture you read.
- A group is a body: its entry says the front, the tail, who has arrived and who is on the way from the plant, and
  the odds are priced on the part in the fight. Every move's words carry the fight the group is in (the odds, what
  it lost in the last 30 s, whether the party can follow), on the moves that keep the fight and on those that leave it.
- A scout's sentence that held under the enemy commander's eyes: "it is a scout: it runs from what can catch it and
  otherwise keeps to its route; being seen is its job".
- The enemy section lists his buildings by place with their guards, the parties that left sight with where and
  when, his biggest party known, and the first of each tier-2 or air type of his the moment it is seen.
Lessons below that were learned before 2026-10-02 name the old hands' options. Read them as the new menu's nearest
move: `send_against` with `how_many` is `send` (one, two, four or eight); `attack_unit` is an air group's `attack`,
and for a ground group the plain `attack`; `raid` is `attack` on his buildings at a place; `advance` and `fight_to`
are `fight to`; `walk` and `station` are `go` and `hold`; `escort` is `follow`; `hunt` is what a `send` detachment
does; `route_seen` is `reached`; `fall_back`, `retreat_home`, `sweep`, `pull_out`, `close_on_shooter` and
`next_extractor` are gone, and a place you name takes their place.
Nothing of this plays without a pick: say in the packet what you want done and the hands weigh it.

**Scouting and raiding with fast units: `rove` (2026-09-27).** `lane` with `{"group_R": "rove"}` hands a group of
fast units (Rovers, Ticks, scout cars; name a plant's output into it with `produce`) to code that runs each of them ten
times a second: it drives to look at what we know least (his start box and base first, then spots nobody has seen,
then the stalest), attacks what it finds unguarded (a constructor, an extractor, a radar with nothing armed in reach),
and never stands inside the reach of anything that can shoot it: a Rover outruns a Pawn and steps off before it is in
reach. It is scouting footwork, not an assault: at a base with Pawns patrolling, a roving Rover steps off from each
and kills nothing (player-20, player-21); a body sent to do damage goes with the evasion rules off, never on `rove`.
Your hands never move a roving group (no hunt, retreat or join), which is what killed the scouts before: in
player-9 three scouts went at his base and none arrived (one turned home by the hands at the first sight of his
commander, one into two Pawns, one onto a hunt). The group's entry says what each rover is doing and what it has
found (`rove`); `"on"` takes it back. Your hands' `scout` move makes a rover of a group's fastest soldier.
[K-micro-a-rover-outruns-what-it-cannot-fight]

**Which map.** Four maps have sections here, Quicksilver Remake, Comet Catcher Remake, Cape Violet V1 and Great Divide V1.
The `map` tool names the one you are on; read that map's section and its opening for the facts of the ground, and
the cases above whatever the map: a lesson from another map is the same situation with different spot numbers.

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
  fifth of the map wide. 80 spots: 14 in the west strip (spot_2, spot_7, spot_14, spot_19, spot_28, spot_30, spot_36,
  spot_45, spot_50, spot_54, spot_62, spot_67, spot_68, spot_74, north to south, at x 540 to 1370), 14 in the east
  strip (spot_5, spot_10, spot_12, spot_17, spot_25, spot_29, spot_34, spot_42, spot_48, spot_51, spot_60, spot_65,
  spot_72, spot_77), 52 in the open middle; which strip is ours the `map` tool says (the arena's `--corner nw` games
  were west, every game with people so far east). Write spot names in full in the packet, as here. Our start is placed within reach of two spots. The opponent's strip is 5,500 to 6,500 elmos east; the
  game puts the AI at an end of its strip (the north-east (6899, 681) or the south-west (1286, 5421), diagonal from
  ours, not straight across), so scout the strip, not the mirror point. Each extractor gives about 2.6 metal a
  second here.
- Constructors are the pace: a constructor a minute from each plant to four a seat by 4:00 and eight by 8:00 (the
  person's count), the allowances written so (`armcv` never capped at one or two); the report's `seats:` line
  carries each seat's constructors beside its extractors. The midline is not a wall: the free spots past it are
  named with their walk and their risk (the nearest party of his, the nearest turret), and the expand job takes
  them by walk, nearest first; "never go to the enemy's strip" means his strip, not the middle of the map.
- Wind is dead: 1 to 4. Solar collectors only (155 metal, a steady 20 energy a second each, no energy to build). A
  factory draws about 80 energy a second while it is being built, and vehicles cost eight to fourteen energy per
  metal (Blitz 900 for 110, Stout 2100 for 225, Mason 1950 for 135), so a plant running steadily wants about eight
  solars behind it; build them beside the commander before and while the plant goes up.
- This opponent opened with a bot lab in 91 of 104 recorded games on this map (a vehicle plant in 13), whatever the
  terrain says: expect Pawns and Ticks early and Maces or Rocketeers later, from the far end of its strip.
- **Splash: which shots hit an area, and why spacing decides** (the bench of 2026-09-28,
  `docs/studies/2026-09-28-body-sizes.md`; K-units-duel-spacing-decides-area-damage). A shot's area of effect as the unit file gives it (the engine's radius is half of it: the
  Fatboy's shells hurt everything within 150 of where they land), and its damage: Fatboy `armfboy` 300 (800 a shell, at 700); Leveler `corlevlr` 144 (190, at 315); Bull `armbull` 130
  (270, at 460); Janus `armjanus` 128 (two of 330, at 380); Mauser `armmart` 120 (260, at 820); Stout `armstump` 48
  (97); Blitz, light turret, Centurion, Sharpshooter 11 to 16 (single-target in effect). The damage falls to 15% at
  the edge of the area (65% for the Mauser, none for the Leveler: full to its edge). A body at our usual 25 to 40
  elmos between neighbours (the pros' 67 to 70) puts six to ten units inside a Fatboy's 300 and three or four inside
  a Bull's or a Janus's: at equal metal 25 Stouts lose to 4 Fatboys by 0.55 at 56 spacing and by 0.22 at 160; 21
  Stouts lose to 5 Bulls by 0.34 at 56 and draw at 160; 6 Stouts against 1 Fatboy go from +0.12 to +0.52. The rule:
  **against area (a Fatboy, Bulls, Janus, Levelers, Mausers), spread and few**: bodies at the dozen, 100 elmos or
  more between neighbours, the long-range units (Shellshockers, Mausers) doing the work from outside his reach;
  **against single-target fire (turrets, Centurions, Sharpshooters, Blitzes), tight and many** at once, every unit in
  reach of the one target. Nothing in the hands holds spacing yet (the spread rule was retired 2026-09-20): a wide
  formation is a `lane` and standing matter for now, and a body against splash is best made small at the source, by
  `produce` groups of a dozen.
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
- **The opening on this map, from the replays** (33 duels with both players at OS 40 and above and thebluegecko's
  13 at 48.5, 2026-09-24; the earlier VAK-Artur game of 2026-09-09 agrees) [K-map-comet-catcher-remake-1-8-two-solars-before-the-plant,
  K-map-comet-catcher-remake-1-8-commander-assists-the-plant, K-open-comet-our-plant-starves, K-open-comet-pro-order].
  **The recommended line is the pros' macro line** (the OS 40+ pool's median and VAK's game, K-open-comet-pro-order;
  restored 2026-09-28 after player-22: the Rover mass is snippet (0) below, an example until the hands can use the
  units). The commander: extractors on the two home spots and the third within 700 of the start (spot_67, spot_74,
  spot_73 from B8; spot_30, spot_28, spot_36 from A4), three solars (0:31, 0:40, 0:49; two at most if the plant is
  up by 0:45), never a fourth before the plant, the plant at 0:43 to 0:58 (standing 1:05 to 1:17), then **on the
  plant from the second it stands** and out only for one solar or one extractor at a time, back after each: the
  fourth solar at about 1:30, the fifth by 3:00, none beyond those while energy is full. Write it, from B8:
  `queue {"commander": ["extractor spot_67", "extractor spot_74", "extractor spot_73", "armsolar", "armsolar",
  "armsolar", "armvp", "assist 20", "armsolar", "assist 60", "extractor spot_68", "assist 60", "armsolar", "assist"]}`
  (a timed `assist N` that begins with the metal store under 20 is skipped by the hands and the next step runs, so
  the step after every timed assist is an extractor or the open `assist`, never a solar). In the turn before
  the game the start inside the box is not known yet: write the same list with a bare `extractor` for each named
  spot (`["extractor", "extractor", "extractor", "armsolar", "armsolar", "armsolar", "armvp", "assist 20",
  "armsolar", "assist 60", "extractor", "assist 60", "armsolar", "assist"]`), the hands taking the free spot the
  commander reaches soonest at each; and when the faction is not known either, with role words in place of the
  internal names (`solar`, `plant`, `lab`, `turret`, `constructor:1`, `raider`: the faction fills them in at the
  start), so the lists exist before the first second. The plant's first unit is a
  constructor vehicle (the third solar is there to power it: thebluegecko, game 5 of 2026-09-27, "the reason to make
  3 solar rather than 2 is so you have enough e to make a fast con"), then Rovers as scouts and Tick-catchers
  (`armfav`, 31 metal, speed 168: the only tier-1 vehicle that catches a Tick at 132; a Blitz at 101 never does),
  a second constructor near 2:00, never three at once, then Blitzes one every ten to fifteen seconds as VAK did
  (thirteen by 5:00): `produce {"plant_N": {"group": "group_A", "units": ["armcv:1", "armfav:3", "armcv:1", "armflash:6", "armcv:1", "armflash"]}}`
  (the hands build the counted entries in order, each to its count, the open entry last as filler). The pool has
  four soldiers by 2:00 and eighteen factory units by 4:00, the bank 152 at 2:00 and 99 at 3:00, the energy store
  under 20 in 9% of the seconds; winners assist less (32 s) and bank less at 3:00 (72) than losers (93 s, 107): they
  spend harder. The first constructor's list: the near spots outward (spot_43 at 2:33, spot_38 and spot_39 by 3:36
  from A4), a light turret at the plant the moment the first Tick or Pawn is in the picture, then the strip, a
  light turret beside each outer pair as its extractor goes up (the winners' 13 by 8:00 against the pool's 9.5).
  Two Rovers stay home from 2:30 against Ticks (BARb's first at 2:08); the rest scout: one along the strip to his
  end by 3:00, on `lane rove`, so his base is in the picture before the first push (E3, F6 of the review skill).
  The economy curve to hold: 7, 13, 17 extractors at 4:00, 6:00, 8:00 (the pool; below p25 is 6, 10, 12), the
  second plant started by 7:20, and the lead spent as groups sent elsewhere and tier 2 (the user's ruling), never
  as more of the same unit past a dozen in a body.
  **Other openings seen on this map, as snippets** (the start of a tree of practical actions by scenario; each
  says when it was played and how it went): (0) *thebluegecko's Rover mass* (OS 48.5; 13 of his 14 Armada sides on
  this map, 10 won, against three people at OS 29 to 43; K-open-comet-thebluegecko-rover-mass): three solars, the
  plant 0:58, the commander on it 140 s of the first 240, a Blitz then Rovers one every three seconds with the first
  constructor seventh; 24 factory units by 4:00 (the pool 18), extractors 4 to 9 (the pool 7), both stores at 0
  from 1:45 with the plant never idle. His Rovers go one by one to the enemy's extractors from 1:54 and trade
  there (2 to 12 alive at 3:00, 10 to 24 lost by 5:00, nearly all in the enemy half). **An example, not the line
  (the user, 2026-09-28, after player-22 and the experienced players' word): the build is valid if the pile is
  capitalised, and that takes Rover micro at his base that your hands do not have.** Four games on it: the body on
  `rove` at his base killed nothing (player-20, player-21: every Rover stepped off from every Pawn within a
  second); the body committed to the assault (player-22) died to Pawns and turrets without paying for itself. Do
  not open with it until a micro mode can use the units; a Rover is a scout and a Tick-catcher on the macro line.
  (1) *The Rascal-constructor line* (VAK, OS 52, against Artur91, won;
  K-open-comet-pro-order): extractors 0:03, 0:10, solars 0:31, 0:40, 0:49, the plant 0:58, a solar 1:29; two Rascals
  1:17 and 1:20, a constructor 1:24, a second 1:41, two Rascals, Blitzes from 2:12 one every ten to fifteen seconds
  (thirteen by 5:00), constructors at 3:09, 4:30 and 5:46; 9, 13, 19 extractors at 4:00, 5:00, 6:52 and three nano
  turrets by 6:37 on one plant. The economy line for a long duel: fewer raiders out, the extractor curve the pool's;
  what BARb's Pawn and Tick raids meet. (2) *The pool's median at OS 40+* (80 sides): the plant standing 1:07, the
  fourth solar 1:33, five solars and eighteen factory units by 4:00, the bank 567 at 1:00, 152 at 2:00, 99 at 3:00,
  the store under 20 in 9% of the seconds; the commander on the plant 50 s. Winners assist less (32 s) and bank less
  at 3:00 (72) than losers (93 s, 107): they spend harder. (3) *thebluegecko's bot lab* (once, against Hellontoast,
  won in 8:01): the lab at 0:03 before any solar, solars 0:21, 0:30, 0:40, then the commander takes extractors
  itself (0:51, 1:35, 1:50, 2:23, 2:37, 2:58, 3:17: eight by 4:00), two Pawns 0:21 and 0:32, five Ticks 0:46 to 1:33,
  Pawns, constructor bots at 2:14 and 2:38, a second lab 3:23, a light turret 3:29. Bots against a vehicle player:
  the Ticks scout and pick off constructors, the commander is the expander. (4) *Ours through player-11*: the
  constructor first (thebluegecko's own advice on the third solar, game 5: "the reason to make 3 solar rather than 2
  is so you have enough e to make a fast con"), then three Rovers, the second constructor near 2:00: 5 to 6
  extractors at 3:00 and 4x his army at 4:00 against BARb, and a lead never spent. (5) *BARb hard_aggressive* (the
  arena opponent, 91 of 104 recorded games): a bot lab, Pawns and Ticks from 2:08, six extractors at 3:00 and fifteen
  at 8:00, light turrets from 4:00 and a Beamer by 8:00, its advanced lab near 11:50.
  **The limit in the first four minutes is metal, not build power**: the store should read about 150 at 2:00 and
  100 at 3:00; a store at 0 with the plant idle or making a constructor is starving, and then the commander's own
  build is the thing to drop, never the assist. Check at 4:00: about eighteen units out of the plant (we had seven
  when the commander walked off to turrets and far extractors, and nine in player-12 behind four solars), five
  solars, the store between 50 and 200, seven extractors. No turrets and no
  extractors beyond the home spots from the commander before 2:00; the first constructor takes the next spots
  (spot_43 at 2:33, spot_38 and spot_39 by 3:36 from A4), a light turret beside each pair after that. From 2:00 the
  commander can walk toward the middle with the second constructor's work behind it, as the VAK game's did (an
  extractor every twenty to sixty seconds at the middle spots nearest home, a turret beside each pair, a radar at each
  outpost); the pros at 40+ mostly keep it nearer the plant until 4:00. Rascals (`armfav`, 31 metal) lose to the
  commander's D-gun; Blitz raids on the far extractors take the resign.
- **The replay survey: 29 duels of players at OS 25 and above on this map (2026-09-23; the claims in
  docs/knowledge/maps.md under "Comet Catcher Remake 1.8: the replay survey", the map file
  docs/knowledge/maps/comet_catcher_remake_1.8.md).** Spot numbers there are the game's 80-spot list, not yours: read
  the grid cell beside each. What the winners did: (1) the turret opening, extractor, extractor, solar, a light turret
  by 0:28, the third home extractor, the factory by 1:05 (18 of 29 winners had the turret before the factory; the
  later plant with it won 18 of 29 sides against the 0:34 plant's 11, though the later-plant player was mostly the
  higher OS) [K-map-comet-catcher-remake-1-8-two-openings, -turret-at-0-27]; (2) the commander takes the three home
  spots and the next pair toward the middle, the constructors take the flank strips (the commander built a quarter of
  the winners' extractors) [K-map-comet-catcher-remake-1-8-home-spots-then-flanks]; (3) from the A4 start the north
  strip B3, B3, A2, A1, B1, B1, C1 by 6:15; from B5 the winners went south-east (C5, C6, C7, D7, D6 by 4:44) and the
  losers took the back corner A6 first [-a4-expansion, -b5-winners-go-south-east]; (4) the game is decided between
  8:00 and 12:00: winners stood at 18, 23, 28 extractors at 8:00, 10:00, 12:00 with 7 constructors by 10:00 and
  9.5 by 12:00, losers at 15, 15.5, 14.5; from 8:00 to 12:00 winners added 11 extractors and losers 1
  [-decided-by-the-8-to-12-gap, -extractor-curve]; (5) a second factory by about 8:30, a vehicle plant, before 10:00
  [-second-factory]; (6) winners spent through the window: at 10:00, 15 of 22 losers had 500 or more metal banked
  against 7 of 22 winners [K-eco-winner-pulls-away-8-to-12]; (7) raids were met with turrets, 9 by 8:00 against the
  losers' 6, every side still lost an extractor (first at 4:18), and five or more scout cars by 3:00 in 38 of 45
  vehicle sides [-raids-met-by-turrets, -first-extractor-lost]; (8) tier 2 in 4 of 58 sides only; Blitzes and
  Incisors are the army through 12:00, line units under 3 % of the count [-tier-2-is-rare, K-army-comet-raiders-to-12];
  (9) a radar by 3:00 and a nano turret by 5:42 in nearly every side, solars only, 16 by 8:00 [K-open-comet-radar-nano-solars-habits].
  Against this opponent the Ticks and Pawns come earlier than the people's scout cars, so the turret at 0:27 and
  the turret pairs on the outer spots matter more here, not less.
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
  groups of 8 and of 48 stalled at home for 35 to 67 s trying to walk out. Solars and every other generator go at home, in the back
  field behind the plants (the hands put them there by themselves from 2026-09-28 when the builder is out on the map;
  at home a builder puts one beside itself), never on the frontier, and the plants apart. A defence at a place of
  your choosing is a `queue` step with the place after it (`armbeamer spot_35`, `armhlt north_gate` after a `mark` of
  the cell): the hands' own turrets only stand beside extractors, and every defence ordered at a place is set 120
  toward the enemy from it, so it covers the approach. Energy sat at 0 at 11:57 with five plants: about eight
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
- escalate-5 (HARD_AGGRESSIVE rematch, Opus 5.5 at medium effort, LOST at 28.5 min, called with the commander
  fleeing south at 35 %): the opening on the players' order again (plant 0:50, the commander out to spot_43/38/39,
  15 extractors and income 32 at 5:13, their commander seen at F1 at 2:43), and then the opposite mistake to game 4:
  **1,600 to 1,850 metal sat in the bank from 6:00 to 10:00** with one plant, a second from 7:12 and three
  constructors. A bank above 500 with income over 30 is build power missing, not a plant missing: the players have
  five constructors by 5:00 and three nano turrets on the one plant by 6:37 and never bank past 700. So: raise the
  constructor cap to `armcv:4` or 5 the moment the second one stands, nanos on the plant from 5:00, and only then a
  second plant. The Advanced Vehicle Plant was started at 9:16 at a marked yard west of home and died unfinished at
  15:08 to two or three Blitzes with no soldier near it (2,600 metal); the second stood at 18:45 and its Mausers
  came at 19:00, after the Stout ball had broken on an E2-E3 turret nest (two heavy laser towers) at 17:08 and been
  fed piecemeal into a Fatboy, Hounds and a sniper at 19:47. Put the tier-2 plant beside the home turrets with a
  Blitz pair standing at it, and start it while the bank is high, not after. Each new `queue` list starts from its
  first step and the build in progress continues; a list whose first step is a turret delays the plant behind it (four
  lists in a minute at 15:08-16:08 each began with `armllt home`, and the plant waited until a list began with it).
  Energy hit zero at 7:08 on eight solars with six Blitzes a minute (the players had twelve solars at 6:00) and again
  at 18:22 when advanced solars, which cost energy to build, crawled in the empty store: plain solars when the store
  is empty. 45 extractors lost, most to single Ticks and Blitzes at spots no soldier stood near; the pickets rule
  above is the answer. Their tier 2 came on the clock of game 4 (Hounds by 14:00, Fatboy 19:00, Razorback 25:48).
- escalate-6 (HARD_AGGRESSIVE, third game, Opus 5.5 at medium effort, LOST at 23.0 min, called with the commander
  fleeing south): the same opening (13 extractors and income 30 at 5:00, their bot lab found at G1 at 3:12 by the
  Rover) and the same bank: 1,357 at 6:00, 1,898 at 7:00, still 1,470 at 10:00, named in a note at 7:46 ("the key")
  and not spent until the raids had taken the income away. Pawn waves of four to eight into the north strip and home
  from 7:00 took 32 extractors, six plants (one half built), eight constructors and five nanos; the Blitz groups
  answered them piecemeal and `send_against` split the army into five to seven detachments that died one by one
  (15:58: "forced one ball, no detachments"). By 12:00 the count was 8 extractors and income 20, and the game was
  decided. Two constructors stood stuck 215-250 s at the second yard (one reclaimed at 13:49); a solar stood in a new
  plant's exit lane (`remove` took it at 9:39-10:06); a constructor's list ran on from an extractor and turret at
  spot_24 that stood at 1-3 % built (14:35-14:44, open). What three games at this tier say together: the opening is
  right and the game is lost between 6:00 and 10:00, when the metal must become constructors and nanos (five
  constructors by 5:00: `produce` counts `armcv:N` as N more from now, so ask for `armcv:3` again when three stand),
  the outer extractors must have a picket and a turret before 5:00, and the first Pawn wave at 7:00 must be met by
  one group at the strip's north end, not by detachments from everywhere.
- escalate-7 (HARD_AGGRESSIVE, fourth game, Opus 5.5 at medium effort, **WON at 16.0 min**, their commander killed at
  G1 (6387, 753) by 37 Blitzes and Stouts on `attack_unit` at 15:43, with their advanced lab, a Gauntlet and a
  beamer in the same three minutes): the first game after the hands were fixed to follow a `produce` ask (a list
  said again restarts its count) and to write the constructor count without a verdict. The plant made constructors
  8 asks of 13 when offered (game 5: 1 of 17): four by 3:19 on `armcv:3`, six by 6:20, and the extractor count
  climbed 11, 14, 17, 20, 22 at 5:00, 6:00, 7:00, 10:00, 13:00 with nine lost all game (games 4-6: 32-45). The bank
  still reached 1,870 at 8:00 (a second plant at 7:16, nanos, an energy stall 6:20-6:45 on seven solars with six
  builders) and drained from 9:00. The rest as the brief says: one ball at spot_39/24 with pickets north and south,
  their block of Rocketeers, Maces and Centurions broken at D4 at 11:33 (2.5k for 1.0k), the far-north route
  (spot_24, 0, 1, 8, 3, 11, 16, 6) past the E2/E3 nest to G1, a home guard of 2-7k at spot_33/39 the whole time, and
  everything onto the commander the moment it was seen. What it still cost: five Blitzes raiding F2 at 6:50 (the
  hands kept the walk after "abandon"), 1.5k chasing a party into the E2/E3 nest at 13:00, twelve Blitzes to LLTs at
  E1 (switch the plants to Stouts when the attack reaches structures, at 12:45 not 14:48), and a `produce` cap on
  `plant_1` that did nothing because the plant is `plant_5440` once it stands (three Rovers came). Open in the hands:
  lists given to constructors mid-walk at 8:19-8:42 were not taken; a plant's engine queue keeps units already
  queued when the allowance changes; the `shelling` place still leads a group under an unseen beamer.

- **Several seats against a person here (bluegecko-3v1-comet-catcher, 2026-09-27, lost at 21:00 from 31 extractors
  and 65 soldiers at 12:00).** What one experienced person did to three seats of ours on the east strip, and what to
  do instead. (1) Raids from 2:06, single Rovers and Blitzes at the outer spots: the north seat lost 17 extractors and
  answered none, because its Blitzes stood in one ball far from the spots. Each seat keeps two Blitzes as pickets among
  its outer extractors from 3:00 and a light turret beside each outer pair; the ball is not the raid answer. (2) The
  block: Stouts and Janus from 7:47, one block of ten to twenty Stouts at the nearest seat's home by 13:00, and the
  north commander died at 14:29 while the 50-Blitz ball of the middle seat sat at spot_48 for six minutes and then
  chased side parties. The three seats' soldiers are one army from the first Blitzes: `join` the middle seat's group
  as they come, station it between the seats (spot_35 or spot_42 on the east strip), and when a block is seen walking
  to any seat, the whole ball goes to that seat's home before it arrives; the threatened seat's commander stays home
  and helps its plant. (3) Shellshockers walked ahead of the screen and were pulled back; artillery stands behind
  Stouts or not at all. (4) Air from 17:00 (Thunder bombers, then Banshees): a flak or two and a Ferret at each seat's
  home by 15:00, before the first bomber. (5) A person scouts and reads: the enemy commander and factories were never
  seen by us all game; one Rover along the west strip by 5:00 and a radar at the middle from each seat.
  From the second game (bluegecko-3v1-comet-catcher-2, lost about 20:30 with the trade 56.7k to 33.5k in our favour):
  (6) Blitzes never go at a commander: two dives cost about twenty to its D-gun (5:33, 8:23); a commander in the
  open dies to Shellshockers and Janus from range, or is left. (7) Janus and Stouts eat packed Blitzes (eleven lost
  to Janus arriving alone at 12:09): from the first Janus seen, every plant makes Stouts and Janus, Blitzes only as
  pickets. (8) The seats' groups broke his second wave when they fought together on our turrets at 13:19-13:58; every
  loss came when one group met him alone. (9) He had a Bull and an air plant by 18:32: with three economies the
  strongest seat starts the advanced vehicle plant by 12:00. (10) A commander told to stay home dies at home when the
  block arrives (15:51, 19:13): from 12:00 each commander stands at its plant with two turrets and a Ferret beside it,
  and when a block is seen walking to its seat it goes to the neighbouring seat before the block is within 1,500.
  From the third game (bluegecko-3v1-comet-catcher-3, lost at 26:00 with 110k destroyed against 77k lost, 34
  extractors and 9.4k of army at 10:45): (11) The lead was never spent on tier 2: his Mauser shelled from out of
  sight at 16:00, twenty Banshees came at 14:00, Bulls at 18:30 and a Razorback at 25:00, and nothing of ours reached
  or outgunned them. The strongest seat's `queue` list carries the advanced plant (`armavp` for Armada, `coravp` for
  Cortex) by 12:00 whatever the front is doing, and its first units are Bulls (or Reapers) and a Mauser (or Tremor)
  for his; flak (`armflak`/`corflak`) and a fighter or two at each home by 14:00. (12) One gathering place before a
  push, and the groups arrive together or the push waits: the groups reached spot_24, spot_37 and spot_60 one at a
  time all game and each was worn down alone. (13) Name places, not parties, when the party's name has changed
  (four attack orders were refused on a stale party name); the hands attack what stands at the place. (14) When the
  seats are of two factions (the `seats:` line says which), each seat's lists and limits take its own faction's
  names: `corsolar` and `corvp` for a Cortex seat, `armsolar` and `armvp` for an Armada one, or role words
  (`solar`, `plant`) that each seat's faction fills in, which is the way to write the lists before the lobby has
  set the factions (human-10: with internal names only, the lists waited for the first report and the hands played
  the prose meanwhile).
  From the fourth game (bluegecko-3v1-comet-catcher-4, lost about 20:30 with the trade 85k to 30k in our favour;
  the opponent [Stud]Irishstud14, OS 48, Cortex, with thebluegecko spectating and advising): (15) The push at 8:00 into his Pounders (riot tanks that beat Blitzes and Incisors head on)
  stopped at his base, and from 11:00 his Incisor swarms of ten to fifteen slipped past the Brute ball and took our
  extractors from 33 to 9 by 19:00 while every fight we had we won. A ball of medium tanks never catches Incisors:
  each seat keeps its raiders (Blitzes, Incisors) as pickets at its outer spots with a light turret beside each pair
  from 4:00, and the ball stands between the seats; a spectator's "push until they die" is advice about his
  army, not about our extractors. (16) Against Pounders and Brutes the answer is Lashers or Janus behind Stouts or
  Brutes, never Blitzes alone; a raid group that meets a Pounder block walks away from it, it does not fight.
- **What the OS-48 players told us in chat, games 3 to 5 of 2026-09-27 (thebluegecko, Irishstud14; the lines in
  `docs/knowledge/_inbox/players-chat-2026-09-27.md`), to follow as written.** [K-open-comet-three-solars-buy-a-fast-constructor-first,
  K-eco-comet-no-converters-store-the-energy-and-two-hundred-a-second-with-nanos,
  K-army-riots-answer-a-light-tank-blob-and-mediums-answer-riots, K-army-an-army-lead-unspent-goes-bad]
  (a) The opening above: three solars, the plant, a constructor first, the commander on the plant. (b) A plant is
  never idle longer than two seconds: the `produce` lists keep every plant queued, whatever the store reads; the
  hands queue the next unit on their own now, your lists say which. (c) No energy converters on this map ("this is a
  metal heavy map"); spare energy goes into an energy store, which pays for bombers later; the economy aims at 200
  energy a second by 6:00 and a construction turret beside each plant by then ("orange and red both need nanos";
  "you dont have enough build power"). (d) Naked extractors of theirs are punished at once: the first raiders go at
  his outer spots and solars from 4:00, both corners at once, and keep moving ("an aggressive response is needed to
  prevent a large metal lead"; "spread out for the raid, attack both corners"; "focus on mexes and solars"; "try top
  corner, not much defence there"). (e) From about 7:30 a person switches to Pounders (Cortex riot tanks, slow, from
  the plant) against a light-tank ball: from the first Pounder seen every plant makes medium tanks (Stouts, Brutes)
  with three to five Lashers or Janus in all, never more ("too many lashers lose to med tanks"; "3 med tanks per
  pounder"), light tanks only for raids where the Pounders are not, or bombers over his solars if energy allows.
  (f) An army lead is spent the minute it shows: when the report says our army is far ahead and income behind, the
  seats' groups go as one line with few groups at his extractors, solars and then his plant ("up by almost 400%
  army value, time to go fight and win; if you dont use that army advantage this will soon go bad"; "make sure your
  units are more of a straight line when attacking, less groups"; "your armies keep not fighting together"), the
  constructors reclaiming behind them ("eat as you push"; "get that reclaim asap"). (g) His commander is sniped by
  light tanks with the target set on it circling it, never by walking a ball into its D-gun. (h) Tier 2 when the
  first heavy unit of his is seen or by 12:00, whichever is first ("either bomb or t2 now" at 6:35 of game 3; "go
  t2" at 20:39, too late); a lab of a seat that no longer needs it is reclaimed ("eat your bot lab"). (i) A few
  resurrection bots over a big wreck field. (j) In chat never say which faction a seat plays or where it started
  ("your opponent will know what faction you are playing and where you spawned"). From game 7 (u6bkep, 14:11):
  (k) The extractor count is the whole difference against a person ("the human player is taking mexes so much faster
  that 3v1 is about fair"): hold half the map while the raiders work his corners, every constructor expanding and
  none at home unless metal is spare. (l) A raid that stops moving gives him time to answer: a raid group's list of
  stops is written whole (spot after spot, a `mark` for each if needed), never one stop at a time. (m) When the
  store rises, build power: constructors first, nanos where the energy allows (they draw a lot). (n) When a seat's
  advanced plant comes online, that seat's other plants pause (`produce` them an empty list) so its income goes
  into tier 2: one constructor, then tanks. From game 9 (thebluegecko, lost 20:16 after leading three to one at
  13:07; the user: dozens of bad trades by the wrong unit in the wrong place): (o) Bulls (reach 460) and beamer
  nests (490) outrange every tier-1 tank; a group that goes at them pays the approach under their fire, so it goes
  in only as one body that outweighs them, never in ones and twos, and never chases into ground it cannot see
  (94 tier-1 tanks died to Bulls for 15 Bulls, 73 more to shooters out of sight). What answers Bulls is our own
  Bulls, Mausers and bombers, or a turret of ours where they must come; against a nest, artillery or the whole
  ball at once. The `enemies_near` line now says when a party or its turrets outrange the group. From game 10
  (thebluegecko, 2026-09-28, lost about 26:10 with the last commander; build d2b5e2c): (p) What we make tells him
  what is safe. His words at 25:10: "this isn't a static problem, it is adversarial, that is, you made many lashers,
  so I was safe to go T2"; and "brutes would have given you the ability to apply more pressure". A riot mass
  (Lashers, Janus) threatens none of his spots, so he techs behind it and his Tzars and Tigers then beat it; Brutes
  and raiders on his spots deny him the tech. Read his army each turn for what it says our mix has made safe for
  him, and change the mix before he does. (q) Two rules of this section were said to us again in that game and had
  not been followed: at 17:27 "Top should stop building other units while doing t2 transition" (rule (n): the
  seat's other plants pause when its advanced plant comes online; the player paused at 17:39, when told) and at
  24:09 "i think you made too many lashers" (rule (e): three to five Lashers or Janus in all, never more). Count
  them in the picture every turn; a rule that is not counted is not followed. (r) The commander assists the plant
  for its first few units (thebluegecko, after the game): the opening list's `assist 25` after the plant now holds
  its 25 s once the plant stands (until 2026-09-28 a queued timed assist ended within a second, a harness fault,
  so the plant's second unit came 25 s after its first with the store at 0); write `assist 40` after the plant and
  a second `assist 30` after the next extractor, and keep the commander at the plant until the first constructor
  and the first two soldiers are out. The condition on it (player-12, the first game the assist held): the assisted
  plant draws about 220 energy a second on a constructor against three solars' 90, and the start's store covers one
  such constructor; so the plant's list is a Blitz, five Rovers, the constructor, ten Rovers, the second constructor
  (`armflash:1, armfav:5, armcv:1, armfav:10, armcv:1, armfav`: his line, never `armcv:2`), and an empty energy
  store from 1:45 behind the assisted plant making Rovers is the assist working, not a call for solars (player-12 answered
  it with four, 620 metal at an income of 8, and stood on three extractors at 3:00).

**This map, Cape Violet V1 (one game with a person, 2026-09-27, three seats of ours on the east strip at H2, H4
and H6 against thebluegecko in the west; lost at 18:18).** [K-map-cape-violet-v1-a-water-map-with-a-third-of-the-spots-under-water]
- 10240 wide by 5120 tall, east against west, 40 % sea. 72 spots: 44 reachable on foot from our strip, 28 under
  water (the picture says which, "under water"). Wind 8 to 14: wind turbines alone starve three plants, solars
  with them. The land ways west run through the D2 passage (about (4130, 860)) and the B3 passage ((2375, 1683));
  the sea runs round both.
- The sea is a third of the map's metal and the person's road: he took every under-water spot with construction
  ships and advanced subs by 10:00 while we took none, brought Welders (amphibious tanks) over the water into our
  strip from 12:00, killed two commanders with Liche atomic bombers at 17:07 (one bomb kills a commander) and
  shelled from Longbow cruisers (range 1,550) at 17:49. What to do from the start: each seat's plant makes a Beaver
  (`armbeaver`, amphibious constructor) as its second or third unit with a list of the under-water spots nearest
  its home; one seat's commander builds a shipyard (`armsy`) at a water mark off its own coast by 4:00 (the site is
  kept on the water; the builder walks to the shore), its first unit a construction ship (`armcs`) for the far
  under-water spots, then Anemones (coastal torpedo launchers) at the spots facing open water and warships as the
  metal allows; a flak turret and two Nettles at each home by 12:00 against the Liches, and a scout plane over the
  sea by 8:00 so the cruisers and subs are seen before they fire.
- The land army stands at the D2 passage side with a radar and two turrets and takes the Welders on our turrets
  (they broke on 40 Stouts at spot_29 at 13:00); it does not walk into the B3 beamer camp, where 6,800 of Blitzes
  died to Beamers and Welders at 8:45.

**Water maps, and SailAway 2 (the first game there, 2026-09-28, three seats against thebluegecko; 10240 by 10240,
wind 5 to 20, the seats on islets: a seat on one had three of 96 spots reachable on foot).** Energy on the water is
the tidal generator (`armtide` / `cortide`, 90 metal, 400 health, a steady energy rate the map sets: the option's
words say how much here), built on the water where the builder stands: a commander that starts at sea builds tidal
generators beside itself and never walks to a shore for a solar (in that game one did, slowly, for its first energy,
because the hands had no tidal generator on their menu until 2026-09-28). Wind at 5 to 20 is worth turbines on the
land spots. The land economy of an islet is a few spots: the rest is the sea's, taken by amphibious constructors
(`armbeaver`; the plant's second or third unit with a list of the under-water spots) and construction ships from a
shipyard at a water mark, as the Cape Violet section says; the army that matters crosses water (hovercraft from a
hover platform, amphibious tanks, ships), and a ball of ground tanks on an islet defends the islet and nothing else.
His start on such a map is unknown until a scout plane or a ship has looked, unless the lobby fixed the positions
(the picture's start_boxes_note says so, and then his start is exact); the lobby's boxes may not be honoured (the
note says when they are not), and the hands' guess is then the mirror of our start. Under the water (the engine's
own rules): a submarine, a ship's hull below the line, or a commander walking the seabed is seen by sonar only, never
by eyes or radar, and is hit by torpedoes and depth charges only; a shooter under the surface hits only what is in
the water, and a commander under water shoots nothing. So each seat on a coast builds a naval radar/sonar tower
(`armfrad`/`corfrad`, the commander can) at its shore by 3:00 and a torpedo launcher (`armtl`/`cortl`) where his ships
and subs come, the picture's odds say when a party is under the water and the group has nothing that reaches it, his
torpedo boats and subs cannot hurt a group on land, and our own commander can cross water unseen along the seabed
when it must move, shooting nothing on the way. From game 11 there (resigned 24:33 to Riptides and Buccaneers;
thebluegecko: "it is better than barbs at water. barbs are bad at water"; the host: "I am told hovers are bad
generically"): the sea is won by a navy, a shipyard (`corsy`/`armsy`) by 4:00 on the seat with the coast, its first
units a construction ship, then subs and Corals (`corfhlt`, a floating defence, 630) at the approaches, frigates
after; hovers are a raid and a landing party, not the army, and a hover artillery (Mangonel) or any tier-1 artillery
is one to three behind a body of line units, never the body. From game 12 there (resigned 14:47; the navy came,
shipyards on every seat by 1:09, and lost anyway): reach decides at sea as on land. The Riptide frigate (`corpship`,
480) outranges the Dolphin (`armdecade`, 280) and the Supporter (`coresupp`), and the report said so from 5:25
("it outranges everything here (480 against our 280)") while every yard was held to Dolphins and Supporters by a
4:09 `produce` never lifted; the first Ellysaw (`armpship`, 500) came at 10:05, after the fleet was gone. From the
first Riptide seen, the yards make Ellysaws (or Riptides of our own on a Cortex seat) and the raider ships stop; a
sea push counts reach, not weight, and goes as one fleet, not three groups an islet apart. The sonar tower by 3:00 was
this section's rule and was ordered at 12:40; his seaplane platform (`corplat`, an air factory) was seen at 9:05 and
the first Naval Nettle stood at 14:05: an air factory of his in sight means anti-air at every seat within two minutes,
at sea as on land. A spot's line now says "under water" for a spot under the surface (a construction ship or an
amphibious constructor takes it) and "on ground our walkers cannot reach" for an islet (a hover or amphibious
constructor; a construction ship only to its shore): read which before sending a ship. The seats divide the work
(the user, after game 12: "One problem with that match was lack of coordination. Each seat built both a shipyard and
a hover lab before beginning to expand, slowing down initial mex capture significantly": every seat had a shipyard
by 1:06 and a hover platform by 3:01, and the side stood at 14 extractors at 8:00 against 51 the game before): one
seat builds the hover platform and gives hover constructors to the other two with `transfer` (`{"units": [...],
"to": "t2"}`), two seats build shipyards, and every shipyard's first units are construction ships that expand the
under-water spots exactly as a constructor does on land, with a `queue` list of those spots by name; the ten
construction ships of game 12 built tidal generators, floating converters and torpedo launchers and three extractors
between them. Tidal generators are the commander's and the hover constructors' work at home; the ships take spots.

**This map, Great Divide V1 (the games with people from 2026-09-27: two seats of ours under you, the north box A1-H2
against the south A7-H8, realtime; the opponent a person who, we are told, will fortify the pass and try to win from
there).** [K-map-great-divide-v1-one-pass, K-map-great-divide-v1-the-choke-is-the-plan,
K-map-great-divide-v1-reach-over-the-divide, K-map-great-divide-v1-not-air-only, K-map-great-divide-v1-our-team-games-stayed-home]
- 3072 wide by 4096 tall: a tall map played north against south. Rows 1-3 and 6-8 are open; rows 4-5 are the divide,
  cliff from edge to edge with one walkable pass at E5 (1570, 2085), 644 wide, 63% of the way from our start to
  theirs. Nothing walks round it: every raid, every army and every constructor of theirs comes through that pass, and
  every one of ours goes through it. No water. Wind 0 to 20: solars, and wind generators only while the readout says
  it blows. 22 spots, eleven a side, so the ground economy is even and the pass decides nothing by itself.
- Our eleven, by walking distance from the north-west start (a seat placed in the east of the box has the east ones at
  its door): home spot_3 B1, spot_2 A1, spot_0 B1 (under 350), then spot_7 C2 (920) and spot_9 B3 (1,200); the middle
  spot_6 E2 and spot_10 E3 (1,700); the east spot_1 G1, spot_8 F2, spot_5 G1 (2,200-2,260) and spot_4 H1 (2,540). The
  twelfth, spot_11 E5, sits at the pass's south mouth, inside whatever they build there: not ours until the camp is
  gone. Theirs are the mirror in rows 6-8 (spot_12 to spot_21), their start most often at the south box's west end
  (about (500, 3700)) or its east end (about (2600, 3750)) in the public duels.
- **Two seats.** The report's `seats:` line names both; each has its own economy and builders. From the first turn:
  the west seat takes spot_3, 2, 0, 7, 9 then 6; the east seat spot_1, 5, 4, 8 then 10; every extractor stands by 6:00.
  One radar and two light turrets at the pass's north mouth (E4, about (1570, 1750)) by 4:00 cover every extractor we
  own, because raiders have one way in: the rest of the north needs no turrets, and a constructor's `never` holds
  everything south of the pass until the camp is broken. Our three team games here (2026-09-20, the old bot) sat at
  home on five extractors for 35 minutes and never named the pass: name it, and the spots, in the packet.
- **Their plan, and the trap in it.** A camp at the pass: light turrets by 3:00, heavy by 6:00, then Pit Bulls or
  Gauntlets with artillery and Dragon's Claws behind, and their army waiting behind the guns for ours to walk in. The
  reach that decides it: light turret 430, Beamer 490, heavy turret 620, Pit Bull 730, Gauntlet 1,220, Cortex Punisher
  1,245. A ball advancing into the pass against standing turrets is the Comet Catcher nest again (16k lost for 6k,
  then 10k for 1k, two games running): it does not happen. The ball holds at the north mouth, out of the camp's reach,
  as the screen for the artillery and the threat that keeps their army home.
- **How the game is won.** Three things in this order. (1) Economy: two seats against one; one seat's tier 2 by about 10:00,
  fed by the other's metal from the moment the plant is placed, its first units an advanced constructor for each
  seat, and mohos on both seats after; twice their income by 12:00, the bank spent. (2) Fire over the camp: Shellshockers (armart,
  710) from a Stout screen at the north mouth outrange light and heavy turrets and kill the early camp for nothing;
  once Pit Bulls or Gauntlets stand, Pillagers (armmerl, 1,300, from the advanced vehicle plant) outrange them from
  the same ground, a dozen at a time with the screen in front. Artillery, not the ball, breaks a camp. (3) Air over
  the divide: an air plant (armap) on one seat by about 8:00, a scout plane first (see the camp before planning
  against it: the report's "known to hold" is only what our units have seen), then bombers (Phoenix, Thunder) in one
  wave at one target, their artillery and the nano-turrets behind the camp, or their extractors in rows 7-8; expect
  flak (850) and Ferrets (950), so a wave, never a trickle. Never air alone: an experienced player's word is that an
  all-air side loses (the answers are cheap, and nothing holds the pass or the spots); air is a lever over the ground
  army and the artillery, not a replacement. (4) When the camp's guns are dead, cross with everything at once
  (`fight_to` onto their base, both seats' groups as one), leaving the north to its turrets: their base is rows 7-8.
- Our seats are Armada (vehicles: the ground through the pass is walkable for bots and vehicles alike); the person
  may be Cortex or Legion, whose twins have the same ranges within a few percent (Wolverine 710, Tremor 1,470).

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
