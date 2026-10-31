# econ-flagship-6: the player's debrief

Match `run/matches/1793413080-econ-flagship-6/00`, build d499981bdb87, against NullAI 0.1 hard_aggressive on Comet Catcher Remake 1.8: Timeout after 20.0 min; 41 turns. Debriefed by claude-opus-5 (effort low) in 19 s, in a shape of its own (DECIDED / NEXT PLAYER / WISHED was asked); since snapshot: 7-day +0.0 points, extra usage +0.00.

This was a pure economy run: NullAI never attacked, so nothing was ever built for defence. The only enemy threat all game was his idle commander at G1, which shoots anything in reach — it killed two advanced constructors at spot_10. Put spot_10 and spot_5 out of bounds in the standing packet from turn one and never send a builder within about 500 of his commander. The 660-metal wreck it leaves there is not worth collecting.

Result: timeout at 20:00 with 894-922 m/s, 78 of 80 spots held, ~40 advanced extractors, three-plus advanced fusions and ~20 advanced converters. For reference, gecko's benchmark is 62/213/651/1,706; our checkpoints ran 52 m/s at 5:18, 135 at 9:13, 185 at 11:49, 255 at 13:14, 430 at 14:34, 572 at 16:32, 724 at 18:07.

What to do differently, in order of cost:

Get the advanced vehicle plant down at about 3:30, not 4:33, with solars placed ahead of it — it draws about 340 e/s and did not finish until 7:23. The bank then floated full from 6:00 to 9:00 for want of energy and build power; fusions and construction turrets both came late. Build the nano block earlier: once it was 19 turrets (3,800 bp) a converter stood in 9 s and an advanced fusion in 82 s, and at 51 turrets a fusion went up in 6 s. That block is the whole reason the late curve climbed, so buy it before you need it.

Know which resource binds and say so each turn. A plant making only constructors spends about 14 energy per metal, so energy binds early — solars cost no energy to build and are the right metal sink while the store is full. Mid-game metal binds; converters (380 m for 10.3 m/s, 37 s payback) beat mohos (620 m for 6.9 m/s, 90 s). Late, energy binds again, so idle advanced constructors should alternate one fusion then converters beside it — roughly 13 m/s per pair, and worth it well before 20:00. Fusions are the right metal sink when metal banks: 5.4 e per metal to build, 750 e/s each.

Two mistakes to avoid. I ordered the T1 plant reclaimed at 13:14 and the advanced plant at 13:54 because metal was the bind; forty seconds later the advanced fusion and converters landed, metal banked, and I had to cancel the second reclaim. Don't reclaim producers off a one-turn reading when builds are already in flight. And watch exit lanes: the plant stalled at 1:53 with the commander standing in its lane, converters blocked it at 5:28, and an advanced constructor sat in the lane for the last 30 seconds. Say explicitly that nothing stands in a plant's exit lane.

The thing that worked best: give every builder its own counted, named list of spots or jobs. Early on, constructors with no list drifted into assisting or toward a bot lab; once each had its own ordered spot list and the packet said never idle, both plants ran without pause. Keep the fall-throughs in the packet too — a constructor with no list helps the nearest moho or fusion frame; at home it builds a solar if energy stalls, a construction turret if not.
