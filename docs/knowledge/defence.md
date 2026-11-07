# Static defence

What armed structures do and what people do with them. From the survey `docs/studies/2026-11-06-static-defence-survey.md`
(five pools, three tools) and the game's unit files under `upstream/Beyond-All-Reason/units` on game source 54199a0d;
the mechanics claims cite the file. "Yield" throughout is the metal of enemy units whose last hit was a defence's, over
the defence metal built: a floor on what a tower earns (it under-credits artillery and towers that soften).

### K-def-people-hold-a-sixth-of-their-army-in-defence
**Claim.** People at every level keep ground defence at 15-16 % of the soldier metal they build, 8 % of everything
standing from 12:00 on, in duels and team games alike; it does not separate winners from losers (14.7 % against
16.6 % in high-OS duels), what separates them is who loses theirs (winners 550 median, losers 2,100).
**Status.** supported (2026-11-06): 1,054 sides of public duels above OS 40 (`run/replays/defence_duels.py`), 2,313
sides of people in our replayed games (`run/defence_pool.py --summary`; OS 35+ team games 15 %, duels 16 %).
**Evidence.** The study §2-3; `run/data/bar-replays/`; the carded replays.
**Would be wrong if.** A pool of winners at one OS band showed a defence share half the losers' at the same minute.
**Used by.** (candidate: the brief's defence paragraph; proposal §9 of the study.)

### K-def-our-seats-hold-half-the-share-and-get-half-the-yield
**Claim.** The flagship builds ground defence at 9-10 % of its soldiers (team and duel), WRDistilled the cheapest tower
only at a third of BARb's defence metal by 15:00, and the flagship's defence kills 0.7 of its metal in team games
and 0.3 in duels against the people's 1.1 and 0.8. The gap in yield is placement: ours stand beside extractors by
rule (H-PLAYER-TURRET-WITH-THE-SPOT, D-SENTRY), the people's at the lanes and the outer pairs the raids come to,
two thirds beyond 1,500 from the start and one in six nearer his start than theirs.
**Status.** supported (2026-11-06): 609 flagship sides, 1,541 heuristic-bot games, the placement over 23,700 duel towers.
**Evidence.** The study §2, §3 (placement), §4.
**Would be wrong if.** A batch with towers sited at the hold point or the raid's lane showed no rise in the yield column.
**Used by.** (candidates: the brief; D-SENTRY's site; the hands' menu.)

### K-def-the-cheap-tower-is-the-tower-and-the-heavy-laser-is-one-in-three
**Claim.** Per high-OS duel side: 14.9 Sentries and 3.3 Beamers (Armada), 14.1 Guards, 5.1 Exploiters and 3.7 Twin
Guards (Cortex: the armed extractor is the outpost tower in 61 % of Cortex sides, five where it is), 28 Pharos
(Legion); 0.4-0.5 pop-ups at tier 2; tier-1 plasma (Gauntlet, Agitator) in 16 of 1,054 sides; tier-2 defence in 110.
In team games the same players add Pit Bulls, Pulsars and (against bonused BARb) Pulsar lines of 16-28.
**Status.** supported (2026-11-06).
**Evidence.** The study §3 "What they build", §2 the pilot.
**Would be wrong if.** A Cortex pool above OS 40 built Twin Guards or Wardens as often as Guards.
**Used by.** (candidate: the hands' menu lacks the Beamer, the Twin Guard and the Exploiter: `roster.rs` `USUAL_*`.)

### K-def-raiders-are-the-turrets-food-and-the-turrets-killers
**Claim.** By metal, what duel defences kill is Incisors and Blitzes first (0.42 M each in the pool), then Brutes,
Grunts, Stouts, Pounders, Pawns; what kills defences, where the engine names it, is the same raiders in numbers
(Incisors 1,581 tower deaths, Blitzes 1,328) and the line tanks, with 6,995 of 15,000 tower deaths unattributed
(artillery, blasts, beams from out of sight). Kills credited to defences run 5:00-20:00, the minutes they are built in.
**Status.** supported (2026-11-06), 527 duels.
**Evidence.** `run/replays/defence_duels.py` second pass.
**Would be wrong if.** A pool showed towers killing mostly skirmishers or artillery.
**Used by.** (the brief's raid paragraphs already say raiders avoid towers; this is the other side of it.)

### K-def-a-tower-is-a-raiders-gun-on-a-line-units-body-at-twice-the-reach
**Claim.** The Sentry (85 metal) is 189 damage a second per 100 metal and 7.3 health per metal at reach 430: a Pawn's
damage per metal (167), a Stout's health per metal (8.0), 2.4 times a raider's reach, for 20 energy a shot. Damage
dealt per metal by dead units in high-OS duels: ground defence 7.4, raiders 6.2, other soldiers 6.5. The structure's
edge is reach and health against what comes inside the reach, not a multiple of a soldier's efficiency; everything
that reaches 435 or more (Rocketeer 475, Aggravator 475, Whistler 550, Lasher 575, artillery) makes it dead metal.
**Status.** supported (2026-11-06): the unit files (`armllt.lua` range 430, reloadtime 0.467, damage 75,
energypershot 20) and 13,358 dead towers' damage rows.
**Evidence.** The study §5; `crates/bot/data/units.json`.
**Would be wrong if.** Damage dealt per metal of dead towers ran twice the soldiers' in some pool.
**Used by.** H-HANDS-TURRETS-WITH-A-PARTY counts a turret at about three times its metal in odds; D-WEIGHT the
same. Three is the fight-on-the-spot number (a tower is not killed first and does not retreat); it is not an
efficiency number.

### K-def-towers-fire-by-radar-and-the-big-ones-carry-their-own
**Claim.** A weapon may target anything its ally team holds in sight or on radar (`RecoilEngine`
`rts/Sim/Weapons/Weapon.cpp` 1045: `LOS_INLOS | LOS_INRADAR`), and the tier-2 towers carry radar past their reach:
the Bastion sees 780, has radar 1,200 and reaches 1,100 (`legbastion.lua`); the Pulsar 780, 1,500, 1,400
(`armanni.lua`). So a Bastion line kills what walks within 1,100 whether or not anything of his sees it, and what
it kills reads in our records as deaths to an unseen attacker (`by_d` -1): human-8v8-1 (the 5v5) t5 lost 62,350 of
metal to unseen attackers and 136,000 to no attacker at all (missiles and blasts), against 16,300 to towers in sight.
**Status.** supported (2026-11-06): engine source, unit files, the 5v5's records.
**Evidence.** As cited; the study §2 the 5v5.
**Would be wrong if.** A unit under a Bastion's reach but outside every radar and sight of his survived there.
**Used by.** (candidates: the report's "turrets covering it reach N" line should carry the tower's radar reach and
say that reach needs no sight; the simulator should give towers their radar.)

### K-def-energy-per-shot-is-the-towers-upkeep
**Claim.** Laser towers pay energy a shot and hold the shot when the store cannot pay: Sentry 20 (reload 0.47 s: 43
a second firing), Beamer 6 at 0.1 s (60 a second), Overwatch 75 at 1.8 s, Pulsar 1,500 at 9.9 s, Bastion 500 at 3 s,
Xyston 1,800 a rail. A line of ten Beamers firing is 600 energy a second; a stalled store is a silent line. The
Cacophony, the pop-ups (Pit Bull, Scorpion, Chimera) and the plasma towers fire for nothing.
**Status.** supported (2026-11-06): the unit files' `energypershot`; `crates/combatsim` charges it (`sim.rs` 728).
**Evidence.** As cited.
**Would be wrong if.** A tower fired in a stall in a replay.
**Used by.** the simulator; (candidate: the brief's defence paragraph: a tower line is an energy bill.)

### K-def-shields-stop-cannon-shells-and-nothing-else
**Claim.** A plasma shield (Overseer `corgate`, Keeper `armgate`, Soteria `legdeflector`: 3,000-3,200 metal) holds
6,175 shield power in a 550 radius, regenerates 130 a second for 562.5 energy a second, starts at 2,090, and is
`smart` (enemy projectiles only). Its `intercepttype` is 1, and the engine gives a weapon's `interceptedByShieldType`
by type when the unit file sets none: Cannon 1, LaserCannon and BeamLaser 2, MissileLauncher 4, AircraftBomb 8,
Flame 16; the game then sets every StarburstLauncher (the nukes, the Arbiter's rockets) to 1024. So a stock-options
shield stops cannon shells (Stouts, Thugs, Fatboys, Mausers, artillery, the pop-ups' and plasma towers' shots) and
nothing else: rockets (Rocketeer, Thanatos, Arbiter), lasers and heat rays, bombs and nukes pass. Under the lobby
option `experimentalshields = absorbeverything` or `bounceeverything` every weapon is set to 1 and everything is
stopped (and bounced at 0.53 of the power); the 5v5 of 2026-10-18 ran `unchanged`, the public 8v8 pilot of
2026-11-06 ran `bounceeverything`. A shield that has taken 6,175 in a few seconds is down until it regenerates.
**Status.** supported (2026-11-06): `corgate.lua`, `legdeflector.lua`, `gamedata/alldefs_post.lua` 1099-1125,
`RecoilEngine/rts/Sim/Weapons/WeaponDef.cpp` 460-538; agrees with F-fight-big-weapons.
**Evidence.** As cited; the report names shields seen (`scout.rs` `shield_lines`).
**Would be wrong if.** A rocket's or a laser's shots were seen stopped by a shield in a replay with stock options.
**Used by.** (candidate: the brief; the user, human-8v8-1 46:50: "within reason really doesn't know what to do with
the shields".)

### K-def-anti-nukes-are-one-interceptor-per-90-s-and-silos-outnumber-them
**Claim.** An anti-nuke (Citadel, Prevailer, Aegis: 1,500 metal) stockpiles one interceptor per 90 s for 7,500 energy
and covers 2,000 (`legabm.lua` `coverage`, `stockpiletime`, `energypershot`); each interceptor takes one missile. A
silo stockpiles one missile per 180 s (Legion and Cortex; Armada 120) for 187,500 energy (Armada 125,000). So N
silos firing within one coverage beat the interceptors stocked there, and the people's eight anti-nukes in the 5v5
(two Prevailers from 27:23, two Citadels from 30:23, four Aegis at 51:00) fell to Apex's 23 silos (14 Supernovas and
9 Armageddons standing at 57:00). A turtle on tier-2 and tier-3 defence ends that way or not at all.
**Status.** supported (2026-11-06): unit files; the 5v5's `al` rows at 57:00 (`record-0.jsonl`).
**Evidence.** As cited; the 5v5's two-sided replay (`defence-5v5-altored-humans`) for the strike clocks.
**Would be wrong if.** An anti-nuke was seen intercepting two missiles within 90 s of each other with one interceptor stocked.
**Used by.** the brief's anti-nuke paragraph (one under every home); `crates/nukes` (the strike engine).

### K-def-walls-are-geometry-not-efficiency
**Claim.** Dragon's Teeth (8 metal, 2,800 health, `armdrag.lua`) and the Fortification Wall (40 metal, 8,900) are
blocking footprints with no weapon: tier-1 units cannot path or shoot low through them, so they funnel and shield.
Nothing in the pools measures their value by last hit; their use is in the placement, which the replay batch has
to read.
**Status.** conjectured (2026-11-06) from the files; unmeasured in play.
**Would be wrong if.** The replay batch showed people's walls never standing where the trickle comes.
**Used by.** nothing yet; the hands' menu has no wall.
