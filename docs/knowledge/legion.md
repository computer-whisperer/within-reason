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
5-14 soldiers at 10:00 against the 16-39 it fields as Armada or Cortex; its random search trains Armada and Cortex
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
**Status.** supported (2026-10-14) — a count over `run/data/replays/manifest.jsonl` (`players[].faction`)
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
**Status.** supported (2026-10-14) for the numbers and build lists (local source); the drone-spawn mechanics are the
descriptions' and the replay's, not read in the gadget
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
