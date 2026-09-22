# The energy draw: what our production would pull at full speed, in the picture

Written 2026-09-22 evening. Every recent loss and both objective wins had the energy store empty for a large share
of the game (evidence-3-bulldogs 42 %, roster-2 46 %, policy-3 57 %, luna-jev 41 %, opus5-1 43 %): the player queues
solars in threes and sixes after the stall has begun, and a tier-2 plant or an aircraft plant outruns them.

## The numbers

A factory or builder draws `build_power x energy_cost / build_time` energy a second while building at full speed:

| what | draw at full speed | solars (20 each) |
|---|---|---|
| Vehicle Plant (150) making Blitzes (900 / 2000) | 68 | 3.4 |
| Aircraft Plant (150) making Stormbringers (4400 / 5000) | 132 | 6.6 |
| Advanced Vehicle Plant (600) making Bulls (13000 / 23000) | 339 | 17 |
| Construction Turret (200) helping with a Bull | 113 | 5.7 |
| the commander (300) building an Advanced Vehicle Plant (14000 / 27000) | 155 | 7.8 |

The picture says the store, the income, the usage and a stall word; nothing says what the plants would take if fed,
so the player learns the size of the deficit only by watching the stall.

## Decisions

1. **Every factory's entry says its draw**: "draws about 339 energy and 25 metal a second at full speed making a Bull"
   for what is on its pad now, else for its queue's first unit, else for the dearest unit it is allowed to make.
2. **Every builder's entry says its draw** while its task is a build: the same formula with the builder's build
   power and the building's cost.
3. **The economy's energy line adds the budget**: the sum of those draws against the income ("our production at
   full speed would draw 540 a second; income 210: short by 330, which is 17 Solar Collectors (20 each), 4 Advanced
   Solar Collectors (75) or a third of a Fusion Reactor (1000)"), naming only generators the faction's roster reaches.
   Numbers only, no advice: the tempo memo holds (the harness states observations; the player judges).

## Not decided

Whether the bot should ever build energy on its own under the player: no. The player has the numbers now; if the
stall column does not move, the brief's general layer gets a line, not the code.
