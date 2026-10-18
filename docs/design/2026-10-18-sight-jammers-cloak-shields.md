# Sight, jammers, cloak and shields: what the engine knows that the bot did not (2026-10-18)

Status: built 2026-10-18 in four commits (protocol e49db76, brain 62fea2a, words 0d70750, briefs: the one that cites this line); the shim must be reinstalled (`run/install_to_bar.sh`) before any of it reaches a game.

## What human-8v8-1 showed

The Legion base at F1 to H2 on Altored Divide was invisible to both our seats for the last twenty minutes. The player
and the commander wrote "a Bastion we can't see", "fire we couldn't see", "the Sol Invictus, the Bastions and the
Astraeus: no sight" turn after turn, planned every assault on "radar sight", and the artillery stood blind. The
record's heightmap (`scratchpad ray.py`) shows why the advanced radar at G4 saw nothing: a ridge at z ~2,330 rising to
450 to 500 elmos lies between it and the north-east; 19 of 1,984 64-elmo squares of E1 to H2 within its 3,500 were in
its line. The engine raycasts sight and radar over terrain (`LosHandler.cpp:92`); jammers are plain circles on top of
that (six Erebus towers were seen there, 760 each). The user, spectating, saw the Soteria shield bubbles and the fire
from out of sight and said in chat "within reason really doesn't know what to do with the shields" and "probably just
attack the thugs into the defence".

The user's rulings (2026-10-18): "The various mountains and hills block line of sight for radar, and the advanced radar
at G4 was blocked from seeing the enemy whether or not they were jammed. The shield bubbles and the fact that our units
were constantly shelled from those locations should have been a dead giveaway however, and both should have been
knowable from our seats. Let's check what parts of that we can get by expanding our shim, then audit for ways in which
jammers and cloaked units break the assumptions our bot layers operate on." Then: "Let's implement all four."

## What the engine's AI interface offers (read 2026-10-18)

- Our side's true coverage grids, terrain-cut: `Map_getLosMap` (BAR `losMipLevel` 3: 64-elmo squares),
  `Map_getRadarMap` (`radarMipLevel` 2: 32 elmos), `Map_getAirLosMap` (`airMipLevel` 4: 128). Index 0 is the
  north-west; a zero means no coverage. The ally team's, so another seat's eyes count.
- Enemy jamming is not exposed: BAR sets `separateJammers = true`, so `Map_getJammerMap` is our own side's jamming.
  Enemy jam is inferred from seen jammers and `UnitDef_getJammerRadius`.
- Per definition: `UnitDef_getLosRadius`, `getJammerRadius`, `isStealth`, `isAbleToCloak`, `isStartCloaked`,
  `getCloakCost`, `getDecloakDistance`, `isDecloakOnFire`, `getShieldDef` then `WeaponDef_Shield_getRadius`,
  `Shield_getPower`, `Shield_getInterceptType`, `isExteriorShield`; per weapon `WeaponDef_getInterceptedByShieldType`.
  A shield stops a shot when `shield.interceptType & weapon.interceptedByShieldType != 0` (`WeaponDef.cpp`):
  Cannon 1, LaserCannon and BeamLaser 2, MissileLauncher 4 (BAR sets StarburstLauncher to 1024 in
  `alldefs_post.lua`); the Soteria, Keeper and Overseer intercept 1, so plasma is stopped and rockets, missiles and
  lasers pass; `exterior` shields stop only shots from outside the bubble.
- Per unit: `Unit_isCloaked` (in LOS only), `Unit_Weapon_getShieldPower`.
- The damage event already gives the bearing and the weapon; the attacker id arrives only in LOS or radar
  (`EngineOutHandler.cpp`). Enemy-damaged events arrive only while the target is in sight or radar.

## The audit: where the layers assumed flat sight or that unseen means empty

- Sight is a circle from the glossary (`scout.rs`, `DEFAULT_SIGHT` 350): the scouting block, spot staleness and the
  "gone" rule (`briefing.rs` `track_enemy_buildings`: a remembered building whose place a unit of ours stands within
  0.8 sight of, not in the snapshot, is gone) all rest on it. A cloaked pop-up beside our units is declared gone by it.
- Radar contacts are counted in parties (`combat.rs` `UNIDENTIFIED_METAL`) but excluded from "enemy in sight"
  (`schedule.rs`) and hunt targets (`execute.rs`); no words said whether a quiet cell was radar-covered or dark.
- The `radar_jammer` glossary flag was read once, for placing ours; `stealth` nowhere; the protocol carried no cloak,
  jam, sight or shield field; a hunt ended on six seconds out of sight whatever the quarry (`micro/src/lib.rs`).
- `shelling.rs` named the weapon, bearing and range and attributed to buildings already seen; nothing kept a standing
  memory of where fire came from, and the brief told the player never to make `shelling` a destination.
- The reclaim and wreck wakes judged safety by enemies in sight; the record's `xi` dropped damage with no attacker id.

## The build, four parts

1. **Shim and protocol.** `Snapshot.sensors: Option<Sensors>` every second (`SENSORS_INTERVAL` 30 frames): three
   bit grids (`Grid { cols, rows, bits }`), LOS, radar, air. `UnitDefInfo` gains `sight`, `jam_range`, `stealth`,
   `cloak: Option<Cloak { cost, decloak_distance, decloak_on_fire, starts_cloaked }>`,
   `shield: Option<Shield { radius, power, intercepts, exterior }>` and `intercepted_by` (the OR over its ordinary
   weapons). The recorder writes the new definition fields in the header and a `sense` row every 10 s with the grids
   as base64 bitmaps; readers skip rows they do not know. `DESIGN.md` and `record-format.md` say so. The shim must be
   reinstalled (`run/install_to_bar.sh`, the user's step) for any of it to arrive; without it every field reads its
   default and the brain falls back to the circle.
2. **Brain.** With a fresh grid, the LOS grid marks the ground and the spots seen (the circle stays as the fallback);
   `CellLook` gains `radar_share`, `contacts` (radar dots in the cell) and `jammed` (a seen jammer's ring covers its
   centre); the "gone" rule needs the engine's LOS over the place and never fires for a cloak-capable definition unless
   one of ours stands within its decloak distance; a hunt on a cloak-capable quarry runs `HUNT_HIDDEN_FRAMES` (30 s)
   out of sight before it ends, and ends with the word "hidden" (code-decisions §2, the user's word above).
3. **Words.** `guns.rs`: fire from out of sight becomes standing places `guns_N` (weapon, range, estimate, hits,
   damage, since, last), kept ten minutes past the last hit, dropped when ours stand on the place; a `fire from out of
   sight:` line in both reports and the cell's scouting entry; a `shields of his:` line and the picture's
   `their_buildings_near` words say the radius and which of our shots it stops; a group entry `firing:` says shots in
   the last 30 s and whether any landed on his units in sight.
4. **Briefs.** `player.md`'s sight passage: terrain cuts sight and radar; the report's radar words; jammers and the
   Juno; cameras and spotters; shields; `guns_N`; walking in. `commander.md` names the three lines as the side's.

## Not in this round

- A wake for shots with nothing landing (words only for now).
- Dropping a `guns` place when the attributed building dies out of sight (the ten-minute memory covers it).
- The hands offering a camera or Juno as a move: the player orders them by `queue` and `fire` already.
- Reading `Unit_isCloaked` or shield power per unit.

## Measured

Nothing yet: the next game with the reinstalled shim is the first measurement.

## Measured: sight-regress-1 (2026-10-18)

The first game with the grids (Comet Catcher, BARb hard_aggressive, won 19:06) carried every new line, and the
gone rule misfired: 107 "is gone" events, 100 naming a building still standing, 96 of them on the thinks that
hold a 15-frame-old grid (the brain thinks every 15 frames, the grid arrives every 30), the rest at the same-frame thinks, where the engine's enemy list runs one frame behind its LOS map (K-scout-sight-flags-lag). The circle rule had the same fault at a tenth of the rate (18 and 9 firings in the two
games before). Fixed the same day: the rule needs two seconds of continuous in-sight absence (`GONE_DWELL_FRAMES`).
Also found and fixed: the weapon words doubled the type name ("a armjanus (armjanus)"), and the `radar now` line
listed sixty cells by runs at the start (now "every cell out of our sight now (60 cells)").

## Measured: apex-v032-altored-3v3-1 (2026-10-18, the dwell alone)

The first team game on the arc (3v3 against Apex v0.3.2 on Altored Divide, won 13:00 on the referee's call): every
new line in all three seats' and the commander's reports, the radar line's "every cell out of our sight now (12
cells)" wording, 50-odd `guns_N` places. The gone rule with the two-second dwell fired 12 times in 13 minutes, 6
of them on buildings still standing by unit id (5 unknowable: pooled from an ally or built inside our sight), with
ticks never late and the grid arriving every 30 frames on the dot; in the one traced, our nearest unit stood 330
from the extractor when it left the enemy list while the grid still covered its square. The grid is the ally
team's, the current frame's (`GET_SENSOR_MAP`), and agrees with the enemy list in 99.4% of same-frame checks; the
engine refreshes its visibility flags every frame, one frame behind the map, so the lag is ours (the 15-frame-old
grid on every other think). The memory held 86% of the truth's buildings in cells seen within the minute (83% in the
+50 game) against sight-regress-1's 61%; the unit-id count of misfires is an upper bound (buildings first seen while
being built raise no `enemy_seen` row). The rule now needs the engine's grid, this seat's own sight circle and the
dwell together, and writes its evidence into the event.

