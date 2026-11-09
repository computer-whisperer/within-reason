# Shields and the other standing units: the survey (2026-11-08)

The user's ask that opened this: expand the defence section with more units, shields in particular; "they have been
used against us on occasion, and now we should learn to use them". The defence survey of 2026-11-06
(`2026-11-06-static-defence-survey.md`) counted towers by metal and yield and left three things unread: what a
shield does for the side that owns it, the standing units that are not guns (jammers, walls, the pinpointer), and the
public team games. This reads those. No code decides anything new here; what went into the brief is listed at the
end, and what would need the hands is listed as proposals.

The script: `run/replays/statics_team.py` (the team pool, cached in `run/data/statics-team.jsonl`; `--ours` for our
own seats' losses). Every number in sections 2 to 4 is its output; rerun it before quoting a number elsewhere.

## 1. What a shield is

Three generators on land, one a faction, built by every tier-2 constructor (bot, vehicle, aircraft): Keeper `armgate`
3,000 metal and 54,000 energy, Overseer `corgate` and Soteria `legdeflector` 3,200 and 55,000; 55,000 of build time
(a Pulsar's 52,000), 3,550 health, a 4 by 4 footprint, 273 of sight. The floating Keeper and Overseer (`armfgate`,
`corfgate`: 3,900 metal, radius 600, 9,500 strength) are the tier-2 naval constructors'. `leggatet3` (14,500 metal,
49,400 strength, stops everything) is in no build list of the stock game; the 18 in the pool came from lobby options.

From the unit files, the game's weapon post-processing and its shield gadget, and the engine
(`units/*/armgate.lua`, `gamedata/alldefs_post.lua` 1098-1177, `luarules/gadgets/unit_shield_behaviour.lua`,
`rts/Sim/Weapons/PlasmaRepulser.cpp`):

- *The bubble.* Radius 550, strength 6,175. It is `exterior` (it takes only shots that come from outside the
  bubble) and `smart` (only the other side's shots): everything of ours inside fires out freely, and anything of his
  that stands inside fires freely at what is inside.
- *What it stops.* Its intercept type is 1 and the engine gives each weapon type its own bit: Cannon 1, laser
  cannon and beam 2, missile 4, bomb 8, flame 16; the game sets every vertical launcher to 1024 and the D-gun to
  512. So under stock lobby options a shield stops cannon shells and nothing else. Cannon is wider than "plasma
  artillery": the Pawn, Blitz, Mace, Stout, Brute, Pounder, Thug, Bull, Tzar, Fatboy, Sheldon, Hound, Mauser, Quaker,
  Shellshocker, Wolverine, Tremor, the Gauntlet, Agitator, Rattlesnake, Persecutor, Eviscerator, the Pit Bull, the
  Banshee gunship, the long-range cannon. What passes: every beam (the Sentry to the Pulsar, the Bulwark and the
  Bastion, the Incisor, the Starlight), every rocket and missile (Rocketeer, Aggravator, Arbiter, Thanatos,
  Ambassador, Catapult, the Scorpion pop-up), lightning, flame, bombs, the nuke, the D-gun.
- *A blast outside does not reach inside.* A unit under a living shield takes no damage from a stopped weapon's
  shot fired by an attacker outside that shield, its area of effect included; an unblocked weapon's blast reaches
  in. Death explosions are nobody's shot and pass: 33 of the pool's dead shields were killed by an advanced fusion
  blowing up beside them.
- *The strength and the bill.* It starts at 2,090 when it finishes and regenerates 130 a second for 562.5 energy a
  second, only while below full and only while the store pays; full from built takes 31 s, full from empty 47.5 s
  and 26,700 energy. It costs nothing standing full and nothing per shot stopped. With the store empty it does not
  regenerate and still holds what it has.
- *The long-range cannon is halved.* The Basilica, Basilisk and Ragnarok carry their own damage to shields: 812,
  1,000 and 812 a shell where a building takes 1,625, 2,000 and 1,625; the Olympus and Astraeus 300 for 600. One
  Basilica or Basilisk is 62 a second on a shield: a single generator's 130 a second outlasts two of them for ever
  at half a fusion's energy. A Ragnarok or Calamity (2,700 a second) empties one in five seconds.
- *What empties it.* The regeneration equals the fire of 1.1 Fatboys, 1.5 Mausers or Quakers, 3 Shellshockers or
  Wolverines, a third of a Tremor, 0.7 of a Rattlesnake. Ten Mausers empty a full one in 8.5 s, three Tremors in
  5.8 s, ten Bulls in 2.7 s. At zero it is off for at least a second and at most twenty, and the overkill is
  regenerated before it returns. It is off while it is being built and while it is stunned (the Paralyzer's missile,
  35 s).
- *Overlap is depth.* A shell is taken by the first living bubble it meets, so three generators within each
  other's radius are 18,500 of strength and 390 a second, spent one after another.
- *The lobby option.* `experimentalshields` is a hidden list option, default `unchanged`. `absorbeverything` and
  `bounceeverything` set every weapon's bit to 1 (the bounce modes at 0.53 of the strength and 0.4 of the
  regeneration). Every lobby game of ours ran `unchanged` (the five start scripts in `run/matches/*-human`), and so
  did the 25 public demos read; the 8v8 pilot of 2026-11-06 ran `bounceeverything` and its 59 shields are not
  evidence for a stock game.

## 2. Who builds one, and when

| pool | sides | with a shield |
|---|---|---|
| high-OS duels (`run/data/bar-replays`, 527 games) | 1,054 | 0 |
| public team games of 15 or 16 seats, people's seats (169 games, 166 on All That Glitters, 87 against AI) | 2,037 | 18 % |
| the same, games of 30 minutes and more | 861 | 38 % |
| the same, games of 40 minutes and more | 412 | 53 % |
| the same pool's AI seats (SMRTBARb and BARb) | 668 | 11 % |

The duel has none: the game is over before the fourth fusion. In the team game it is a late tier-2 purchase at every
level: 31 % of seats under OS 10 and 44 % above 30 in games of thirty minutes, the first at 24:51, 29:14 and 34:58
(quartiles), one, two and five a seat (quartiles; the most 27). A seat's first shield finds it with four fusions, one
tier-2 tower and, in 53 %, an anti-nuke; by its own faction the Keeper is in 37 % of Armada seats, the Overseer 32 %,
the Soteria 33 %, twice each, at 30:00 to 32:00. Winners and losers build them alike (people against people 32 %
and 36 % of seats; against AI 46 % and 44 %): the shield does not decide games, it is part of what a long game's
seat owns.

## 3. What stands under one, and where

Of the people's 1,327 shields, by the kind holding the most metal among the side's structures within 550 five
minutes after the shield stood: energy 37 %, tier-2 towers 32 %, construction turrets 7 %, factories 7 %, long-range
cannon 5 %, the rest under 3 % each. The towers are Pulsars before anything (1,478 under shields; then 355 Pit
Bulls, 290 Bulwarks, 227 Scorpions, 212 Rattlesnakes, 164 Persecutors, 155 Basilicas, 124 Bastions). A shield covers
17,000, 33,000 and 69,000 of structures (quartiles): eleven times its own price at the median. 62 % stand within
550 of another shield. They stand 610, 1,570 and 2,600 from the seat's start (quartiles), 9, 23 and 38 % of the way
to the nearest enemy start: a quarter over the fusion block at home, the rest over the tower line.

43 % died, after a median 4.4 minutes; half of those name no killer (missiles, blasts, shots from out of sight), then
Titans, Ambassadors, advanced fusions' explosions, nukes, Razorbacks, Catapults, Basilicas, Juggernauts: what passes
a shield or carries more than it holds.

The towers under one live longer. On the sides that had shields, tier-2 towers under a living shield: 1,285 built,
28 % died; not under one: 1,346 built, 65 % died. Counting only towers finished after the side's first shield (the
same minutes of the game): 27 % of 894 against 59 % of 659. The killers shift the way the mechanism says and less
than the survival does: cannon-only killers are 7 to 8 % of the deaths under a shield and 9 to 12 % outside. The
survival gap is therefore mostly where shields are put (the part of the line the seat is keeping) and only partly
the shells stopped; the pool cannot separate the two, and nothing in a record counts the shots a shield absorbed.
Fusion reactors show no difference (28 % and 27 % after the first shield): they die to explosions and unnamed
strikes, which pass.

**Against us.** Three records hold shields on the other side. The lobby 5v5 of 2026-10-18 (stock options): eleven,
seven of them SSFA's Soterias from 36:50 to 56:37 over the north-east Bastion clusters, 98,000 to 164,000 of
structures under each, two to three overlapping; our side lost 224,000 of units within 800 of them, 81,000 of that
Titans whose cannon the bubble took while their beams passed, and the line fell to silos at 56:40. The lobby 8v7 of
2026-11-07: one Overseer at 19:23 by an OS-25 person over two Wardens, a Persecutor, a Scorpion and seventeen
soldiers; nothing of ours came near it. The player's turns in the 5v5 name shields only as things killed; the report
line `shields of his` has said since 2026-10-18 which of our types' shots each stops.

## 4. What a shield would have stopped of our own losses

Our own seats' structures in our team games (30 games: the Glitters 8v8 series, the defence 8v8s, the lobby games;
1,149,000 of structure metal lost), by the killer's guns and reach:

| the killer | share |
|---|---|
| cannon only, reach 550 or less (Stouts, Thugs, Brutes, Pounders, Blitzes: they walk inside the bubble) | 23 % |
| no cannon, reach 550 or less | 19 % |
| no killer named | 14 % |
| no cannon, reach over 550 (Thanatos, Banishers, Catapults, Jaguars, Whirlwinds) | 13 % |
| a death explosion | 11 % |
| cannon and other, reach over 550 (Sol Invictus, Shiva, Sharpshooter) | 10 % |
| cannon only, reach over 550 (Tzars, Sheldons, Wolverines) | 5 % |
| long-range cannon (one Ragnarok, 34,600) | 3 % |

A shield stops the 5 % outright, a part of the 10 %, and none of the Ragnarok. Against what we have met so far a
shield over the hold would have saved about a twentieth of what our seats lost. The bet it makes is his artillery,
and BARb's and Apex's trickle is not artillery.

## 5. The other standing units people build and we do not

Share of the faction's seats in games of thirty minutes (861 seats), median count, median first clock:

| unit | share | count | first | note |
|---|---|---|---|---|
| tier-1 jammer: Castro 100 m, Sneaky Pete 240, Nyx 140 | 49, 44, 44 % | 2, 1, 2 | 9:09 to 10:10 | not in the brief's derived layer at all; not on the hands' usual menu |
| tier-2 jammer: Veil, Shroud, Erebus | 26, 25, 37 % | 1 to 2 | 23:30 to 25:00 | on the usual menu |
| Dragon's Teeth | 33, 32, 31 % | 15, 15, 11 | 5:18 to 7:28 | the hands' menu has no wall |
| anti-nuke | 50, 40, 42 % | 1 | 22:00 to 25:00 | the brief says "by 25:00 against people" |
| heavy anti-air: SAM, Chainsaw, Eradicator, Mercury, Screamer, Xyston | 12 to 22 % | 1 to 2 | 15:00 to 31:00 | the air article's |
| Pinpointer (halves radar wobble for the side) | 12 % | 3 | 24:22 | on the usual menu, unsaid in the brief |
| Beholder camera | 13 % | 1 | 7:28 | |
| Catalyst missile launcher, Paralyzer | 5, 3 % (of all seats) | 2 | 28:00 to 29:45 | the Paralyzer's stun switches a shield off |
| Basilica | 4 % (of all seats) | 2 | 33:49 | |

The jammer is the largest gap. The defence article's own mechanism ("the tower fires on radar, and the big ones bring
their own") has a counter that half the people buy at nine minutes for 100 to 240 metal: a jammer hides everything
of its side within 360 to 500 from his radar, so his Bastions, Pulsars and artillery need sight of what stands
under it (K-scout-jammers-hide-from-radar-only). Dragon's Teeth are a third of seats at fifteen blocks from the
fifth minute; where they stand is still unread.

## 6. What this says

1. A shield is the cover for what cannot move and is out-reached by cannon: the tier-2 tower line and the fusion
   block of a long team game, bought after the fourth fusion and beside the anti-nuke, two overlapping where the
   ground under them is worth thirty thousand. It is no part of a duel.
2. It answers artillery, plasma towers, long-range cannon singly, and the cannon of anything firing from outside.
   It does nothing against rockets, beams, bombs, nukes, units that walk in, or a Ragnarok.
3. Against what our seats have lost to so far it is worth about a twentieth; against people past thirty minutes it
   is what four seats in ten own. The trigger is his cannon artillery seen (Wolverines, Sheldons, Mausers, Tremors,
   Rattlesnakes, a Basilica), not the clock.
4. Against his shield nothing changes from the article's item 8, with one correction to our own habit: a Pulsar,
   Bastion or Bulwark of ours fires through his bubble.
5. The jammer over the hold belongs in the article beside the radar tower.

## 7. What was written, and what is proposed

Written (2026-11-08): the fact layer's shield paragraph (`docs/briefs/facts/fights.md` F-fight-big-weapons), the
defence article (`docs/briefs/derived/defence.md`: the shield and the jammer as mechanisms, items 10 and 11, the wall
item's census, two ways it fails), the claims in `docs/knowledge/defence.md`.

Proposed, not built:

- *The hands' usual menu* (`roster.rs` `USUAL_*`) has no shield and no tier-1 jammer. The player can queue either
  by its internal name with a place today (`armgate <place>`: a shield carries a weapon definition, so the hands
  place it where it is asked); the usual menu is what Jev is offered when the player whitelists nothing. Unjudged
  in a game.
- *A line for our own shields.* The report says his shields and nothing of ours. The AI interface gives a shield's
  definition and not its present strength; the game publishes on and off as a unit rules parameter. Whether the
  shim can read that is unread.
- *An instrument for shots absorbed.* The replay dump widget could write each shield's strength with the state
  rows (`Spring.GetUnitShieldState`); the drops are what it stopped. Without it a shield's yield is not in any
  record.
