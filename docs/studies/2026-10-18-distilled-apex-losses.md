# Where the distilled bot's losses to Apex v0.2.9 come from, against the Opus player's game (2026-10-18)

The user's ask (2026-10-18): "Take a look at the recent losses and see where its behavior deviates from the main
bot." Read: the two batteries against Apex v0.2.9 on Comet Catcher (`apex-v029-distilled-comet-1`, 3-5 at +0;
`apex-v029-distilled-comet-sym50-1`, 1-7 at +50; sixteen games, seeds 1-8, sides alternating) against the Opus
player's game on the same map against the same opponent at +0 (`apex-v029-comet-sym0-opus-1`, WON 16:35, seed 3,
Armada mirror), and one five-minute diagnostic game of the bot on the +0 battery's seed 2 with the commander's
engine state logged each second (`distilled-commander-diag`). Instruments: `run/bot_sides.py` (both sides by minute
from the census lines; new), `run/bot_orders.py` (every build order by game clock from the host's build trace; new),
`run/bot_curves.py`, `run/compare_census.py` and `run/analyze_match.py` for the player's game. Nothing was changed
in the bot; what to build is the user's call (§7).

## 1. The two bots at 6:00 and 10:00, same opponent, same map, +0

Ours / his. "Power" is plants plus construction turrets. The player's game first, then the eight distilled games.

| game | result | 6:00 extractors | 6:00 power | 6:00 soldiers | 10:00 extractors | 10:00 power | 10:00 soldiers |
|---|---|---|---|---|---|---|---|
| the Opus player, seed 3 | WON 16:35 | 14 / 10 | 2+1 / 1+0 | 11 / 10 | 24 / 9 | 3+3 / 2+4 | 43 / 13 |
| distilled 00, seed 1 | won 27:30 | 9 / 9 | 1+0 / 1+0 | 10 / 8 | 15 / 10 | 1+0 / 1+0 | 9 / 1 |
| distilled 01, seed 2 | lost 13:30 | 7 / 15 | 1+0 / 1+2 | 11 / 13 | 7 / 23 | 1+1 / 2+5 | 10 / 30 |
| distilled 02, seed 3 | won 14:48 | 10 / 6 | 1+0 / 1+0 | 12 / 6 | 20 / 6 | 1+0 / 0 | 17 / 2 |
| distilled 03, seed 4 | lost 19:36 | 7 / 10 | 1+0 / 1+1 | 11 / 7 | 16 / 18 | 1+0 / 2+2 | 8 / 22 |
| distilled 04, seed 5 | won 38:22 | 12 / 7 | 1+0 / 1+0 | 14 / 7 | 19 / 10 | 1+0 / 2+0 | 22 / 10 |
| distilled 05, seed 6 | lost 16:36 | 10 / 11 | 1+0 / 1+2 | 11 / 8 | 21 / 20 | 1+1 / 2+3 | 10 / 32 |
| distilled 06, seed 7 | lost 24:54 | 10 / 7 | 1+0 / 1+0 | 9 / 8 | 18 / 14 | 1+0 / 2+3 | 18 / 19 |
| distilled 07, seed 8 | lost 29:54 | 10 / 11 | 1+0 / 1+0 | 13 / 7 | 18 / 21 | 1+0 / 2+1 | 25 / 23 |

(The 10:00 power column leaves out the advanced plant's frame, begun at 6:00-9:30 in seven of the eight; it made
nothing before 10:00.) The player had a second plant at 5:50, its first construction turret at 5:32 and a third
plant at 8:45, and made 43 soldiers by 10:00 against his 13. The distilled bot had one plant and no construction
turret at 10:00 in every game, 8-25 soldiers, and in the five losses stood against 19-32 of his. His extractors
tell the same story from the other side: 9 against the player at 10:00, 14-23 in the distilled losses.

## 2. Finding 1: the commander is told to build a construction turret, which it cannot build, and stands at the plant for the rest of the game

Neither commander has the construction turret among its build options (`armcom`, `corcom`: 26 options each,
`armnanotc`/`cornanotc` not among them; the constructors have it). The bot's build-power ladder (`economy.rs`
`build_power_job`) does not look at the builder's options: the first job it gives when power is short is
`Turret(plant)`, to whichever builder is free, and at +0 that is the commander in every game.

`run/bot_orders.py apex-v029-distilled-comet-1`: the first construction turret was ordered at 2:00-2:33 in all eight
games, by the commander in all eight. After it the commander issued no further order in four games and one to four
orders after 19:06 in the other four. Its position from the census, every minute from 3:00 on, is one spot within
ten elmos, 130-143 from the plant's centre (the commander's build distance is 145): game 01 at (6706,818) against
the plant at (6576,816) from 3:00 to the end at 13:30; game 02 at (1474,5316) from 3:00 to 14:00 and beyond; the
same in every game.

The diagnostic game shows the mechanism second by second (`run/matches/1792299289-distilled-commander-diag`, the
`[diag]` lines in `00/engine.log`): at 2:19 the commander's job became `Assist` (guard the plant, low priority) and
it walked to the plant; at 2:39, arrived, the ten-second review found power short and gave it `Turret(9194)`; the
host found a site at (1480,5352) and the engine accepted the build command without an error (no `CommandRejected`
event, so the order stayed recorded); the commander never moved again (velocity 0 at (1498,5365) from 2:44 to the
cap at 5:00), never went idle (`idle=false` every second), and no turret frame appeared (`turrets=0`). The bot
re-asks a builder with a build order only when it is idle, so the phantom order is never replaced. The engine
keeps the guard order the commander already had and holds the impossible build; what the engine does with a build
order for a type the builder lacks was not read in its source, only observed.

Two more costs follow from the same order. The commander's priority flag follows its recorded job (D-PRIORITY: low
while assisting): the phantom `Turret` job flips it to high, so from 2:44 it guards the plant at full draw, 300
build power on the plant's soldiers. The diagnostic's energy store went from 1,328 at 2:39 to 0 at 3:04 and sat
there; in the battery the store at 3:00 read 39, 8, 3, 30, 0, 1, 119 and 1 in the eight games. That stall makes
`energy_short` true, which sends two or three constructors to solars instead of extractors (the ladder's first
rung) in minutes 3-6.

At +50 the same thing happens later. The first turret there is ordered at 2:00 by a constructor (the piling-store
clause fires at 1:30 while the commander is still on the plant), and the commander takes the second plant at
2:00-2:24, which it can build; it is given a turret job at 5:00-6:00 in games 01, 02, 03, 04 and 07 and stands
still from then (5:00-14:00 in game 02, 7:00-12:00 in 03, 5:00-9:00 in 07), and in games 00, 05 and 06, where it
was idle when the job came, the engine dropped the order at once and the bot re-gave it: 52, 182 and 65 turret
orders to the commander in a row (10:03-10:09, seven within one half-second window).

## 3. Finding 2: the phantom order holds the bot's one build-power slot

`build_power_wanted` allows one build-power job at a time (`parallel` = 1, one more for every 1,000 metal banked,
every builder when the store is over 90%), counting every recorded `Plant`, `Turret` and `AdvancedPlant` order. The
commander's dead `Turret` order is counted. So in every +0 game no plant and no turret is ordered by anyone until
the bank reaches 1,000, and the second plant comes when the store fills, not when power is short:

| game | first turret that stood | second plant ordered | metal banked then |
|---|---|---|---|
| 00 (won) | 11:00 | 20:00 | 5,623 at 20:00 |
| 01 (lost 13:30) | 10:00 (ordered 9:38 by a constructor) | never | 1,099 at 10:00, full (1,697) at 11:00 |
| 02 (won) | 13:00 | never | 938-1,043 at 10:00-12:00 |
| 03 (lost 19:36) | never | never | 0-689 through 15:00 |
| 04 (won) | 14:00 | 14:37 | 4,818 at 15:00 |
| 05 (lost 16:36) | 10:00 | 10:00 | 1,840 of 2,050 at 10:00 |
| 06 (lost 24:54) | 17:00 | 17:11 | 1,995 at 17:00 |
| 07 (lost 29:54) | 13:00 | 19:49 | 2,494 at 20:00 |

The advanced plant's `AdvancedPlant` order (a constructor's, from 6:00-9:30 until the plant stands two or three
minutes later) occupies the same slot, so the lock holds through the transition too.

## 4. Finding 3: the tier-2 transition at income 22 pauses the only plant while his army triples

D-T2-GATE fires at income 22 from 6:00; at +0 the gate fired at 6:00-9:30 in seven of eight games (game 05 at
10:00). D-T2-PAUSE then stops the tier-1 plants until two advanced constructors are made. In game 01 the plant was
paused from 6:36 to 12:00 (`paused=1` on the 7:00-11:00 lines; `made` flat at 23 from 6:00 to 12:00); the home
guard stood at ten Blitzes, 1,100 metal, under the late launch weight of 1,200, and never went out; his soldiers
went from 13 at 6:00 to 55 at 12:00 and the game ended at 13:30. Game 07 paused 10:00-12:00 (`made` 41 flat),
game 03 9:00-10:00, game 06 at 10:00, game 05 at 12:00. The player's game began its advanced plant at 15:49, after
the field was won. `bots/distilled/README.md` already records that against hard_aggressive "the transition does
not pay at either gate" (1 of 24 games); the default arm is still `t2=22`.

## 5. Finding 4: the body goes out at 4:00 with five raiders, comes back, and stays

D-LAUNCH sends the home guard out at 550 metal of soldiers before 8:00: five Blitzes. `strike=5` at 4:00 or 5:00 in
seven of eight +0 games, into his 7-13 soldiers; D-STEP-BACK brings it home, and the guard then sits at the rally
point until 9:00-13:00 (`home_guard` 9-18, `strike` 0 on the 6:00-9:00 lines of games 01, 05, 06, 07). Meanwhile
his expansion goes unpunished: 15-23 extractors at 8:00-10:00 in the five losses. The player's Rovers and Blitzes
were on his extractors from 7:22 (engagement #1 of its report, a mex, a sentry and a solar at G3) and the body went
as one at 10:25 with 43 soldiers against 13; he never had more than 13 extractors. The three distilled wins are the
games where his army stayed at 1-10 soldiers at 10:00 and his extractors at 6-10: the games Apex played small.

## 6. The +50 battery

The same bot at +50 both sides (1-7): two plants and a turret stand by 3:00 in every game (the piling clause), the
advanced plant goes up at 6:00 in all eight (standing 7.7-9.0), the commander is lost to the turret job at 5:00-6:00
or to the order loop at 10:00, and his soldiers at 10:00 are 14-39 against our 6-35 with 1,272-2,614 metal banked in
four of the losses. The late bank the README names (4,658 at 15:00 at the median) is the same slot lock: with the
commander's phantom order and the project's order in it, nothing adds power until the store passes 1,000, and at
+50 income it passes 1,000 and refills faster than one plant at a time can spend it.

## 7. The deviations, side by side (not built; the user's call)

| | the distilled bot | the Opus player (same opponent, +0) |
|---|---|---|
| build power | one plant, no turret, until 1,000 metal is banked (9:38-20:00); the commander's impossible turret order holds the one slot | second plant 5:50, turret 5:32, third plant 8:45; 3+3 at 10:00 |
| the commander | at the plant from 2:30, high priority, never ordered again | builds the second plant, expands, helps |
| energy at 3:00 | store at 0-39 in seven of eight games (the commander's full-draw assist) | stalls in minutes 2, 6-7 and 12-13, each short |
| tier 2 | at income 22, 6:00-9:30; the only plant paused one to five minutes | 15:49, after the field |
| the body | five raiders at 4:00, home by 6:00, out again at 9:00-13:00 | pressure on his extractors from 7:22, one push at 10:25 with 43 against 13 |
| his extractors at 10:00 | 14-23 in the losses | 9 |

The smallest change the evidence points at is one condition: give the turret job only to a builder whose options
include the turret (the commander then takes the plant, which it can build, as it does at +50), with a guard
against any order a unit holds without executing (not idle, no frame begun, no movement for some seconds: drop it
and refuse the pair). That alone changes every +0 game from 2:30 on. The gate figure, the pause and the launch
weights are separate rows to run after it, since the batteries so far were played with no build power at all.

## 8. Built the same day, on the user's word ("apply the fixes you see fit ... take inspiration from the successful flagship matches"), and what the batteries then said

Every row is in `docs/experiments.md` (apex-v029-distilled-comet-2 to -10); the rules in `bots/distilled/README.md`.

**Built.** D-BUILD-POWER's rung goes only to a builder whose options include it (§2's commander), with a guard behind
it for any job; D-ORDERS: an order the engine drops at once counts as refused, a builder standing two minutes on a
build order with no frame beside it drops it, four failures in a row sideline a builder for a minute, an energy order
moves round the back field and a turret site round the plant; the advanced constructors' energy rung offers only what
they can build (neither has the advanced solar: §2's 38-order loop); D-BUILD-POWER's short test net of what the
builders draw; D-FIELD and the `gate=field` default (tier 2 only with a pushing body out, the player's 15:49); D-RAID
(the body out at the base weight, raiding his known extractors at two-to-one odds while lighter than what it has seen
of his army, pushing when heavier); D-LANE (the shared `crates/micro` lane under the army's orders, `lane=off` to
test without). Instruments: `run/bot_exchange.py` beside `run/bot_sides.py` and `run/bot_orders.py`.

**Two things the batteries taught about measuring.** A twenty-second held-order watchdog fired 27-65 times a game on
constructors standing in range of their spots: the engine starts no frame while it first reclaims a wreck on the site
or waits for a blocking unit to move off, and drops a blocked build itself after thirty tries (`CBuilder::StartBuild`,
`CBuilderCAI::ExecuteBuildCmd`); frames begin 4-18 s after an order in the usual case (distilled-held-diag-2), so the
net is two minutes. And Apex carries its exchange record between games (the arena's `run/data/opponent-state`): §1's
3-5 row started from 2 pairs, every row after it from 35-145, so the day's rows were put back to the 2-pair copy. From
that state the old bot went 0-8 on the seeds of its 3-5 (comet-6): the three wins of §1 were games Apex collapsed on
his own (1, 2 and 10 soldiers at 10:00). An eight-game row against v0.2.9 scatters by three wins for an unchanged bot.

**What moved and what did not.** Over sixteen cold games on seeds 1-16, the tree less the lane went 3-12-1 against
the old bot's 0-16 (comet-8 against comet-6 and -9). Underneath: the first turret at 3:05-5:00 by a constructor and
the second plant at 7:00-9:30 (the pros' 5:56 and 8:11), extractors 20-24 at 10:00 (§1: 15-21), the advanced plant at
7:33-9:30 where the body had come to outweigh what it had seen, his extractors at 10:00 a little lower. What did not
move is the exchange: by 10:00 the bot lost 10-25 soldiers for 6-24 of his in every battery of the day, under the old
rules, the raid at even odds and the raid at two to one alike (`run/bot_exchange.py`), where the player's record shows
7 lost (454 metal) for 9 of his by 10:00, then the push at 10:25 with 43 against 13 that traded badly in metal and won
the game. That is why the lane was wired in last (D-LANE): the flagship's hands fight under it, and against BARb it
turned 0-12 into 3-7-2 (micro-ab2). Its sixteen-game row (comet-10) went 1-14-1 with the exchange by 10:00 the same (ours 12-24 dead for his 5-20), the games longer (20-40 minutes against 15-26): as wired, the lane keeps soldiers alive a little and wins nothing with them, so it is off by default (`lane=on` for a row). What the bot still lacks against the player's game is not footwork but the player's restraint before the push: 7 soldiers lost by 10:00 against the bot's 10-25 under every rule set tried, and that is the next thing to design, with the lane's debug lines read first.
