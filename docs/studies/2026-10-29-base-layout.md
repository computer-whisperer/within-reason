# Base layout: how the people lay a base out, measured (2026-10-29)

The research behind `docs/briefs/derived/base.md` (D-base-layout), asked for by the user on 2026-10-29 as the first
step of the base-design work: find the key patterns and structures of the problem before any process for our play is
reviewed. Tool: `run/base_layout.py` (`pool`, `block`, `summary`). Data: `docs/studies/data/base-layout-2026-10-29/`
(`comet-duels-35os.jsonl`, one line a seat; `gecko-and-ours.jsonl`; `comet-duels.txt`, the 70 replay directories read,
all under `/ceph/christian/within-reason/run/matches-archive`; `unitfacts.json`, footprints, blasts and health parsed
from the game's unit files).

## 1. What was read

- **The pool**: the replay survey's Comet Catcher Remake 1.8 duels with records, 70 games, both seats where the player
  is at 35 OS or above: 118 seats, median game 13:12. Records are the dump widget's (`created`, `finished`,
  `destroyed`, `cmd` with every build order's site, `s` with every own unit's position each second).
- **thebluegecko**: his seat of the Center Command 2v4 (OS 34, Cortex, 21:12; archive
  `1792466765-replay-center-command-2v4b/record-0.jsonl`) and his game against InactiveAI on Comet (21:42; played
  again through `run/replay_match.py` as `run/matches/1793321961-replay-gecko-inactiveai-comet-2026-10-21-base`, the
  first record having gone from `run/matches` and the archive; ledger row of the same name).
- **Ours**: advance-3 and advance-4 (the flagship under the player, seed 1, Comet: one opening measured twice).
- **The game's files**: footprints and yardmaps (`units/`), death blasts (`weapons/Unit_Explosions.lua`), the engine's
  rules for them (`WeaponDef.cpp:71`: `areaOfEffect` is a diameter; `GameHelper.cpp:132-149`: linear falloff to the
  radius, measured to the victim's surface; `Builder.cpp:374`: a builder's reach is `buildDistance + buildeeRadius`;
  `SolidObject.h:38-49`: the yardmap states, `e` exit-only "closed for walk into, closed for build", `b` build-only).
- **The community's writing**, by a research agent (web only): the official guides (economy, advanced mechanics, how
  to start), the unit pages, RogueL1ke's construction-turret analysis, the crdhq articles; nothing found on nano-farm
  layout in so many words, the idiom being "con-turrets" and "build spacing".

## 2. Method

Per seat: the commander's path and still seconds until the first factory stands plus 30 s (at most 150 s); its build
orders and their batching (orders within 2 s of the last are one batch); sites in reach of its arm (145 plus half the
footprint) when ordered; the distinct standing places (within 24 elmos) it built the pre-factory structures from; the
gap from one structure finishing to the next order. Turrets: nearest-neighbour distance at finish; distance to the
nearest standing factory's edge; count within reach (400 plus the factory's half footprint) of each factory at its
death or the game's end; clusters at single linkage 120. Spacing: nearest-neighbour centre distance by kind and
across kinds. Own reclaims: a death marked `reclaimed`, or by one of our own builders. Chain deaths: a building that
died within 3 s of an exploding neighbour whose blast at full damage, by the falloff rule, reaches half the victim's
health. Lanes: a factory's facing from the first products' movement five seconds after they finish; what of its own
stood in a lane 320 deep and the factory's width, with the near edge's depth; the turrets' side of the factory. The
block mode adds the bars (clusters at 100), what stood with the turrets and how many were in reach when each was placed
and finished, the frame times, and the chain bill (how many finished buildings one death at full health kills).

Caveats: the pool's games are short (13 minutes), so its late game is thin (four turrets a seat, no fusion); the chain
count is an arithmetic on positions and full health, not the engine's kill attribution (the record names no attacker
for a blast death, K-mech-missile-and-blast-deaths-unattributed), and its half-health threshold counts likely kills;
footprints are the unit files', the record's `footprint` field being absent in these records; the facing is inferred,
not read (these records carry none).

## 3. The pool (118 seats, 35 OS and above, Comet duels)

| Opening | median | spread |
|---|---|---|
| first factory finished | 1:11 | |
| commander still, of its first 101 s | 87 s | |
| commander path in them | 399 elmos | |
| standing places for every structure up to the plant | 1 (53 seats), 2 (49), 3 (13), more (3) | |
| build orders in the horizon / batches | 12 / 4 | batch depths: 1 (231), 2 (62), 3 (52), 5 (37), 4 (23), 6 (15), 7-8 (14) |
| sites in reach of the arm when ordered | 39 % | |
| step between consecutive sites | 176 | 82 to 329 between quartiles |
| gap, a structure finished to the next order | 0.6 s | 0.3 to 0.8 |
| first six structures | mex mex solar solar vp (37), mex mex solar llt mex solar (33), mex mex mex solar solar solar (8), mex mex solar lab (6) | |

| Turrets and spacing | |
|---|---|
| turrets a seat | 4 median; per factory: bot lab 2 (max 18), vehicle plant 2.5 (19), air plant 3.5 (12), advanced vehicle plant 11 (20) |
| turret to the nearest factory's edge | 129; within 400 of a factory: all, in 98 of 98 seats with turrets |
| turret nearest neighbour | 87 (one footprint and a bit); largest standing cluster 2 |
| solar nearest neighbour | 80 (touching: the footprint is 80) |
| converter nearest neighbour | 48 (touching) |
| energy storage nearest neighbour / to a factory / to a turret | 179 / 227 / 152 |
| solar to the nearest factory | 302 |

| Lanes (225 land factories with a facing read) | |
|---|---|
| facing the enemy's half | 205 |
| with a building of their own in a lane 320 deep | 95 |
| in-lane near edge from the front | quartiles 128 / 176 / 256; 18 % within 100 |
| in-lane kinds | solar 97, extractor 27+, turret 33, plants 10 |
| turrets within reach of a plant, by side | beside 636, behind 296, in front 197 |

| Reclaims and chains | |
|---|---|
| seats that reclaimed something of their own | 62 of 118; 330 solars, 31 vehicle plants, 9 bot labs, 39 turrets, 26 laser towers, 23 extractors; minute median 15:00 |
| building deaths | 5,343; chain victims by the arithmetic 198 (3.7 %) in 56 seats: solars 88, extractors 53, converters 20, turrets 17; sources: the laser tower (80), energy storage (36), converter (21), plants (30+) |

## 4. thebluegecko

**Center Command 2v4 (21:12).** Opening tide, mex, mex, lab from three stands, 5 of 7 sites in reach when ordered,
gaps 6.0 / 2.2 / 1.8 s. 45 turrets, 43 at 48 from their nearest neighbour, in two blocks of 22 (384 x 320) and 20 (240 x
240), 4,500 elmos forward of the first lab where the advanced vehicle plant stood: 36 in reach of it, 30 of the air
plant, 42 of the advanced air plant placed sixteen elmos from the eaten advanced plant (lab 7:07, advanced lab 13:21,
advanced plant 18:45). His 30 turbines and 3 tide generators had no turret in reach. Converters at 48, advanced
converters at 64 (touching); fusions 358 apart and 81 from the nearest turret, a fusion's death killing 24 finished
buildings by the arithmetic; energy storages spread (232).

**Against InactiveAI on Comet (21:42).** Opening mex mex mex solar solar solar vp from two stands, 11 of 14 sites in
reach when ordered, gaps 0.8 / 8.3 / 0.3 / 0.7 / 0.7 / 0.7 s, the plant finished 1:20. 304 turrets, 303 at 48, in bars
four turrets wide (192) and 576 to 960 long (80, 64, 48, 48, 35, 12 in a cluster), the two largest north-south; 155
advanced converters at 64 in rows between the bars (120 from the nearest turret), 30 advanced fusions at 96 (touching
one another, 76 from the nearest turret), 8 fusions at 80 to 96 (69 from the turrets). Turrets in reach when placed:
converters 46 median (max 102), frames 5.5 s; advanced fusions 53 (max 121), frames 35 s; the first of each 13:48 and
13:24. Eight air plants placed inside the block 21:00 to 21:36 with 35 to 87 turrets in reach each. 17 % of his turrets
were ever within 400 of a factory; his 25 solars had none in reach. Reclaims: the plant 4:41, the advanced plant
10:22 (29 turrets in reach then), 17 solars 13:44 to 15:24, 4 advanced solars. Chain bill at full health: an advanced
fusion's death kills 174 finished buildings median (220 at most) and the three advanced fusions it touches; a
converter's eight (five converters); a fusion's 19.

## 5. Ours (advance-3 and advance-4, one opening on seed 1)

mex mex solar solar vp from three stands, 480 elmos walked, gaps 3.5 / 1.0 / 0.8 / 5.8 s, every order alone (the
hands' list issues one step at a time), 4 of 6 sites in reach when ordered; the plant finished 1:03. Turrets 7 and 8
in 26 to 31 minutes, nearest neighbour 181 and 234, 56 to 69 from the plant's edge, no cluster above 3; solars 97 and
128 apart; nothing in any lane (the 320 keep-out); reclaimed a plant once (advance-3, 9:24) and nothing else; no
chain victims.

## 6. What it says

1. The people's opening is a stand, not a walk: everything the arm reaches, queued, from one or two places; dead time
   between structures is under a second. Ours walks three stands and loses eleven seconds to gaps on the same
   sequence.
2. Packing is touching: solars, converters, turrets, advanced fusions all at their footprint. The only spacing the
   people keep is in front of a plant, about a hundred elmos, and round energy storages.
3. The turret pile begins with the advanced plant (two or three on a T1 plant, eleven and up on a T2 one), stands
   where that plant stands, and serves every later plant placed on the same ground; the late block is laid for the
   frames (converters, advanced fusions) and the plants come to it, air plants last.
4. The late block is a powder train by its owner's choice: touching turrets chain once two die together, touching
   advanced fusions chain outright, and one advanced fusion's death takes 174 buildings in his block. The pool pays
   under four percent of its building deaths to chains in duels; the bill is a late-game one.
5. Reclaim buys ground, not metal: a solar's 155 is under a second of a 14-minute income; seventeen solars are
   forty-seven turret footprints.

## 7. What it does not say

Why the people pitch at 48 (habit, the grid's default, or a measured preference), whether a block at 96 costs anything
in frame clocks (no frame in his game had more than 121 turrets in reach, and 68 fit at 96), how the people lay bases
on sloped or windy maps (the pool is Comet), and anything about transport of turrets in play (none seen in these
records; the capability is in the unit files). A pool of longer games at 40 OS and above, and the Glitters 8v8
records, would test the block findings against more than one player.
