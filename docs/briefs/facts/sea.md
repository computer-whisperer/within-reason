<!--meta
id: F-sea-intro
layer: fact
status: live
-->
## The sea: what the game's files say of a base and a fight on water

The unit tables carry every ship and sea building; the map model says how deep water must be for each kind and how
water slows what wades. This document adds the three things a seat at the sea needs beside them: what its commander
is under water, who builds the sea kinds, and who out-reaches whom among the tier-1 things that fight there.

<!--meta
id: F-sea-commander
layer: fact
premises: [F-map-ground]
sources: {code: [upstream/Beyond-All-Reason/units/corcom.lua:186-251 (the two lasers: the land one 75 a shot, the under-water one 200 a shot at reload 1 and range 300, `firesubmersed = true`, 100 against submarines), 252-292 (the Disintegrator: `firesubmersed = false`, range 262), 294-308 (the weapon slots), upstream/Beyond-All-Reason/units/armcom.lua:187-239, 265-278 (the same three), run/matches/1794105281-human/00/record-1.jsonl (a Cortex commander 95 under water: 24 to 25 elmos a second over its first thirty seconds)]}
status: live
-->
### The commander under water

Both commanders walk the sea floor at any depth, at two thirds of their land speed from 47 deep (25 elmos a second;
F-map-ground). Submerged, a commander fights with a second laser made for it: 200 damage a shot, one shot a second, at
300, half that against submarines. Its D-gun does not fire while it is under water. So on the floor it is 3,700 health
and 200 damage a second at 300 with no D-gun, and it builds at its full 300 of build power.

<!--meta
id: F-sea-builders
layer: fact
premises: [F-map-build, F-eco-build]
sources: {code: [crates/bot/data/units.json (`made_by`, `build_time`, `build_power`, `speed`, the costs; run/unit_stats.py from the game's unit files at commit 54199a0d3)]}
status: live
-->
### Who builds what at sea

- *The commander* builds the whole tier-1 sea base from the floor: the extractor (at any depth), the tidal generator
  (85 metal for Cortex, 90 for Armada; 7 seconds at its 300), the shipyard (450 metal and 950 energy, 17 seconds: the
  cheapest tier-1 factory, against Cortex's 470 for a bot lab, 570 for a vehicle plant, 630 for an aircraft plant), the
  floating torpedo launcher (170 metal, 12.5 seconds), the floating radar and sonar tower, the floating anti-air
  turret, the naval hovercraft platform, the floating converter and the under-water storages. No solar, wind turbine,
  land plant or land turret stands in water (F-map-build).
- *The construction ship* (from the shipyard: 200 metal, 2,000 energy, 25 seconds at the yard's 150) has 125 of
  build power at speed 60 and builds every sea kind, the naval construction turret and the land plants within its
  reach of the shore. It builds no solar and no wind turbine.
- *The amphibious constructor* (Cortex's Muskrat, Armada's Beaver, from the vehicle plant: 170 and 150 metal, 46 and
  45 seconds at the plant's 150) has 85 and 80 of build power at speed 40 and builds both the land base and the sea
  kinds. It is the slowest and weakest of the tier-1 builders and takes twice a construction ship's time to make.
- *The land constructors* (the construction vehicle and bot) build the shipyard and the coastal torpedo launcher
  from the shore, and an extractor on an under-water spot within their build distance of it. They build no tidal
  generator, no floating turret and no floating radar, and they do not enter water deeper than 20.
- *The construction hovercraft* (from either hovercraft platform, 190 to 200 metal) has 115 and 110 and builds both.

<!--meta
id: F-sea-reach
layer: fact
premises: [F-fight-range]
sources: {code: [crates/bot/data/units.json (range, dps, health, speed, costs), crates/bot/data/units.json `gloss` and `prose` (what each weapon can hit: written from the unit files by a reader and not re-read here: the torpedo launchers hit ships and submarines and nothing on land, in the air or hovering)]}
status: live
-->
### Reach among the tier-1 things that fight at sea

| what | metal | reach | damage a second | health | speed |
|---|---|---|---|---|---|
| gunboat: Supporter (coresupp), Dolphin (armdecade) | 100, 175 | 280 | 91, 135 | 480, 970 | 110, 105 |
| the commander, submerged | | 300 | 200 | 3,700 | 25 |
| patrol boat: Skater (armpt) | 150 | 300 | 66 (71 against air) | 780 | 93 |
| frigate: Riptide (corpship), Ellysaw (armpship) | 420, 380 | 480, 470 | 120, 117 | 2,350, 2,150 | 82 |
| floating torpedo launcher: Urchin (cortl), Harpoon (armtl) | 170 | 500 | 187 | 1,440, 1,300 | |
| submarine: Orca (corsub), Eel (armsub) | 580, 440 | 500 | 137, 86 | 1,060, 840 | 57, 66 |
| coastal torpedo launcher, on land: Jellyfish (cordl), Anemone (armdl) | 250, 240 | 600 | 150 | 2,350, 2,200 | |
| missile boat: Herring (corpt) | 210 | 650 | 24 (60 against air) | 890 | 69 |
| destroyer: Oppressor (corroy), Corsair (armroy) | 960, 880 | 710, 700 | 154, 147 | 3,950, 3,700 | 62, 67 |

A torpedo launcher hits ships and submarines only. The gunboat is among the cheapest damage for its metal
(0.91 and 0.77 a second per metal; an Incisor is 0.78) and a shipyard makes a Supporter in 9 seconds and a Dolphin in 16; the destroyer
out-reaches everything else on the list, the launchers included, and takes 71 seconds of a yard. A yard making
Supporters without pause draws 11 metal and 107 energy a second.
