<!--meta
id: D-air-not-optional
layer: derived
premises: [D-metal-is-the-game, D-energy-second, D-surprise-weapons, D-reach-nests, F-eco-energy, F-eco-build, F-fight-damage, F-fight-death, F-fight-sight, F-map-metal, F-unit-armap, F-unit-corap, F-unit-armfig, F-unit-corveng, F-unit-armthund, F-unit-corshad, F-unit-armkam, F-unit-corape, F-unit-corcrwh, F-unit-armca, F-unit-corca, F-unit-armck, F-unit-armrl, F-unit-corrl, F-unit-armjeth, F-unit-armsam, F-unit-armcir, F-unit-armsolar, F-unit-corsolar, F-unit-armwin, F-unit-armmakr, F-unit-armnanotc, F-unit-corbw, F-unit-armatlas, F-unit-corvalk, F-unit-armhawk, F-unit-corvamp, F-unit-armliche, F-unit-armawac, F-unit-corawac, F-unit-armllt, F-unit-armbeamer, F-unit-armmex, F-unit-armmoho, F-unit-armack, F-unit-armcv, F-unit-armcom, F-unit-armpw, F-unit-armflash, F-unit-armstump, F-unit-armplat, F-unit-armsb, F-unit-armsfig, F-unit-armaap, D-tier2-sprint, D-raid-answer, M-pool-air, M-pool-energy-and-power, M-gecko-vs-us-comet, M-gecko-team, F-barb-how-it-decides, F-barb-profiles, M-barb-sparring, M-format-8v8]
lessons: [L-sheet-labs, L-other-u6bkep-nests-and-air, L-gecko-bots-vs-vehicles, L-energy-converters, L-tier2-go, L-mex-game-over, L-sheet-surprises, L-gecko-air-details, L-gecko-one-more-con]
sources: {people: [thebluegecko, u6bkep, Irishstud14, the user], dates: [2026-10-02, 2026-10-23, 2026-10-25, 2026-10-27], relay: thebluegecko's spoken notes on what this article left out (2026-10-27, L-gecko-air-details); before them, the user's account of the experienced players on air (2026-10-25 night, "my relay here is imperfect, but the core points should be correct"); thebluegecko's sheet rows on the aircraft plant; u6bkep's chat in human-10; code: [upstream/Beyond-All-Reason/units/ArmAircraft/armthund.lua (the bomb: 5 x 110, area 144, edge 0.55, reload 6, burstrate 0.25), upstream/Beyond-All-Reason/units/ArmBuildings/LandEconomy/armsolar.lua (damagemodifier 0.5 while closed), upstream/Beyond-All-Reason/weapons/Unit_Explosions.lua (metalmaker 660 over 210; nanoboom 530 to construction turrets over 128, edge 0.75; windboom 89 over 180), upstream/RecoilEngine/rts/Game/GameHelper.cpp:131-150 (falloff from the surface), crates/terrain/src/sheet.rs (spots by air alone), upstream/Beyond-All-Reason/units/CorAircraft/corbw.lua (the stun: 600 a shot, reload 1.2, reach 220, paralyzetime 7), upstream/Beyond-All-Reason/gamedata/modrules.lua (paralyzeOnMaxHealth, paralyzeDeclineRate 40), upstream/RecoilEngine/rts/Sim/Units/Unit.cpp:996-1013, 1253-1305 (stunned while the stun passes the maximum health; it falls by a fortieth of the health a second and is capped at 7 s past the last hit), upstream/Beyond-All-Reason/units/ArmAircraft/T2/armliche.lua (3,150 to commanders), upstream/Beyond-All-Reason/units/ArmBuildings/LandUtil/armnanotc.lua (mass 700, cantbetransported false, reach 400), upstream/Beyond-All-Reason/units/ArmAircraft/armatlas.lua (transportmass 750, transportsize 3), upstream/RecoilEngine/rts/Sim/Units/UnitDef.cpp:351 (a unit's mass is its metal cost unless set)]; the pool cut and the passes: /tmp scratch scripts air_cut3.py and bomb_pass.py of 2026-10-25 night over the 1v1 cards re-cut from the archived records; the Glitters 2v6 record run/matches/1792971120-human/00/record-1.jsonl}
status: in the manifest since 2026-10-26 (the user read it: "This looks good, let's add it"); amended 2026-10-27 from thebluegecko's notes (the Shuriken, Banshees on extractors, transports, tier-2 air and seaplanes past the ground's anti-air, the sea; the gunship line corrected): the arithmetic from the unit files and the engine, none of it drilled; the bomb pass is worked by hand from the unit files and the engine's falloff, not measured in a drill; the matched-income cut and the team pool's cut not done
-->
### Air is not optional

**The heuristic.** thebluegecko's sheet on the aircraft plant (L-sheet-labs): "fighters are the strongest anti bomber
per cost. bombers are the lowest cost way to destory oponents E production after the map is split, or if ground forces
are hard to push through. This means most pro players make air lab around 30m/s on many maps. Stronger on maps with low
e cost". u6bkep, watching his bombers break our side on Gecko Isle (L-other-u6bkep-nests-and-air): "figs is the only
reliable way to stop a large bomber force like that. this is something you need to intuit or scout your opent" ...
"and pre emptively go air". Irishstud14 with our energy in surplus (L-tier2-go): "either bomb or t2 now". The plain
line: from about 30 metal a second, an air plant is part of the economy, built before his air is seen; it answers his
bombers with fighters, takes the spots and the paths the ground cannot, and is the one army that goes round a front
nothing on the ground can break.

**The quantity.** Two clocks, as in D-surprise-weapons: the minutes his aircraft act unanswered over our energy, our
builders or our commander, against the minutes ours act unanswered over his. The answer's price is fixed and small
(a plant and a few fighters); what an unanswered wing takes grows with what stands under it. Not the plant's cost, which
at 60 metal a second is eleven seconds of income.

**The mechanism.**

- *The entry fee, and what it buys per metal.* An aircraft plant is 650 metal, 1,100 energy and 5,450 of build: 68
  seconds for one tier-1 constructor, 18 for the commander alone. Against aircraft a Falcon does 139 a
  second for 73 metal, 1.9 a metal; a Nettle 0.85 (68 for 80), a Chainsaw 0.83, a Crossbow 0.44, a Whistler 0.32
  (F-fight-damage: most ground weapons fire upward only with nothing on the ground in reach). Per energy the Nettle is
  better (0.075 a second per energy against the Falcon's 0.048), and a Falcon has 167 health. So the sheet's "per cost"
  holds in metal, and twice over in reach: a turret covers 765 around itself and must stand at every field; a fighter
  wing at 289 a second crosses 3,000 elmos in ten seconds and covers every field at once. Two Falcons (146 metal) kill a
  Stormbringer (150 metal, 670 health) in about two and a half seconds; a bomber at 250 a second crosses a Nettle's
  1,530 of reach in six, taking about 400, so one Nettle does not stop one bomber and a field wants two.
- *One bomber over an energy field* (the unit files; the engine measures a blast from the target's surface, so a bomb
  falling between packed buildings hits each of them nearly in full). A Stormbringer drops five bombs of 110, 62 apart
  along its path, each reaching 72 from a building's edge, every six seconds.
  - Solars close when hit (F-eco-energy) and take half damage closed. One pass over a packed field (gaps of 0 to 16)
    kills none and hits 13 to 16 of them; each stays dark about ten seconds, 260 to 320 energy a second gone, and a
    bomber that keeps passing keeps them dark until it is shot down. Two passes kill one, three passes three of a block
    of 24; a field 48 apart is hit four at a time. Against solars the bomber's damage is the blackout, not the kills:
    the wave that killed 19 solars in one minute in the pool (M-pool-air) was fifteen bombers.
  - Wind turbines (196, no armour): touching, one pass kills 11 of 24 and two kill 21; 16 apart, 4 and 14; 48 apart,
    two or three.
  - Converters (167): two bombs kill one, and its burst does 660 falling to nothing at 105 from its centre
    (F-fight-death), which kills a converter or a turbine whose edge stands within about 78. A block of converters 54
    or less apart dies whole from one pass; 60 apart, four of 24.
  - Construction turrets (560): a turret's death bursts 530 to other construction turrets alone (11 to anything else),
    falling to three quarters at 64 from its centre: 461 to a touching neighbour, 374 at 16 apart, so an unhurt
    neighbour survives one burst and a neighbour already hit for a hundred or so does not. A block of 24 touching or
    16 apart takes one bomber's first two passes with nothing dead; on the third, twelve seconds in, the first dies and
    the burst takes all 24; 32 apart, 3 to 15 go; 48 apart, the burst reaches none. Any damage that kills one turret of
    a block already hit (a wave, artillery, a raid's splash) starts the same chain.
  - This is "vulnerable to chain death if packed" (the user): the converter field's property and the construction
    turret block's, and the turbine field's in part; a solar's burst is 89 and chains nothing.
- *What the pool's bombers did* (M-pool-air). In the 40 Comet duels at 40 and above, bombers killed in six games, every
  kill from minute 12 on and most of them solars; the bombing side won all six. Their largest pass killed six, in three
  passes; the field raids were waves of nine to fifteen.
- *The wave that goes round the front.* A ground army meets turrets and the other army on the way in; an aircraft meets
  only anti-air and fighters, and arrives at 234 to 250 a second where the defender's army walks at 40 to 90. A side
  ahead in economy that cannot break a front of turrets (the sheet's "if ground forces are hard to push through") has
  the air as its way to his energy, his builders and his commander; D-surprise-weapons works the Banshee snipe and its
  clock, and the same clock governs a bomber wave: it pays where his fighters and his field anti-air do not yet
  stand, and is a waste where they do.
- *The Shuriken* (Cortex's stun drone: 58 metal, 83 health, speed 280; L-gecko-air-details: "some of the strongest
  units in the game"). Each shot is 600 of stun at 220 every 1.2 seconds; a unit is stunned while the stun on it passes
  its health, and wakes at most 7 seconds after the last shot. One shot stuns a Pawn (370), a Grunt (280) or a Thistle
  (335); two a Blitz (730), a construction bot or a light laser turret (620); three a Beamer (1,430); four a Stout
  (1,800); seven at once a commander. A stunned unit neither shoots nor moves, and the shot that stuns it holds it the
  full 7 seconds, so at best one Shuriken keeps about five Pawns or three Blitzes down at once (a new one every 1.2
  seconds; whether the drones spread their shots over units not yet stunned is unchecked); nothing dies of it. Hence
  his three uses, each with the same condition:
  - *with a ground attack at the same moment*: the stunned army is shot by ours without shooting back, which is how "a
    losing-sized army" trades up. Stun alone kills nothing, and stun that lands before or after our army arrives buys
    nothing.
  - *against run-bys*: a few hold the raiders at our extractors until our army arrives (D-raid-answer's clock, with the
    raid's damage stopped while it runs).
  - *against a turret nest*: a light laser turret answers aircraft at 10.7 a second (it shoots them last, and weakly),
    a Beamer at 20, so Shurikens stun a nest's turrets with nothing hurting them, and the ground army walks into a nest
    that does not fire (D-reach-nests). A Thistle kills a Shuriken in about 1.2 seconds of fire and the wing crosses the
    545 between its 765 and the Shuriken's 220 in about two, so a spread wing loses a couple a Thistle and stuns it with
    the first shot that lands; flak or fighters unseen are what make it "fall apart badly". Spread them, stun the
    anti-air first.
- *Banshees on extractors* (Armada's gunship: 135 metal, 560 health, 38.6 a second, reach 350, speed 185). A Banshee
  kills a tier-1 extractor (270) in about seven seconds and is answered by a light laser turret at 10.7 a second
  (52 seconds to kill it), so the extractors of the map's middle and far side, guarded by turrets and no anti-air, are
  its targets, one Banshee to one extractor and all at once across the map. Against units it does too little before
  fighters come: a Falcon kills a Banshee in four seconds, two in two, for 73 metal each. A tier-2 extractor (2,800)
  takes it 73 seconds: not its target. When his fighters come it either takes a last extractor or two each, or comes
  home to defend against run-bys over our anti-air, where fighters that follow die cheap. The window is the wide map
  whose front is about to settle and has not (L-gecko-air-details); once it has, bombers on his energy are the plain
  answer (above).
- *Past the ground's anti-air: tier-2 air and seaplanes.* His tier-1 anti-air prices a tier-1 aircraft: a Thistle or
  a Nettle at 67.6 a second kills a Stormbringer (670) in ten seconds, which is why a field wants two (above). The
  advanced aircraft plant's units are priced against fighters instead:
  - a Wasp (Cortex's tier-2 gunship: 370 metal, 1,560 health, 111 a second at 410) takes 23 seconds of a Thistle and
    28 of a Crossbow (55) and kills the Thistle (335) in three; a tier-1 extractor dies in 2.4 seconds, a commander
    (3,700) to ten Wasps in under four.
  - a Liche (Armada's atomic bomber: 2,200 metal, 46,000 energy, 2,300 health) drops 3,150 on a commander: two bombs
    kill a whole commander, one a commander already hurt for 550.
  - the radar planes (Oracle and Condor, about 175 metal, radar to 2,400-2,500, sight 1,250) see the commander as a
    radar dot through its cloak, which is how the snipe finds it.
  - tier-2 fighters beat tier-1 ones by far: a Highwind (150 metal, 370 health) does 536 a second to aircraft against
    the Falcon's 139 (3.6 a metal against 1.9) and kills a Falcon in a third of a second, so fighters that stop his
    tier-1 air are answered by tier 2.
  - seaplanes (the Seaplane Platform, 1,450 metal, made by the sea, amphibious and air constructors) are the tier-1
    form of the same bet "to some extent": a Tsunami bomber has 1,170 health against the Stormbringer's 670 (17 seconds
    of a Thistle), a Cyclone fighter 252 a second to aircraft against 139.
  Both cost energy above all (D-energy-second), and both are what the 60-rated players rush "to at least threaten
  sniping the comm" (L-gecko-air-details): the commander snipe of D-surprise-weapons with ground anti-air no answer to
  it.
- *Transports* (the Stork and the Hercules: 68 to 74 metal, about 1,400 energy, 265 health, speed about 200; each
  lifts one unit of mass up to 750, and a unit's mass is its metal cost unless set). A construction turret (230 metal,
  200 of build power at 400 of reach, mass 700) and a tier-2 constructor bot (430 metal, mass 430, 210 of build power,
  speed 33) both ride. A tier-2 constructor alone builds an advanced extractor (14,900 of build) in 71 seconds and walks
  33 a second between spots; with two turrets dropped beside it, 24 seconds, the three moved at about 200 by three
  transports (204 metal). Per build power a turret on a transport (298 metal for 200) costs about what a construction
  vehicle does (135 for 90) and moves four times as fast; its price is attention (the drops), which the hands do not
  yet offer. On a wreck field a dropped turret takes everything within 400 at 200 a second with no walking between the
  piles, against a construction bot's 80 at 130 that walks to each (L-reclaim-field). And turrets that cannot all stand
  where they are needed can be built where there is room and carried.
- *Air constructors.* 110 metal, 2,100 energy, 50 of build power against a construction bot's 80, but 136 a second
  against 36 and above every building and cliff: across a crowded base, to a far spot, onto ground nothing else stands
  on. Some maps have spots that only they reach: on Project SD-129, 25 of 148 stand on islands in the void, beyond the
  build distance of anything that walks, drives, hovers or sails (the map sheet's "built by air alone"; none on the six
  other maps we hold terrain for). In the pool they were few (5 of the 22 air sides, three each).
- *His air against us.* thebluegecko had an air plant at 11:29 to 17:06 in the replayed Comet games against our seats,
  Stormbringers 3 to 10 and Banshees 6 to 20 from 14:00 (M-gecko-vs-us-comet). On All That Glitters on 2026-10-25
  (M-gecko-team) his air was tier 2: Wasps from 27:17, 62 in sight at once at most, Dragons from 37:07, and air
  constructors from 26:57, 30 in sight at once. Our six seats built no air plant in 44 minutes; their anti-air came
  after the Wasps and was all ground (44 Furies, the tier-2 anti-air vehicle, 65 Lashers, nine Eradicators on the flagship's seat;
  Thistles, Sentinels and a Birdshot on the others'). Three of our commanders died to his Tzars and Shivas from 39:57,
  the last three under his gunships from 42:07. We had lost before that (the user: "the games were always lost before
  that"); the air ended it, and no ground anti-air we built stopped it. Our own model-played games built an air plant in
  4 of the last 30.

**Consider, and why it may work.**

1. *An air plant from about 30 metal a second in a game that will last past 12 minutes.* The pool's strong players
   build it 3 to 7 minutes after first reaching 30, at 46 to 82 of income (11:48 at the median); in games that last 15
   minutes, 17 of 30 sides do. The sheet's 30 is where it begins, not the median. On a map where energy is cheap
   (wind, tidal) it comes earlier and pays more.
2. *Fighters as the field's anti-air,* one wing for the base rather than two Nettles at every field, once the plant
   stands; Nettles where a field stands alone and far.
3. *Fields that a pass cannot take whole.* Converters 60 or more apart, never in a block; construction turrets 48
   apart, or in small groups with gaps between (a block of nanos round a plant is one target, its whole build power
   at once); wind turbines with a gap of a turbine; solars may stand packed, since they die slowly, but a packed solar
   field is the bomber's blackout target and wants its anti-air first.
4. *Bombers when his energy stands exposed and his front holds.* His field without anti-air and fighters is the
   lowest-cost target on the map (the sheet's line; L-gecko-bots-vs-vehicles: "bomb the enemy's E earlier, you can
   spiral that E advantage"); a full energy store is the cue ("you can use it for quick bombers", L-energy-converters).
5. *Air constructors where the ground is slow or closed:* the spots only they reach, a base too crowded for a
   constructor to cross, the far spots of a wide map.
6. *Scout his air plant.* Its standing is the clock on everything above (u6bkep's "intuit or scout"); an aircraft plant
   of his that we have not seen is the reason to build fighters before his first wing, not after it.
7. *Shurikens as Cortex, in time with the ground:* over his army the moment ours reaches it, over raiders at our
   extractors, over a turret nest ahead of our army; spread, his anti-air stunned first.
8. *Banshees on his extractors on a wide map whose front has not settled,* one to an extractor across the map, and home
   over our anti-air when his fighters come.
9. *Tier-2 air when his answer to our air is ground anti-air,* and tier-2 fighters when his fighters stop our tier-1
   air: the Wasps on extractors and commanders, the Liche on the commander, a radar plane to find it.
10. *Transports with an air plant already standing,* to carry the tier-2 constructor and its construction turrets
    from spot to spot, a turret onto a large wreck field, turrets to where there is no room to build them.

**Where it bends.** (1) Short games: the pool's duels end at 12:00 at the median, and in the 40 Comet duels an air
plant was built by 22 of 80 sides; the plant belongs to the game that goes past the first army's fights, and an air
plant at 8:00 that delays the second ground plant is the early trap of D-metal-is-the-game. (2) Against BARb, whose
static defence is on a clock with anti-air at 520 s on medium and whose counter table answers air with anti-air
(F-barb-how-it-decides, F-barb-profiles, M-barb-sparring): bombers meet anti-air early and the lesson of the people's games travels less. (3) In
a team one seat's air plant covers the side (M-format-8v8): fighters for every seat's fields from one plant, the seat
that holds the side's energy (D-energy-second) the one that builds it. (4) Gunships are not bombers: they kill one building at a time, not a field. A Banshee's target is
an extractor, never an army (above); a mass of Banshees or Wasps at his commander is D-surprise-weapons' snipe
("could they switch into 10 banshees and kill my com", L-gecko-one-more-con). (5) At sea: ships are dear, so anti-air
is seldom built before it is needed, ships are weak to Shurikens, and a sunk fleet's wrecks lie in the winner's water,
so the first sea battle is worth more than a first battle on land (L-gecko-air-details; not worked out here, nor the
torpedo aircraft).

**How it fails** (for the episode index): *not built* (the store full, no air plant past 15:00: the Glitters
lobby game, 2026-10-25); *not seen* (his air plant unscouted until his wing came); *not answered* (his bombers or
gunships over our fields and commanders with ground anti-air alone, built after they came); *packed* (a converter,
construction turret or turbine block lost to one pass's chain); *Banshees on units* (sent at an army, dead to his
fighters before they kill enough); *stun alone* (Shurikens over an army with nothing of ours on the ground to shoot it);
*tier-1 against tier 2* (Thistles and Crossbows as the answer to Wasps or a Liche).

**The test.** Kill-test: the pool's sides that reached 30 metal a second and played five minutes more, with an air plant
against without, at matched income: with one they won 12 of 20, without 12 of 24, a lean that the income confounds
(the side with the plant is usually the richer); the six bombing games went to the bomber six times, once on the lower
income (67 against 86 at 10:00). Not done: the matched-income cut, and the same cut on the team pool (the Glitters
8v8 cards). Status: the heuristic's first half (the plant around 30, most players) holds as "from 30, at about twice
it, in the long games"; fighters per metal holds; the single bomber that destroys a field holds for converters,
construction turret blocks and packed turbines, and not for solars, which it darkens instead. thebluegecko's
additions of 2026-10-27 (the Shuriken, the Banshee's window, tier-2 air, transports, the sea) are his account with our
arithmetic from the unit files and the engine's stun rule; none is measured in our games or drilled, and the
transports cannot be played by our hands yet.
