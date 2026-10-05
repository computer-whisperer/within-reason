# WRDistilled: a heuristic bot fitted to the flagship's winning games

A bot of `bots/`: code decides everything, no model is asked, a game costs nothing (`../README.md` for the regime).
Vehicles, tier 1, Armada or Cortex, any map with metal spots; one seat.

## Where the rules come from

Seven games the flagship won against BARb hard_aggressive on Comet Catcher (Opus 5.5 medium as the player, the same
start): going-opus-1, again-opus-1, same-opus-1, same-opus-2, standing-cut-1, wire-cut-1, stragglers-1
(`docs/experiments.md`). Every played action of those games was read from the hands' logs (`jev-0.jsonl`, the `played`
entries of `pass` and `plan` lines). Half to three quarters of them were the player's build lists, carried out by
code with no question to Jev: those are the economy and production rules below, read off directly. The rest were
Jev's picks, mostly for the army; an earlier attempt to fit rules to Jev's answers recovered whether a party needs
answering and not which move to make (2026-10-05, 1.5M sampled answers, whole games held out), so the army rules
here are a guess over the verbs Jev picked most, to be judged by results.

## The rules

| ID | Rule | Values | Read from | Code |
|---|---|---|---|---|
| D-OPENING | The commander builds two extractors, two solar collectors and the vehicle plant, in that order, decided from what stands | | Identical in all seven games: extractors at 0:02 and 0:08, solars at 0:16 and 0:24, the plant at 0:34 | `economy.rs` `next_job` |
| D-COMMANDER | After the opening the commander builds a solar when energy is short, an extractor on a free spot near home, and otherwise helps the plant | `COMMANDER_WALK = 700` | The commander's lists: `assist` from 0:55, then solars and the home spots | `next_job` |
| D-EXPAND | A constructor takes the free spot that is nearest, the walk counted whole and the distance from home half; not a spot with an armed enemy near or one refused in the last minute | `SPOT_DANGER = 600`, `REFUSED` 60 s | 39 to 51 extractor steps a game; orders by 5:00, 8:00, 10:00: 12-19, 21-30, 25-37 | `free_spot` |
| D-SENTRY | An extractor farther than 900 from home gets a sentry on its enemy side, queued behind it for the same builder | `SENTRY_FROM = 900` | 27 to 54 sentry steps a game, most written as the step after an extractor at the same spot | `order` |
| D-ENERGY | A builder builds a solar (a constructor the advanced solar, from 15 metal a second) while the energy store is under 30 %, or falling and under 60 %, or the income is under 1.25 times what the soldiers the metal income pays for draw; two builders at a time and one more for every four constructors | `ENERGY_MARGIN = 1.25`, `ADVANCED_FROM = 15`, `ENERGY_BUILDERS = 2`, per 4 | 31 to 70 solar steps a game, about one an extractor, and 2 to 16 advanced solars; the draw is the game's own arithmetic (a soldier's energy over its metal, times the metal spent) | `energy_short` |
| D-BUILD-POWER | From 2:30, while the plants and their construction turrets cannot turn 85 % of the metal income into soldiers (build power times the soldier's metal over its build time), or the metal store is over half: one construction turret behind each plant, then another plant, up to eight, then turrets up to three a plant. An idle turret helps its plant. One builder at a time | `SOLDIERS_SHARE = 0.85`, `PLANTS = 8`, `TURRETS_A_PLANT = 1`, at most 3 | 3 to 11 plants and 5 to 11 construction turrets a game, the second plant at 4:47 to 7:42; v0's games, where 1,900 to 4,500 metal stood unspent from minute 8 | `build_power_job`, `can_spend`, `run_turrets` |
| D-PRODUCTION | Scout, scout, constructor; two more scouts; then soldiers with a constructor after every two (under three constructors) or four, until the constructors are 3 before 3:00 and 3 + 1.2 a minute after, 14 at the most; raiders before 8:00, then five tanks to one artillery | `TANKS_FROM` 8:00, `SCOUTS_FIRST = 4` | The first nine units of six games: fav fav cv fav fav cv flash flash cv; constructors made by 3, 5, 8, 10 min: 3, 4-5, 7-10, 9-15; the first Stout at 5:08 to 9:07 (median 8:00) | `next_unit` |
| D-GUARD | Soldiers wait 500 out from home; a mobile enemy within 900 of a building of ours is fought by all of them | `DEFEND = 900`, `RALLY = 500` | A guess. Jev's `send` and `attack party` picks, 405 in the seven games | `army.rs` `raid` |
| D-LAUNCH | With no raid on, the home guard goes out as the strike body at 550 metal of soldiers before 8:00 and 1,200 after; with the body out, at 400 it follows | `LAUNCH_EARLY`, `LAUNCH_LATE`, `REINFORCE` | A guess. The ruling that the first handful runs across the map (2026-10-03) | `launch` |
| D-GATHER | A body with under 60 % of its soldiers within 600 of its middle closes on its middle before anything else | `BODY = 600`, `BODY_SHARE = 0.6` | Jev's most frequent army pick (`gather`, 273); the share is the flagship hands' own | `run_strike` |
| D-STEP-BACK | A body outweighed 1.3 to 1 within 900 (a turret three times its metal) walks home and joins the guard; it goes again with 1.5 times what drove it back, the memory halving every 3 minutes | `OUTWEIGHED = 1.3`, `TURRET_WEIGHT = 3`, `AGAIN = 1.5` | The flagship's rules text for the hands (outweighed groups step back; a turret counts three times) | `run_strike` |
| D-ADVANCE | A body that is together fights its way to the nearest known building of his, else the nearest enemy in sight, else the nearest spot on his half nobody has looked at, else his start | | A guess over `fight to` and `go` (391 picks) | `run_strike` |
| D-SCOUTS | An idle scout drives to the spot on his half nearest to it that nobody has been sent to | | The player's standing words for Rovers ("they are scouts, being seen is their job") | `run_scouts` |

Not in it: radar, converters, tier 2, air, sea, reclaim, the control lane's
footwork (`crates/micro`), detachments, shelling, any thought for the commander's safety, more than one seat.

## Results

Comet Catcher Remake 1.8, eight games a row, sides and factions alternating, 40-minute cap.

| Version | BARb easy | BARb medium | BARb hard_aggressive |
|---|---|---|---|
| v0 (2026-10-05): D-OPENING to D-SCOUTS, plants by the clock, energy by the store alone | 8-0, 15 to 23 min | 8-0, 16 to 23 min | 1-6, one timeout |
| v2 (2026-10-05): D-BUILD-POWER, and D-ENERGY ahead of the draw | 8-0, 16 to 21 min | 7-1, 13 to 21 min | 4-11, one timeout, over two rows of eight on different seeds |

Against hard_aggressive, medians over a row's games (`run/bot_curves.py <label>`):

| | Metal unspent at 8, 10, 12, 15 min | Metal income at 10 min | Extractors at 10 min | Units ordered by 10 min | Soldiers alive at 10, 15 min |
|---|---|---|---|---|---|
| v0 | 940, 2,466, 2,863, 3,392 | 61 | 26 | not logged | 10, 24 |
| v2, seeds 1-8 | 0, 77, 150, 1,469 | 42 | 18 | 46 | 12, 26 |
| v2, seeds 101-108 | 4, 22, 86, 646 | 44 | 20 | 46 | 12, 26 |

What v2 shows:

- **The spending is mended and the result is not.** The store stays near empty through minute 12 and the metal
  spent follows the income, but the body alive at 10 and 15 minutes is the size it was, and sixteen games give four
  wins where eight gave one: inside what eight-game rows scatter by.
- **The soldiers are made and then lost.** About 46 units are ordered by 10:00 and about 100 by 15:00 against 12 and
  26 soldiers alive: D-ADVANCE walks the body into his sentries (20 or more by 10 minutes) again and again.
- **He raises our dead.** BARb hard_aggressive keeps 7 to 54 resurrection bots (`cornecro`) and the census shows him
  owning our units: 12 Stouts at 15:00 and 59 at 32:00 in seeds 1-8's game 5. Every soldier that dies on his ground
  is his next soldier. This is the first thing to mend now: where the body fights, and what it does with wrecks.
- **Expansion pays for the build power.** With one builder on turrets and plants and more on energy, extractors at
  10:00 fell from 26 to 18-20 and the income from 61 to 42-44.

A lesson for the rule set, from distilled-v1-hard (3-4-1, the version between): build power piled on one plant does
not spend. Its game 1 had one plant with six construction turrets and a full store for four minutes; a plant's yard
has to clear between units. Hence one turret a plant before the next plant.

## Running it

`target/release/arena --our-ai WRDistilled:0.1 --matches 8 --parallel 8 --speed 50 --profile medium --map "Comet Catcher Remake 1.8" --label <label>`.
Its own line a game minute is in `engine.log` (`[distilled ai=N] f=... extractors=... strike=...`).
