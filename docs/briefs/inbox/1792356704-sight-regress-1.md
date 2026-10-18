# sight-regress-1: the player's debrief

Match `run/matches/1792356704-sight-regress-1/00`, commit 4e8a96a, BARb hard_aggressive on Comet Catcher Remake 1.8: Win after 19.1 min, 98 turns. Debriefed by claude-opus-5 (effort low) in 39 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief — sight-regress-1, Comet Catcher Remake 1.8, arm, corner W, vs BARb hard_aggressive

Won at 19:06, 98 turns. The win came from one thing: walking the whole body around his turrets to his base at H1 and killing the commander. Everything between 11:00 and 17:40 was wasted metal. Read the middle section before you give your first push.

## What worked, keep it

**The opening.** Mex, mex, solar, solar, armvp at ~0:34, commander assisting the plant at low priority, then two more extractors. Plant list: Rovers to rove his half, Blitzes for the raid answer, a constructor every four or five units (I had 4-5 by 6:00). Give the plant its list directly and set its soldiers to group by type, so the Rovers have their own group and the Blitzes stay home. Give the first constructor a fixed expansion list immediately — the hands will otherwise send it to assist the plant.

**Economy caught up by 4:28** (7 extractors, 18 metal/s — the pro median at 4:00) even after losing three extractors to raids. Being two extractors down at 3:00 is recoverable; do not panic-build defence everywhere.

**The finish.** From 17:37 on, orders were: *walk* to the next spot, don't stop to fight, next stop. That got the body to H3 (killed his first moho, two extractors, a Sentry), then spot_25 → 17 → 12, then his base at H1/H2. At 18:34 the whole body on his commander with fan footwork on for the D-gun; dead by 19:00.

## His shape on this map

- **0:50–5:00: two or three Rovers in your base.** They killed my spot_74 extractor, a spot_69 frame, and a solar frame. **Blitzes cannot catch a Rover** (1.7x speed). The answer is the commander plus a light turret on the exposed spots, not more Blitzes. Build the turret before the extractor on anything west/north of home.
- **Turret nests at E3, F3, F4, F5.** Beamers, LLT, HLT, Gauntlet (reach 1,220), Dragon's claw, plus Shellshockers. A T1 body trades ~1.5:1 *against* you into these, every time. He also keeps two Rattlesnakes at G2 (1,380) in front of his base.
- **Tier 2 at ~15:00** (first Bull). His economy stays weak — at 17:56 I was at 116 income against his ~40 — so his army is the only thing that grows. Don't let the clock run while you feed it.

## The expensive mistakes — all the same mistake

I gave three pushes (10:16, 16:20, 17:17) and all three were dragged into turrets, costing roughly 2,300, 3,500 and 1,150 metal in single minutes. The mechanism is always one of these:

1. **A lone bait unit in sight near a nest** (a kiting Blitz at 11:05, the Bull at 15:31) pulls the body in through Jev's "attack a party in sight". A never-list on the turret does **not** hold against an in-sight party.
2. **A stale attack order** on a party that then walks west into the nest (17:27, 17:37). The body follows it.
3. **Route stops inside turret reach.** Check every waypoint against the Gauntlet's 1,220 and the Rattlesnakes' 1,380 before you write the route.

**So:** route the body *entirely* around E3/F3/F4, or bring Shellshockers (710) to outrange the Beamer (490) first. When you want it to move, order a walk and say not to stop and fight; re-issue every turn it drifts. Prefer going round and killing his economy and his commander — his base at H1/H2 was defended by two turrets and nothing else.

## Mechanics worth knowing before you need them

- **Footwork stalls the body at contact.** Flee kept 14 of 39 out of a fight with a 275-metal post (14:21). March/form/slot held the body 20-400 short of its waypoint for 30-40 seconds under fire (16:30, 17:07, 17:17). For a big body on the move, drop flee and march and set the lane raw.
- **Thrash:** group_B cycled spot_35/spot_42/gather/hold on 8-second loops because its route's first stop was already "reached". Rewrite the route to start at the *next* spot, with no gather/hold.
- **Whistlers die to a single Janus** (splash on a packed group) — 2:1 against us for three minutes. Stop building them; Stouts and Blitzes, plus Shellshockers for turrets.
- **Reinforcements trickling alone** past his forward units were most of my late losses. `join_in 6` so new units walk out six together.
- **Blocked build sites:** spot_74 refused twice — first the wreck of the old extractor, then one of *our own* light turrets standing on it. Reclaim or take apart the thing on the spot; the commander's queued teardown doesn't always take, so check it happened.
- Four plants showed idle at 17:27 with metal flowing in; re-issue lists when income is positive and nothing is being spent.

## If I sat down again

Same opening. Turret-first on the exposed spots. Then **no push into his half before Shellshockers**, and when the body does go, send it on a walk around the east side (spot_25 → 17 → 12 → 5/10/6) straight at his base, with footwork off and no attack orders left standing. That line, which won the game at 19:00, was available at 14:00.
