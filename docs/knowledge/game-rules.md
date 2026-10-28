# Game rules and engine semantics that shape play

### K-rules-commander-death-ends-game
**Claim.** With the default death mode (`deathmode=com`, "Kill all enemy Commanders") a *side* loses when it has no
commander left: the gadget keeps one live-commander count per ally team and kills every seat of the side when it
reaches zero. A seat whose own commander dies plays on while an allied commander stands; in a duel the side is one
seat, so the game ends with that commander. `own_com` is the mode in which a seat dies with its own commander. A dead
commander's wreck can be resurrected.
**Status.** supported (2026-09-19) for the duel; corrected 2026-10-18 for several seats (the user; the gadget read)
**Evidence.** `luarules/gadgets/game_team_com_ends.lua` (`aliveComCount[allyTeamID]`, `spKillTeam` over the ally
team's teams when it is 0; the `own_com` branch separate), `modoptions.lua` (`deathmode` default `com`, "When a team
has no Commanders left, it loses"). baseline-easy/00, /05, /06 (duels): each match ended right after the commander's
death. human-8v8-1 (a 5v5): his Armada commander died at 34:56 and that seat's Fleas, Pawns and labs went on dying to
our army for the next minute; both of our commanders stood at the end. Until 2026-10-18 the player's role text said
"lose when ours dies" for every game.
**Would be wrong if.** A seat of a several-seat side lost its units when its own commander died under `com`.
**Used by.** H-COM-LEASH, H-COM-RETREAT; the player's role text (the game-end rule); the brief's tier-2 sprint (the
commander reclaimable when the side keeps another).

### K-rules-ai-start-position
**Claim.** The game places AI teams itself inside their start box; an AI does not need to send a start position.
**Status.** supported (2026-09-19)
**Evidence.** `luarules/gadgets/game_initial_spawn.lua` (GuessStartSpot for AI teams, ~line 662); every arena match spawned
at (2032,1188) NW or (5576,5360) SE on Quicksilver.
**Would be wrong if.** A map or lobby setup left our commander unspawned or outside its box.
**Used by.** (none; it is why the protocol has no start-position command)

### K-rules-factory-shift-means-five
**Claim.** For factory build orders the SHIFT option bit means "build five", CTRL "build twenty"; factory orders always append.
**Status.** supported (2026-09-19)
**Evidence.** `rts/Sim/Units/CommandAI/FactoryCAI.cpp:150`. v3-easy/04: 5 constructors and 0 army at minute 4; after the fix
(v4-easy) openings produced fighters first.
**Would be wrong if.** A factory given one unshifted order produced more than one unit.
**Used by.** Enforced in the shim (`engine.rs`, `Command::Build`), so no heuristic can trip on it.

### K-rules-mex-upgrade-in-place
**Claim.** An advanced extractor ordered at the exact position of an existing extractor is allowed, and the old one is
reclaimed when the new one finishes. Advanced extractors yield 4x (extractsmetal 0.004 vs 0.001). An ally's advanced
extractor built over our extractor is given to us when it finishes (human-8v8-1: 22 `given` rows of Apex-built mohos to
our seats from 14:02 to 47:12; both readers took the frames for a harness fault until the review).
**Status.** supported (2026-09-20) through the AI interface: t2-first-look, both factions, a build order for the advanced
extractor at the tier-1 extractor's exact position is accepted (`Map_isPossibleToBuildAt` says yes) and the new one
is created in place (record: `cmd armmoho [5648, 5360]`, `created armmoho 5648 5360`). BARb does the same and falls
back to a site within 32 elmos, then to reclaiming the old one (`CircuitAI` `MexUpTask.cpp`)
**Evidence.** `luarules/gadgets/unit_mex_upgrade_reclaimer.lua`, `cmd_mex_denier.lua:53`; unit files armmex/armmoho.
**Would be wrong if.** The engine rejects the build order because the square is occupied (the AI interface path may differ
from the player UI path).
**Used by.** (planned tier-2 economy; needs an exact-position build in bot-protocol)

### K-rules-impossible-orders-vanish
**Claim.** A build order for something outside the builder's build options is accepted by the AI interface (return code 0) and
then dropped without any event. The builder is idle again on the next tick.
**Status.** supported (2026-09-19)
**Evidence.** dropped-orders batch: 560-890 dropped orders per 20-minute match, 103 of 160 logged ones were the commander told
to build `armadvsol` (constructor-only, see `units/ArmBots/armck.lua` vs `units/armcom.lua`). After guarding by
`UnitDefInfo.build_options` (dropped-orders-2): 7-38 per match, minute-20 metal income +38-44 on 16-19 extractors.
**Would be wrong if.** (mechanical)
**Used by.** Guard in `economy.rs` `run_economy`; H-ECO-ENERGY-BY-STORAGE / H-ECO-FALLBACK-ENERGY pick per builder.

### K-rules-extractor-must-be-exact
**Claim.** The game rejects an extractor order that is not exactly on a metal spot, or where an allied extractor already stands
(`luarules/gadgets/cmd_mex_denier.lua`, `AllowCommand`); the rejection is silent. Letting the engine shift the site to the
"closest buildable" position therefore produces doomed orders when the exact spot is blocked.
**Status.** supported (2026-09-19)
**Evidence.** dropped-orders/01: spot (5640,5352) placed at (5648,5360) was built; three later orders for the same spot were
placed at (5664,5264) and all dropped. Extractor site search radius is now the build-grid snap (16).
**Would be wrong if.** A shifted extractor order ever produced an extractor.
**Used by.** `EXTRACTOR_SNAP` in `economy.rs`.

### K-rules-unreachable-orders-loop
**Claim.** A move, fight or build order to a place the unit cannot path to makes it walk to the nearest reachable point, raise
`UnitMoveFailed`, and go idle. A brain that re-issues the same order to idle units loops forever. Large crowds also raise
move failures by jostling, so failures alone do not prove unreachability.
**Status.** supported (2026-09-19)
**Evidence.** move-failures batches: 1,300-6,800 `UnitMoveFailed` per match. Units stuck at the factory exit (2224,1188) when the
station was 250 ahead of an outpost extractor; constructors re-sent to extractor spots such as (3224,520), (6184,520);
25-57 attackers hovering mid-map for 20 minutes with no enemy in sight.
**Would be wrong if.** (mechanical)
**Used by.** `economy.rs` `note_unreachable_sites`; `army.rs` `note_station_failures`, `note_unreachable_targets` (gives up a
target only after 90 s of failures with no attacker within 600 of it).


### K-rules-wreck-blocks-site-search
**Claim.** `Map_findClosestBuildSite` refuses a position holding a wreck, though `Map_isPossibleToBuildAt` allows it and a
build order there works (the builder reclaims the wreck first). A dead extractor's wreck therefore hides its spot from
the site search for good.
**Status.** supported (2026-09-19)
**Evidence.** nosite-diag: `no site ... wanted=(3336,3256) possible_at_exact=true feature:cormex_dead`.
**Would be wrong if.** An extractor ordered onto a wreck-covered spot were never started.
**Used by.** The shim places extractors with `Map_isPossibleToBuildAt` at the exact spot and searches nowhere.

### K-rules-site-search-ignores-metal-spots
**Claim.** The engine's site search knows nothing of metal spots: base buildings anchored at the start point get put on
the nearest spots, which are then lost to us.
**Status.** supported (2026-09-19)
**Evidence.** nosite-diag: from the SE start every game refused the spot at (5640,5352) from minute 2, and the diagnostic
found three or four of our own wind generators on it. After the keep-out, SE extractors at minute 5 went from 4 to 6.
**Would be wrong if.** "no site" for extractors reappeared with our own buildings listed at the spot.
**Used by.** Shim `SPOT_KEEPOUT` (100 elmos).

### K-rules-early-orders-are-lost
**Claim.** An order given to the commander in the first second of the game (frame 15) is dropped by the engine, and an
order takes more than one tick to show in the unit's command queue, so a brain that re-plans every "idle" builder each
tick overwrites its own fresh orders.
**Status.** supported (2026-09-19)
**Evidence.** Found by the build-order study replaying our recorded openings, confirmed from
`run/matches/1789868387-v17-truth-medium/00/record-0.jsonl`: extractor ordered at f=15, another at f=45, a wind generator
at f=60, each replacing the last; the first extractor was finally built at f=2355. The lost order also held its metal
spot's claim. With no orders before f=60 and a 45-frame grace after each order, both extractors stand at 1:00.
**Would be wrong if.** Records showed overlapping build orders for one builder again.
**Used by.** H-ECO-OPENING (`FIRST_ORDER_FRAME`, `ORDER_GRACE_FRAMES`).

### K-rules-mex-position-is-an-area
**Claim.** An extractor does not have to stand on the spot position the engine reports: BAR accepts any position from
which the extractor's radius covers every metal cell of the patch (`common/upgets/api_resource_spot_finder.lua`
`IsBuildingPositionValid`, enforced by `luarules/gadgets/cmd_mex_denier.lua`). The engine's spot list gives the patch's
top-left best cell, and we always build there.
**Status.** observed (2026-09-20); first noticed by the user in the viewer.
**Evidence.** v24, 24 games: our extractors sit at the engine's spot + (8, 8) in 30 of 30 placements; BARb's, from its own
spot finder, at + (40, 72) in 41 of 42. Both are accepted, so the valid area spans at least 32 x 64 elmos.
**Would be wrong if.** Orders at other positions inside that span were refused.
**Used by.** Nothing yet. Candidate: send each spot's valid region in `Hello` and build at the valid position nearest
the builder (saves 1-2 s of walking per extractor; may put the first extractor inside the commander's build range).

### K-rules-wrecks-are-visible
**Claim.** The AI interface lists the features in sight (`getFeatures`), each with its type's metal
(`FeatureDef_getContainedResource`), the share left (`Feature_getReclaimLeft`) and the unit it can be raised into
(`Feature_getResurrectDef`); single features can be reclaimed (`COMMAND_UNIT_RECLAIM_FEATURE`) and raised.
**Status.** supported (2026-09-20), rec-1/rec-2: 25-55 wrecks of 25 metal or more known at a time from minute 10,
1200-4200 metal in 2-6 fields.
**Used by.** H-REC-FIELDS, H-ECO-RECLAIM.

### K-rules-resurrect-needs-the-unit-limit
**Claim.** Through the AI interface a resurrect order must carry the feature's id PLUS the engine's unit limit
(`Unit_getMax`). The engine's command takes a feature as id + limit; the interface adds the limit for
`COMMAND_UNIT_RECLAIM_FEATURE` and leaves it out for `COMMAND_UNIT_RESURRECT` (AISCommands.cpp; `BuilderCAI::ExecuteResurrect`
subtracts it), so a bare id names a unit and nothing happens, without an error.
**Status.** supported (2026-09-20): rec-1, 57 resurrect orders with the bare id, no unit raised; rec-2, with the limit
added in the shim, 17 raised in 4 games. An engine fault (2026.07.04); not reported upstream.
**Used by.** shim `engine.rs` `issue`.

### K-rules-raised-units-arrive-broken
**Claim.** A resurrected unit arrives with about a twentieth of its health (48-56 of 755-1042, 102 of 1890); raising is
free of metal but is not a unit until it has been repaired.
**Status.** supported (2026-09-20), rec-2, 17 units.
**Used by.** H-REC-RESURRECT.

### K-rules-extra-units-pack-is-off-in-normal-play
**Claim.** The tier-2 construction turrets (`armnanotct2`, `cornanotct2` and their platform forms), the tier-2 wind
turbines (`armwint2`, `corwint2`), the tier-3 gates and some two dozen other units reach a build list only under the
lobby option `experimentalextraunits` ("Extra Units Pack", default off), which
`unitbasedefs/experimental_extra_units.lua` applies by appending them to builders' and factories' build options;
serious play leaves it off (the user, 2026-10-09). The tier-3 constructors and factories in the unit files
(`armhack`/`armhalab`, `armhacv`/`armhavp`, `armhaca`/`armhaap`, `armhacs`/`armhasy` and their Cortex forms) are
built only by each other and reach no standard build list at all. Our arena's start script sets no option but
`maxunits`, and the player's roster line is built from the engine's build lists at game time, so none of these units
reach the player or the hands; `crates/combatsim/data/units.json` carries at least one pack unit (`armshockwave`),
used by no study.
**Status.** supported (2026-10-09): local source; the "serious play" half is the user's word.
**Evidence.** `modoptions.lua` (`experimentalextraunits`, `def = false`); `unitbasedefs/experimental_extra_units.lua`;
`units/ArmBuildings/LandUtil/armnanotct2.lua`; `crates/arena/src/script.rs` (`[MODOPTIONS] { maxunits=... }`);
`crates/bot/src/strategist/shared.rs` (the roster from build lists); a walk over every `buildoptions` of the standard
Armada and Cortex unit files on 2026-10-09: nothing standard builds `armhalab`, `armhavp`, `armhaap`, `armhasy` or
their Cortex forms.
**Would be wrong if.** A lobby with people had the option on (the roster line would then show the units, which is
the right behaviour), or a standard factory file listed a tier-3 factory.
**Used by.** The brief's unit tables (no pack unit in them); K-t2-mobile-build-power-by-cost; the combat simulator's
data (to be filtered).

### K-rules-a-stockpile-takes-no-build-power
**Claim.** A stockpiled round (a silo's nuclear missile, an anti-nuke's interceptor) advances at a fixed `1 / stockpiletime`
a frame while the team can pay that frame's share of the round's cost, all or nothing for the frame, and nothing a
constructor or a construction turret does shortens it: the engine's `CWeapon::UpdateStockpile` is the only code that
advances a stockpile, it reads no build power, and the builder code (`Builder.cpp`, `BuilderCAI.cpp`) never names a
stockpile; BAR's two stockpile gadgets limit the count and tell the unit's script of it. So a missile is a question of
resource flow, above all energy: Armada's 125,000 E over 120 s and Cortex's 187,500 over 180 s are both 1,042 E a second
for the round's length, beside 8.3 and 8.3 metal a second; an interceptor is 83 E a second for 90 s. A massed crew on a
silo is wasted (the silo's frame, 178,500 build time, is where the crew pays).
**Status.** measured (2026-10-09 afternoon): a silo given alone and a silo with two construction turrets guarding it
from its first second (`docs/data/scenarios-2026-10-09-nuke/silo-alone.json`, `silo-nanos.json`; the `nuke-check-alone`
and `nuke-check-nanos` duel batches, `--economy fed`) had their first missile ready at the same 142 s, the round
energy-bound on one freefusion's 850 a second plus the start's 1,000; the turrets changed nothing. The engine's source
and the game's gadgets say why (below).
**Evidence.** `upstream/RecoilEngine-2026.07.04/rts/Sim/Weapons/Weapon.cpp` (`UpdateStockpile`: `p = 1 / stockpileTime`,
`UseResources(cost * p)`), `WeaponDef.cpp` (`stockpileTime` in seconds scaled by `GAME_SPEED`); `grep stockpile` over
`rts/Sim/Units/UnitTypes/Builder.cpp` and `CommandAI/BuilderCAI.cpp` (nothing); `luarules/gadgets/unit_stockpile_limit.lua`,
`unit_stockpile_script.lua`; `units/ArmBuildings/LandDefenceOffence/armsilo.lua` (`metalpershot = 1000`,
`energypershot = 125000`, `stockpiletime = 120`), `corsilo.lua` (1500 / 187500 / 180), `armamd.lua` (150 / 7500 / 90).
**Would be wrong if.** A BAR gadget reimplemented stockpiling with build power (the engine's `stockpileTime = 0` hook is
made for one; none is in the tree), or a game showed turrets beside a silo shortening its rounds.
**Used by.** `crates/buildorder` (`units.rs` STOCKPILES, the simulator's stockpile draw, no help on a stockpile); the nuke
line of the frontier tables (docs/design/2026-10-09-late-economy.md §9); the brief's late-game section to come.

### K-rules-a-stockpile-fills-itself-to-its-limit
**Claim.** Nobody orders a round. BAR's "Stockpile control" gadget sets every unit that can stockpile to its unit
file's limit the moment it is created or given (`stockpilelimit` in the weapon's custom parameters: 10 for the silos;
99 where the file says nothing) and queues rounds to keep stock plus queued at that count, so a standing silo draws its
missile's 1,042 energy a second from its first second until ten missiles sit in it (twenty minutes of rounds, 1,250,000
energy), whatever else the economy wants. A stockpile order from a player or an AI (the engine's `CMD_STOCKPILE`; our
`Command::Stockpile`) does not queue a round: the gadget swallows it and moves the unit's kept count by one (five with
shift, twenty with control, a hundred with both; down with the right mouse button), clamped to 0 and the limit, then
re-queues to match. The engine's own meaning (one order is one round) applies only to orders from synced Lua, which the
gadget's own re-queuing is. The anti-nukes work the same way: an interceptor is stockpiling from the moment the
anti-nuke stands, and is ready 90 s later.
**Status.** measured (2026-10-09 afternoon, `docs/data/scenarios-2026-10-09-nuke/`, the `nuke-check-*` duel batches): a
silo given at full health showed stock 0, queued 10 in the first second's snapshot with no order from anyone; its first
missile was ready at 142 s and the moment it was fired (180 s) the queue read 10 again, and the fed team's energy store
sat at 4 to 30 for the whole fight. Cortex's anti-nuke (`corfmd`, limit 20) had its first interceptor at 90 s and its
second at 180.
**Evidence.** `upstream/Beyond-All-Reason/luarules/gadgets/unit_stockpile_limit.lua` (`UnitCreated`: the desired
target is the limit, `UpdateStockpile` queues to it; `AllowCommand`: a `CMD_STOCKPILE` not from synced Lua moves the
target and returns false); `rts/Sim/Units/CommandAI/CommandAI.cpp` `case CMD_STOCKPILE` (the engine's +1 / -1 / x5 /
x20 by option); `units/ArmBuildings/LandDefenceOffence/armsilo.lua` (`stockpilelimit = 10`).
**Would be wrong if.** A silo given in a game stood with an empty queue until ordered, or a `Command::Stockpile`
added a round beyond the kept count.
**Used by.** `Command::Stockpile` (its semantics, `DESIGN.md`); the simulator's stockpile model (a standing stockpiler
draws from its first second, `crates/buildorder/src/sim.rs`, which matches); the brief's late-game section; the
design's nuke harness (`docs/design/2026-10-09-late-economy.md` §10): the player's lever is the kept count, not a
stockpile order, and a silo left at its limit is a 1,042 energy a second draw for twenty minutes.

### K-rules-the-silo-fires-on-an-attack-at-the-ground
**Claim.** A silo's missile goes by a plain attack order at a place, not by the engine's manual-fire command. BAR's
nuke weapon is `commandfire` (the engine's `manualfire` tag: the weapon never picks a target of its own), but the engine
allows the manual-fire command only to a unit type with `canManualFire`, which BAR gives the commander (its D-gun) and
not the silo, and for a type without it the weapon code skips the manual-fire check altogether, so an attack at the
ground fires the missile. From an AI the order is `COMMAND_UNIT_ATTACK_AREA` with the place and no radius (`CMD_ATTACK`
with a position; `Command::FireAt`), and `COMMAND_UNIT_D_GUN_POS` is refused with no effect. Measured: the order went
out at 171 s, the silo launched at 180 (nine seconds of launch sequence), the warhead landed at 192 (the file's 400
frames of flight) and the five buildings 2,500 away, four solars and a light turret, were gone: `x (wiped) 192s`. With
a Cortex anti-nuke (`corfmd`, coverage 2,000) standing 260 from the targets with an interceptor ready, the same missile
did nothing: the anti-nuke's ready count went from 2 to 1 between 180 and 184 s and every building stood at full health
at 230 s.
**Status.** measured (2026-10-09 afternoon): `docs/data/scenarios-2026-10-09-nuke/silo-alone.json`, `silo-nanos.json`,
`silo-antinuke.json` with the scripts `fire-170.json` and `nanos-fire-170.json`; the duel batches `nuke-check-alone`
(`run/matches/1791571513-...`), `nuke-check-nanos` (`1791571529`), `nuke-check-antinuke` (`1791571546`); the three
before them (`1791571207`, `1791571228`, `1791571245`) sent the manual-fire command and nothing fired.
**Evidence.** `rts/Sim/Units/CommandAI/CommandAI.cpp` (`AllowedCommand`: `CMD_MANUALFIRE` needs `ud->canManualFire`;
`ExecuteAttack`: a position goes to `AttackGround`), `rts/Sim/Weapons/Weapon.cpp` `CWeapon::TestTarget` (the
manual-fire check applies only when `owner->unitDef->canManualFire`), `rts/Sim/Units/UnitDef.cpp` (`canManualFire` from
the unit file's `canManualFire` or `canDGun`, default false; the silo's file and BAR's `gamedata` set neither),
`rts/ExternalAI/AISCommands.cpp` (`COMMAND_UNIT_ATTACK_AREA` is `CMD_ATTACK` with the position and the radius pushed);
`units/ArmBuildings/LandDefenceOffence/armsilo.lua` (`commandfire = true`, `flighttime = 400`).
**Would be wrong if.** A silo fired on no order (the weapon auto-targeting), or a manual-fire order from an AI fired it
in another engine version.
**Used by.** `Command::FireAt` (the host picks the engine command by `UnitDef_canManualFire`: manual fire for the
commander, an attack at the ground for the silo; `DESIGN.md`); the duel harness's `fire` script command; the design's
nuke harness (`docs/design/2026-10-09-late-economy.md` §10).

### K-rules-interception-needs-the-aim-point-in-coverage
**Claim.** In BAR an anti-nuke takes a missile only when the place the missile is aimed at lies within the anti-nuke's
coverage (2,000 for `armamd`, `corfmd`, `legabm`; measured flat, from where the anti-nuke stands at that moment, so a
mobile one's cover moves with it). The engine alone would also take a missile whose flight passes within the coverage
or whose projected impact does (four cases in `CInterceptHandler::Update`); BAR's gadget "Don't target flyover nukes"
refuses every case but the aim point. So a missile aimed at a place more than 2,000 from every anti-nuke lands, and an
intercepted one proves an anti-nuke stood within 2,000 of where it was aimed.
**Status.** read from source (2026-10-27), not measured.
**Evidence.** `rts/Sim/Misc/InterceptHandler.cpp` (`Update`: the four cases, each behind
`eventHandler.AllowWeaponInterceptTarget`); BAR's `luarules/gadgets/unit_interceptors.lua` (`AllowWeaponInterceptTarget`
returns whether the target's ground position is within the interceptor weapon's `coverageRange` of the interceptor
unit, in x and z); the unit files (`coverage = 2000`, `reloadtime = 2`, `stockpilelimit = 20`, `stockpiletime = 90`).
**Would be wrong if.** A missile aimed beyond every anti-nuke's coverage were taken in flight, or the gadget were
disabled in a lobby.
**Used by.** D-NUKE-FIRE (`bots/distilled/src/nuke.rs`): no place within his known anti-nuke's coverage is struck.

### K-rules-a-bot-sees-no-interception
**Claim.** Nothing tells an AI that its missile was intercepted. The Skirmish AI interface has no projectile access and
no interception event, and an enemy unit's stockpile count is given to allies only (`AICallback::GetProperty`,
`AIVAL_STOCKPILED`), so his anti-nuke's ready interceptors cannot be read. What an AI does get: its own silo firing
(`EVENT_WEAPON_FIRED`, raised for a unit firing under an attack order), and the damage and kills the warhead does
(`EnemyDamaged`, `EnemyDestroyed`, both naming our silo), but only for units in our ally team's sight or radar (BAR's
buildings are not radar-stealthy). A strike on a place nobody of ours watches has no observable outcome. The true
heightmap is given in full (`Map_getHeightMap` reads the synced one), so a crater tells a landing anywhere; it is off
when the lobby disables map deformation. A player gets the same bit by another road. The game draws a glowing crater in
the fog where a missile landed; it is not the real deformation, which shows only once the place is in sight, but a
deliberate signal of success. The missile's flight and its explosion, in the air when intercepted and on the ground
when not, are seen by all (the user, 2026-10-27, from games with people: "the terrain deformation from a succesful nuke
is vary obvious", then "In the game gui there is a very obvious glowing crater that appears in the fog-of-war terrain
content. It is not exactly the real deformation (getting real vision reveals the real terrain damage), but the
deliberate success signal is still there"). So the bot reads the real ground for that one bit, landed or not, and
nothing more. Whether the glowing mark is drawn on a lobby with deformation off, where the bot's ground tells nothing,
is not known.
**Status.** read from source (2026-10-27), not measured; the user, 2026-10-27: "Let's run with the expectation of not
knowing", then that the crater test is "all above board".
**Evidence.** `rts/ExternalAI/Interface/SSkirmishAICallback.h` (no projectile callbacks), `AISEvents.h`,
`rts/ExternalAI/EngineOutHandler.cpp` (`UnitDestroyed`, `UnitDamaged`: the enemy events need the unit in the ally
team's sight or radar unless cheat events are on), `rts/Sim/Units/CommandAI/CommandAI.cpp` (`WeaponFired`),
`rts/ExternalAI/AICallback.cpp` (`GetProperty`, `GetHeightMap`).
**Would be wrong if.** An engine version added projectile or interception events, or the enemy events came without
sight.
**Used by.** D-NUKE-FIRE's instrument (`nuke:` lines: what the events saw of each strike, and whether the place was in
our sight or radar); `run/nukes.py` judges every strike from the truth file instead.

### K-rules-a-landed-nuke-digs-a-crater
**Claim.** A nuclear missile that lands digs a crater the ground's height shows at once, and one intercepted leaves the
ground as it was. On Comet Catcher an Armada missile aimed between four solars and a light turret took the ground down
52 at the aim, 37 at 150 from it, 7 at 300 (21 on the mean over the aim and two rings of eight); the same missile taken by
a Cortex anti-nuke 260 from the aim left every point as it was, to the tenth. The engine digs the crater over the frames
after the blast (`EXPLOSION_LIFETIME`), squares under a building move with the building as a block, and the readings
25 and 50 s after the impact were the same.
**Status.** measured (2026-10-27), one run each, one map; a harder map digs less (the engine divides by the ground's
hardness), and a lobby with map deformation off digs nothing.
**Evidence.** The duel harness's `probe` script command (`Command::Probe`, answered with the synced ground's height);
`docs/data/scenarios-2026-10-09-nuke/silo-alone-witness.json` (silo-alone with a fifth solar outside the blast, so the
duel outlives the landing) and `silo-antinuke.json`, script `crater-170.json` (probes at 150, 215, 240 s, the missile at
170); `run/matches/*-duel-nuke-crater-witness`, `*-duel-nuke-crater-antinuke`; `rts/Map/BasicMapDamage.cpp`.
**Would be wrong if.** An intercepted missile were seen to move the ground, or a landed one on another map moved the
mean by less than 2.
**Used by.** D-NUKE-VERDICT (`bots/distilled/src/nuke.rs`, `CRATER_DEPTH = 2`).
