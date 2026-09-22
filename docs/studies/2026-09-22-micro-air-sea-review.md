# Review: the hands, the lane and the routes against aircraft, hovercraft and ships

Written 2026-09-22 by Claude (Fable 5.1) on branch `roster`, from the code only (no game run). The question (the user):
"the micro system will be good to go on those new unit types. Air and sea units move differently." The three games the
findings are ranked against: **A** kill the commander with Stormbringers (armthund, from armap), **B** with Bulls
(armbull, from armavp), **C** with Crocodiles and Possums (armanac, armmh, from armhp). Line numbers are the worktree's.

The short answer: nothing in the bot knows a movement domain. `is_army` (mod.rs:495) admits every armed mobile unit,
so every rule written for a Pawn now runs on a bomber, on ground fields built for a Pawn, with combat numbers that a
bomber does not have. Game A is the one that breaks; B and C mostly degrade.

## Findings, worst first

### 1. The lane steps aircraft about on the ground grid and breaks their bombing runs (A)

- `micro.rs:328` admits aircraft to the lane (`is_army`); `flee` (378-427) sends a `Move` to the least threatened
  *ground* cell within 100 (`passable()` is the Pawn's, 405-406) and re-issues it every 6 frames / 32 elmos. A
  Stormbringer at 250 elmos/s covers 125 elmos in the 15-frame lookahead (379), so it is always "heading into" any
  stamped reach it approaches, and the step is 100 elmos: a new `Move` every tick, each of which `CAirCAI::ExecuteMove`
  finishes at once (it is inside the plane's turn radius, AirCAI.cpp:262-274), emptying the queue.
- `fan` (449-495) consults no commitment and fires on every soldier inside the commander's D-gun reach + 260. A bomber
  approaching the commander, the one thing game A is for, is given sidesteps `snap_to_reachable`'d onto the ground
  (475) every 6 frames while inside 510 of it; its run never lines up.
- A `move_to` group is `Commitment::None` (241-258): its bombers flee every remembered turret (LLT, 430 reach +
  90 tail) on the way in, whether or not that turret can hit them (see 2).

Fix: the lane skips units without a move class until it has air rules. At `micro.rs:328`:
```rust
let soldiers: Vec<&OwnUnit> = snapshot.own_units.iter()
    .filter(|u| !u.being_built && self.is_army(u, &kit) && self.world.def(u.def).is_some_and(|d| d.move_class.is_some()))
    .collect();
```
and the same guard in `note_standing_orders` (205) so a bomber's order is never "given back" by `release`. Hovercraft
stay in the lane (their footwork is a tank's) but need finding 6's fields for the flee step.

### 2. Anti-air is invisible to the threat grid; ground guns are stamped as threats to aircraft (A, and B against AA)

- `combatsim/src/units.rs:85-94` `hits_ground()` drops every `onlytargetcategory = VTOL` weapon; `reach()`/`dps()` (152-165)
  sum only those. `threat_sources` (`micro.rs:676-680`) requires `reach() > 0`, so a Ferret (950) or Nettle (765) is
  never a `Source`, never stamped, never fled, never priced. For a bomber group the one threat that matters is the one
  the grid cannot see.
- Conversely a Bull's cannon and the Possum's launcher are `SURFACE` (armbull.lua:141, armmh.lua:150) and cannot touch
  air, yet every armed enemy in sight is stamped as a threat to every soldier of ours. The LLT and the D-gun are
  `NOTSUB` (armcom.lua:297-306; combatsim table) so by category they *may* hit aircraft; whether a beam laser reaches
  cruise altitude 165 is an engine question (see open questions).
- `known_enemy_force` (`army.rs:299-323`) adds every `weapon_count > 0` building's metal to `turret_metal` at
  `TURRET_WORTH` 3: Ferrets weigh against Bulls, LLTs against bombers, and `base_words` (`menu.rs:426-435`) tells Jev
  "we outweigh it" for six bombers against four Ferrets.

Fix: give `Unit` a second pair, `reach_air()`/`dps_air()` (weapons whose category admits VTOL: everything but `SURFACE`,
`NOTAIR`, `NOTSUB`-with-`canattackground=false` aside), keep two grids in `Lane` (`ground`, `air`), stamp each source
into the grid its weapons reach, and read the grid by the unit's domain. `Force` gets `turret_metal_air`; `power`
(`combat.rs:60-82`) picks by the opposing force's domain.

### 3. Nothing can order the new units to attack the commander (A, B, C)

- The group menu has no "attack this unit" option: `engage` is `Fight` to the party's centre (`hands.rs:235`), `fight_to`
  a `Fight` to a place. A bomber on `CMD_FIGHT` bombs the *closest valid target* within 500 × moveState of a point 20
  frames ahead on its line (AirCAI.cpp:360-378), so a Fight at the commander's party bombs whichever wind or extractor
  is nearest that point first. (For ground units the engine's fight-move picks targets the same way.)
- The only `Command::Attack` issuer is `focus` (`micro.rs:502-586`), and it excludes aircraft: `survey_sim_defs`
  (`contact.rs:145`) skips every mobile unit with no move class, so `sim_stats(armthund)` is `None` and a bomber is
  never a shooter. Hovers get a stand-in (finding 4). `focus` also ranks "what the brain told a member to attack"
  first (536, 541), so a standing `Attack` order on the commander would make the whole group converge on it, if
  anything issued one.

Fix: a group option `attack_unit` (whom = a party; target = its dearest member, or the commander when it is in the
party) issuing `Command::Attack { queue: false }` to every member, kept as the standing order; `GroupTask::Engage`
gains `target: Option<UnitId>` and follow re-issues the `Attack` rather than a `Fight` (see 5). For bombers the
engine keeps attacking until the target dies or is lost (AirCAI.cpp:400-403), which is the behaviour wanted.

### 4. The simulator's table has no hovercraft and no aircraft; hovers get a random ground unit's numbers (C, A)

- `combatsim/data/units.json` holds 156 ground units; 55 Armada mobile units of the glossary are missing, among them
  every hovercraft (armanac, armmh, armch, armah, armsh, armlun) and every aircraft. `armbull` is present.
- `survey_sim_defs` (`contact.rs:148-153`) gives a missing unit the nearest ground unit *by metal* of the same
  mobile/armed kind. A Crocodile (370 reach, 70 dps per the glossary) and a Possum (710 reach, 50 dps) are then
  kited, focused and priced with some 300-metal bot's reach and dps: `kite` (604) compares the wrong reaches, `focus`
  (512, 538, 554) puts shooters on targets "in reach" that are not, the flee's `friends_dps` (345) is wrong, and an
  enemy hover stamps the wrong disc.

Fix: regenerate the table for every reachable mobile unit (the glossary's `run/unit_stats.py` already parses the unit
Lua; the simulator's `Unit` needs `weapons[]` with categories, `air`, and the movedef's slope/depth, all in the same
files), and make `sim_stats` fall back to the glossary's `range`/`dps`/`speed` for a name the table lacks rather than
a stand-in: a stand-in was right for "a Pawn of theirs of a type we lack" and is wrong for a class we now field.

### 5. Groups mix domains; the march "holds" a bomber by emptying its queue, which is an attack order (A)

- Adoption is by distance to the nearest centre (`groups.rs:119-136`): bombers rolling off an aircraft plant in the
  yard join the ground ball beside it. `centre_of` (93-99) then averages a plane at 250 elmos/s with tanks at 62, so
  `ARRIVED` (175), `H-HANDS-STALL` progress (169-172) and `nearest_of` for `split`/`send_against` are all off.
- `march` (`march.rs:37-44`) stops the leaders: `Command::Stop`. For an aircraft an empty queue is not a hold: with
  `AUTO_GENERATE_ATTACK_ORDERS` (AirCAI.cpp:27, 185-190, 224-260) a plane with fire-at-will and not hold-position
  attacks the closest valid target within 1000 × moveState on its next slow update. The "held" bombers go and bomb
  whatever is nearest, one at a time, into the AA. Every `Hold` (`hands.rs:217`) and `give_up` `Stop` (`groups.rs:184`)
  does the same; the `hold` option's words "stand where it is" (`menu.rs:436`) are false for aircraft.
- `Engage` follow (`groups.rs:200-204`) re-issues `Fight` every 2 s when the party moves 150: each one restarts a
  bomber's approach (`ExecuteFight` re-selects a target from the new line).

Fix: adopt by domain (`move_class.map(|m| m.kind)`, `None` = air): a newcomer joins only groups of its own domain, and
`join_group_*`/merge (139-156) never cross domains. Aircraft skip `march` (exclude them from `distances` and from
`held`). `Hold` for an air group means a `Move` to the centre with move state hold-position (a new
`Command::SetMoveState`), or is left as "patrols here" in the words.

### 6. One movement class for every soldier: fields, snapping and reachability are the Pawn's (B, C)

- `survey` builds `passable`, `from_home`, `from_enemy` and the spot fields from `kit.raider`'s class only
  (`routes.rs:186-234`); `walker_of` (343-345) knows two walkers, Bots and Commander. Bull is `HTANK4`
  (movedefs.lua:267: `maxslope = SLOPE.MINIMUM` = 27) against the Pawn's `BOT2` (`SLOPE.DIFFICULT`; the shim reports
  105 for armpw and 28 for armbull in the simulator's scale). Every "reachable" judgement for a Bull group is a Pawn's:
  `snap_to_reachable` (457, used by `move_to`/`split`/`scout`/`fan`/`kite`), `reachable_on_foot` (399: which spots
  are places at all, `picture.rs:353,387`), `on_the_way_to`/`forward_of_home` (the turret line and the yard), and the
  flee step's `passable` (`micro.rs:405`). A `fight_to` a place a Pawn reaches over a slope a Bull cannot stalls
  until `GIVE_UP_FRAMES` (90 s) and a flee step onto such a slope is a move failure.
- Hovercraft cross water (`terrain/src/lib.rs:52` has the rule) but no field is ever built for `MoveKind::Hover`, so
  for a Crocodile group water is impassable, an island spot is never a place, and a mark in the water reads as
  unreachable to the picture's checks. `Ship` likewise (53; and `depth` is `minWaterDepth` for ships,
  MoveDefHandler.cpp:242, which `passable` reads correctly).
- Aircraft have `move_class: None`: `walker_of` gives them the Bots fields; `seconds_to_spot` (360-363) prices a
  Construction Aircraft's trip along a walking route at 136 elmos/s, and its `walk_to` and `BuildingAt` sites are
  snapped to walkable ground (`hands.rs:101,126`).

Fix: `Walker` becomes `enum Walker { Air, Ground(MoveClassKey) }` keyed by (kind, max_slope, depth); `survey` builds
`passable`+`from_home` per distinct class among the definitions we can build (lazily, on the field thread), and the
spot fields likewise; `walker_of(def)` reads `def.move_class`. `snap_to_reachable(pos, walker)`, `reachable(pos,
walker)` and `passable(walker)` take the walker; for `Air` they return the point, `true`, and `None`. Group calls use
the group's domain (finding 5). The picture lists a spot as a place when *any* builder we have can reach it, and says
which cannot.

### 7. Air constructors and sea yards on the builder menu (A's aircraft plant is fine; the shipyard is not)

- `place_planned` (`economy.rs:371-391`) puts every factory at `beside_builder(forward_of_home(LAB_YARD))`: right
  for armap and armhp on land. A shipyard passes the `on_water` gate (`menu.rs:254`, `water_within` reach+300) and is
  then anchored on land with `search_radius` 1000 (`build_site_for`, 520): a `BuildSiteNotFound` or a yard on the
  wrong side of the map. Fix: for an `on_water` definition anchor at the nearest under-water cell within reach
  (`world.water_within` already scans them; return the cell).
- `gap_around` (534-536) gives `LAB_GAP` only to `kit.lab`/`kit.plant`; armap (9×6), armavp and armhp (6×6 with
  exit-only `e` cells, UnitDef.cpp:855) get `BUILDING_GAP` 5, so neighbours may sit on the hover platform's exit
  lane. Fix: `if self.world.is_factory_def(def) { LAB_GAP }`.
- armca's `free_spots` (`menu.rs:496-514`) and words say "s of walking" on the Pawn's routes (finding 6).

### 8. Parties, odds and the picture do not say what can hit what (A, C)

- `enemy_parties` (`picture.rs:216`) includes enemy aircraft; `parties` in `contact.rs:171` excludes them. Under the
  pianist only the former runs: Jev is offered `engage`/`send_against` on a gunship party with any group, and
  `odds_words` (257-277) prices it by metal with `Matchups` that have no row for any air, hover or tier-2 vehicle
  (`combat.rs:34-38`: missing = margin 0 = parity). The party's `composition` names types but not "flies".
- `class_words`/`unit_words` carry the glossary class ("bomber", "line hovercraft"), so the *roster* reads right;
  the odds beside it do not.

Fix: `Force` split by domain and the turret split of finding 2; `odds_words` says "cannot hit it" when one side has
no weapon for the other's domain; parties carry a `flies` flag in their words.

### 9. Enemy aircraft are never remembered as threats, and hits from them read as shelling (A, B)

`threat_sources` (`micro.rs:687-699`) drops any source with `stats == None`, and `lane.seen` (696) only records
sources, so a Banshee that dived and left is forgotten at once; `enemy_soldiers` (`briefing.rs:167`) does keep it.
Fix follows finding 2 (air-capable stats for enemy gunships) and 4 (the table).

### 10. The record cannot tell a bomber from a Pawn (all)

`recorder.rs:333-345` `class()`: an armed aircraft is "army", armca "builder". `run/floor.py`'s raider-reaction and
army-share rows will count bombers as ground army. Fix: `"aircraft"`/`"air_builder"` when `move_class.is_none() &&
speed > 0`, `"hover"`/`"ship"` from the kind; the viewer gets a glyph.

### 11. Rot found beside the above (not air-specific)

- `picture.rs:463-466`: the mark warning `off > 150.0` can never fire. `Field::snap` (`terrain/src/lib.rs:157-159`)
  only searches `SNAP_CELLS` = 6 cells (96 elmos at the 16-elmo cell) and returns `None` beyond that, and
  `snap_to_reachable` then returns the point unchanged, so `off` is either ≤ 96 or 0. A mark in open water is passed
  raw to `Move`/`Fight` (which is right for air and hover and a 90-second stall for tanks, with no warning).
- `menu.rs:297-302` offers a builder the `attack` when `force_of([unit])` outweighs the party: `Force::add` counts
  metal with no weapon check, so an unarmed constructor (armck, armca) can be told to `Fight` a lone scout; the engine
  rejects `CMD_FIGHT` for a unit without `canFight` (AirCAI.cpp:284 asserts it). Guard with `weapon_count > 0`.

## What is fine as it is

- `Command::Fight` works for bombers as an attack-move: `CAirCAI::ExecuteFight` picks a valid target ahead on the line
  and pushes an internal `CMD_ATTACK` (AirCAI.cpp:279-381); `IsValidTarget` (543-549) keeps non-fighters off enemy
  aircraft. `Command::Attack` on a unit is honoured until it dies or is lost (384-403). `Move` finishes inside the
  turn radius (262-274).
- `kite` never claims an aircraft (no `sim_stats`), which is right: a bomber has no standoff weapon; `focus` never
  claims one either (right for the wrong reason, see 3).
- `note_standing_orders` (`micro.rs:201-216`) and `release` handle any armed mobile unit; nothing there assumes ground.
- `Pick::Engage` uses the party's true centre, not a snapped point (`hands.rs:235`); the commander's retreat and
  waypoint (`mod.rs:394-451`, `routes.rs:319-340`) touch only the commander, whose class is its own.
- `terrain::passable` has the Hover (slope-or-water) and Ship (depth) rules and `MoveKind` from the shim maps the
  engine's four speed-mod classes in order (engine.rs:170-187); `UnitDef_isMoveDataAvailable` is false for aircraft
  (assumed; the shim's `None` comment says so). The `Terrain` heights sign (water < 0) matches `water_within`.
- The glossary has every unit of the three games with the right flags (`flies`, `hover`+`floats`, none for the Bull;
  `on_water` for the shipyard), and the `on_water` gate (`menu.rs:254`) keeps the shipyard off a dry map's menu. The
  policy's legality rule reaches `armap`, `armhp`, `armavp` and their units by internal name.
- Enemy aircraft with weapons enter `enemy_soldiers` and the picture's parties; the `flies` word is available from the
  glossary for the fix in 8.
- `hands.rs:138` `RetreatHome` for a non-commander goes straight home: fine for air.

## Open questions about the engine (not settled from the source)

1. Do `NOTSUB` beam lasers (LLT, the commander's laser) and the `DGun` hit a bomber at cruise altitude 165? The
   categories allow it; whether the weapon can elevate is in the weapon's `turret`/`tolerance` handling
   (rts/Sim/Weapons/Weapon.cpp `TryTarget`). A duel (`docs/harness/duels.md`) settles it in a minute.
2. BAR's default move state for aircraft (the auto-attack radius is 1000 × moveState, AirCAI.cpp:228,246): not found in
   `luarules/gadgets`; likely the Chobby/widget default states. If maneuver (1), idle bombers hunt within 1000.
3. Where an idle bomber lands: `autoLand` in `CStrafeAirMoveType::Update` (the landing branch, StrafeAirMoveType.cpp
   ~466-520) or BAR's air-repair-pad gadget. Affects whether "idle" aircraft sit under our AA or theirs.
4. Whether `Unit_getCurrentCommands` counts the internal auto-generated `CMD_ATTACK` (it is pushed to `commandQue`,
   AirCAI.cpp:254, so probably yes: an auto-bombing plane reads as not idle).
5. A commander walking on the sea floor (the briefing's amphibious retreat): does the game give it a category the
   bomb's `NOTSUB` excludes? If so game A cannot end while it is under water, and the picture should say so.
6. The exit-only `e` cells of armhp (UnitDef.cpp:855): whether a building within `BUILDING_GAP` of the platform's
   edge blocks hovercraft leaving it.
