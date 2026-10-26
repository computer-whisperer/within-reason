# The distilled bot's air (2026-10-25)

**Status.** Built and measured (see "Measured" at the end); baseline dist15-19 (the bot as it stood at bd926e2), smokes
dist20-23 (the changes they brought are marked below), benches dist24-28 (the changes it brought marked "from
dist28"), dist29-33, dist34-35 (`air=off` against `air=on` over sixteen games against BARb, which found a bug of
D-AIR-POWER's) and dist36-40 on the rules as committed.

## Why

The user's eight-seat game on All That Glitters Extended (2026-10-25): his air came at 27:00, 23 Wasps and 5 Dragons
at least, and killed 771 of our units after 28:00. The distilled seats answered with at most four Nettles by their
plants, and one seat's commander held its anti-air order for 747 s while the one-builder rule kept every other builder
off anti-air.

The user, the same day: "by 30 metal/s an air lab is considered obligatory. Once up, fighters are apparently the
cheapest anti-air by far, bombing the enemy economy can be very strong in moderation, and other possibilities open
up", and "bombers on the enemy economy (solar or wind fields) are extremely strong due to how standard base layouts
let them chain-explode on death". The distilled bot is mine to design, the territory bot the user's to oversee
(the user's separation, 2026-10-25); these are the game knowledge it is built on (`knowledge/army.md`
K-army-air-plant-by-30-income, K-army-fighters-cheapest-anti-air, K-army-bombers-chain-energy-fields).

## What is true today

- D-ANTI-AIR: once any armed aircraft of his has been seen, Nettles by the plants, one more than the plants, at most
  four, one builder at a time. Not sized: the ledger of his aircraft (`army.his_air`, metal and frame) is kept and
  read by nothing.
- His aircraft weigh nothing to the army (D-WEIGHT) and are no raid (D-HOLD).
- No air plant in the kit. An armed aircraft of ours would join the guard (`enlist` takes every armed mobile
  non-builder).
- The bot's income reaches 30 a second at 7-11 min against Apex v0.3.2 on Comet and 7-12 against BARb
  hard_aggressive (dist14); his first armed aircraft were seen at 10-14 min against Apex.

## The numbers

From `glossary/units.csv` (the strength is T-AIR's, √(damage a second to aircraft × health) / metal):

| Unit | Metal | Energy | Build time | Health | To aircraft a second | Strength a metal |
|---|---|---|---|---|---|---|
| Falcon / Valiant (fighter) | 73 | 2,900 / 2,800 | 3,500 / 3,330 | 167 / 156 | 139 | 2.09 / 2.02 |
| Nettle / Thistle (turret) | 80 | 900 / 850 | 1,800 / 1,750 | 330 / 335 | 68 | 1.87 / 1.88 |
| Stormbringer / Whirlwind (bomber) | 150 | 4,400 / 4,600 | 5,000 / 5,050 | 670 / 680 | - | - |
| Aircraft Plant | 650 / 630 | 1,100 | 5,450 / 5,380 | | | (build power 150) |

- A fighter does 2.2 times a Nettle's damage to aircraft per metal, and 0.65 times per energy (0.048 against 0.075
  a second per energy point). Its edge is metal and reach: a turret guards 765 round itself, the fighters every
  place, and they gather on the strike. Banshees and Wasps cannot hit aircraft at all; a Dragon does 95 a second.
- A plant of 150 build power makes a Falcon every 23 s, and draws 124 energy a second doing it. Fighters are built
  as fast as build power and energy let them, not as fast as metal does.
- Death blasts (radius / damage): solar and wind 90 / 89 (solar 340 health, wind 196-230), converter 105 / 660
  (167), energy storage 210 / 980, Cortex's advanced solar 130 / 220. A Stormbringer drops five 110-damage bombs in
  a line. An Armada wind dies to a bomb and one neighbour's blast (Cortex's and Legion's to two), a solar to a bomb and three;
a converter takes its neighbours with it.

## Engine facts it rests on

From the Recoil source (`rts/Sim/Units/CommandAI/AirCAI.cpp`, `rts/Sim/Weapons/BombDropper.cpp`):

- An aircraft with an empty queue, fire state fire-at-will and move state not hold, attacks on its own the closest
  target within 1,000 × its move state; a fighter (`IsFighterAirUnit`) looks only for aircraft. Hold position
  stops it.
- A fight order makes a fighter take the enemy aircraft nearest its line within 1,000 × its move state; under
  manoeuvre (move state 1) one more than 1,000 off the line is not taken and an attack it took is dropped past 1,500.
- An attack on a unit ends when the target has been out of sight and radar for a while (`UpdateTargetLostTimer`):
  bombers sent at a building out of sight turn back. An attack at a place (`FireAt`, the engine's area attack with
  radius 0) needs no sight and never ends on its own: the bombers make pass after pass until told otherwise.
- Bombs only fall: a target above the bomber is out of range (`CBombDropper::TestRange`). The combat tables credit
  the bombers with damage to aircraft (their weapon's category is `NOTSUB`); it is not real. Noted for the shared
  `combatsim`, not changed here.

## The design

The rules replace D-ANTI-AIR, in `bots/distilled/src/air.rs`.

**D-AIR-SEEN.** His armed aircraft as seen, by type and the frame last seen, for five minutes (`SEEN_FOR`, as his
army), the dead left out: the air shown, in metal and in health; and apart, his fighters (a weapon only against
aircraft) and what they do to aircraft a second.

**D-AIR-PLANT.** From 30 metal a second of income (the user's figure) with two vehicle plants standing, one aircraft
plant, at home, built by the first builder to come free that can build it, one at a time. From dist28: at 30 alone the
bot, which reaches 30 on one vehicle plant at 6-7 min, put the aircraft plant where the second vehicle plant had been
(begun 6.1-7.7 min, the second vehicle plant at a median 11 against 7.6; 41 soldiers made by 10:00 against 50, 1,750
metal banked against 430, the army never launched, four of eight lost to BARb hard_aggressive against none); and
300 behind home, in the open, BARb's raiders killed it five times in eight games.

**D-FIGHTERS.** The air plant makes fighters to a count: four; twice his fighters shown, counted in ours by strength
(√(damage to aircraft × health)); and enough to kill the rest of the air shown within ten seconds (its health over
the fighter's damage to aircraft times ten). His fighters twice over came from dist20 game 01: eight Falcons, sized
to kill seven Whirlwinds and four Valiants in ten seconds, met them at even numbers of fighters and lost five; by
Lanchester's square law twice his number keeps three quarters of ours. They stand at the air station, halfway from home to
the hold point, on manoeuvre, so that they take aircraft that come within 1,000 on their own. His aircraft in sight
on our half, within 1,500 of something of ours and clear of his known anti-air, are fought at: the fighters are sent
on a fight order to the one nearest them. Nothing past the middle, nothing over his anti-air: a Falcon has 167
health.

**D-AIR-POWER.** While fighters are short of what his air shown asks for, a builder whose job would be to help a plant
helps the air plant instead,
and D-BUILD-POWER's construction turrets may go beside the air plant (it counts among the plants for their sites, not
for the step to three turrets a plant once three plants stand, and an idle turret helps it only with no vehicle plant
in reach: counted whole it made the third plant and turrets stacked where the third vehicle plant should have come,
2 plants and 10 turrets at 20:00 against 4 and 6 with `air=off`, dist35 against dist34); the air plant's draw while it works (its power and its helpers' times the fighter's
energy over its build time) is added to D-ENERGY's draw. What answers nothing yet (the four fighters before he has
flown, the bombers) is made only with the energy store over half and is not in the draw (from dist28: the plant on
fighters drew 124 a second, 40 % of the energy income at 8:00, and the store stood at 856 against 1,748). The turrets came from dist22 game 01: at 28:30 his tier-2
air (a Dragon, two Nighthawks, a Wasp, a Valiant) took the fighters wanted from 6 to 30, the energy store sat full,
the commander helped the air plant, and every constructor was building turrets by the vehicle plants; 1-4 fighters
stood.

**D-ANTI-AIR (rebuilt).** Until the fighters are as many as wanted, Nettles by the plants as before, but to a count
sized to the air shown (turrets of three times their metal for what he has shown, as T-AIR weighs them), at least
one and at most eight, two builders at a time. The turrets are the bridge: before the air plant, and while the
fighters are being made.

**D-BOMB.** With the fighters as many as wanted, the air plant makes a squadron of four bombers (Armada's
Stormbringer, Cortex's Whirlwind; Legion has no tier-1 land bomber). They wait at the air station on hold position.
A sortie goes when all four stand within 700 of it (they circle), a minute after the last, at the densest field of his
known economy buildings (generators, converters and energy storage; his buildings within 200 of a point, by metal
plus energy over 70, at least three of them) where his known anti-air would do the squadron no more than one and a
half bombers' health (each turret's damage a second for the time the straight line there and back runs within its
reach, and for four seconds over the target if it covers it: one Thistle beside the field, about 690, is taken, two
are not), and while his fighters shown could not kill the squadron within ten seconds. The first form wanted the
target and the line clear of his anti-air: Apex stands a turret by his generators early, and in dist21 game 00 no
field was ever clear in 30 minutes. The bombers are
sent with an attack at the place and called back to the station once each has been within 500 of it for four
seconds, or the sortie has lasted its flight time and twenty seconds. A sortie that loses half the squadron stops
the bombing, and the making of bombers, for five minutes; after two sorties the squadron is made up again only while
the economy buildings its sorties were seen to destroy (metal plus energy over 70) are worth the bombers lost (from
dist28: 58 bombers made in eight games against BARb, 33 lost for 43 buildings, four games losing more than they
took). The moderation is the squadron's size, the minute between sorties, the pause and the account.

**The death instrument.** A shot-down aircraft dies in its crash (the engine's damage type 8), which names no killer:
every one of ours read `killer=other(8)` in dist20-22. The last weapon that hit each of our aircraft is kept, and a
crash is put down to it.

**Not in it.** The advanced air plant and tier-2 air (Highwind, the flak turret), anti-air soldiers in the army
(Whistler, Lasher), air scouts, constructor aircraft, escorts for the bombers, bombing anything but the energy fields.

## The bench

`run/distilled_bench.sh FIRST TAG <arena args>`: Comet Catcher, All That Glitters and Quicksilver, eight games each
(sides and factions alternating by seed), 30:00, played out. Against Apex v0.3.3 `--profile standard
--opponent-option apex_plan_force=5` (its "Mass air" plan), with Apex's exchange record put back to one saved copy
(165 lines, 2026-10-25) before every batch, so neither arm meets an opponent that learned from the other. Then Comet
against Apex v0.3.3 with no plan forced and against BARb hard_aggressive, eight games each, both arms, for what the air
plant costs when he flies little or not at all.

Read: wins, losses and timeouts; our metal lost to his aircraft (the death instrument's killer); his air built (the
truth files); when the air plant stood, fighters made and lost, sorties flown and his economy buildings destroyed by
them (the `air:` and `sortie` lines).

## Measured

The air bench (24 games against Apex v0.3.3 on mass air, the record put back before each batch), and the regression
rows on Comet. Each later row carries the changes the one before it found; dist36-40 are the rules as committed.

| Rows | Mass air, won-lost-cap | Our metal lost to aircraft over his air built | Apex, no plan | BARb hard_aggressive |
|---|---|---|---|---|
| Baseline (dist15-19) | 12-5-7 | 2.38 (134,170 / 56,355) | 6-2-0 | 3-0-5 (8 games) |
| First form (dist24-28) | 7-1-16 | 0.74 (104,388 / 140,639) | 2-2-4 | 1-4-3 |
| dist28's changes (dist29-33) | 12-1-11 | 0.73 (57,543 / 78,306) | 6-1-1 | 2-3-3 |
| `air=off` / `air=on` (dist34-35) | | | | 10-6-0 / 6-6-4 (16 games each) |
| The turret fix (dist36-40) | 9-3-12 | 0.70 (85,177 / 122,554) | 6-0-2 | 7-3-6 (16 games; `air=off` 10-6-0) |

- What we lose to his aircraft per metal of his air fell to under a third in every run of the rules. How much he
  flies moves from batch to batch (his air built 56,000-141,000 over the same 24 seeds; games that last longer see
  more of it), so the totals alone mislead; the ratio does not move with it.
- Against mass air the losses fell (5 of 24 to 1-3); fewer of the games are won outright before the cap.
- Against BARb, which does not fly, the eight-game rows were noise (the same seed diverges in the first minute):
  sixteen games with `air=off` lost six. With the turret fix the rules lose three and win three fewer: even within
  that noise. The first form's cost was real and had two causes, both fixed: the aircraft plant in the second
  vehicle plant's place, and the aircraft plant counted as a third plant by D-BUILD-POWER.
- The bombing pays where his anti-air is thin: against Apex with no plan forced 8 bombers lost for 73 of his
  economy buildings (18,898, eleven to one); against Apex on mass air 25 for 27 on Glitters (where he had fighters
  and anti-air everywhere) and 2 for 32 on Comet; against BARb 32 for 57 (1.4 to one). The bot sees about two
  thirds of what its sorties destroy; the truth file's count is in `run/distilled_air.py`.
- The fighters cannot keep up with his tier-2 air at the scale of the user's game (dist22-23: a Dragon and
  Nighthawks took the wanted count to 26-35 with one aircraft plant making them); the flak turret and the advanced
  aircraft plant are what is left out.
