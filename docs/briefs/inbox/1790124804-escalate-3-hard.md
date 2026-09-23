# escalate-3-hard: the player's debrief

Match `run/matches/1790124804-escalate-3-hard/00`, commit 8a10934, BARb hard on Comet Catcher Remake 1.8: Win after 17.3 min, 56 turns. Debriefed by claude-opus-5-5 (effort low) in 20 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief: escalate-3-hard, Comet Catcher, arm corner W vs BARb hard. Won at 17.3 min.

## What won and what nearly lost it

We won on economy and volume, not on skill at the front. By 8:00 we had 18 extractors and an army of ~3,500 metal against ~225 seen, and never lost that lead. The fighting itself traded **even for the whole game**: 11,591 destroyed against 11,442 lost at 16:50. The win came when the main group went around the enemy fortress along the north edge, found their advanced bot lab at G2 (6248,856), and then their commander at G1 (6291,747).

Hard fights differently from medium. It builds a **fortress at E3/F3**: a Gauntlet plasma battery (range 1,220), a heavy laser tower (620 reach from F3), a beam turret, walls, and later an artillery turret at F2. It stacks Maces, Rocketeers and resurrection bots in front. We spent roughly 11:40–15:00 attacking into it or stalling beside it, at an even trade.

## Do this

1. **Opening: keep the players' order.** Two extractors (spot_45, spot_50), three solars, vehicle plant, then the commander goes out to spots 43/52/61 with turrets and radar. The plant builds 1 Rover and 2 constructors, then Blitzes. This was on schedule at 1:00 and 2:00.
2. **Solars before the stall, not after.** Energy hit zero at ~4:13 with metal banking. Each Blitz costs 900 E, and each Stout costs ~2,100 E. It stalled again at 9:51 with four plants running. Budget roughly one solar per 900 E of queued production. Queue solars when a plant is added, before the drop.
3. **Base location: north/north-east, G1–G2.** Don't spend Rovers on the south-east (G4–H8). Several scouts found nothing there. The enemy traffic at E2–F3 pointed at the base the whole time.
4. **Route: the north edge, 1,300+ from F3.** Go spot_3 → 6 → 10 → 5 → 12 → 17. That path clears the Gauntlet and the heavy laser tower. Go this way from the first commit (~11:40), not after two failed pushes. Expect an artillery turret at F2 near the end.
5. **Keep Pawn hunters at home: Blitzes, not Stouts.** From ~13:00 Pawn raids took extractors from 20 to 16. They also killed a plant near spot_36 and a constructor. Stouts cannot catch Pawns. Keep a small Blitz group at home through the midgame, and put turrets at the outer spots (36, 52, 61, 22).
6. **Cap factories explicitly.** The hands built four plants: home, spot_36, and two near spot_39. Say in the packet which builders may build plants and how many. The commander cannot build nanos, since they are not on its roster, and it substituted a plant.

## How the hands (Jev) misread orders

- **Stale walks persist.** A group ordered to "gather at spot_39" sat home for 72–87 s, and a new raid order didn't reach it. At 16:05 group_B kept walking west into beamer fire after a rewrite. The fix that worked: **name the old walk as wrong and abandoned** in the packet ("group_B's walk to spot_13 is abandoned; advance east to spot_6"). It took effect within one turn.
- **"Advance to X first" sends groups home** if X is behind them. Give only forward waypoints.
- **Groups split their artillery off.** group_B detached 4 Shellshockers against two Maces, and they died alone. Assign support units to a named group explicitly, and give the side threat to a different group.
- **Flee footwork holds groups under light turrets.** Turn it off for a group you are committing.

## Numbers to check against

| time | extractors | income | note |
|---|---|---|---|
| 3:33 | 9 | — | energy falling, first solars added |
| 5:31 | 13 | 28 | second plant |
| 7:57 | 18 | 36 | 24 Blitz raiding |
| 10:30 | 20 | 42 | peak; Stouts arriving |
| 14:31 | 17 | — | Pawn raids biting |
| 16:12 | 16 | — | bypass under way |

If you are at 20 extractors and ahead on the army at 11:00, the game is yours to lose. Go north around the fortress, keep a Blitz squad home, and stop feeding it into F3.
