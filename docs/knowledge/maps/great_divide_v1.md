# Great Divide V1

Read 2026-09-27 from the map as the bot sees it (`rehearsal-2v1-realtime`'s first `map` picture: terrain, spots,
passages), our three team games there (2026-09-20, `docs/experiments.md` team-2v2-*, commander-team-1), the public
replay listing (`run/replays/pick.py`, 2026-09-27) and the user's notes from experienced players. The user, 2026-09-27:
this is one of the maps the coming games with people will be on; a person there "will be trying to use the terrain to
set up a choke to win from there". No card from a public replay yet (four low-OS duels kept, none replayed).

## The ground

3072 wide by 4096 tall, north box A1-H2 (rows 1-2, our seats in the arena and the rehearsal), south box A7-H8. Rows
1-3 and 6-8 are open ground; rows 4-5 are the divide, cliff across the whole width but one walkable pass at E5
(1570, 2085), 644 wide, at 63% of the way from the north start to the south. Pockets at A5-C5 and G4-H5 are cut off
from the north (`x` in the picture: reachable from the south side only, the east strip G5-H5 opens southward). No
water. Wind 0-20.

Spots, 22, eleven a side, by the bot's walking distance from the north-west start (320, 315):

- North, home: spot_3 B1 (128), spot_2 A1 (182), spot_0 B1 (342), spot_7 C2 (921), spot_9 B3 (1196).
- North, the middle and east: spot_6 E2 (1676), spot_10 E3 (1712), spot_1 G1 (2185), spot_8 F2 (2227), spot_5 G1
  (2256), spot_4 H1 (2544). A second seat placed in the east of the box has these at its door.
- The pass's south mouth: spot_11 E5 (2732), the one spot between the two sides.
- South: spot_12 G6 (3225), spot_13 C7 (3334), spot_15 C7 (3545), spot_14 F7 (3702), spot_16 B8 (3968), spot_18 A8,
  spot_17 G8, spot_21 B8, spot_20 G8, spot_19 H8 (4169-4499).

## Claims

### K-map-great-divide-v1-one-pass
**Claim.** Every walking route between the two sides goes through the E5 pass, 644 wide; whoever holds it decides who
crosses, and nothing walks round. The map's economy is symmetric (eleven spots a side, the twelfth at the pass's
south mouth), so a side that holds the pass and keeps its eleven is not out-expanded by ground alone.
**Status.** observed (2026-09-27): the bot's own passages list (one entry) and the terrain picture; the spot list.
**Evidence.** `run/matches/1790465040-rehearsal-2v1-realtime/00/strategist-0.jsonl`, the first prompt's `map`.
**Would be wrong if.** Hovers or amphibious units had a way round (no water: they do not), or the cut-off pockets
connected to the north somewhere the 32-column picture does not resolve.
**Used by.** the brief's Great Divide section (`docs/briefs/player.md`).

### K-map-great-divide-v1-the-choke-is-the-plan
**Claim.** A person on this map fortifies the pass (light and heavy turrets first, Gauntlets or Pit Bulls and
artillery behind them later) and wins from there: an army sent into the pass dies to static defence it cannot
outrange, as ours did at Comet Catcher's nest (F3 in the review skill: player-8 15.9k lost for 5.6k, player-9's
commit 10.2k for 1.3k).
**Status.** conjectured (2026-09-27), from the user's word of what the coming opponent intends and our two games
against a nest; not yet seen on this map.
**Would be wrong if.** The public duels on the map showed winners crossing early on tier 1 before any camp stood.
**Used by.** the brief's Great Divide section.

### K-map-great-divide-v1-reach-over-the-divide
**Claim.** Two things pass the divide without walking the pass: aircraft, and fire that outranges the camp. By the
unit table (`reach`, this game's `unit_defs`): Armada Shellshocker 710 and Mauser 820 outrange light turrets (430),
Beamers (490) and heavy turrets (620) but not Pit Bulls (730) or Gauntlets (1220); the Pillager (1300) outranges the
Gauntlet; Cortex the same with Wolverine 710, Tremor 1470 against Punisher 1245. Bombers (Phoenix, Thunder, Shadow:
reach 1280, speed 250) reach anything; gunships (Brawler 380) reach the camp's back. Against air the camp answers
cheaply: flak 850, Ferret 950, Chainsaw 1200.
**Status.** conjectured (2026-09-27): the user, relaying experienced players ("you can fly over the mountains with
air units, and long-range units can be setup to bombard encampments"); the ranges are the game's.
**Would be wrong if.** The camp's own artillery (Gauntlet 1220, Pit Bull 730) sat far enough behind the pass that
Pillagers at the north mouth could not reach it, or a fighter screen made bombers a loss.
**Used by.** the brief's Great Divide section.

### K-map-great-divide-v1-not-air-only
**Claim.** Going all air against the camp loses: flak and fighters are cheap against bombers and gunships, and a side
with no ground army holds neither the pass nor its own eleven spots. Air and artillery are levers over a ground army
that stands at the north mouth as the screen and the threat, not a replacement for it.
**Status.** conjectured (2026-09-27): the user, relaying experienced players ("fully going air-only is a bad move").
**Would be wrong if.** A game on the map were won on air alone against a fortified pass.
**Used by.** the brief's Great Divide section.

### K-map-great-divide-v1-our-team-games-stayed-home
**Claim.** In our three team games on the map (2026-09-20, the heuristic bot, BARb medium) no seat of ours crossed
the pass: commander-team-1 held 5 extractors and 39 soldiers at home for 35 minutes against a standing opponent, and
the Sonnet commander never named the pass (it was shown the terrain picture once and never opened `map`).
**Status.** observed (2026-09-20), `docs/experiments.md` rows team-2v2-smoke, team-2v2-board, commander-team-1.
**Evidence.** `run/matches/1789914919-commander-team-1/00` (`run/analyze_match.py`: 5 extractors from minute 5 to
35, army 6-7k against under 2.5k seen).
**Would be wrong if.** Those games had truth files showing the opponent across the pass (they have none).
**Used by.** the brief's Great Divide section (name the pass in the packet; two seats take all eleven north spots).
