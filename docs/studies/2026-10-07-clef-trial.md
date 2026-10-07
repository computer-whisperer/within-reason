# Cloudflare Clef on our Jev tasks: the hosted trial (2026-10-07)

Workers AI, paid tier, from this machine, the day the user made the token. Two scripts: `run/clef_replay.py`
(a recorded game's calls through Clef, compared with Jev's recorded answers) and `run/clef_spatial.py` (the spatial
battery of `2026-10-07-haiku-spatial.md` as typed questions, with the pictures through Clef's `images` field, and
Jev on the text encodings for the comparison the user asked for). Spend: $1.22 on the replay, about $0.16 on the
battery, $0.01 of Jev.

## 1. The replay: human-26 seat 0, all 399 calls (2.84M Jev tokens, 3.72M Clef tokens), every 10th sent twice

The request is what the hands sent Jev: the logged state with the packet and the standing text put back, the
questions as logged; a call past 64 questions (Clef's limit, enforced: "Dictionary should have at most 64 items")
goes as batches of 64 with their latencies summed, as the pianist would wait.

| | clef-flash | clef | Jev (recorded) |
|---|---|---|---|
| latency, median, calls under 8k tokens (n=293) | 428 ms (p90 605) | 888 ms (p90 1,334) | 181 ms |
| 8-16k (n=56) | 858 ms | 2,074 ms | 219 ms |
| 16-32k (n=39) | 1,780 ms | 4,611 ms | 271 ms |
| 32k and over (n=11) | 2,775 ms | 7,189 ms | 356 ms |
| cost of the game's seat | $0.33 | $0.89 | about $0.12 |
| nouls on the same side of 0.5 (7,359 compared) | 0.85 | 0.86 | |
| median absolute difference in a noul | 0.13 | 0.13 | |
| nouls Jev put over 0.5: 1,128; Clef over 0.5 on | 224 (0.20) | 259 (0.23) | |
| nouls Clef put over 0.5 | 443, Jev over 0.5 on 224 (0.51) | 391, Jev over 0.5 on 259 (0.66) | |
| choices, same pick (437 compared) | 0.64 | 0.67 | |
| Clef's median probability for Jev's pick | 0.57 | 0.58 | |
| the keep world (w1) picked | 100 of 437 | 230 of 437 | 168 of 437 |
| repeated calls (40): nouls that moved | none (median difference 0.000) | none | Jev wobbles hundredths between minutes (K-jev-answers-drift) |

What the disagreements are. Of Jev's over-0.5 nouls that Clef puts under, 558 (flash) / 516 (clef) are the hunt's
`forbidden` marks, which Jev reads at 0.75-0.81 and Clef at 0.4-0.7 on the median gate; then `change` (105 / 72)
and `place` (85 / 113). The choice disagreements run opposite ways by model: where flash and Jev differ (157
picks), Jev chose keep in 88 and flash in 20, so flash leans to a change; where Clef and Jev differ (143), Jev chose
keep in 22 and Clef in 84, so Clef leans to keep. Neither is "wrong" on the evidence here: the same-side rate on
nouls is dominated by the many nouls both put low, and the picks have no human verdict beside them in this run.

## 2. The spatial battery, as typed questions (12 instances a kind, seed 1, the same truth)

The free position of `gap` is not asked (a decision model gives no numbers). `cover` is a noul per site, the top
two taken; `rally` a noul per unit and a choice over the points; `gap` and `route` a choice.

| kind, measure | encoding | clef-flash | clef | Jev | Haiku, thinking off, free text (for reference) |
|---|---|---|---|---|---|
| gap: the spot | coords | 0.17 | 0.58 | **0.92** | 1.00 (2.0 s) |
| | cells | 0.25 | 0.17 | 0.25 | 0.67 |
| | ascii | 0.25 | 0.75 | 0.92 | 0.75 |
| | facts | 0.08 | 0.67 | **1.00** | 1.00 |
| | image / image_coords / image_reach | 0.75 / 0.50 / 0.75 | 0.50 / 0.75 / **1.00** | (no images) | 1.00 (with working) |
| cover: covered / best | coords | 0.46 | 0.60 | 0.64 | 1.00 (4.4 s) |
| | facts | 0.66 | 0.89 | **0.93** | 1.00 (0.6 s) |
| | image / image_coords / image_reach | 0.57 / 0.60 / 0.48 | 0.62 / 0.57 / 0.57 | | 0.97 / 0.96 / 1.00 |
| rally: stragglers / point | coords | 0.97 / 0.75 | 0.93 / 0.83 | 0.98 / 0.83 | 1.00 / 0.92 (0.6 s) |
| | facts | 0.95 / 0.92 | 0.96 / 0.92 | **1.00 / 1.00** | 1.00 / 1.00 |
| | image / image_coords / image_reach | 0.65 / 0.17, 0.97 / 0.67, 0.97 / 0.58 | 0.61 / 0.17, 0.98 / 0.92, 1.00 / 0.83 | | 0.63 / 0.50, 0.86 / 0.75, 0.89 / 0.83 |
| route | coords | 0.33 | 0.33 | 0.42 | 0.83 (3.2 s) |
| | facts | 0.33 | **1.00** | **1.00** | 1.00 (0.6 s) |
| | image / image_coords / image_reach | 0.50 / 0.25 / 0.33 | 0.58 / 0.75 / 0.83 | | 1.00 / 0.75 / 1.00 |
| latency, median | text / image | 250-550 ms / 450-700 | 350-520 / 500-720 | 180-330 | 600 ms with facts |

## What it says

1. **Clef speaks our requests unchanged** and answers deterministically (40 repeats, not a hundredth moved), where
   Jev drifts. The 64-question limit is the one client change (batch by count), and the batches cost their sum.
2. **It is slower than Jev from here, 2-3x for flash and 5-20x for Clef**, and the gap widens with size: a 20k-token
   gate is 1.8 s on flash, 4.6 s on Clef, 0.27 s on Jev. The vendor's 39 ms is a short-state number.
3. **It reads our pictures of the game differently, not obviously better.** Same side of the flag on 85% of nouls,
   but Jev's over-0.5 answers are mostly not Clef's (only a fifth are), most of all on the forbidden marks, and the
   picks agree two thirds of the time with flash leaning to change and Clef to keep. Which is right needs the
   reviewed moments (`run/jev_ab.py`'s recorded set with human verdicts), not this run.
4. **On the spatial battery Jev beats both Clef models on every text encoding**, and with code's facts Jev is at
   0.93-1.00 on every kind. Clef-flash is weak on geometry at any encoding; Clef reaches Jev only with the facts on
   `route` and `rally`.
5. **The pictures are Clef's one edge, and only with the reach drawn.** `gap` by image with reach circles: Clef
   1.00 against 0.58-0.67 on its own text; `route` 0.83 against 0.33. The bare picture is worse than the numbers on
   `rally` for every model (0.17 on the point). As with Haiku: a drawing is one more encoding of code's facts, for
   what one look can see.
6. **Cost**: flash is about 3x Jev a game, Clef about 7x; the paid tier is needed past about one flash seat-game a
   day.

So for the pianist's seat today, Jev stays: faster, cheaper, and better on the geometry we can pose in words. Clef's
case is the open weights (fine-tuning on our games, local lockstep play) and a drawn picture for the few spatial
asks where the eye beats the sentence, both untested here.

## What this does not say

One game, one seat, no human verdict beside the disagreements; the battery is synthetic (12 instances a cell, a
difference under 0.15 is noise); Clef's latency is one day's from one machine; nothing played in the engine.
