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

## Images (added the same day, after the user asked whether any real multimodal input was tried)

Three more encodings: `image` (a 1,024-pixel PNG of the map, 8 elmos a pixel, grid every 1,024, named markers, no
numbers in the text), `image_coords` (the PNG and the numbers), `image_reach` (the PNG with every turret's reach drawn
as a circle, and the numbers). Haiku bills the picture at about 1,100 input tokens.

Free text, thinking off, 4,096-token cap:

| kind | image | image_coords | image_reach |
|---|---|---|---|
| gap: spot / free | 1.00 / 1.00, 2.7 s, 440 out | 1.00 / 1.00, 1.8 s | 1.00 / 1.00, 1.8 s |
| cover: ratio | 0.97, 4.1 s, 805 out | 0.96, 4.5 s | 1.00, 4.3 s |
| rally: stragglers / point | 0.63 / 0.50, 0.7 s | 0.86 / 0.75, 0.7 s | 0.89 / 0.83, 0.8 s |
| route | 1.00, 4.3 s, 740 out | 0.75, 4.3 s | 1.00, 3.2 s |

The JSON alone (structured output, no working), thinking off:

| kind | image | image_reach | (coords, from above) |
|---|---|---|---|
| gap: spot / free | 0.83 / 0.83, 1.4 s | 1.00 / 1.00, 1.4 s | 0.67 / 0.75 |
| cover: ratio | 0.53, 1.3 s | 0.68, 1.3 s | 0.66 |
| rally: stragglers / point | 0.63 / 0.33, 1.3 s | 0.92 / 0.75, 1.2 s | 1.00 / 0.83 |
| route | 0.75, 1.3 s | 0.58, 1.3 s | 0.58 |

6. **The picture is a coordinate source to Haiku, not a shortcut.** Given the image alone and room to work, it reads
   pixel positions off the markers ("enemy turrets at pixels A(578,516), B(452,616)...", "reach 450 elmos is about 56
   pixels") and then does the same distance arithmetic as on the numbers, to the same scores (gap 1.00, cover 0.97,
   route 1.00) and the same 3-4 s. Its pixel reads are good to a few pixels, 20-40 elmos.
7. **By eye, at a second, only the drawn gap is reliable.** With the working denied, the one case the picture answers
   is the turret line with the reach circles drawn: the hole between the circles is visible and it scored 1.00 where
   the numbers alone gave 0.67. Cover and route by eye are at the level of the numbers by eye (0.53-0.75), and the
   rally ask is worse on the picture than on the numbers (ten labelled dots to tell apart: 0.33-0.75 against 0.83).
8. **So drawing is for what the eye can see in one look** (a hole in a ring of circles, a body and its stragglers,
   which side of a line something is on), and for those a picture with the geometry drawn by code is one more
   encoding of code's facts. For anything that needs a distance compared with a reach, the numbers plus code's
   measurement remain the one-second path.

## What this does not say

Mock instances with clean geometry, one turret kind, no terrain, synthetic pictures with no terrain or units drawn; 12 instances a cell, so a difference under about
0.15 is noise; nothing played in the engine.
