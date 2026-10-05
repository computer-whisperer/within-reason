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
| D-ENERGY | A builder builds a solar while the energy store is under 30 %, or falling and under 60 %; two builders at a time and one more for every four constructors | `ENERGY_BUILDERS = 2`, per 4 | 31 to 70 solar steps a game, about one an extractor | `energy_short` |
| D-PLANTS | A second plant from 6:30 at 10 metal a second, a third from 10:30 at 22, and one more whenever the metal store is over half; eight at the most | `PLANTS = 8` | Second plants at 4:47 to 7:42 (median 6:30), 3 to 11 plants a game | `plants_wanted` |
| D-PRODUCTION | Scout, scout, constructor; two more scouts; then soldiers with a constructor after every two (under three constructors) or four, until the constructors are 3 before 3:00 and 3 + 1.2 a minute after, 14 at the most; raiders before 8:00, then five tanks to one artillery | `TANKS_FROM` 8:00, `SCOUTS_FIRST = 4` | The first nine units of six games: fav fav cv fav fav cv flash flash cv; constructors made by 3, 5, 8, 10 min: 3, 4-5, 7-10, 9-15; the first Stout at 5:08 to 9:07 (median 8:00) | `next_unit` |
| D-GUARD | Soldiers wait 500 out from home; a mobile enemy within 900 of a building of ours is fought by all of them | `DEFEND = 900`, `RALLY = 500` | A guess. Jev's `send` and `attack party` picks, 405 in the seven games | `army.rs` `raid` |
| D-LAUNCH | With no raid on, the home guard goes out as the strike body at 550 metal of soldiers before 8:00 and 1,200 after; with the body out, at 400 it follows | `LAUNCH_EARLY`, `LAUNCH_LATE`, `REINFORCE` | A guess. The ruling that the first handful runs across the map (2026-10-03) | `launch` |
| D-GATHER | A body with under 60 % of its soldiers within 600 of its middle closes on its middle before anything else | `BODY = 600`, `BODY_SHARE = 0.6` | Jev's most frequent army pick (`gather`, 273); the share is the flagship hands' own | `run_strike` |
| D-STEP-BACK | A body outweighed 1.3 to 1 within 900 (a turret three times its metal) walks home and joins the guard; it goes again with 1.5 times what drove it back, the memory halving every 3 minutes | `OUTWEIGHED = 1.3`, `TURRET_WEIGHT = 3`, `AGAIN = 1.5` | The flagship's rules text for the hands (outweighed groups step back; a turret counts three times) | `run_strike` |
| D-ADVANCE | A body that is together fights its way to the nearest known building of his, else the nearest enemy in sight, else the nearest spot on his half nobody has looked at, else his start | | A guess over `fight to` and `go` (391 picks) | `run_strike` |
| D-SCOUTS | An idle scout drives to the spot on his half nearest to it that nobody has been sent to | | The player's standing words for Rovers ("they are scouts, being seen is their job") | `run_scouts` |

Not in it: construction turrets, advanced solars, radar, converters, tier 2, air, sea, reclaim, the control lane's
footwork (`crates/micro`), detachments, shelling, any thought for the commander's safety, more than one seat.

## Results

v0 (2026-10-05), Comet Catcher Remake 1.8, eight games a row, sides and factions alternating, 40-minute cap:

| Opponent | Result | Game length |
|---|---|---|
| BARb easy | 8-0 | 15 to 23 min |
| BARb medium | 8-0 | 16 to 23 min |
| BARb hard_aggressive | 1-6, one timeout | the win at 22 min, the losses at 30 to 36 min |

Extractors standing at 5:00, 8:00, 10:00: 6-11, 15-24, 20-33 (the flagship's orders: 12-19, 21-30, 25-37).

What the hard_aggressive games show: from 8 to 10 minutes on the metal store is full (1,900 to 4,500 unspent) with
energy at zero for minutes at a time, so the army is a fraction of what the income could pay for (0 to 8 in the
strike body at 10 to 15 minutes), and the game is lost after 30 minutes to his tier 2 (Goliaths, fusion reactors).
The flagship wins that pairing in 12 to 20 minutes. Spending is the first thing to mend.

## Running it

`target/release/arena --our-ai WRDistilled:0.1 --matches 8 --parallel 8 --speed 50 --profile medium --map "Comet Catcher Remake 1.8" --label <label>`.
Its own line a game minute is in `engine.log` (`[distilled ai=N] f=... extractors=... strike=...`).
