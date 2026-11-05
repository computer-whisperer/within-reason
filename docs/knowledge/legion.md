# Legion: the third faction

Seeded 2026-10-14 from the local game source (`upstream/Beyond-All-Reason` at 54199a0d, 2026-09-19; unit paths relative
to `units/Legion/`), the installed AIs, the replay survey's manifest and its two carded Legion games. Numbers are the
unit files' raw values (`alldefs_post.lua` changes none of the ones quoted; the same caveat as `units.md`). Nothing
here has been seen in a game of ours: **we have never played or faced Legion** (`docs/experiments.md` has no Legion row).

### K-leg-exists-only-under-a-mod-option
**Claim.** Legion is loaded only when the start script sets `experimentallegionfaction=1` (or `ruins`, scavengers,
zombies, `experimentalextraunits`, `scavunitsforplayers` or `forceallunits`): `unitdefs.lua` skips every file whose path
contains `legion` otherwise. A `Side=Legion` team in a game without the option is given a random Armada or Cortex
commander by the spawn gadget (`legcom` is not among its valid start units), so the seat plays another faction
without any error. Our arena writes only `maxunits` into `[MODOPTIONS]` and its `--side` takes `armada` or `cortex`,
so no game of ours has ever had Legion units defined. The replay survey's Great Divide games that had a Legion side
carried `experimentallegionfaction=1` in their scripts, and our shim and record handled the Legion defs there without
change (`record-1.jsonl` of the carded games below names `leg*` units).
**Status.** supported (2026-10-14) — game source and the carded replays; the "random commander" branch read, not run.
The arena's switch built the same day: `--side legion` / `--their-side legion` write the option, and leg-smoke-distilled
(`docs/experiments.md`) had a Legion commander and a Legion base on our side under it
**Evidence.** `gamedata/unitdefs.lua:47-78`; `luarules/gadgets/game_initial_spawn.lua:165-185` (`validStartUnits`),
`:265-280` (`isUnitValid` else `getValidRandom`); `gamedata/sidedata.lua` (sides Armada, Cortex, Random, Legion, in
that order: "Don't change the ordering ... otherwise AI will crash"); `crates/arena/src/script.rs:207`,
`crates/arena/src/main.rs:819-825`; `run/matches/1791940127-replay-f90fb66a6c09d540a2efdcf14f3fb980/_script.txt`.
**Would be wrong if.** A script with `Side=Legion` and no mod option spawned a `legcom`.
**Used by.** (the arena's Legion switch, when built: the option goes in whenever a seat's side is Legion)

### K-leg-barb-cannot-play-it-apex-can
**Claim.** (As first written, 2026-10-14 morning.) The stock BARb cannot play Legion: its script leaves the Legion type
mask commented out and none of its config files name a Legion unit; Apex v0.2.2 can.
**Status.** refuted (2026-10-14 afternoon) — the evidence looked only at the engine directory's copy of BARb's config,
which holds the defaults alone; BARb loads the game's shipped per-profile config (`game_config=true` in its
`AIOptions.lua`, "Config override: legion" in the engine log), and that has the Legion files. The kill-test
leg-smoke-barb (`docs/experiments.md`) seated BARb medium on `Side=Legion`: the log says "Inserting Legion", and at 6:00
it stood a Legion commander, 3 extractors, 4 solars, 2 advanced solars, a vehicle plant, a constructor vehicle,
6 Alaris, 3 Pharos and a radar. The Apex half holds (K-leg-both-stock-ais-play-legion). The arena guard built on this
claim was removed the same afternoon. The prose agent's report pointed at the game-shipped configs.
**Evidence.** `run/matches/1791985810-leg-smoke-barb/00/engine.log` (the BARb lines), the census at f=10800.
**Would be wrong if.** n/a (refuted).
**Used by.** nothing now.

### K-leg-both-stock-ais-play-legion
**Claim.** Both stock AIs play Legion when a seat is given `Side=Legion` with the mod option on. BARb reads the game's
shipped config `luarules/configs/BARb/stable/config/<profile>/` (every profile has `behaviour_leg.json`,
`build_chain_leg.json`, `commander_leg.json`, `economy_leg.json`, `factory_leg.json`; `game_config` defaults to true
in `AIOptions.lua`, and the engine directory's `config/` holds only the shared defaults, no profile directories), and
its medium profile opened with the vehicle plant, Alaris and Pharos on Comet (above). Apex v0.2.2's `standard` profile
carries the same five `*_leg.json` files and in leg-smoke-apex opened with the bot lab: Goblins, Phobos, a Ballista,
two Pharos and an advanced solar by 8:00. So a Legion seat of ours can be tested against either AI, and either can
field Legion against us (`--their-side legion`, or `--mirror` with `--side legion`).
**Status.** supported (2026-10-14) — one six-minute game of each, then eight 40-minute games of Apex standard as Legion
(apex-v024-legion-distilled-comet-1: the bot lab first in all eight, Phobos, Satyrs and Goblins, never a vehicle plant,
5-14 soldiers at 10:00 (median 9) against the 1-25 (median 15) it fields as Armada or Cortex on the same seeds; its random search trains Armada and Cortex
mirrors only, so its Legion is the untrained side); what BARb's hard profiles do as Legion is unseen
**Evidence.** leg-smoke-barb and leg-smoke-apex in `docs/experiments.md`; `run/engines/2026.07.04/AI/Skirmish/BARb/stable/AIOptions.lua`
(`game_config`); `upstream/Beyond-All-Reason/luarules/configs/BARb/stable/config/*/`;
`run/engines/2026.07.04/AI/Skirmish/Apex/Unstable/config/standard/*_leg.json`.
**Would be wrong if.** A hard or hard_aggressive BARb on `Side=Legion` crashed or built nothing past the commander.
**Used by.** the arena's `--their-side` and `--mirror` (`crates/arena/src/script.rs`)

### K-leg-pro-pool-is-thin
**Claim.** The replay survey cannot teach Legion at the standard the Armada and Cortex baselines use (OS 40-50). Of the
manifest's 317 games (665 sides) 21 have a Legion side (27 sides). The best Legion player in a kept game is OS 27
(Warmoob, two games on Great Divide V1 against OS 25 Cortex, both won, both carded); one OS-41 Legion side sits in a
5-minute game that was not kept; the rest are OS 8-23, most on Great Divide and Comet Catcher. Legion knowledge
therefore comes from the unit files, from Apex's Legion scripts, and from our own games, until the fetcher is pointed
at Legion sides of high OS on purpose.
**Amended 2026-11-05.** Our manifest has grown to 576 games, 103 with a Legion side (240 sides): 14 sides at OS 30 or
above, all 8v8 lane seats on All That Glitters from the human-8v8 set ([Cookie]Maco at 53.7 in four games, Air_China
46.1, NullRay and Minigoal 33, Eclipse 32.5, Thors_Hammer 31.2, elderWhite 30.9; all carded), and 14 duel sides none
above Warmoob's 27.1. The friend's bar-replays processing of the public replay bucket (http://b-dtop.b.gecko.network:8770;
a DuckDB of the engine replay's events, 1,000 recent public duels above OS 40 processed) holds the whole catalogue:
public duels with a Legion side number 39 with both players above OS 40 (29 of them in 2026, 6 since June), 63 above
35, 164 above 30, 7 Legion mirrors above 40, against 14,212 public duels above 40 in all; so Legion is about one duel
in forty at that level. Two of the 39 are read (K-leg-open-os49-canis-river), the other 36 are queued there as
selection 6, legion-1v1-os40 (2026-11-05, about one CPU-hour), and `run/replays/bar_replays.py` reads a processed game.
**Status.** supported (2026-10-14) — a count over `run/data/replays/manifest.jsonl` (`players[].faction`); the
bucket-wide counts 2026-11-05 from the bar-replays catalogue preview
**Evidence.** The manifest; the two carded games `run/matches/1791940127-replay-f90fb66a6c09d540a2efdcf14f3fb980`
(22 min) and `run/matches/1791939987-replay-a915b66a6f0eb566c5821435469ccba9` (6 min).
**Would be wrong if.** The BAR replay API, asked for Legion sides above OS 35, returned a usable pool (not asked yet).
**Used by.** (the replay survey, if it is extended to Legion)

### K-leg-economy-is-the-cortex-economy
**Claim.** Legion's tier-1 economy costs what Armada's and Cortex's do, so the opening arithmetic (K-open-*) transfers:
extractor `legmex` 50 M / 500 E (the same); solar `legsolar` 150 M / 0 E for 20 E/s (Armada 155); wind `legwin` 43 M /
175 E (Armada 40, 230 hp against 196); bot lab `leglab` 470 M / 1050 E (= Cortex's); vehicle plant `legvp` 570 M /
1650 E (Armada 590 / 1550); converter `legeconv` 1 M / 1150 E at 70 E to 1 M (= Armada's); construction turret
`legnanotc` 230 M, 200 build power (the same); constructor bot `legck` 100 M, 75 build power, speed 39 (Armada 110 /
80 / 36); constructor vehicle `legcv` 125 M, 85 build power, speed 57 (Armada 135 / 90 / 54); storages `legmstor`
340 M, `legestor` 175 M. The next step up in energy: advanced solar `legadvsol` 465 M / 4080 E for 100 E/s (Armada's 350 M /
5000 E for 80; a tier-1 constructor builds either), 4.65 against 4.4 metal per energy a second; fusion `legfus` 4000 M /
25000 E for 950 (Armada 3350 / 18000 for 750, Cortex 850), 4.2 against 4.5, so the Legion fusion is the cheaper one
per energy (the 75 and 1000 first written here, 2026-10-14 morning, were wrong; the prose agent read the files); advanced bot
lab `legalab` 2600 M / 16000 E; advanced constructor `legack` 410 M, 195 build power; advanced extractor `legmoho`
640 M / 8100 E (= Cortex's). The commander `legcom` is the same hull (2700 M, 3700 hp, speed 37.5, 300 build power,
30 E/s, laser 300 at 175 per 0.8 s, D-gun 250) plus a torpedo and an anti-air missile, both at 300: the only commander
with a weapon that targets aircraft.
**Amended 2026-10-14 (leg-player-1's review).** The buildings cost the same; the vehicles do not. Legion's tier-1
vehicles cost 30 to 60 % more energy per metal than their twins (energy over metal from the unit files): Alaris 13.1
(Blitz 8.2, Incisor 9.2), Decurion 12.0 (Stout 9.3, Brute 10.2), Lance 15.0 (no twin; Whistler 1150/150 = 7.7),
Wheelie 15.2 (Rover 11.9), Helios 15.0, Barrage 12.8 (Shellshocker 16.3), constructor vehicle 16.8 (Armada's 14.4).
A Legion vehicle army therefore wants more energy per metal of income than an Armada one, and the game showed it:
energy use matched income in minutes 10-14, the metal store sat at its cap for 146 s and spent 2,203 metal into
nothing while the advanced solars ordered at 5:52-6:48 had been overwritten by later lists (the first stood at
13:13). The opening arithmetic holds; the army's energy does not.
**Amended 2026-11-05 (every economy, constructor, lab and commander file of the three factions dumped with Lua, and
the energy per metal of every factory's list computed).** The buildings' prices are the same to within a few percent,
as above, and the "30 to 60 %" is the twin pairs named, which are the extremes, not the roster: by factory, the median
energy per metal of what it makes is level between the factions everywhere but the tier-1 vehicle plant (Legion 15.0
against Armada 13.0 and Cortex 13.2; tier-1 bots 11.4 against 11.1 and 10.6; tier 2, air and tier 3 within a point).
The consequence holds for a Legion vehicle army and inverts for a bot one (K-leg-lab-energy-inverts). Where Legion's
economy differs from the other two is structural, not in price: K-leg-economy-structures.
**Status.** supported (2026-10-14) — local source; the energy-per-metal consequence seen in one game
**Evidence.** `Economy/legmex.lua`, `legsolar.lua`, `legwin.lua`, `legeconv.lua`, `legadvsol.lua`, `legfus.lua`,
`legmoho.lua`, `legmstor.lua`, `legestor.lua`; `Labs/leglab.lua`, `legvp.lua`, `legalab.lua`; `Utilities/legnanotc.lua`;
`Constructors/legck.lua`, `legcv.lua`, `legack.lua`; `legcom.lua` (weapons table: `aa_missile` with
`onlytargetcategory = "VTOL"`, `torpedo` with `waterweapon`); the Armada and Cortex files beside them.
**Would be wrong if.** The definitions the shim reports in a Legion game (`Hello`) differed from these.
**Used by.** the Legion `Roster` and its usual menu; the brief's Legion section (the energy line)

### K-leg-t1-bot-roster
**Claim.** The bot lab `leglab` (8 options) makes: constructor `legck`; Zagreus `legrezbot` 130 M, 200 build power,
resurrects (= Lazarus); Goblin `leggob` 30 M / 200 hp / speed 75 / range 261, two 12-damage laser shots per 0.9 s, the
game calls it a light skirmish bot; Satyr `leglob` 60 M / 550 hp / 51 / 400, plasma 75 per 2.5 s, area 24, the Mace's
place at under half the metal; Phobos `legcen` 140 M / 840 hp / 90 / 180, two 75-damage gauss shots per 1.4 s, a fast
assault bot (the speed of a Pawn with four times its metal); Ballista `legbal` 160 M / 900 hp / 43.5 / 460, one
250-damage rocket per 5.5 s, area 70 (the Rocketeer's place: it outranges the light towers' 430); Karkinos `legkark`
310 M / 1725 hp / 42 / 360, a heat ray 160 per 1.8 s plus a shotgun at 251 (the Centurion's place); Toxotai `legaabot`
120 M, anti-air only, range 760, amphibious. There is no tier-1 scout bot: the scouts are the vehicle plant's Wheelie
`legscout` 23 M / 75 hp / speed 160 / sight 635 and the drone plant's Noctua `legfig` 40 M. The tier-1 heat rays and Legion
lasers list no energy per shot; the tier-2 and tier-3 ones pay (Prometheus, Telchine, Incinerator 10 a frame while
firing, the Bastion 500 a shot, the Phoenix 17 a shot, Sol Invictus 17 a frame; the agent's reading of the files,
2026-10-14).
**Status.** supported (2026-10-14) for the numbers (local source); the "place of" comparisons are by cost and range only,
no duel has been run
**Evidence.** `Labs/leglab.lua:buildoptions`; `Bots/leggob.lua`, `leglob.lua`, `legcen.lua`, `legbal.lua`, `legkark.lua`,
`legaabot.lua`, `legrezbot.lua`; `Vehicles/legscout.lua`; `language/en/units.json` for the names.
**Would be wrong if.** The duel harness, run on these against the Armada/Cortex tier-1 table, placed them elsewhere
(the Goblin's 261 range and 30 M may make it a swarm skirmisher rather than a Grunt).
**Used by.** (candidate: the Legion `Roster`'s `raider` and `line`; a Legion column in the brief's unit table)

### K-leg-t1-vehicle-roster
**Claim.** The vehicle plant `legvp` (10 options) makes: Wheelie `legscout` (above); constructor `legcv`; Otter
`legotter` 150 M amphibious constructor (44 options: the amphibious complex, the seaplane platform and the sea
buildings besides the land set); Alaris `leghades` 65 M / 425 hp / speed 99 / range 211, a five-pellet shotgun (75 a blast
every 2.1 s; its file defines a gauss cannon too but mounts only the shotgun, `leghades.lua` weapons table) (the
Blitz's place at 60 % of its metal and health); Helios `leghelios` 160 M / 800 hp / 75 / 320, heat ray 160 per
1.3 s, a skirmisher tank; Decurion `leggat` 300 M / 2550 hp / 45 / 361, a six-shot gatling (12 each) per 0.4 s, the
assault tank (the Stout's place, 300 against 225 M); Barrage `legbar` 250 M / 1500 hp / 40.5 / 535, napalm cannon 60
per 5 s, area 120 (short of the Shellshocker's 710; it is a riot-artillery piece, not a tower killer); Lance `legrail`
240 M / 1100 hp / 46.5 / 650, railgun 200 per 7.5 s and an anti-air railgun at 750 (a long skirmisher that also shoots
aircraft; the game groups it `weaponaa`); Cetus `legamphtank` 200 M amphibious tank; Sapper `legmlv` 52 M minelayer.
**Status.** supported (2026-10-14) for the numbers (local source); the comparisons are by cost and range only
**Evidence.** `Labs/legvp.lua:buildoptions`; `Vehicles/leghades.lua`, `leghelios.lua`, `leggat.lua`, `legbar.lua`,
`legrail.lua`, `legamphtank.lua`, `legmlv.lua`; `Constructors/legotter.lua`.
**Would be wrong if.** Duels placed them elsewhere; or the Lance's 650 proved enough to kill light towers unanswered
(it does outrange 430).
**Used by.** (candidate: the Legion `Roster`'s `vehicle_raider` and `vehicle_line`)

### K-leg-defences
**Claim.** Legion's light tower is the Pharos `leglht`: 70 M / 550 E, 550 hp, heat ray at 430 for 200 per 1.1 s, no
energy per shot (Sentry 85 M, 620 hp, 430, 20 E a shot): cheaper, a weaker hull, and it keeps firing in an energy
stall (contrast K-units-laser-towers-need-energy). The rest of the tier-1 set a constructor builds: Bramble `legrl`
80 M anti-air at 765 (= Nettle); Dragon's Jaw `legdtr` 290 M pop-up riot cannon, 400 range, 240 per 2 s, area 140,
stealthed; Cacophony `legmg` 380 M gatling at 575 against land and air, 17 x 6 per 0.4 s; Rhapsis `legrhapsis` 280 M
salvo anti-air at 950; Lupara `leglupara` 900 M flak at 1125; Amputator `legcluster` 850 M cluster artillery at 1000
(the Gauntlet's place, 1250 M at 1220); Hive `leghive` 350 M, carries six drones at 15 M / 500 E each; Argus `legeyes`
32 M camera; radar `legrad` 60 M; jammer Nyx `legjam` 140 M; Juno `legjuno`. Tier 2 (advanced constructors): Chimera
`legapopupdef` 750 M pop-up at 730 (the Pit Bull's place), Bastion `legbastion` 4200 M energy weapon at 1100 (the
Annihilator's place), Eviscerator `legacluster` 2300 M pop-up cluster artillery at 1380, Pluto `legflak` 820 M flak,
Xyston `leglraa` 1600 M long-range anti-air at 2400, Perdition `legperdition` 1250 M napalm at 2300, Aegis `legabm`
1500 M anti-nuke, Supernova `legsilo` 7700 M nuke (Armada's 8100 M, Cortex's 9100 M), Olympus `leglrpc` 5200 M
long-range cannon at 4800, Starfall `legstarfall` 63000 M.
**Status.** supported (2026-10-14) for the numbers (local source); roles from the game's one-line descriptions
**Evidence.** `Defenses/*.lua`, `Utilities/*.lua`; `Constructors/legck.lua:buildoptions`, `legack.lua:buildoptions`.
**Seen 2026-10-14 (leg-player-1):** Pharos dealt 1,877 damage at 6:13-6:51 with the energy store at 0-3 (the review).
**Would be wrong if.** The Pharos duel row against the Sentry's did not show it firing through an empty store.
**Used by.** (candidate: the Legion usual menu; the brief's tower reach line, where 430 still holds)

### K-leg-drone-plant-is-the-air-plant
**Claim.** Legion's tier-1 air factory is a Drone Plant `legap` (430 M / 1100 E; Armada's aircraft plant 650 M /
1100 E) and its tier-1 air is drones: constructor aircraft `legca` 100 M, 45 build power; Noctua `legfig` 40 M fighter
and scout; Martyr `legkam` 65 M, flies into the target and explodes (area 1280 listed as its range); Blindfold `legcib`
100 M, drops a Juno bomb; Mosquito `legmos` 110 M gunship with stockpiled rockets; two transports. Separately, the Hives
(`leghive`, `legfhive`), the Mantis truck `legvcarry`, the Syracusia destroyer, the Hecate anti-nuke carrier and the
Hecatoncheir seaplane spawn light drones (15 M / 500 E each, `legdrone` 325 hp, speed 280, about 27 a second at 300),
the Rampart and the Myrmidon mech heavy ones (`legheavydrone`, 90 M / 1000 E, 1650 hp, 125 a second at 430); a
carrier's drones are released when it dies and then lose 20 health a second (50 for heavy ones;
`luarules/gadgets/unit_carrier_spawner.lua`). That is how the carded OS-27 player made his tier-2 army (6-12 drones
from 10:00). The engine will show them as units of ours that no factory made. The Fortifier with a drone pad
(`legmohobp`) is in no build list; the reachable Fortifier is `legmohocon`, an advanced extractor carrying an
unselectable 400-build-power construction turret and no drones (the agent's reading, 2026-10-14; first written here as
`legmohobp`).
**Amended 2026-11-05 (the gadget read).** A Hive's drones are a picket, not an army: they fight within 1,100 of it and
never leave 1,000, dock to heal, and the Hive restocks a lost one every ten seconds at 15 metal and 500 energy through
its stockpile (K-leg-economy-structures item 3). The OS-27 player's "army of drones from 10:00" was Hive pickets.
**Status.** supported (2026-10-14) for the numbers and build lists (local source); the drone-spawn mechanics read in
the gadget 2026-11-05 (`unit_carrier_spawner.lua`), not timed in a game
**Evidence.** `Labs/legap.lua`, `Air/*.lua`, `Defenses/leghive.lua`, `Economy/legmohobp.lua`; the 22-minute carded
game's `army_by_type` (below).
**Would be wrong if.** A Hive of ours produced nothing, or its drones could be built from a factory menu.
**Used by.** (candidate: the hands' "a unit no factory made" path, which the Fortifier and the Hives will exercise)

### K-leg-t2-roster
**Claim.** Advanced bot lab `legalab` (16): constructor `legack`; Proteus `legaceb` 310 M all-terrain combat engineer;
Hoplite `legstr` 355 M / 1780 hp / speed 84, fast raider; Telchine `legamph` 600 M amphibious assault at 600; Phalanx
`legshot` 470 M shielded riot bot at 280; Incinerator `leginc` 2300 M / 9000 hp heavy heat ray at 725, speed 24;
Arquebus `legsrail` 750 M all-terrain railgun at 850; Belcher `legbart` 550 M napalm skirmisher at 625; Infestor
`leginfestor` 250 M spider; Thanatos `leghrk` 750 M salvo rockets at 1200; Aquilon `legadvaabot` 750 M anti-air;
Snapper `legsnapper` 60 M amphibious bomb; mobile radar, jammer, spy, decoy commander. Advanced vehicle plant `legavp`
(15): constructor `legacv`; Quickshot `legmrv` 250 M / speed 135 raider; Gladiator `legaskirmtank` 450 M burst
skirmisher at 600; Prometheus `legaheattank` 1250 M / 7700 hp heat-ray assault at 450; Medusa `legmed` 1500 M salvo
rockets at 1000; Cleaver `legamcluster` 460 M cluster artillery at 930; Inferno `leginf` 1700 M napalm artillery at
1200; Boreas `legavroc` 920 M stealthy rockets at 1300; Triton `legfloat` 650 M tank/boat; Mantis `legvcarry` 450 M
drone carrier; Charon `legvflak` 470 M flak truck; Hera `legavantinuke` mobile anti-nuke; Aceso `legafcv` 200 M light
constructor buggy; radar and jammer vehicles. Gantry `leggant` 8400 M (8): Sol Invictus 23500 M dual heat ray,
Praetorian 7000 M shotgun mech, Daedalus 6500 M rail tank, Astraeus 11000 M siege cannon at 3100, Myrmidon 3950 M
drone-carrier mech, Charybdis 1350 M hovertank, Javelin 1200 M amphibious raider, Keres 2600 M anti-swarm tank.
**Status.** supported (2026-10-14) for the lists and numbers (local source)
**Evidence.** `Labs/legalab.lua`, `legavp.lua`, `leggant.lua`; `Bots/T2 Bots/*.lua`, `Vehicles/T2 Vehicles/*.lua`,
`T3/*.lua`.
**Would be wrong if.** The lists differed in the game we run (`test-31357`), which is older than the checkout by a day.
**Used by.** (none yet; the glossary when regenerated with Legion)

### K-leg-open-os27-great-divide
**Claim.** The one Legion player of the pool above OS 25, in two wins on Great Divide V1 against OS-25 Cortex, opened
the same way both times: extractor 0:03, extractor 0:12, wind 0:19, wind 0:25, solar 0:32, bot lab 0:42 (the bot lab
before 1:00, as 41 of 58 Armada/Cortex sides do); the lab made a constructor, then Goblins (two to four), a Zagreus and a
second constructor; the commander walked to the next cluster for an extractor, a Pharos and a solar by 1:43-1:57, then
another Pharos at 2:19-2:23. His army was Goblins to 2:00, then Satyrs with one to three Ballistas from 4:00 to 10:00
(no Phobos, no Karkinos, no vehicle plant), the advanced bot lab at 8:08 in the long game, and from 10:00 drones from
Hives (6-12 standing) with one Phalanx and one Aquilon. The 22-minute game was decided with that; the 6-minute one
before any tier 2.
**Status.** reported (2026-10-14) — one player, OS 27, two games against the same opponent; well below the OS 40-50
baselines, so a sketch of what Legion play looks like, not a standard
**Evidence.** `card.json` of `run/matches/1791940127-replay-f90fb66a6c09d540a2efdcf14f3fb980` and
`run/matches/1791939987-replay-a915b66a6f0eb566c5821435469ccba9` (`build_order`, `army_by_type`, `factories`).
**Checked 2026-11-05.** The two OS-49 duels opened the same way (K-leg-open-os49-canis-river): the bot lab at 0:47, a
constructor, then Goblins, with Satyrs from 7:25 and no Ballista and no Phobos; the shape holds across 22 OS.
**Would be wrong if.** Higher-OS Legion games, once fetched, opened with the vehicle plant or Phobos pressure instead.
**Used by.** (candidate: the brief's opening snippet for a Legion seat; the Legion `Roster`'s `raider`/`line` picks)

### K-leg-role-words
**Claim.** A first mapping of the bot's role words (`crates/bot/src/brain/roster.rs`, `ROLE_WORDS`) onto Legion, by the
cost-and-range places above and the OS-27 games: commander `legcom`, extractor `legmex`, converter `legeconv`, lab
`leglab`, turret `leglht`, constructor `legck`, plant `legvp`, vehicle_constructor `legcv`, advanced_lab `legalab`,
advanced_constructor `legack`, advanced_extractor `legmoho`, raider `leggob`, line `leglob`, vehicle_raider `leghades`,
vehicle_line `leggat`, rez `legrezbot`, solar `legsolar`, wind `legwin`, air_plant `legap`, radar `legrad`, nano
`legnanotc`, metal_storage `legmstor`, energy_storage `legestor`. The two doubtful picks are `raider` (Goblin, the
30 M swarm unit the OS-27 player massed; Phobos `legcen` is the fast one) and `line` (Satyr, his mid-game body; Karkinos
`legkark` is the heavy one). Every suffix also differs from the other factions' where it matters: `leglht` not `legllt`,
`legeconv` not `legmakr`, `legscout` not `legfav`, `legrezbot` not `legrectr`, so a suffix rule (`bots/distilled`'s
`kit_of`) does not reach Legion.
**Status.** conjectured (2026-10-14)
**Evidence.** The claims above; `roster.rs:106-130`; `bots/distilled/src/lib.rs:94-125`.
**Would be wrong if.** Legion games with the player showed `raider` wanted as Phobos (a Pawn-speed unit) or `line` as
Karkinos; the words are the player's, so the first games decide.
**Measured 2026-10-14 (leg-player-1, WON 25:54 against BARb hard_aggressive Armada).** The mapping stands as built and
was barely used: the player wrote Legion internal names in its lists from the first turn (leglht 84 times, legadvsol
46, leggat 31, leghades 22, legrail 11 over 46 `queue` and 26 `produce` calls) and role words for `extractor` (92, a
list step rather than a role word), `solar` (29), `constructor` (25), `raider` (4, at the vehicle plant, where it
resolves to the Alaris) and `plant` (1); `line`, `turret` and `lab` never. (Counted once per tool call; the strategist
log holds each call three times, and a first count here was three times too high.) It built no bot lab in the game (vehicle plants only, four of them, then the advanced
vehicle plant), so the Goblin/Satyr half of the mapping is unexercised.
**Used by.** the Legion `Roster` (`crates/bot/src/brain/roster.rs`)

### K-leg-economy-structures
**Claim.** Where Legion's economy differs from Armada's and Cortex's is five things it has and one it lacks, not the
prices (K-leg-economy-is-the-cortex-economy); unit files at 54199a0d, the two gadgets read.
1. *The Fortifier `legmohocon`* (the advanced constructors' lists): a tier-2 extractor (the moho's 0.004 a square, 20
   energy a second of upkeep, 600 storage, 3,900 hp) carrying a 400-build-power construction turret of 400 reach that
   assists, repairs and reclaims; 1,060 M / 14,500 E / 19,400 build time, against the moho's 640 / 8,100 / 14,100 plus
   two construction turrets' 460 / 6,400 / 10,600 (1,100 / 14,500 / 24,700 bought apart). The game builds it as one
   frame and at completion swaps in an extractor and a turret unit (`unit_attached_con_turret_mex.lua`). The advanced
   constructors' `area_mex_def` is the plain moho, so an area upgrade never makes one, and our role word
   `advanced_extractor` resolves to `legmoho`; it has to be named. `legmohobp`, a moho with a drone pad, is in no
   build list.
2. *The Rampart `legrampart`* (the advanced constructors' lists; needs a vent): 600 energy a second with 4,000
   storage, an anti-nuke (coverage 1,600, one missile stockpiled in 90 s for 150 M / 7,500 E), radar 2,100, a jammer and
   three heavy drones `legheavydrone` (90 M / 1,000 E each, 1,650 hp, range 430, kept within 1,500); 2,600 M / 38,000 E,
   8,600 hp, 5x5. Armada's vent building past the geo is the Prude `armgmm`, 1,150 M for 750 a second with 13,900 hp
   and nothing attached; Cortex's the Cerberus `corbhmth`, a geothermal plasma battery, 3,100 M for 450. Legion's plain anti-nuke `legabm` (1,500 M / 40,000 E)
   exists beside it.
3. *Hives and carriers.* A Hive `leghive` (the tier-1 constructors' lists) is 350 M / 9,000 E and holds up to six
   drones `legdrone` (15 M / 500 E, 325 hp, speed 280, a two-shot 12-damage gun at 300 every 0.9 s): three at
   completion, the rest and every replacement one per ten seconds at the drone's cost through the Hive's stockpile (the
   weapon's `metalpershot` 15, `energypershot` 500, `stockpiletime` 10; the gadget's stockpile branch). The drones
   engage within 1,100, never leave 1,000 of the Hive, dock to heal, and lose 20 hp a second once the Hive is dead. The
   Mantis `legvcarry` (450 M, advanced vehicle plant) is the mobile form (three at start, six, within 900) and the
   Rampart's heavy drones the same mechanism. So Legion turns energy into a standing picket at 33 energy per metal with
   no factory and no build power on it: a Hive restocking draws 50 energy and 1.5 metal a second.
4. *Energy in bigger lumps at the same price.* Advanced solar 100 a second for 465 M / 4,080 E / 12,500 build (5x5)
   against Armada's 80 for 350 / 5,000 / 7,950 (4x4) and Cortex's 80 for 370 / 4,000 / 8,150: 4.65 metal per energy a
   second against 4.4 and 4.6, and 41 seconds of its own output to charge against 63 and 50. Fusion 950 for 4,000 /
   25,000 / 66,000 against 750 for 3,350 / 18,000 / 54,000 (Cortex 850 for 3,600 / 22,000 / 59,000): 4.2 against 4.5
   and 4.2. Advanced fusion 3,300 for 10,500 / 69,000 / 340,000 against 3,000 for 9,700 / 69,000 / 312,500: 3.2 against
   3.2. Converters identical (`legeconv` = `armmakr`; `legadveconv` = `cormmkr`, 370 M, 600 into 10.3). The ladder per
   metal a second: fusion with its converters 280 against Armada's 296, advanced fusion with five and a half
   converters 220 against 224. The fusion frame on one advanced construction vehicle: 244 s against 186.
5. *Constructors cheaper, weaker and faster, and a 200-metal claim buggy.* Bot `legck` 100 M / 1,600 E / 75 build power /
   speed 39 / 700 hp (Armada 110 / 1,600 / 80 / 36 / 760; Cortex 120 / 1,750 / 85 / 34.5 / 820); vehicle `legcv` 125 /
   2,100 / 85 / 57 / 1,330 (Armada 135 / 1,950 / 90 / 54 / 1,380); aircraft `legca` 100 / 2,200 / 45 (Armada 110 /
   2,100 / 50); advanced bot `legack` 410 / 195 / 36 (430 / 210 / 33); advanced vehicle `legacv` 530 / 270 / 55.5 (550 /
   290 / 52.5; Cortex 580 / 310 / 49.5); advanced aircraft `legaca` 330 / 115 / 203. Their own energy make is 5 (bot,
   aircraft) and 7 (vehicle) against Armada's 7 and 10. The Aceso `legafcv` (advanced vehicle plant) is 200 M / 2,900 E,
   120 build power at speed 110, and builds only the extractor, solar, wind, radar, Pharos and dragon's teeth: the slot
   of Armada's Consul (250 M / 150 / 78) and Butler (210 / 140 / 75), cheaper and faster. The Proteus `legaceb` (310 M,
   all-terrain, 125 build power) builds tier-1 basics, towers and units in the field, as Cortex's Twitcher and
   Armada's Consul do. The drone plant `legap` is 430 M against the aircraft plants' 650 and 630.
6. *No tier-1 extractor upgrade.* The Overcharged extractor `legmext15` (250 M / 5,000 E, 0.002 a square, 30 energy a
   second of upkeep, 1,110 hp; the game's own faction-substitution table pairs it with Armada's Twilight `armamex` and
   Cortex's Exploiter `corexp`) is in no builder's list at this commit, BARb medium's Legion config marks it "disabled,
   lack of efficiency", and the `legionsimplifiedmexes` lobby option only blocks it further. Legion's step from one to
   four times a spot is the moho or the Fortifier. ([Cookie]Maco at 0:31 of an 8v8: "only legion doesn't get the extra
   t1 mex".)
Beside these, the tier-1 heat rays cost no energy a shot (K-leg-defences), so a Legion base stays armed through a stall.
**Status.** supported (2026-11-05) — local source and the two gadgets; the Hive's rate is the file's and the gadget's
branch, not timed in a game; none of the six built or met in a game of ours
**Evidence.** `units/Legion/Economy/legmohocon.lua`, `legmohoconct.lua`, `legmohoconin.lua`, `legmohobp.lua`,
`legrampart.lua`, `legadvsol.lua`, `legfus.lua`, `legafus.lua`, `legmext15.lua`; `Defenses/leghive.lua` (the carrier
weapon's customparams), `Air/legdrone.lua`, `Air/T2 Air/legheavydrone.lua`, `Vehicles/T2 Vehicles/legvcarry.lua`;
`Constructors/*.lua`; `luarules/gadgets/unit_attached_con_turret_mex.lua`, `unit_carrier_spawner.lua:595-660`;
`gamedata/alldefs_post.lua:376-383`; `luaui/Include/blueprint_substitution/definitions.lua:59`;
`luarules/configs/BARb/stable/config/medium/behaviour_leg.json:318`; the Armada and Cortex files beside them
(`ArmBuildings/LandEconomy/`, `CorBuildings/LandEconomy/`, `ArmVehicles/T2/armacv.lua`, `CorVehicles/T2/coracv.lua`);
`run/matches/1792687833-replay-1787cd6a347311cd1b9559af95fc7a4e/card.json` (Maco's chat). The dump: `lua5.1` over the files, one `print`
per field (the session's scratch, 2026-11-05).
**Would be wrong if.** A Legion game's Hello carried other numbers; a Hive restocked at another rate in a game;
`legmext15` appeared in a constructor's menu in the version we run.
**Used by.** F-eco-legion (docs/briefs/facts/economy.md); the Legion bends of D-energy-second, D-tier2-sprint and
D-late-economy; the glossary's `legdrone` and `legheavydrone` entries (run/unit_stats.py follows `carried_unit`)

### K-leg-lab-energy-inverts
**Claim.** The lab article's premise, that bots cost about twice the energy per metal of vehicles (Pawn 16.7, Grunt
19.1 against Blitz 8.2, Incisor 9.2), does not hold for Legion. Its raider pair is level, Goblin 14.0 (30 M / 420 E)
against Alaris 13.1 (65 / 850), and its lines go the other way: Satyr 8.3 (60 / 500), Karkinos 7.7 (310 / 2,400),
Ballista 12.0 (160 / 1,920) against Decurion 12.0 (300 / 3,600), Helios 15.0 (160 / 2,400), Lance 15.0 (240 / 3,600),
Barrage 12.8 (250 / 3,200). By factory list the medians are tier-1 bots 11.4 and tier-1 vehicles 15.0 (Armada 11.1
and 13.0, Cortex 10.6 and 13.2). At the plant alone (150) and with the commander (450): a Goblin draws 70 and 210
energy a second (a Pawn 82 and 245), a Satyr 62 and 188, an Alaris 77 and 232 (a Blitz 68 and 202), a Decurion 112
and 338 (a Stout 93 and 278). So for Legion the bot lab is the cheap-energy opener and the vehicle plant the dear one,
and "the vehicle plant on solar maps" has no Legion reading.
**Status.** supported (2026-11-05) for the arithmetic (unit files); as a statement about play, refuted the same day:
the 36 Legion sides above OS 40 open the vehicle plant 24 times and the bot lab 12 (K-leg-duel-openers-os40), and the
vehicle openers run a Wheelie mass or Alaris from the first minute, so the lab is chosen for the raiders' speed
(Wheelie 160, Alaris 99, Goblin 75) and the player's habit, not for the energy bill; BARb medium as Legion opens the
vehicle plant and Apex the bot lab (K-leg-both-stock-ais-play-legion)
**Evidence.** `Bots/leggob.lua`, `leglob.lua`, `legkark.lua`, `legbal.lua`; `Vehicles/leghades.lua`, `leggat.lua`,
`leghelios.lua`, `legrail.lua`, `legbar.lua`; the Armada and Cortex twins; every factory's `buildoptions` at 54199a0d
(the draws are energy times build power over build time, F-eco-build).
**Would be wrong if.** (as a play statement) Legion duels above OS 40 opened the vehicle plant as Armada's do: they do,
2 to 1, on solar and wind maps alike.
**Used by.** D-lab-choice's Legion bend; M-leg-duel-opening (docs/briefs/meta/legion.md)

### K-leg-open-os49-canis-river
**Claim.** The two highest-rated Legion duels available, ozvideo (OS 49.0 and 48.4) as Legion against maratus (OS 49.1
and 49.5) as Armada on Canis River v1.4 (wind 4 to 14, tidal 10), 2026-10-03, game version test-31479, both won by Legion at 13:25 and 12:11,
opened identically: three extractors at 0:03, 0:10 and 0:17, a solar at 0:24, turbines at 0:34 and 0:40, the bot lab
at 0:47, then seven to nine turbines to about 1:57 and nothing else; the lab's first product a constructor at 1:26
(a second at 1:43 in game two), then Goblins without a pause, one every three to six seconds, 173 and 179 by 10:00; a
construction turret on the lab at 2:09 (three or four by 10:00); Zagreus four and seven; a second bot lab at 5:08 and
6:36, a third at 6:21 and 7:56, a fourth at 9:29; eight or nine solars from 4:11 and one advanced solar at 8:57 and
7:11; energy storages at 5:29 and 8:15 in game two; Satyrs joining the stream from 7:25 (26 by 10:00) in game one;
four to fifteen Pharos, four to nine radars; no vehicle plant in either game. Won without an income lead: cumulative
metal at 10:00 16,750 against his 17,630 and 12,850 against 15,490, energy 187,000 against 175,000 and 162,000
against 152,000, units produced 326 against 192 and 286 against 176; army value behind at 8:00 (2,700 against 3,863;
1,530 against 1,943) and ahead at the end (7,210 against 3,300 at 12:00; 3,174 against 1,665 at 10:00); Goblins died
by the dozen to Blitzes (61 and 73 of them a game) and Stouts, and the last minute of each game is his base dying (25
and 28 extractors, 18 and 21 solars). The opponent: bot lab at 0:28 for Fleas and Pawns, a vehicle plant at 3:13 and
3:45 for Blitzes (28 and 47 by 10:00) and Stouts.
**Status.** reported (2026-11-05) — one player, one opponent, one map, one session; read from the bar-replays
processing of the engine replay, not our cards; the 36 other Legion duels above OS 40 in that catalogue are queued
there and are the test
**Evidence.** http://b-dtop.b.gecko.network:8770/match/d011c16a8ebd392b093b7dcb82c9d784 and
/match/9d0ec16a69d1a0c01178d7a69ef203ea (`/api/match/<id>` and `/api/game-event/<id>` for the events); read with
`run/replays/bar_replays.py <id>`.
**Checked 2026-11-05 (the queued games read, K-leg-duel-openers-os40).** The vehicle plant opens 24 of the 36 sides at
this level and the bot lab 12; ozvideo's Goblin stream is one of the bot lab's forms, and Altf4_ubivator (OS 57) plays
it with Phobos. The claim stands as what this player does, not as the Legion opening.
**Would be wrong if.** The queued 36 showed vehicle-plant openers at this level, or lab-first Legion losing its duels.
(They did, 24 of 36; the lab-first sides won 7 of 12.)
**Used by.** M-leg-duel-opening (docs/briefs/meta/legion.md); D-lab-choice's Legion bend; K-leg-open-os27-great-divide
(checked against it)

### K-leg-duel-openers-os40
**Claim.** Over the 36 Legion sides of the 29 public duels with both players above OS 40 that replay under the current
engine (selection 6 of the bar-replays tool, 2026-02 to 2026-10, game versions test-29538 to test-31723; nine maps,
sixteen players, OS 43 to 60), Legion opens the vehicle plant 24 times and the bot lab 12, never air. The openers:
- *Vehicle plant* (24 sides, 10 won; placed at 0:41 median, 0:31 to 0:52, after three or four extractors and a solar or
  two turbines). Two forms. (a) *The Wheelie mass*, the Legion form of thebluegecko's Rover mass (K-open-matt-order-
  timings): two or three constructors, then Wheelies without a pause, 34 to 49 by 5:00, the first Alaris only at 4:44
  to 7:09 and 45 to 88 of them by 10:00; the Wheelies kill his scout cars and bare extractors (PRO_Fiddler112, OS 54
  to 55, 21 to 74 kills a game, 59 to 117 Wheelies lost) and the Alaris then kill his raiders and extractors by the
  hundred (Blitzes 73 and 97 in two games). Played by PRO_Fiddler112 (six sides), Chomolungma, Altf4_ubivator (one
  side), phosphophyllite, cooolcat1239, BM_Lovish, erevos, mostly on Flats and Forests (wind to 12) and Hades Ponds
  (14). (b) *Alaris first*: a constructor, a Wheelie, then Alaris from 1:15 to 1:53, 14 to 23 by 5:00, 51 to 111 by
  10:00, Helios from 5:00 ([APM]Glados OS 51, four sides, four wins on Eternal Consequences; GlorpChud, modulo,
  cooolcat1239). An Alaris army killed 231 Goblins in one game and still lost to the stream (Rosetta, GlorpChud
  against iyc).
- *Bot lab* (12 sides, 7 won; placed at 0:31 median, 0:20 to 0:47, after one to three extractors and a solar or a
  turbine). Constructors first (two to five before the first Goblin in eight of twelve), then Goblins from 1:07 to
  2:18, 24 to 57 by 5:00 and 50 to 242 by 10:00; Altf4_ubivator (OS 57, three sides on Hades Ponds, two won) adds
  Phobos, 8 to 15 by 10:00, and the Satyr comes in the long games. Played by ozvideo, Altf4_ubivator, Haundreyy and
  VonComet (an Argent Strata mirror), iyc, GlorpChud, cooolcat1239, I0NST0RM, C3BO; on Canis River, Hades Ponds,
  Carrot Mountains, Argent Strata.
- The opener follows the player more than the map: Flats and Forests is vehicles in 8 of 8, Eternal Consequences 6
  of 7, Hades Ponds 4 vehicles (0 won) and 3 bot labs (2 won), Rosetta (a solar map, wind to 10) 3 vehicles and 1
  bot lab (the bot lab won their meeting). In the two Legion mirrors where the openers differed the bot lab won both
  (Rosetta, Hades Ponds). The wins by opener are 10 of 24 and 7 of 12, within noise of each other and of the OS
  gaps in the pairings.
- Common to both: five constructors by 5:00 (median 5.5 and 5), 15 extractors by 5:00 and 34 to 40 by 10:00, one
  or two construction turrets by 5:00, two or three Pharos by 5:00, the second factory a second vehicle plant (14),
  the drone plant (8, at 6:08 to 11:08) or a bot lab (7). No Hive in any side by 10:00. The energy is the map's:
  turbines by the dozen where the wind tops 14 or more (28 to 68 by 10:00), solars on Flats and Rosetta (15 to 45).
**Status.** supported (2026-11-05) as a count of what strong players do; the win rates are noise-level (36 sides,
mirrors and OS gaps mixed in); the energy-bill arithmetic (K-leg-lab-energy-inverts) does not decide the Legion
lab, the player's habit and the Wheelie's and Alaris's speed do
**Evidence.** `run/replays/legion_openings.py` over `run/data/bar-replays/selection6.ids` (the cached
`/api/match` and `/api/game-event` of the 29 games; the ten June 2025 games of the selection crashed the engine
in replay, signal 11, and are not in the count). The per-side table is the script's output.
**Would be wrong if.** A larger cut (above OS 35: 63 games in the catalogue) reversed the 2:1, or the bot lab's
wins held up over more mixed mirrors.
**Used by.** M-leg-duel-opening (docs/briefs/meta/legion.md); D-lab-choice's Legion bend; K-leg-open-os49-canis-river
(one of its two forms)
