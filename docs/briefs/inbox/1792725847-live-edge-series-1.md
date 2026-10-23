# live-edge-series-1: the player's debrief

Match `run/matches/1792725847-live-edge-series-1/02`, commit 16ec55b+, BARb hard_aggressive on Comet Catcher Remake 1.8: Win after 22.6 min, 119 turns. Debriefed by claude-opus-5 (effort low) in 38 s; since snapshot: 7-day +0.0 points, extra usage +0.00.

# Debrief — live-edge-series-1, Comet Catcher Remake 1.8, Cortex, corner W, vs BARb hard_aggressive (win, 22:37)

## What to copy

**The opening, unchanged.** mex, mex, solar, solar, corvp; then the plant on constructor, 2 Rascals, constructor, 3 Rascals, constructor, Incisors with a constructor every few. We start at B8. 4 extractors at 2:10, 5 at 2:50, 7 at 3:36, 9 at 4:05, 33 at 11:17 (income 77) — ahead of the people's median the whole way. Nothing in the opening needed rethinking.

**Name the plant before you produce.** My first `produce` call was refused: `plant` is not an actor name. Use `all` until the plant stands, then `plant_N`. I lost the first turn to this.

**Fix energy twice, early.** It stalls at 2:10 (80 in against 88 out, store empty) and again at 3:36 (160 against 307). Both times the answer is solars on whichever builder is nearest, ahead of the next extractor. It stalls a third time at 12:25 (energy 296) — nine solars on two constructors.

**Groups by unit type.** Setting the plant's soldiers to one group per type is what let the Rascals rove while the Incisors stayed home. Do it at the plant's build list.

## What cost us the most

**Standoff shelling is the whole story of 19:53–21:43.** A Lasher army given "fight to toaster" or "attack his buildings at X" stops about 400 short and shells from its own reach — which is inside a Persecutor's 1390. It lost 20 tanks in 10 seconds at 19:53, then ~8,700 metal in the following minute, and the front moved 747 to 719. The charge I thought I had ordered never happened.

What worked, at 21:58: name the gun itself as the target from the bearings of its shots, order **one body driving onto it**, and add **"never stand still at a distance under the Persecutors' fire."** That phrasing produced a real advance. Write it that way the first time; do not spend two minutes discovering it as I did.

Second-best answer: Tremors from the advanced plant (1470) outrange the Persecutor (1390). I started them too late to matter — the strike went in first. If you see a Persecutor before 20:00, start Tremors then rather than trading into it.

**Brutes lose to Pounders.** Confirmed twice (9:35, 12:25: bled 6k to kill 2.6k at F2/G2; 13:48 still dying). Lashers (575) outrange Pounders and kite them; Tigers come from the advanced plant. By 13:48 all three tier-1 plants made only Lashers and the footwork was kite/flee/fan. Get there sooner — switch the plants the first time a Brute trade goes badly, around 9:30.

**Home extractors have no turrets and he knows it.** A lone Incisor was inside our base at 3:18 and killed a half-built mex at B5 plus constructor_10396 (4:05–4:20); hunts stopped at their 900 limit and the raider was faster than the builder. Worse at 16:39: 3–4 Gators took **10 extractors in 3 minutes down the west column**. Equal-speed hunters cannot catch raiders, and single Guards die to them. Twin Guards (corhllt) at the west spots ended it. Put turrets on the west column before 15:00.

## Timeline worth knowing

- His tier 2 lands ~10:44. Ours: advanced vehicle plant raised by commander plus two constructors starting ~9:55, third corvp at spot_69 at 11:17.
- Fronts are F2/G2/E3; his vehicle plant at G2, his base box G1/H1. His commander was at H3 at 2:50 — scout G1/H1, which is where it ends up.
- Bank caps repeatedly from 15:59 on (2.3k, 3.3k, 4.2k). Spend it: mohos on both advanced constructors, then a fusion.
- Watch for label collisions — `#fus1` was refused with "stands already" and the reactor silently never started for a minute. Use a fresh label and check the next turn that it is actually building.

## How the game was won

His commander walked into sight at G1 (6709, 763) at 22:10 with three constructors under three turrets, outweighed 6:1 counting turrets triple. The standing instruction that paid for itself: *his commander, whenever it is in sight, is the main group's business above everything — the whole body attacks and chases it, Lashers shelling from outside its D-gun.* Carry that sentence from your first packet. On this map against this opponent the army is big enough to cash it by 22:00, and nothing else closed the game faster — we were up on economy from 8:00 and still bleeding 9k a minute to his guns at 20:00.
