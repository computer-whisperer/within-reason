# What spatial input Haiku 5.5 can work with at one second (2026-10-07)

The user's question: Jev cannot restructure an army, find a gap in a defensive line or place turrets well; can Haiku,
at about a second a response, and on what kind of spatial input? Mock scenarios with code-computed ground truth,
`run/haiku_spatial.py`, 12 random instances a kind (seed 1), 8 calls at once, effort low. No engine, no game.

## The scenarios and the encodings

| kind | the ask | the truth |
|---|---|---|
| gap | a turret line (5-6 turrets, reach 450) with one uncovered stretch of 400-850 elmos: pick the candidate spot that closes it (6 spots), and give a free (x, z) | the spot at the stretch's middle; a free x whose reach covers the whole stretch (half credit inside it) |
| cover | 8 extractors, 10 candidate sites between pairs of them: the two sites covering the most extractors | brute force over the 45 pairs; the score is covered / best |
| rally | 10 units, 7 within 250 of a centre and 3 strung out 900-1,800 away, an enemy party 2,500 away: name the stragglers and the rally point among 5 | the stragglers (Jaccard); the point at the main body's centre, against the whole group's centroid, the base, a point toward the enemy, a straggler |
| route | 3 enemy turrets between a group and its target, 4 routes via one waypoint each: the one out of every turret's reach | sampled exposure; exactly one clean route, two with over 300 elmos inside |

Encodings: `coords` (x, z numbers), `cells` (512-elmo cells as the picture names spots, "G7"), `ascii` (the numbers
plus a 32x32 character map, 256 elmos a character), `facts` (the numbers plus what code computed: each turret's covered
x-range, each site's covered extractors, each unit's distance to the densest centre, each route's elmos inside reach).

## The results (score is the mean over 12 instances; wall the median)

Free text, thinking off, 1,024-token cap (the model writes its working before the JSON; `cover` and `route` on bare
numbers overran the cap):

| kind | coords | cells | ascii | facts |
|---|---|---|---|---|
| gap: spot / free | 1.00 / 1.00, 2.0 s | 0.67 / 0.38, 3.6 s | 0.75 / 0.75, 2.1 s | 1.00 / 1.00, 1.6 s |
| cover: ratio | 0.31, 4.4 s (8 of 12 cut) | 0.53, 4.1 s | 0.33, 4.4 s (8 cut) | 1.00, 0.6 s |
| rally: stragglers / point | 1.00 / 0.92, 0.6 s | 0.92 / 0.83, 0.6 s | 0.94 / 0.83, 0.6 s | 1.00 / 1.00, 0.6 s |
| route | 0.67, 4.1 s (4 cut) | 0.25, 5.0 s (9 cut) | 0.75, 4.0 s | 1.00, 0.6 s |

Bare numbers with room to work (4,096-token cap) and with adaptive thinking:

| kind, coords | thinking off, 4,096 cap | adaptive thinking, effort low |
|---|---|---|
| gap: spot / free | 1.00 / 1.00, 2.0 s, 318 out | 1.00 / 1.00, 2.8 s, 509 out |
| cover: ratio | 1.00, 4.4 s, 1,014 out | 1.00, 6.5 s, 1,384 out |
| rally: stragglers / point | 1.00 / 0.92, 0.6 s | 1.00 / 0.92, 1.9 s |
| route | 0.83, 3.2 s, 610 out | 1.00, 5.9 s, 1,188 out |

Structured output (the JSON alone, no working; `output_config.format`), thinking off:

| kind | coords | cells | ascii | facts |
|---|---|---|---|---|
| gap: spot / free | 0.67 / 0.75 | 0.58 / 0.08 | 0.58 / 0.58 | 0.58 / 0.58 |
| cover: ratio | 0.66 | 0.47 | 0.56 | 1.00 |
| rally: stragglers / point | 1.00 / 0.83 | 0.90 / 0.92 | 0.87 / 0.75 | 1.00 / 1.00 |
| route | 0.58 | 0.25 | 0.42 | 1.00 |
| wall | 1.1-1.9 s | | | 1.1-1.3 s |

## What it says

1. **Haiku does the geometry right only when it writes it out, and that takes 2-6 s.** On bare coordinates it works
   the distances by hand in the text (80 distance checks for `cover`, about 1,000 tokens) and gets them right when it
   has the room: gap 1.00, cover 1.00, route 0.83 with thinking off and a 4,096 cap; everything 1.00 with thinking on.
   Denied the working (structured output) it answers in about 1.2 s and drops to 0.58-0.75 on gap, cover and route.
2. **With the geometry computed by code it is right and fast.** The `facts` encoding scores 1.00 on every kind at
   0.6 s with free text (the answer is then 10-30 tokens and the first token comes at the floor). This is the
   project's own idiom (code enumerates and computes, the model picks) and it is what the one-second budget allows.
3. **Pictures do not help; cells hurt.** The ASCII map scored at or below the numbers alone on every kind, and cost
   a third more input. The 512-elmo cell names lose the precision the gap and route asks need (a 400-850 gap inside
   a 512 cell) and scored worst everywhere: the picture's "spot_6 (G1)" idiom is fine for naming places, not for
   geometry.
4. **Where no arithmetic is needed it is fast on any encoding.** Stragglers and the rally point (a cluster read off
   the numbers) came at 0.6 s and 0.92-1.00 on coordinates, with no working written.
5. **Structured output costs about 600 ms a call here** (1.2 s against 0.6 s for the same 10-30 token answers) on top
   of the accuracy it costs. A JSON shape asked for in the prompt and the last JSON object parsed out of the text is
   the cheaper way (`parse` in the script; the first version took the first brace and mis-scored 9 of 12 `cover`
   answers).

So the spatial awareness Jev lacks is not bought by handing Haiku the map: at one second Haiku picks what code
measured, as Jev does, but reads the facts in a wider frame (one call over every actor, the packet in the same
context) and can be asked the compound question ("which units are strung out and where should they gather"). The
geometry that is not there yet, the gap in a line, a route's exposure, a site's coverage, is code to write in the
picture's words. Where a decision can wait 3-6 s (a turret's placement, restructuring a line), one slow call with
thinking on and the bare numbers gets it right.

## What this does not say

Mock instances with clean geometry, one turret kind, no terrain; 12 instances a cell, so a difference under about
0.15 is noise; nothing played in the engine.
