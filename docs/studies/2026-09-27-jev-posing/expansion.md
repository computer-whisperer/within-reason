# Expansion: why our seats could not take spots fast enough, against thebluegecko (2026-09-27)

A read-only study for the Jev-posing mining (README beside this file). The hypothesis under test, from the user: thrash in
who moves where, with constructors going back to base when they were needed further out. Sources: all three seats'
records, jev logs and the player's transcript for games 2-9 (the ninth is `1790479031-...-8`); his replay records and
cards for games 2, 3 and 9, which agree with our records to within one extractor. Figures cover the first 10 minutes.

## Summary

1. **Per seat we were a third of him; as a side we were not behind on extractors.** At 8:00 a seat held 5-17 against his
   18-27 (the pool: 15-16 per player). The side held 20-35 and stayed level or ahead through 14:00 in all three of his games.
   The side's metal income led his until 6:00 (game 3) or 12:00 (games 2 and 9). He pulled ahead after 14:00 on income per
   extractor (tier 2), not on extractor count.
2. **Going home is small:** 2 % of constructor time. Between consecutive extractors, our constructors went home in 9 of 69
   intervals, his in 0 of 52.
3. **The thrash runs between seats.** 114 extractor orders were abandoned. Another seat of ours took the spot in 57 of
   them, and in 27 of those the ally's extractor was already started when we gave the order. About 2,600 constructor-seconds
   (7.6 %) went on walks to extractors that were never built. Between consecutive spots our constructors walked 800-1,640
   elmos over 54-185 s (medians); his walked 500 over 30-39 s.
4. **Too few constructors per seat.** The player's allowances kept most seats at 2-4 constructors. He had 4 by 4:00 and
   9-13 by 10:00. Seat-games with 6 or more constructors held 15-21 extractors at 10:00; those with 4 or fewer held 7-13.
5. **Expansion stopped at the midline and at the end of each seat's list.** We never held more than 5 spots west of the
   middle, while 16-23 stood free there at 8:00. The middle seat stalled at 7-9 extractors in every game.

## Extractors by minute

Per seat and in total from our records; his from his cards. The pool is the strong duellists' median per player, with the
winners' median in brackets (`run/pro_baseline.py`, brief).

| game | | 2:00 | 4:00 | 6:00 | 8:00 | 10:00 | 12:00 | 14:00 | 16:00 |
|---|---|---|---|---|---|---|---|---|---|
| 2 | t1 G1 / t2 H8 / t3 G4 | 3/3/4 | 7/5/4 | 4/5/6 | 9/8/8 | 12/10/9 | 12/11/8 | 12/15/8 | 3/16/8 |
| | ours total / his | 10/3 | 16/6 | 15/11 | 25/18 | 31/24 | 31/25 | 35/34 | 27/40 |
| | constructors ours / his | 1/1 | 7/4 | 8/4 | 8/5 | 8/7 | | | |
| 3 | t1 / t2 / t3 | 3/4/3 | 3/5/5 | 5/6/3 | 6/9/5 | 9/17/7 | 9/17/8 | 11/18/8 | 8/14/7 |
| | ours total / his | 10/4 | 13/8 | 14/14 | 20/27 | 33/29 | 34/32 | 37/33 | 29/32 |
| | constructors ours / his | 2/2 | 4/4 | 10/6 | 14/8 | 14/9 | | | |
| 9 | t1 / t2 / t3 | 4/3/4 | 5/5/4 | 7/10/7 | 10/17/8 | 12/21/8 | 12/21/8 | 12/19/8 | 12/17/8 |
| | ours total / his | 11/3 | 14/10 | 24/19 | 35/19 | 41/22 | 41/28 | 39/33 | 37/32 |
| | constructors ours / his | 3/2 | 9/4 | 8/8 | 13/10 | 13/13 | | | |
| pool | per player | | 7 | | 15-16 (18) | (23) | 23 (28) | | |
| 4-8 | ours total | 9 | 12-20 | 21-26 | 26-34 | 29-42 | 24-46 | | |

The per-seat gap opens by 4:00 (his two constructors by 1:29 and three home spots give him 8-10 against our 3-7), and
widens to 8:00 (he adds 8-17, a seat 1-7). At 16:00 he drew 112-132 metal a second from 32-40 extractors (tier 2 from
11:18 in game 9, 13:58 in game 3); our side drew 60-76 from 27-37.

Free spots, from all four teams' replay records, east (ours) and west (his):

| game | 6:00 free east / west | 8:00 free east / west | 10:00 free east / west | ours west of the middle, max |
|---|---|---|---|---|
| 2 | 25 / 31 | 15 / 23 | 9 / 19 | 0 |
| 3 | 26 / 26 | 19 / 16 | 7 / 11 | 1 |
| 9 | 16 / 22 | 6 / 20 | 4 / 15 | 5 |

## Constructor time budgets

From each constructor's commands and position every second to 10:00, as % of constructor-seconds. Walk and build ext:
toward or beside its extractor order; help: guard or repair; away: moving after a retreat or step-away play; stuck: under
25 elmos in 6 s on a walking order. Abandoned: extractor orders never built (another seat took the spot; replaced by
rule / plan / list).

| seat | cons | con-s | walk ext | build ext | at site | walk other | build other | help | away | reclaim | idle | stuck | abandoned | walked on them |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 3 t1 | 4 | 1086 | 21 | 19 | 4 | 5 | 31 | 15 | 3 | 0 | 2 | 1 | 3 (1; 1/2/0) | 51 s |
| 3 t2 | 10 | 2210 | 36 | 16 | 3 | 3 | 18 | 20 | 2 | 1 | 2 | 1 | 5 (1; 3/2/0) | 140 s |
| 3 t3 | 4 | 1244 | 20 | 22 | 4 | 6 | 27 | 5 | 1 | 10 | 4 | 1 | 2 (2; 0/2/0) | 59 s |
| 9 t1 | 3 | 1357 | 24 | 29 | 4 | 4 | 31 | 3 | 0 | 0 | 2 | 2 | 6 (3; 2/1/3) | 214 s |
| 9 t2 | 6 | 2027 | 25 | 28 | 4 | 4 | 21 | 14 | 0 | 0 | 3 | 1 | 5 (1; 0/2/3) | 120 s |
| 9 t3 | 5 | 1477 | 22 | 20 | 4 | 6 | 33 | 11 | 0 | 0 | 2 | 1 | 11 (6; 1/8/2) | 192 s |
| 2 t1-t3 | 9 | 3499 | 14 | 18 | 3 | 7 | 34 | 9 | 4 | 4 | 4 | 2 | 3 (3; 1/2/0) | 98 s |
| all 24 seat-games | 107 | 34719 | 21 | 20 | 4 | 5 | 30 | 11 | 2 | 2 | 4 | 1 | 114 (57; 22/53/22) | ~2,630 s |

His replays carry no commands, so both sides are also classed by position alone:

| | moving | build ext | build other | still | extractors his/our constructors started |
|---|---|---|---|---|---|
| his, games 2 / 3 / 9 | 26 / 21 / 19 % | 20 / 20 / 11 % | 36 / 40 / 31 % | 18 / 20 / 40 % | 21 / 28 / 19 |
| ours g3, t1 / t2 / t3 | 41 / 48 / 44 % | 19 / 16 / 22 % | 31 / 18 / 27 % | 10 / 19 / 8 % | 9 / 15 / 11 |
| ours g9, t1 / t2 / t3 | 36 / 44 / 39 % | 29 / 28 / 20 % | 31 / 21 / 33 % | 4 / 7 / 8 % | 12 / 21 / 7 |

He spends as much time on turrets and solars as we do. We differ in walking, and in constructors per seat.

What replaced an extractor order before it was built (all games):

| replaced by | count | seconds already walked |
|---|---|---|
| another extractor: rule re-target / pick / list step | 17 / 17 / 10 | 558 / 524 / 218 |
| a pick to help a plant | 19 | 386 |
| a turret, solar, radar, nano or converter | 23 | about 400 |
| retreat or step away | 8 | 129 |

## The mechanisms that cost the most

**1. Seats walk to each other's spots, and the words do not say so.** Game 9, the middle seat's constructor_2914, 6:49 to
8:07. It got five extractor picks in 78 s and built nothing. It walked 36 s to spot_49 (the south seat built there at
7:24), then went for spot_37 (south, 7:26), spot_46 (south had held it since 5:23) and spot_32 (south, 7:59). At 8:07 it
was picked to "help plant_11741", at a confidence of 0.36. The words each time: "constructor_2914 builds a metal
extractor at spot_46 (14 s of walking, ground ): our 9th (1 under way already)". They name neither the ally's extractor
nor its constructor. `free_spots` (`pianist/plan.rs`) filters on our own extractors, our own seat's tasks and enemy
buildings. Allies count only through H-TEAM-ALLY-GROUND, for spots nearer the ally's start. The team board's
`spot_claims` is never filled and `team_mates` is never read (`brain/allies.rs`, `brain/mod.rs`), so the pooled claims
that H-TEAM-BOARD registers are empty. Worst is the middle seat (game 7: 15 abandoned, 13 ally-taken). **Needed:** allies' extractors and builders' targets in each seat's picture and on its spot options, and
per-seat free spots in the report.

**2. First lists: a menu of the enemy's base, then steps that vanish.** Game 9, 1:29. The middle seat's first constructor
was offered only "spot_36 (108 s of walking, ground ): our 4th" (the default) and "spot_45 (115 s of walking)", both in
the person's home cells. The 0:59 packet had written "Middle seat's: spot_34, 29, 35, 42, ...". Only a literal `spot_N`
counts as named (`diet::names`), so the named free spots were the scout targets "spot_36, spot_19, spot_45, spot_2". The
rule played spot_36, and the north seat's first constructor went to spot_2 (B1) at 1:37 the same way. The middle
constructor drove west for 45 s. Then the 2:05 list landed at 2:20 ("extractor spot_41", "extractor spot_40", "armllt
spot_41", ...), and its two extractor steps were neither played nor said to be skipped. The constructor built the turret
first, and its first extractor came at 3:37. The south seat's list lost its leading extractor steps the same way at 2:20.
Over 139 constructor lists, 39 extractor steps vanished like this while a later step of the same list played.
`play_lists` (`pianist/lists.rs`) pops the step before `execute_builder`, whose early `return None` (H-HANDS-STARTED, or
no site from `build_site_for`) drops it with nothing said. The median time from a
constructor's birth to its first list step was 86 s: lists usually landed on a constructor already sent elsewhere. **Needed:** a vanished step said like any other skip, and the nearest free spot offered beside far named spots.

**3. Two or three constructors per seat, set by the player's allowances.** In game 3 at 1:31 the player sent
`produce {"lab_15738": ["armpw"], "lab_9212": ["corak"]}`, with no constructor. t1 and t3 had one constructor until
about 5:00. In game 9 at 2:05 each plant got `["armcv:2", "armflash"]`, with "constructors while allowed, then raiders
... without pause". Game 7, the best expansion of the series (46 extractors at 12:00), is the one where the allowance was
raised to `corcv:2` at 5:04 and `corcv:3` at 8:09. The brief (for one duellist) says "a second constructor near 2:00,
never three constructors at once". **Needed:** each seat's constructor and
extractor counts against the per-player curve in the report, since the brief's curve reads as if the side were one player.

**4. The midline and the end of the list.** From 2:20 in game 9 the packet said "Constructors stay in the eastern half of
the map; they never go to the enemy's strip" and gave each seat 8-11 named spots. t1 plateaued at 12 from 9:00 and t3 at 8
from 8:00; by 10:00 4 spots were free in the east and 15 in the west. His advice: "need to hold more of the map" (game 9)
and "try and hold half the map while raiding the corners" (game 7). **Needed:** once our half is full, the free
middle and far spots each turn with what stands near them, so where next stays the player's live call.

**5. The hypothesis as stated: going home.** Real but small: 77 retreat and step-away plays (49 rule, 28 pick), nearly
all under raids between 3:20 and 5:56, 2 % of constructor time. The larger form is "help the plant", 11 % of time, with
87 picks while an extractor was on the menu. The help option says "adds its build power to whatever it
makes (metal must be coming in faster than the factory spends it)". The extractor option says "(19 s of walking, ground ):
our 10th", with nothing on its return (about 2.6 metal a second here) and the ground left blank. The store held under 100
metal at 42 % of those picks. **Needed:** each spot option's return, and the plant's real spending headroom.

## What his method does that ours does not

- **A constructor about once a minute from one plant.** In game 3: 1:17, 1:29, 2:32, 3:22, 4:19, 5:23, 6:22, 6:50.
- **Field constructors run strips and never come home.** In game 3, constructor 1 took B3, B3, C3, C3 and E3 with turrets
  and radars between. Constructor 3 took C1, C1, B1, B1, A1. Constructor 4 took B8, B7, B8, B8. The median gap was 33 s and
  504 elmos between consecutive extractors, and 0 of 52 intervals went back home. Home ring first, then three strips.
- **Base work has its own constructors.** One constructor built ten solars and an energy store from 5:18 to 9:53; another
  built nano turrets and solars from 7:21. None of the field constructors did either. His "cons at home only if you have
  extra metal to spend", as a division of labour.
- **Vehicles.** His Masons move at 54. Our game-3 seats opened bot labs, whose constructors move at 36.

## What the tools could not read

- His replays carry no commands. His "still" time mixes assisting, idling and waiting, and his reversals cannot be counted.
- A reversal's source is the last play within three frames of the new command; "an ally took it" means an ally's
  extractor was started there by 30 s after our order changed.
- The cause of the 39 dropped list steps is untraced; finding it needs the bot's log or a replay of the pass.
- No opponent cards for games 4-8; games 5 and 8 ended before 10:00.
- `run/floor.py`, `hands_window.py` and `raid_ledger.py` read seat 0 only; these numbers come from scratch scripts.
