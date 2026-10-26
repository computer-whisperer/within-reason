<!--meta
id: D-air-not-optional
layer: derived
premises: [D-metal-is-the-game, D-energy-second, D-surprise-weapons, D-reach-nests, F-eco-energy, F-eco-build, F-fight-damage, F-fight-death, F-fight-sight, F-map-metal, F-unit-armap, F-unit-corap, F-unit-armfig, F-unit-corveng, F-unit-armthund, F-unit-corshad, F-unit-armkam, F-unit-corape, F-unit-corcrwh, F-unit-armca, F-unit-corca, F-unit-armck, F-unit-armrl, F-unit-corrl, F-unit-armjeth, F-unit-armsam, F-unit-armcir, F-unit-armsolar, F-unit-corsolar, F-unit-armwin, F-unit-armmakr, F-unit-armnanotc, M-pool-air, M-pool-energy-and-power, M-gecko-vs-us-comet, M-gecko-team, F-barb-how-it-decides, F-barb-profiles, M-barb-sparring, M-format-8v8]
lessons: [L-sheet-labs, L-other-u6bkep-nests-and-air, L-gecko-bots-vs-vehicles, L-energy-converters, L-tier2-go, L-mex-game-over, L-sheet-surprises]
sources: {people: [thebluegecko, u6bkep, Irishstud14, the user], dates: [2026-10-02, 2026-10-23, 2026-10-25], relay: the user's account of the experienced players on air (2026-10-25 night, "my relay here is imperfect, but the core points should be correct"); thebluegecko's sheet rows on the aircraft plant; u6bkep's chat in human-10; code: [upstream/Beyond-All-Reason/units/ArmAircraft/armthund.lua (the bomb: 5 x 110, area 144, edge 0.55, reload 6, burstrate 0.25), upstream/Beyond-All-Reason/units/ArmBuildings/LandEconomy/armsolar.lua (damagemodifier 0.5 while closed), upstream/Beyond-All-Reason/weapons/Unit_Explosions.lua (metalmaker 660 over 210; nanoboom 530 to construction turrets over 128, edge 0.75; windboom 89 over 180), upstream/RecoilEngine/rts/Game/GameHelper.cpp:131-150 (falloff from the surface), crates/terrain/src/sheet.rs (spots by air alone)]; the pool cut and the passes: /tmp scratch scripts air_cut3.py and bomb_pass.py of 2026-10-25 night over the 1v1 cards re-cut from the archived records; the Glitters 2v6 record run/matches/1792971120-human/00/record-1.jsonl}
status: in the manifest since 2026-10-26 (the user read it: "This looks good, let's add it"); the bomb pass is worked by hand from the unit files and the engine's falloff, not measured in a drill; the matched-income cut and the team pool's cut not done
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

**Where it bends.** (1) Short games: the pool's duels end at 12:00 at the median, and in the 40 Comet duels an air
plant was built by 22 of 80 sides; the plant belongs to the game that goes past the first army's fights, and an air
plant at 8:00 that delays the second ground plant is the early trap of D-metal-is-the-game. (2) Against BARb, whose
static defence is on a clock with anti-air at 520 s on medium and whose counter table answers air with anti-air
(F-barb-how-it-decides, F-barb-profiles, M-barb-sparring): bombers meet anti-air early and the lesson of the people's games travels less. (3) In
a team one seat's air plant covers the side (M-format-8v8): fighters for every seat's fields from one plant, the seat
that holds the side's energy (D-energy-second) the one that builds it. (4) Gunships are not bombers: Banshees and Wasps
fight armies and commanders rather than fields; the Banshee's own clock is D-surprise-weapons'.
(5) Water maps carry their own air (seaplanes, torpedo aircraft), not derived here.

**How it fails** (for the episode index): *not built* (the store full, no air plant past 15:00: the Glitters
lobby game, 2026-10-25); *not seen* (his air plant unscouted until his wing came); *not answered* (his bombers or
gunships over our fields and commanders with ground anti-air alone, built after they came); *packed* (a converter,
construction turret or turbine block lost to one pass's chain).

**The test.** Kill-test: the pool's sides that reached 30 metal a second and played five minutes more, with an air plant
against without, at matched income: with one they won 12 of 20, without 12 of 24, a lean that the income confounds
(the side with the plant is usually the richer); the six bombing games went to the bomber six times, once on the lower
income (67 against 86 at 10:00). Not done: the matched-income cut, and the same cut on the team pool (the Glitters
8v8 cards). Status: the heuristic's first half (the plant around 30, most players) holds as "from 30, at about twice
it, in the long games"; fighters per metal holds; the single bomber that destroys a field holds for converters,
construction turret blocks and packed turbines, and not for solars, which it darkens instead.
