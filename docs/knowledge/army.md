# Army

### K-army-crowds-are-never-idle
**Claim.** A large group ordered to a rally point is never all idle at once, so "count idle units at the rally point" is a
broken launch condition; group membership has to be tracked explicitly.
**Status.** supported (2026-09-19)
**Evidence.** v2-easy-0704/00: army grew to 148 with only 7 waves sent in 35 minutes. After membership-based waves
(v3-easy/03): 8 waves by minute 18.
**Would be wrong if.** (mechanical; would need an engine change)
**Used by.** H-ARMY-WAVES.

### K-army-fighters-before-constructors
**Claim.** The first factory units should be fighters; an opening of several constructors loses the early game to raids.
**Status.** narrowed (2026-09-19): true of five constructors and no fighters, false of two. Against medium, two
constructors before the first fighter cost nothing in army by minute 4 and led by minute 8 (v21-early-expand-ab10, see
K-open-sim-constructors-first). Confounded at the time with K-rules-factory-shift-means-five (both changed in v4).
**Evidence.** v3-easy/04 (5 constructors, 0 army at minute 4, lost) vs v4-easy wins spread over both corners and factions.
**Would be wrong if.** Constructor-first with correct single-unit orders did as well over 24+ matches.
**Used by.** H-PROD-BATCH.

### K-army-piecemeal-midmap
**Claim.** Our waves meet BARb's army in the middle of the map and trade there; reinforcements arrive one by one and die.
**Status.** conjectured (2026-09-19)
**Evidence.** v5-probe/00: attacker centroid stays around (3500-3900, 3700-4300) on a 7168-wide map, attacker count
oscillating 37 -> 20 -> 34 with few idle.
**Would be wrong if.** Attacker positions showed them reaching the enemy base in strength and losing there instead.
**Used by.** (none yet; motivates regrouping before contact)

### K-army-home-defence-does-not-protect-outposts
**Claim.** Keeping the army at home (our default until a wave is ready, or a `defend` stance) does not protect extractors: raids
kill the outer ones, the constructor sent to rebuild dies too, and the economy freezes at 2-3 extractors.
**Status.** conjectured (2026-09-19) — one strategist game plus the medium diagnosis
**Evidence.** opus-first/00 (docs/transcripts/2026-09-19-opus-first.md): 2-3 extractors and +6-9 metal for 20 minutes with 6
turrets and 8-17 idle bots at home; Opus identified it at 4:55, 7:10 and 8:55 and asked for a directive to station the army
at a map point as an escort. Same shape as the v5-medium losses (K-opp-medium-wins-by-20).
**Would be wrong if.** Stationing the home group forward, between the outposts and the enemy, did not raise extractor survival.
**Used by.** (candidate: army `station` directive / forward rally near the most exposed extractor cluster)

### K-army-waves-chased-raiders
**Claim.** Targeting "the visible enemy nearest the enemy start" sends waves to wherever an enemy was last seen, which is
usually a raider inside our own half; the army then spends the game near home.
**Status.** supported as a description of the old behaviour (2026-09-19); the replacement is untested
**Evidence.** dropped-orders-2/00: attacker centroid (2000-2900, 1300-2400) with home at (2032,1188) through minute 20; earlier
batches logged waves launched at (1593,1804), (2326,2943). Replacement: remembered enemy buildings, nearest to us first,
else the presumed enemy start (H-ARMY-TARGET rewritten).
**Would be wrong if.** With building targets the attacker centroid still stayed in our half.
**Used by.** H-ARMY-TARGET.

### K-army-base-maze
**Claim.** Base buildings placed around one anchor with a 3-square (24-elmo) gap form a maze that a T1 army cannot leave:
units fail their moves inside the base, never reach the station, and waves "launch" without going anywhere.
**Status.** supported (2026-09-19)
**Evidence.** v10-sites: attackers' average position sat within 350 of our start for 7-15 whole minutes in 6 of 8 NW
games (up to 64 attackers parked). nw-stuck-diag (8 NW games): ~17,000 soldier move failures, all within ~500 of our
start; after the yard layout (nw-layout, 8 NW games) 149, none clustered at home. It did not explain the corner gap
(K-maps-quicksilver-corner-asymmetry). The false alarm it raised: station after station was declared
unreachable (the failing units were in the base, not at the station) until the station fell back to the start point
itself, in the middle of the maze.
**Would be wrong if.** Soldier move failures clustered at home again with the yard layout.
**Used by.** H-ECO-BASE-LAYOUT.

### K-army-dead-waves-are-resurrected
**Claim.** BARb easy builds resurrection bots (armrectr / cornecro) and raises our dead attackers in its half; the units
that finally kill our base are partly our own.
**Status.** supported (2026-09-19) — inferred from unit names, not from watching a resurrection
**Evidence.** nw-layout fight ledgers: in all 4 losses the enemy fielded units of OUR faction (7-42 of them killed by us,
2-49 of our losses caused by them); in the 3 wins and the timeout, none. `killed armrectr`, `killed cornecro` appear.
**Would be wrong if.** BARb could build both factions' units by some other route (capture, a shared lab).
**Used by.** (none yet) — candidate: do not feed waves into a defended base; fight where we can reclaim the field.

### K-army-waves-die-to-static-defence
**Claim.** Our waves arrive strung out and die to static defence they never see. In a 40-minute timeout we lost 739
units in the enemy half for 104 kills; 427 of those to attackers we had no sight of, 90 to the enemy commander, 114 to
LLT/HLT/HLLT turrets.
**Status.** supported (2026-09-19) — one game read closely (nw-layout match 01), others look alike
**Evidence.** `run/matches/1789857672-nw-layout/01/bot.log` fight ledger.
**Would be wrong if.** Waves gathered at a staging point before engaging lost just as badly.
**Used by.** (none yet) — candidates: gather before the assault, skip targets under turret cover while the army is
small (BARb's own 0.75-power rule), artillery against turrets, a scout for sight.

### K-army-commander-sniped-after-wave-leaves
**Claim.** Many of our losses are not collapses: an enemy group of ~15 walks into a healthy base just after a wave has
left and kills the commander in 10-20 seconds, which ends the game.
**Status.** supported (2026-09-19)
**Evidence.** v11-layout match 14: wave 7 (32 units) left at f=34560; at f=36105 the commander had 3535 health, at
f=36495 it was dead to a pack of armwar, with 8 extractors, 4 labs and 28 soldiers alive, 10 of them at home. v10-sites
losses 04, 08, 12: the commander dies at home to corthud / corstorm from under 300 away.
**Would be wrong if.** With H-ARMY-RECALL, losses with a healthy economy (8+ extractors at the end) still happened as often.
**Used by.** H-ARMY-RECALL.

### K-army-we-never-raid
**Claim.** Marching every soldier in one group at one target leaves the enemy's extractors alone all game, while BARb
medium's small fast groups strip ours.
**Status.** supported (2026-09-19)
**Evidence.** v13-medium: per game we killed 3.1 enemy extractors from NW and 7.4 from SE, and lost 22.2 and 18.5.
**Would be wrong if.** With raid squads the enemy extractors killed per game did not rise.
**Outcome.** It did not (3.1 -> 4.9, noise): BARb medium guards its extractors with a turret each, and squads of three light
raiders die to them. The claim stands as a description; raiding as a remedy is retired.
**Used by.** (none; H-ARMY-RAID retired)

### K-army-defence-is-positioning-and-mix
**Claim.** In the first Sonnet commander game the loss was mostly defender positioning and unit mix, with a smaller
part played by units that huddled at a cliff edge they could not cross.
**Status.** reported (2026-09-19) by the user from the engine replay (he has StarCraft 2 experience, not BAR).
**Evidence.** `docs/transcripts/2026-09-19-sonnet-commander-1.md`: posts set reactively, after an extractor cluster was
already raided; a posted squad drawn out of its radius; light armpw/armrock/armham against corthud/corstorm. The
huddling predates the terrain work (K-maps-terrain-not-straight-lines): squad posts and orders were straight-line
points, some on the far side of a cliff.
**Would be wrong if.** With posts snapped to walkable ground and a sound mix, the exchange ratio in our half stayed as bad.
**Used by.** (none yet) — positioning and mix are the levers meant for the LLM ([[llm-levers]] in project memory).

### K-army-verdicts-v17
**Claim.** Against BARb medium our losses have two interlocking shapes. (1) Cheap lone raiders (armfav 31 metal, armflash,
corak) farm extractors and constructors while the home group and the turrets stand in one clump at the lab; the bot
rebuilds the same spot into the same raider, up to five times. (2) A wave takes the entire army at a target it has not
weighed, arrives in instalments, dies, and the raid that follows meets nothing at home. Fight quality is secondary:
in two losses the army metal traded 1:1 and the game was still lost on replacement rate (2 extractors against 26).
**Status.** supported (2026-09-19): 14 of 14 losses of one batch, each read by an analyst from curves and scenes; three
claims spot-checked against the raw records by me.
**Evidence.** `run/tally_verdicts.py run/matches/1789868387-v17-truth-medium`: primary cause expansion_raided_undefended 7,
blind_wave_into_defence 3, defenders_out_of_position 3, economy_never_grew 1; defenders_out_of_position contributes in
9 more. Specifics: from the NW start the third and fourth metal spots, (2144,2144) and (2320,2352), lie 963 and 1199
from home, beyond the base turret line (650) and inside OUTPOST_DISTANCE (1200), so no rule ever gives them a turret;
they are the spots killed 3-5 times a game (match 18: 2:44, 5:11, 8:11, 11:50). Recall needs 6 intruders, so a single
raider never triggers it. The wave rule is a head count (20, 25, 30...) that leaves 0 soldiers at home. The
constructor target (3 + extractors/2, earlier 2 + extractors/4) is lowest exactly when extractors are being lost.
In every NW loss "they first have 1.5x our army" falls in minute 6-8, even on level extractors.
**Would be wrong if.** With spots remembered as hot, cover before expansion, a home guard that stays, and a wave gate
on known enemy value, the same tags still led the tally.
**Used by.** (next changes)



### K-army-combat-prediction
**Claim.** Once a fight is decisive (a side with a real force on the spot loses at least half of it), the side with the
higher fighting power, metal value weighted by the duel table with turrets at 3x (1.5x at first: 92 % right; 95 % at
4x, `run/predict_check.py`), loses the smaller share in 9 cases
of 10. The matchup weighting adds only a point or two over plain metal in our games, because both sides field a narrow
set of units. It says nothing about skirmishes and raids, where who loses less is a coin toss (56 %), and nothing about
what we have not seen.
**Status.** supported (2026-09-19)
**Evidence.** `run/predict_check.py` over v17-truth-medium and v18-verdict-fixes (48 games): 273 decisive engagements,
89 % right (plain metal 88 %), 93 % beyond 1.5:1, 94 % beyond 2:1, 96 % beyond 3:1; correlation of log power ratio with
log loss-share ratio 0.85. The selection is by outcome (decisive fights only), so this is the accuracy given that a
fight goes to the finish, not the accuracy of "should we start it".
**Would be wrong if.** With retreat and launch decided by these odds, our share of metal lost in their half did not fall.
**Used by.** H-ARMY-WAVE-GATE, H-ARMY-RETREAT, the posted squads' intruder response (`brain/combat.rs`).

### K-army-verdicts-v18
**Claim.** With raided spots closed, turrets on the ring and a wave gate, the lone-raider losses are gone and the bot
reaches minute 10 level; what decides games now is (1) one mobile enemy ball of 700-3000 metal that our split-up
defence meets in packets, (2) a wave gate that weighs only what stands at the target while their army is a roaming
block it has never scouted ("1500 known" against a real 3205), (3) our force arriving in speed order, light raiders
first and rocket infantry 600 elmos behind, so we lose fights we outweigh, (4) constructor attrition: 16-31
constructors lost a game, the bigger bill behind every extractor count, and (5) a far line of spots 2000-2400 from
home bought and swept in every game.
**Status.** supported (2026-09-19): 12 losses of v18-verdict-fixes read by three analysts; the production-stall and
global "turret on its way" bugs they reported were confirmed in the code and logs.
**Evidence.** `run/tally_verdicts.py run/matches/1789870160-v18-verdict-fixes`. Bugs found: the army parked in the lab
yard jams the factory exits (1500 metal banked for five minutes with two labs alive, soldiers' moves failing beside the
base); `unguarded_outpost` treated any turret under construction anywhere as cover for every outpost; the
quiet-at-home clause of the gate never clears under continuous raiding.
**Would be wrong if.** After the fixes (station clear of labs, positional cover test, gate ceiling and remembered
army, scout, expansion reach) the same tags led the next tally.
**Used by.** H-ARMY-STATION, H-ARMY-WAVE-GATE, H-ARMY-SCOUT, H-ECO-REACH, H-ECO-OUTPOST-TURRET.

### K-army-won-games-are-recalled
**Claim.** The south-east timeouts at three to five times the opponent's army were our own retreat rule: it weighed the
handful of attackers in contact against what they met and, finding them outmatched, sent the whole wave home.
**Status.** observed (2026-09-20) in the logs; fix (H-ARMY-REGROUP) unmeasured.
**Evidence.** v26-radar-startguess-ab/03 (timeout, 13,800 v 3,100 army at minute 20): "wave 11: 271 units to (2336,
3008)" at f=67110, then at f=67140 "retreat: 6 attackers at odds 0.60 ...; everyone home"; the same one second after
waves 10 and 12, with 4 and 2 attackers. Every retreat is followed by a 90-second ban on waves.
**Would be wrong if.** With the fix the south-east still times out at the same rate.
**Used by.** H-ARMY-REGROUP.


### K-army-groups-arrive-strung-out
**Claim.** A group given one order strings out by unit speed and meets the enemy a few at a time; the loss happens in
the first seconds of contact, before any judgement can be made.
**Status.** observed repeatedly (2026-09-19/20); the fix (H-ARMY-MARCH) is unmeasured.
**Evidence.** Commander game 8c, 32:09-32:26: 10 Centurions reached the enemy base about 350 elmos ahead of 36 Hammers;
army value fell 9750 to 6930 in 17 seconds. Commander game 6 (lost): three squads ordered to one rally point arrived one
after another and fought alone. v26: waves recalled because the six fastest attackers met something alone
(K-army-won-games-are-recalled). BARb moves a squad on one shared path at its slowest member's speed
(docs/studies/barb-comparison.md).

### K-army-posted-squads-chase-singles
**Claim.** A posted squad that turns, all of it, on the nearest intruder is led around by single cheap units.
**Status.** observed once, from the record (2026-09-20, Opus analyst, code path confirmed).
**Evidence.** Commander game 7/01, 10:02-11:56: one 43-metal Grunt walked all 38 members of `mainforce` from (1485,3376)
to (3528,3027), about 1800 elmos, for 226 metal killed. The post code re-aimed every member at the single nearest
enemy inside the radius every 2 seconds. Now: the biggest group, met by the nearest members good odds take.

### K-army-small-waves-restage-the-army
**Claim.** Re-staging every committed attacker whenever a wave leaves holds the army at the staging point for as long
as small waves keep leaving, which under an `attack` stance (waves of three) is for good.
**Status.** observed once (2026-09-20), commander game 9 south-east (won anyway, by the commander taking the soldiers
into a squad).
**Evidence.** bot.log: wave 1 of 65 at f=28410, then waves 2-27 of 3-4 units about every 15-30 s; attackers 76 -> 101
around (3059, 2995)-(3133, 3108) from f=32400 to f=37800, target 1300 away. The commander's note at 20:44: "stalled
exactly at D4 for many minutes despite attack_target".
**Used by.** H-ARMY-REINFORCE.

### K-team-seats-alone-split-the-army
**Claim.** Two seats of ours that do not talk each hold their own wave against their own target and their own estimate
of the defenders, and neither goes when together they would.
**Status.** observed (2026-09-20), team-2v2-smoke match 0 on Great Divide: both seats held 20-29 soldiers at odds
0.71-1.12 for three minutes against (584, 3592) and (2568, 3592); when they went, they went 2900 frames apart to
different places.
**Evidence.** `run/matches/1789913869-team-2v2-smoke/00/bot.log`, the `wave held` lines from f=20700. With the board
(team-2v2-defend) 12 of 14 waves left within 15-300 frames of the partner's, at the same target.
**Used by.** H-TEAM-BOARD, H-TEAM-WAVES.

### K-team-a-partner-dies-beside-an-idle-army
**Claim.** A seat that only defends its own base and extractors lets its partner be overrun next door.
**Status.** observed once (2026-09-20), team-2v2-board match 0: seat 1 was down to 2 extractors and 2-5 soldiers from
minute 8 with 17-28 enemies in sight, while seat 0 kept 19-26 soldiers at its station 2200 away; lost at 20 minutes.
**Evidence.** `run/matches/1789914072-team-2v2-board/00/bot.log`, the per-minute lines of both seats.
**Used by.** H-TEAM-DEFEND.

### K-army-one-post-for-a-wide-front
**Claim.** A home group that stands on one post cannot answer raids on a wide front: from Quicksilver's north-west start
23 of 24 soldiers stood at one extractor while raiders arrived across 2000 elmos, and 12 of 19 constructor deaths had
nothing of ours within 600.
**Status.** observed (2026-09-20, the early-game post-mortem of commander games 8-11); the remedy (H-ARMY-DETACH) is
built and fires, its own effect is not isolated from the territory grid's.
**Used by.** H-ARMY-DETACH.

### K-army-enemy-army-seen-is-the-worst-estimate-we-have
**Claim.** `enemy_army_seen` — the biggest enemy force in one look within the last two minutes, which the wave gate
uses as its estimate of what our wave will meet — is the worst of the four estimates available to us. Over 430
Quicksilver games its mean absolute error against the opponent's real army metal is 1741; the clock alone scores 1247,
simply summing every enemy soldier ever identified minus the ones we watched die scores 1190, and a fitted model
scores 637. It understates: 2287 against a real 4853 in minutes 20-40. The fault is the two-minute window, not the
measurement — the same sum without the window is the single most informative observation we have (39 % of the
time-only error removed, more than any other feature).
**Status.** measured (2026-09-20), offline; the replacement is unmeasured in the arena.
**Evidence.** `docs/studies/tempo-model.md`, tables "What the bot actually has in hand" and "Held-out error".
`ENEMY_ARMY_MEMORY_FRAMES = 120 s` (`brain/army.rs:62`); the gate at `army.rs:465-478`. This is the mechanism behind
K-army-verdicts-v18's "1500 known against a real 3205".
**Would be wrong if.** With the gate reading an unwindowed sum (or the tempo estimate), waves still launched into
armies they had not counted at the same rate.
**Used by.** H-ARMY-WAVE-GATE (as a criticism of its input, not yet a change).

### K-army-blob-attack-move-wastes-the-raiders
**Claim.** Sending a wave to one point makes it fight as a blob, and a blob is the worst formation our tier-1
raiders can be in. Spreading the same army over a block 110 elmos between neighbours is worth about a fifth of the
metal it trades, and essentially all of that belongs to Pawns and Grunts: over 28 tier-1 pairings in the engine,
metal killed per metal lost went 1.09 to 1.30, with a mean margin gain of +0.47 for Grunt and +0.47 for Pawn
against ±0.05 or less for every other unit. Two mechanisms, not one: the blob feeds two or three units to every
area shell (Mace and Thud 36 elmos, Rocketeer and Aggravator 48), and — the larger effect — a blob of 180-range
raiders cannot get more than its front rank inside its own range of a target, so it walks into a tower line a few
at a time.
**Status.** supported (2026-09-20) in the engine duel harness; **it does not reach the arena** — the
`micro2-spread` A/B is flat (K-army-a-real-wave-is-not-a-blob says why).
**Evidence.** `docs/studies/micro-combat.md`; batches `micro2-off` / `micro2-on` (28 pairings x 6 duels an arm,
equal metal at 1200, spawn spacing 56), compared with `run/duel_ab.py`. Biggest cells: Pawn against Guard −0.182
to +0.570, Grunt against Sentry −0.070 to +0.613. The behaviour was confirmed before the result: `spread_x` in
`duels.csv` (RMS distance of an army's units from its own centre at the first shot) rose from a mean 81 to 131
elmos while the opponent's stayed at 94. Consistent with K-units-duel-spacing-decides-area-damage, measured from
the other end (both sides loose) in the 2026-09-19 tables.
**Retracted from the first pass.** An earlier run of this A/B, on an implementation since corrected, put
Rocketeers against Grunts at −0.254 and it was reported as the policy's one clear cost. Re-run it is −0.056 and
the Rocketeer's row mean is +0.02: six duels a cell cannot separate a fifth of a margin from noise. The engine's
own spacing tables still say spreading hurts that pairing (−0.03 tight against −0.23 wide) and have more duels
behind them, so the question is open, not settled either way. **The raw data of the first pass was lost with the
worktree it lived in and cannot be re-examined.**
**Would be wrong if.** An arena arm with H-MICRO-SPREAD off traded metal at least as well as one with it on, or if
the gain in the duels came from the fight taking longer rather than from the formation (contact time rises with
the spread order and was not separated).
**Used by.** (none; H-MICRO-SPREAD, retired 2026-09-20).

### K-army-withdrawing-a-hurt-soldier-saves-metal-and-loses-the-fight
**Claim.** Walking a unit out of a fight when its health falls below a third is the most metal-efficient policy
tested and still loses ground: over 135 simulated tier-1 cells it took metal killed per metal lost from 1.41 to 2.41
and cost 0.039 of margin. Fewer of ours die; the ones that leave stop shooting, so fewer of theirs die too, and what
walks away walks away hurt. The two measures disagree by exactly the question of whether a hurt soldier is ever
repaired, and ours are not. Only when outnumbered (0.7x metal) is it not negative: +0.005 of margin, 0.66 to 1.02.
**Status.** conjectured (2026-09-20) — simulator only, never run in the engine. It also rests on an unchecked
assumption: that a unit under a `Move` order keeps firing at what comes into range, which is what the engine's
default fire state should do but was not verified.
**Evidence.** `combatsim micro --reps 16`, `docs/studies/micro-combat.md`.
**Would be wrong if.** A duel arm with `withdraw` orders traded better *and* won as often, or if soldiers were
repaired at home, which would move the true measure from margin towards metal.
**Used by.** (candidate: H-MICRO-WITHDRAW, after H-ECO-REPAIR is extended to soldiers)

### K-army-focus-fire-cannot-be-ordered-and-would-not-pay
**Claim.** There is no attack-unit command in the protocol: `Fight` names a point and the engine picks the target,
roughly the nearest. Simulating the command we do not have says not to add it — a side that all shoots the enemy with
the fewest hit points left scores 0.165 of margin *worse* than the engine's own targeting over 135 cells, because
nothing stops a salvo already in the air and the overkill is most of a Rocketeer's 3.8-second volley.
**Status.** conjectured (2026-09-20) — simulator only. The absence of the command was `supported` until 2026-09-20
evening: `Command::Attack` exists now (added for deliberate turret kills, 8ffc510), so the claim is only the second
half, that naive lowest-health focus would not pay; a focus rule that counts damage in the air is H-MICRO-FOCUS's
business (`docs/design/2026-09-20-micro-lane.md`).
**Evidence.** `combatsim micro --reps 16`; `finishing_the_weakest_target_wastes_shots_on_the_dead` in
`crates/combatsim/tests/mechanics.rs`.
**Would be wrong if.** A focus rule that avoided overkill (counting damage already in the air towards a target)
priced out positive; the policy tested is the naive one.
**Used by.** (nothing — it is a reason not to extend the protocol)
**Amended 2026-09-25.** Measured directly in the duel harness (the lane driving one side, 8 duels an arm on the
same sites): H-MICRO-FOCUS cost the Blitz pack against Pawns 0.21 of margin (+0.098 with it, +0.308 without, the
plain order +0.154) and nothing in a 12-against-12 Stout mirror (`docs/studies/2026-09-25-formation-micro.md`); the
rule is retired. The "cannot be ordered" half is stale (the protocol has `Command::Attack`); the "would not pay"
half is supported.

### K-army-a-real-wave-is-not-a-blob
**Claim.** Our waves already fight spread out, so a formation policy has nothing to fix. Measured over the 930
engagements of the `micro2-spread` batch with at least 300 metal of ours present, our soldiers stood at a mean 358
elmos from their own centre with H-MICRO-SPREAD firing 155 times a game and 356 without it — a rule that shapes
every attack order moves the number by two elmos. In the enemy's half, where the army is most concentrated, it
reaches 303 against 270. The duel harness, which spawns an army in ranks 56 apart and sends it at one point,
fights at a mean 81 of the same measure, and the spread orders that beat it by a fifth of the metal traded only
reach 131. **The blob the duel tables price is an artefact of the harness.** By the time a wave is in contact it
has walked a thousand elmos, been marched, regrouped, detached and lost its fastest, and it is three times more
scattered than the formation the policy exists to break up.
**Status.** supported (2026-09-20). Both numbers are the same statistic (RMS distance from the group's own centre):
`present_before.our_fighters_spread` in `run/analyze_match.py --json`, and `spread_x` in the duel harness's
`duels.csv`.
**Evidence.** Batch `micro2-spread` (48 games) against batches `micro2-off` / `micro2-on` (336 duels).
`docs/studies/micro-combat.md`. An earlier, weaker version of this measurement (315 against 325) was taken on a
batch whose raw data has since been lost; this one replaces it and says the same thing with the rule firing
seventeen times as often.
**Would be wrong if.** The arena measure were inflated by the way engagements are cut out (fighters within 1100 of
the centre are counted, so the statistic is bounded well above what was seen — but 327 is far from that bound), or
if waves that arrive together after H-ARMY-MARCH and H-ARMY-STAGE improve showed a lower number.
**Used by.** (none; it explained why retired H-MICRO-SPREAD's engine gain did not reach the arena); a caution for any future
formation or spacing rule priced on the duel tables.

### K-army-distance-decides-a-raid-response
**Claim.** Whether an answer to a raid on an outpost kills anything is decided by where the answer starts from, not by
how big it is: a raider is killed in 79 % of raids when one of ours was within 600 of the extractor ten seconds before
it died, 52 % when the answer came from 600-1500 away, 28 % from beyond 1500 and 23 % when nobody came (the outpost's
turret did it). Parties are not small: 200-1000 metal within 900 of the extractor on average. The chase simulator
reproduces those rates from the units and distances alone (89 / 47 / 23 / 19 %) and agrees with the played outcome in
71 % of single episodes; it under-predicts what our answer loses (304 metal against 506 close by, 89 against 189 from
mid-range) — it knows only the party within 900, not what follows it.
**Status.** measured (2026-09-20), 1106 raids in 48 games (now-quicksilver, now-isidis, now-mithril), medium BARb.
Extractor losses in these episodes are not a fair test of the model: an episode is found by an extractor dying.
**Update 2026-09-20.** With pursuers walking at where their side last saw the party (not at where it is), the same episodes predict 83 / 39 / 22 / 19 %: nearer the played 79 % from within 600, farther from the played 52 % from 600-1500, where play has the team's whole sight and radar to re-order by and the scenario has only the units in it.
**Evidence.** `run/raid_episodes.py <batches>`, `combatsim chase-file`, `run/raid_episodes.py --compare`.
**Would be wrong if.** Episodes found another way (every party that came within 900 of an extractor, died or not) gave
different rates by distance.
**Used by.** H-ARMY-CONTACT (`docs/design/2026-09-20-army-response.md`): answer from near or not
at all, guard where parties go, and price the answer's own losses higher than the model says.

### K-micro-a-tick-is-a-tower-margin
**Claim.** At the 2 Hz tick a Pawn (87 elmos a second) walks 43 elmos between decisions, most of the 90-elmo margin
between "outside a light laser tower's reach" and "dying in it" (reach 430, 160 damage a second against 370 health;
the commander 300 and about 390 a second); at 10 Hz it walks 9. So no rule at 2 Hz can keep a Pawn out of a defence it
did not mean to enter, and the four raid faults found on 2026-09-20 (the dive at the commander after sighting it, the
fight under a tower with no flee, the leader waiting for joiners under fire, the party walked through the base on a
Move) are all decisions taken too late at the unit level.
**Status.** supported (2026-09-20 evening) on the mechanism measures: micro-ab2 (12 games an arm, the whole lane)
by minute 20 cut deaths to turrets and the commander from 10.9 to 4.9 a game and the exchange from 1.27 to 0.78, and
won 3 to 0 (wins alone are within noise at this size). First A/B (micro-flee-ab, 12 games an arm): the flee alone cut deaths to turrets and the commander from 4.2 to 3.2 a game by minute 10 and soldier-seconds
under fire from 328 to 216, but also halved what the army killed (648 against 1174) because wounded units left unit
fights their side was winning; wins 1 against 3. The claim stands for static defences; for unit fights it is
K-army-withdrawing-a-hurt-soldier-saves-metal-and-loses-the-fight that holds, and the rule now leaves only losing fights.
**Evidence.** `crates/combatsim/data/units.json` (armpw, armllt, armcom); rush-26 records: 17.8 Pawns lost to 3.0
buildings killed by minute 10; micro-flee-debug (one game, the first lane): every Pawn death traced to a decision the
lane made or failed to make (`docs/design/2026-09-20-micro-lane.md`, status).
**Would be wrong if.** The lane's A/B (`--ab-disable H-MICRO-LANE`, 24 games) showed no fall in soldiers lost to
turrets and the commander, or in soldier-seconds under fire (`run/micro_ledger.py`).
**Amended 2026-09-25 (lane-ab, 24 games an arm on Comet Catcher against medium, the heuristic bot).** By this
claim's own test the lane no longer shows: deaths to turrets or the commander 6.2 a game with the final lane, 5.4
with the old, 5.2 without; soldier-seconds under fire 1,741 / 1,731 / 1,655. What the lane shows instead is the
trade: metal lost per metal killed 0.79 [0.65-0.95] against 0.94 [0.83-1.05] without, deaths elsewhere 36 against
45, muzzled 6.4% against 11.5%, spacing at contact 51 against 40. The tower margin is what the flee was built on;
on this map against medium the fights are unit fights (the raid's turret dives are Quicksilver's), and the flee's
tower cases are few (0.4 deaths in reach a game by minute 10). The claim stands for what it measured (the 10 Hz
margin at a tower); its "would be wrong if" no longer separates the lane from no lane here.
**Used by.** H-MICRO-LANE, H-MICRO-FLEE.

### K-micro-the-engines-slower-drop-ended-every-hunt
**Claim.** A rule in the engine that second-guesses the hands' pick ends the pick without anyone deciding it: the
hunt's "slower" drop (a hunter whose type is no faster than the quarry's leaves the hunt at its first tick) turned every
hunt the hands offered "without catching it" into "ended after 0 s: no hunters", and with a hunt a group of its own,
each such pick split one or two units off that then held where they stood.
**Evidence.** player-24-hunts (`run/matches/1790657006-player-24-hunts/00/jev-0.jsonl`): 47 hunt ends, 46 "after 0 s:
no hunters" (party_10 21, party_7 9, party_9 9, party_15 3, party_16 2, party_5 2), the one kill a Rover; the hunters a
Blitz (101) after a Blitz (101) or a Rover (168), or a Rover after a Rover; twelve splits (2:50, 5:31, 6:38, 6:48-6:50,
7:03, 7:10-7:13) and ten groups of one at 7:38, 990 metal within 1,100 of home, when eight Blitzes and a Rover (911)
drove into the base and killed 2,371 metal. The hands' words had said "they drive it off in 3 s from 264 away without
catching it (101 against its slowest 168)" and, from the second pick on, "its last hunt of it ended 1 s ago: no hunters".
**Status.** fixed 2026-09-29: the engine's "slower" and "hurt" drops deleted (H-MICRO-HUNT amended); whether the
hunters can catch or survive the quarry is the hands' call in the state's words. Recheck in the next player game: no
hunt ends "no hunters"; a hunt by a Blitz after a Blitz runs until dead, lost or the leash.

### K-micro-a-hunt-by-id-catches-a-raider
**Claim.** A raider is caught by hunters faster than it when the order is an attack on its unit id re-issued every
tick with no footwork over it (H-MICRO-HUNT), and not by a fight-to-point at its last place re-issued every two seconds
(the hands' engage before 2026-09-26): the point is where the raider was, the attack is where it is.
**Status.** measured (2026-09-26) in the duel harness's raid scenario (`duel --scenario raid`, `docs/harness/duels.md`):
four Rovers and two Blitzes at a station in the middle of a strip of four extractors 600 apart, a Tick and a Pawn on a
scripted route that runs from anything within 350; the picket answers by the standing raider rule (two pickets faster
than the raider, at a raider within 250 of a building of ours and 1,200 of the station), the same ends judged either
way (dead, out of sight 6 s, a 900 leash from where the hunt began, a third of health).
**Evidence.** raid-on-8 and raid-on-8b (16 raids, the hunt): the first raider killed at a median 11.0 s from the first
orders (11 s in ten of sixteen; 17-33 s in the rest, when the Rovers' first hunt on the Tick ended by the leash or both
Rovers dropped hurt under the Pawn's gun), both raiders dead in 15 of 16 by 22-72 s, extractors lost 0.19 a raid (0 in
14), hunters lost 1.3 (Rovers to the Pawn). raid-off-8 and raid-off-8b (16 raids, fight-to-point every 2 s): the first
kill at a median 76 s (23.7-150.8), both raiders dead in 6 of 16, ten raids to the 180 s cap with a raider alive,
extractors lost 1.25 a raid, hunters lost 1.3. The hunts' ends show the mechanism: with the hunt a run ends "dead" in
1 s after the hunters reach the quarry; without it the runs end "leash" 6-15 s after they begin, eight to sixteen
times a raid, the hunters running to where the raider was. The 900 leash from the hunters' start ends the first hunt
on the far extractor (1,060 from the station) either way; the second hunt, from where they then stand, catches it.
**Would be wrong if.** The fight-to-point arm reached the first kill as soon on the same raids, or the hunt arm lost
more extractors.
**Used by.** H-MICRO-HUNT; `docs/design/2026-09-26-threat-response.md` (ladder step 1).

### K-micro-a-rover-outruns-what-it-cannot-fight
**Claim.** A fast unit (a Rover at 168 a second, a Tick at 132) run by code ten times a second, kept at every shooter's
reach plus 1.5 s of that shooter's speed (at least 80; 60 beyond a building's) and sent only along ways that pass no
such margin, gets to see his start box and base and kills the unguarded economy it finds there without dying to
anything it had in sight: seen is harmless, only reach kills, and a Rover gains 81 a second on a Pawn. The hands'
scouts, walked on Moves under the one-second pick, do not: in player-9-posing (lost 36:32, his base at G1-G2 never
found) three scouts went at it and none arrived: a Rover at 3:27 on an eight-spot route was turned home at 3:58 by the
pick ("falls back to our base", 0.58) the second it sighted his commander, four stations short; a second Rover on a
north route at 4:22 drove into two Pawns beside his lab (party_6, two armpw at (6138, 1016) at 5:36) and died at 5:38,
500 from the lab; the hands' own Blitz scout at 7:05 was put on a Pawn hunt at 7:18 and died at 7:31, 1,300 short.
**Status.** conjectured (2026-09-27). Built as H-MICRO-ROVE; unit tests only: in a straight-line kinematic model a
Rover chased for a minute by a Pawn over a map of 25 spots came no nearer than 302 (the Pawn's reach 180) and
looked at 19 of them, all ten of his among them.
**Evidence.** `run/matches/1790513490-player-9-posing/00` (`run/hands_window.py ... 3:50 4:02 group_B`: the pick at
3:58, `retreat=0.58`; 5:30-5:40: party_6 at group_B); `crates/micro/src/rove.rs` tests.
**Would be wrong if.** In games with a roving group, rovers die to shooters that were in their sight or remembered
(the lane's sources) more often than once in ten minutes of roving, or his base and start-box spots are still unseen
at 6:00 with a rover out from 3:00; or the engine's turning and pathing carry a rover into reach through the half
second of lookahead (deaths with the evasion firing in the seconds before).
**Used by.** H-MICRO-ROVE.

### K-micro-a-unit-under-the-guns-shoots
**Claim.** A soldier already inside the enemy's reach should stay and shoot even when their fire on it would kill it
and its side is locally outgunned: a unit that walks does not fire, and their fire on a ball at contact is "lethal"
for every unit in it at once. The lethal flee is for the unit on its way into a losing fight, judged by strength
(damage a second times health), not by damage a second.
**Status.** measured (2026-09-25) in the duel harness with the lane driving one side (`duel --lane old`, one site,
`WITHIN_REASON_MICRO_DEBUG` naming what held each muzzled unit). 11 Blitzes against 22 Pawns: the lethal flee as
written on 2026-09-20 (damage a second odds, no "inside" test) fled seven to ten Blitzes at contact in a fight the
plain order wins (+0.26, 0% muzzled): 24-33% of the Blitzes' in-reach seconds muzzled, 21 of 36 muzzled seconds
under the flee claim (form-smoke-old, -old3). With strength odds alone it still fled hurt units before contact
(form-smoke-old3); with the inside test as well, 0% muzzled (form-smoke-old4), and over 8 duels the amended branch
costs nothing measurable beside the formation (form4-blitz-on-nofocus +0.308 +- 0.023 with it, form3-blitz-on
+0.282 +- 0.018 without).
**Evidence.** `docs/studies/2026-09-25-formation-micro.md`; the debug lines `muzzled: ... held by H-MICRO-FLEE`.
**Would be wrong if.** An arena A/B with the inside test showed more soldiers lost to unit fights the side was losing
than without it over 24 games.
**Used by.** H-MICRO-FLEE.

### K-army-pickets-across-the-front
**Claim.** Tick and scout-car raids on the outer extractors are standard and expected (BARb from about 5:00 on Comet
Catcher, outermost spots first), so the answer is decided before they come: single soldiers or pairs standing across
the front at each outer spot cluster, told to attack on sight, answer a raider in seconds; a guard at home sent after
the extractor is attacked arrives after it is gone. One responding unit is enough for a single Tick; more is wasted
and pulls the group about.
**Status.** stated by the user 2026-09-22 late, watching escalate-4-hard-aggressive; not measured over games.
**Evidence.** `run/matches/1790125719-escalate-4-hard-aggressive`: 33 extractors lost, most to single Ticks at spots no
soldier stood near while the Blitzes held as a guard group at home or at spot_43 (the player's notes 5:19, 10:00); the
experienced players' game lost one extractor in seven minutes with a light turret beside each pair.
**Would be wrong if.** Pickets of one across the front lost as many extractors to Ticks as the home guard did, or died
in numbers to the parties that follow the Ticks.
**Used by.** H-HANDS-DETACH (`how_many` 1), the player's guide and brief (pickets from the first Blitzes).

### K-army-air-plant-seen-means-aa
**Claim.** An enemy Aircraft Plant in sight is the whole warning: its first gunship is about ninety seconds behind
it, tier-1 tanks and bots do a fifth of their damage to it and only when the ground is clear, and an 80-metal Nettle
beside each pair of extractors or Whistlers in the plant before it arrives is the answer; after it arrives the
extractors are already gone.
**Status.** stated by the user 2026-09-23 ("WReason should have noticed the air lab I was building, and could have
countered the build directly"); one game.
**Evidence.** `run/matches/1790132629-human-9`: the plant seen at 2:16, reported as a grid without its type, the first Banshee at 3:54, Nettles
from 4:42, eight Blitzes to six Banshees at 8:22.
**Would be wrong if.** Nettles at the extractors from the sighting still lost them to gunships, or the plant were
seen and no aircraft came in a run of games.
**Used by.** H-HANDS-ENEMY-EVIDENCE (the factory type on the report line); the player's brief.

## Comet Catcher Remake 1.8: the replay survey (2026-09-23)

Claims from the game cards of 29 public 1v1 duels (both players OS 25 and above, games of 2026-09-20 to 2026-09-23 on BAR test-31357 to 31383), filed by the synthesis agent against `docs/knowledge/questions.md` and checked with `run/replays/check.py`; the map file `docs/knowledge/maps/comet_catcher_remake_1.8.md` and the index `docs/knowledge/replays.md` point here. Spot numbers are the cards' (the game's own 80-spot list), not the bot's (the engine's 75); they map to the bot's by position (nearest within 130), and every spot carries its grid cell (8x8 over 8192x6144). Unit names: armfav Rover, corfav Rascal, armflash Blitz, corgator Incisor, corak Grunt, armpw Pawn, armflea Tick, armstump Stout, corraid Brute, armllt/corllt the light laser turret. "W"/"L" are the winners' and the losers' sides. Remaining card limits: `fights` is the eight costliest cell-minutes per game; the cards carry no unit positions over time (held back or chasing cannot be read); the killer of the first extractor is "?" on 10 sides; one manifest start (bcaeb16a, Chronopolize) disagrees with the card's B5 and the card is used. Fixed after the survey (cards re-cut 2026-09-23 evening): the faction field, the spots' cells, the first-enemy-seen field (removed: a replay sees everything), the factories' order, the last 15 s of kills counted apart, cells on every building, the first tier-2 unit.

### K-army-comet-raiders-to-12
**Claim.** Tier-1 raiders are the army through 12:00. Armada sides (average per side): 3.1 Rovers and 0.7 Blitz at 2:00
(25 sides), 5.2 Rovers and 3.6 Blitz at 4:00, 8.1 Blitz and 3.6 Rovers at 6:00, 13.1 Blitz at 8:00 (21), 17.3 at 10:00
(18), 24.1 at 12:00 (15). Cortex: 2.4 Rascals at 2:00 (33), 4.4 Grunts, 4.2 Rascals, 2.5 Incisors at 4:00, 7.2 Incisors
and 6.1 Grunts at 6:00, 13.2 Incisors at 8:00 (29), 19.8 at 10:00 (26), 25.6 at 12:00 (21). Line units (Stout, Brute,
Thud, Pounder, Janus, artillery) were in 5 of 25 winners' armies at 8:00, 11 of 22 at 10:00, 10 of 18 at 12:00, at a
median 3% of the count at 12:00; Brutes first appear at 10:00 (median, 17 Cortex sides) and Stouts at 12:00 (7 Armada
sides). Scout cars fall from the lead unit at 2:00-4:00 to under three per side from 8:00 (Rascals under one).
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29 (`army_by_type` every two minutes).
**Would be wrong if.** Games on a later version with line units by 8:00 in most sides. **Refines**
K-open-comet-flash-raids-take-the-resign: the Blitz (and the Incisor for Cortex) is the army all game, not only the
opening. What beats what cannot be read from the cards: kills and losses are whole-game totals by type; a per-fight
composition (the types dying on each side in each fight cell) would answer it.
**Used by.** (candidate: the unit-mix lever in the brief)

### K-team-the-script-parser-took-clan-tags-for-sections
**Claim.** The shim's start-script reader took any `[word]` for a section header, so `name=[gecko]thebluegecko;`
and `skill=[6.02];` inside values started phantom sections and every section after them lost its name: in all
four games with people (2026-09-26/27) the person's seat read "team 0 (who plays it is not in the script)", our
first seat (`[ai0]`, listed after his `[player0]`) was not counted as ours, so `seats_of_ours()` said two of three,
the first turn fired at frame 1 with two seats, the sides line never named the third, and the player wrote a
two-seat opening three games running ("I found a third seat" at 0:21 in game 4).
**Evidence.** The demo's start script (`WR_SCRIPT=<file> cargo test -p ai-shim -- --ignored probe --nocapture`
printed a `[gecko]` section of 4,583 bytes and `controllers` without teams 0 and 1); game 4's first two turns.
**Status.** Fixed 2026-09-27 (a section's brace must follow its name with only whitespace between; regression
test with the clan tag). A lobby seat on side Random keeps the word `Random` in the engine's side, so the bot now
reads its own faction from the commander's type. Exploited by [[H-PLAYER-SEAT-NAMES]], [[H-PLAYER-SIDES]].

### K-team-one-store-for-all-seats-loses-a-seat
**Claim.** Every store in `Shared` that one seat overwrote instead of keying by team lost the other seats to the
player's tools: the standing list (drained by one seat, 8cc9ac6), the hands (aca9d65), the field's roster
(6872ff4) and the unit cards the `remove` tool searches (bluegecko-3v1-comet-catcher-3, 18:55: the extractor
`cormex_344` blocking a Cortex lab's exit "nothing of ours has that name" twice, the lab idle with a Thug in its
lane). The pattern: `*shared.x.lock() = mine` in a per-seat publisher. Fixed by keying on `hello.team` and reading
the union.
**Evidence.** `run/matches/1790471259-bluegecko-3v1-comet-catcher-3/00/strategist-0.jsonl` (turn at 18:55);
`grep -n "lock().unwrap() = " crates/bot/src/brain` for the pattern.
**Status.** Cards fixed 2026-09-27 (`own_cards: BTreeMap<i32, Vec<UnitCard>>`). Exploited by
[[H-PLAYER-SEAT-NAMES]].

### K-team-random-side-gives-the-seats-two-factions
**Claim.** A lobby seat on side Random draws its faction at start, so seats of ours can be of two factions in one
game, and the tools checked names against the lead seat's roster alone: in bluegecko-3v1-comet-catcher-3
(2026-09-27) t1 was Armada and t2 and t3 Cortex, the `queue` tool refused `corsolar` as "not a step" and `produce`
refused `corfav` as "not a unit of our roster", both Cortex seats opened on the hands' defaults with no list, t2 had
no plant at 2:00, and the player spent two turns guessing which faction's names the tools wanted. The report's
roster is the merged field's, which took the lead seat's roster only. The earlier games with people had every seat
on side Random too and drew the same faction by chance.
**Evidence.** `run/matches/1790471259-bluegecko-3v1-comet-catcher-3/00/strategist-0.jsonl` (turns 1-3), the demo's
start script (`side=Random` on every team of ours).
**Status.** Fixed in code the same night (the merged field's roster is the union of the seats' rosters, the report
names each seat's faction when they differ, a seat skips a step it cannot build); for the games with people the
lobby sets each seat's faction explicitly (docs/harness/lobby.md). Exploited by [[H-PLAYER-SEAT-NAMES]].

### K-team-one-name-for-two-seats-loses-a-seat
**Claim.** With two seats of ours under one player and one name space, the player cannot steer the second seat: both
commanders were "commander", lists keyed by name were drained by whichever seat ticked first, and the report's hands
section showed the lead seat's actors only. In bluegecko-2v1-great-divide (2026-09-27, a person in the north box
against two seats of ours in the south, realtime) the first turn fired at frame 1 with one seat published, the player
took itself for the south-east Cortex seat alone and wrote a Cortex opening, the Armada seat drained that list and
could build none of it, the Cortex seat got nothing, and neither seat played one list step in the first three minutes;
both ran the hands' default opening (a bot lab each at 0:39 and 0:40) on a map whose brief section asked for a
vehicle plant.
The same drain took the `standing` changes: the 4:16 `never spot_0..11` for constructors landed on the west seat only (the east seat's constructors kept spot_8, spot_9 and spot_11 on offer and built at spot_9 at 5:34 and spot_8 at 6:35), and the 9:31 `set station spot_16` and the 10:47 `clear` for group_A went to the other seat's group_A, so the west group's `station spot_15 (tool)` stood until `clear all` at 12:06 and the group flipped thirty times between its station under the turrets and home (-2,910 for 540).
And the packet's paragraphs never reached a tagged group or commander: the reader took `group_A_t2:` as `group_A` and knew only a plain `commander` (bluegecko-3v1-comet-catcher, the review: group_B_t3's slot carried no station from six packets that gave it one, the 50-Blitz ball holding at spot_48 from 5:30 to 13:33 while the block killed the north seat; fixed 98cac09+, `standing.rs` `group_names`, `builder_names`, `rules_for`).
**Status.** observed (2026-09-27), two games.
**Evidence.** `run/matches/1790465811-bluegecko-2v1-great-divide/00`, `run/matches/1790467496-bluegecko-2v1-comet-catcher/00` (`00/review.md`) (`strategist-0.jsonl` turns 1 and 2, `jev-0.jsonl`
and `jev-1.jsonl` with no `list` plays, `bot.log`).
**Would be wrong if.** The lists had reached the Cortex seat and its hands had refused them for another reason.
**Used by.** H-PLAYER-SEAT-NAMES.

### K-army-riots-answer-a-light-tank-blob-and-mediums-answer-riots
**Claim.** A ball of Blitzes or Incisors is beaten by Pounders (Cortex riot tanks, slow, made in the plant) from
about 7:30 ("this is about the right time to switch to pounders, which will be hard to fight with light tanks like
blitz"; Irishstud14 after game 4: "pounders solve the blobbing it does"), and the answer is medium tanks with a few
Lashers, never many ("once you have maybe 3 or 5 lashers pure med tank is probably better; too many lashers lose to
med tanks"; "pounders for defence and med tanks for offence, maybe 3 med tanks per pounder, or all pounder if I am
getting pushed"), or air over their solars ("air switch to bomb solar is strong in the tank phase if you have the
e"). Light tanks keep raiding where the Pounders are not ("pounders are made in the main lab and slow, so if you go
other places it is safer for those fast light tanks"; "spread out for the raid, attack both corners"; "focus on
mexes and solars"; "try top corner, not much defence there"). Game 4: our push of Blitzes and Incisors stopped on
Pounders at 8:41 and our Lasher-heavy groups lost to his Brutes.
**Evidence.** `docs/knowledge/_inbox/players-chat-2026-09-27.md`; game 4's turns 8:41-11:10.
**Status.** supported (the players' words and game 4), unmeasured as a rule. Exploited by
[[H-PLAYER-PLAYERS-ADVICE]].

### K-army-an-army-lead-unspent-goes-bad
**Claim.** An army lead over a person with the better income has to be spent at once: on his extractors and
solars, as one line with few groups, the constructors reclaiming behind it, or it goes bad as his income tells
("you are up by almost 400% army value, time to go fight and win; you are down in metal income; if you dont use
that army advantage this will soon go bad": thebluegecko, game 5, 6:50, and the push won the game by 9:42; game 3:
"red not mobilizing fast enough is what lost you the game", "your armies keep not fighting together", "make sure
your units are more of a straight line when attacking, less groups", "eat as you push", "get that reclaim asap";
game 4: "Irish expanded faster, even fight on first contact, then Irish's raids did better damage"). The
commander is sniped by a circle of light tanks with the target set on it ("set target the commander and drive into
a circle around it to snipe it"); an early lead in raids answers naked extractors ("an aggressive response is
needed to prevent irish gaining a large metal lead").
**Evidence.** `docs/knowledge/_inbox/players-chat-2026-09-27.md`; games 3, 4 and 5.
**Status.** supported (2026-09-27; game 5 is the one game where it was done). Exploited by
[[H-PLAYER-PLAYERS-ADVICE]].

### K-army-tier-1-trickled-into-bulls-and-turrets-is-the-loss
**Claim.** Against thebluegecko in bluegecko-3v1-comet-catcher-9 (lost about 20:16 after leading three to one at
13:07) the loss was dozens of trades by the wrong unit in the wrong place (the user): our army losses were 61.5k
against 54.7k of his seen, and the killers were his Bulls (94 of ours: 32 Pounders, 26 Stouts, 23 Janus, 13
Blitzes, for 15 Bulls), shooters out of sight (73: Bulls and Mausers beyond our sight, beamers), beamers (24
Stouts and Blitzes) and his Janus and Stouts (27 Blitzes). Pounders (reach 315) and Stouts (350) walked at Bulls
(460) and beamers (490) in ones and twos and died on the approach; the odds called it "we outweigh it" because the
matchup table has no rows for tier 2 or turrets and the square law counts no reach (a Bull's damage times health
is worth about 670 tier-1 metal against its 950). A turret that outranges a group is no bar to pushing in and
killing it (the user): the cost is paid on the approach, once, by a group that goes in as one body.
**Evidence.** `run/matches/1790479031-bluegecko-3v1-comet-catcher-8/00` records (the `destroyed` events by killer
type); the worth probe `cargo test -p bot -- --ignored worth_probe --nocapture`.
**Status.** supported (2026-09-27, one game with the numbers). Exploited by [[H-HANDS-ODDS-REACH]],
[[H-PLAYER-PLAYERS-ADVICE]].


### K-team-one-party-had-a-name-per-seat
**Claim.** Until 2026-09-27 evening each seat named the enemy parties it saw with its own counter and a seat tag
(`party_46_t2`), so one party seen by two seats had two names, the `standing` tool's `engage_party` check read the
lead seat's sightings only and refused the others' names ("not a party in the picture": games 3, 4, 6, 9), and each
seat's Jev priced its own group alone against a party the next seat's group stood beside (three armies that could
never be "together", games 4, 6, 9).
**Evidence.** `docs/studies/2026-09-27-jev-posing/README.md` (I2; game 3's refused engage orders traced to
`mcp.rs`'s standing branch); the games' `jev-N.jsonl` (the same units under two names).
**Status.** fixed 2026-09-27 evening by [[H-HANDS-SIDE-PARTIES]] and [[H-HANDS-ALLIED-GROUPS]] (the check: a party
in two seats' pictures under one name in the next game with several seats).

### K-team-the-trade-counted-each-death-per-seat
**Claim.** Every seat records every enemy death it sees, and the report's trade line summed the seats' figures, so
in a three-seat game a death all three saw counted three times: every ledger row's "destroyed" figure of
2026-09-27 is inflated on that side (the 3v1 trades of 110k:77k, 85k:30k and the like are not what they say).
**Evidence.** `docs/studies/2026-09-27-jev-posing/README.md` (I3, verified in `seats.rs`'s merge); the records'
`enemy_destroyed` events carry the unit id, so the true figure can be re-derived.
**Status.** fixed 2026-09-27 evening by [[H-PLAYER-TRADE-ONCE]]; the ledger rows' correction is a separate
cleanup (changes doc 3.1).

### K-team-the-seats-thrashed-over-spots
**Claim.** In the games with three seats of ours, constructors abandoned 114 extractor orders, 57 of them for spots
another seat took first, because no seat knew the others' claims: the team board posted `spot_claims` from a map
nothing filled and `free_spots` counted the seat's own orders only; going home was small (2 % of constructor time)
and the seats had too few constructors, stopping at the midline.
**Evidence.** `docs/studies/2026-09-27-jev-posing/expansion.md` (verified in code: `Brain.spot_claims` never filled,
`team_mates` never read).
**Status.** fixed 2026-09-27 evening: the claims are filled from the build tasks and read by `free_spots` and the
picture (H-TEAM-BOARD amended); the constructor count and the midline are the brief's (changes doc 13.4, 13.5).

### K-micro-a-rover-tours-the-map-after-one-look
**Claim.** Ranking his box and base first only while unseen or older than three minutes, the rover looked at his base
once and then toured every never-seen spot on the map; what a scout is for, once his base is known, is its perimeter:
what leaves it, what is built at its edge, the constructors and extractors around it.
**Status.** observed (the user, 2026-09-28, watching player-10-routes: "the rovers tend to explore the whole map rather
than bother the perimeter of the known enemy base"); the three rovers reached within 511-636 of his start at 3:55 and
were back in our half by 4:55 (`run/minutes.py`).
**Evidence.** `crates/micro/src/rove.rs` (the tiers before 2026-09-28); player-10-routes 3:00-5:00.
**Would be wrong if.** A rover on the perimeter died to the base's turrets faster than it found things, or the map's
other spots carried what mattered (his expansion away from his base).
**Used by.** [[H-MICRO-ROVE]] (amended 2026-09-28: a minute's staleness for his places; ranking his recently seen places ahead of the map's unseen ones was tried the same evening and reverted: with the base's own spots behind turret reach it left two edge spots, and the rovers walked between them in the first human game).

### K-army-what-we-make-tells-him-what-is-safe
**Claim.** The unit mix is adversarial, not a static answer to his: a mass of riot units (Lashers, Janus) puts no
pressure on his spots, so he reads it as safe to go tier 2, and his Tzars and Tigers then beat the riot mass; a mass
that threatens his spots (Brutes, raiders) denies him the tech. thebluegecko, game 10 (2026-09-28): "this isn't a
static problem, it is adversarial, that is, you made many lashers, so I was safe to go T2"; "brutes would have given
you the ability to apply more pressures"; "i think you made too many lashers". The same day the user, on the duel:
tier 2 is rare because the attempt opens a fatal weak point, which the opponent's army mix decides whether he can
afford. Pairs with K-map-comet-catcher-remake-1-8-tier-2-is-rare (the count) and the user's calculus by scenario.
**Evidence.** `docs/knowledge/_inbox/players-chat-2026-09-28.md`; `run/matches/1790528069-bluegecko-3v1-comet-catcher-10`
(the brief's Lasher cap of three to five, in force since games 3-5, was not followed).
**Status.** supported (the player's words, 2026-09-28), unmeasured: count Lashers made per seat in the record against
the cap, and his tier-2 plant's start against our mix at the time.
**Would be wrong if.** A riot mass with Brutes in front had held his tier-2 units, or he had teched at the same clock
against Brutes.
**Used by.** the brief's bluegecko section (game 10) once the arena game ends and the brief can be edited.

### K-army-artillery-outranges-but-is-never-the-body
**Claim.** Artillery is worth having because it outranges what it faces (the Mangonel hover, `cormh`, 700, over the
tier-1 artillery tanks' 710 in practice by where a hover can stand), but it cannot take a straight fight: one to three
of them behind other units in front, never the body of the army. thebluegecko, after game 11 on SailAway 2: "mongonals
are better than t1 arty because they outrange things. They can't get in a straight fight, so you want 1-3 of them
with other units in front. Same with t1 artillery -- it shouldn't be the primary unit body." And on the map: "it is
better than barbs at water. barbs are bad at water."
**Evidence.** `docs/knowledge/_inbox/players-chat-2026-09-28.md`; game 11 (`run/matches/1790530604-bluegecko-3v1-comet-catcher-11`):
our seat 0 stood with 13 Mangonels among 65 units at 13:00 and the hover army lost to his Riptides and Buccaneers;
comet-5 (2026-09-23): 24 Shellshockers without a screen eaten at 23:06.
**Status.** supported (the player's words, 2026-09-28), unmeasured: count artillery as a share of each seat's army
per minute against the losses of that seat's fights.
**Would be wrong if.** A mass of artillery held a line on its own in a recorded game, or the 1-3 with a body in front
lost to the same fleet.
**Used by.** the brief's unit-mix case and water-map paragraph (2026-09-28).

### K-team-one-hover-lab-two-shipyards-and-the-ships-expand
**Claim.** On a water map with several seats, every seat building both a shipyard and a hover platform before it
expands halves the early extractor count; the division of labour is one seat with the hover platform giving hover
constructors to the others by `transfer`, two seats with shipyards, and every shipyard's first units construction
ships that expand the under-water spots as a constructor does on land.
**Status.** supported (the user, 2026-09-28, after game 12 on SailAway 2: "One problem with that match was lack of
coordination ... One player could have built a hover lab and two could have built shipyards, gifting hover
constructors to the other two that need them. The shipyard should build constructors and expand with mexes like
normal"); measured on one game: shipyards by 1:06 and hover platforms by 3:01 on all three seats, 14 extractors at
8:00 against game 11's 51, ten construction ships that built three extractors between them.
**Evidence.** `docs/knowledge/_inbox/players-chat-2026-09-28.md`; `run/matches/1790532515-bluegecko-3v1-comet-catcher-12`.
**Would be wrong if.** A seat without its own hover platform could not get hover constructors in time by transfer, or
construction ships could not take the under-water spots the hands offer them.
**Used by.** the brief's water-map paragraph (2026-09-28).

### K-army-the-pros-body-ceiling-is-a-dozen
**Claim.** In the experienced players' Comet Catcher duels (40 games, OS 40+, `run/replays/composition.py`) the
typical largest body of one type in a game (the median over games of each game's largest body, chained at 200) is a
dozen of a raider or line tank (13 Blitzes, 13 Bulldogs, 11 Instigators, 13 Grunts, 10 Pawns, 9 Thuds) and eight
Stouts; seven Whistlers or Mortars; four Hammers or Levelers; two Slashers. Rovers alone are massed (16-17 in one body
at 3:50-5:10). The bodies over 30 are late (after 14:00) and mixed (two or three types together), never 30 of one
thing. Ours: 24 Stouts in one body at 17:13 of player-15 (three times the pros' typical largest), 31 owned at once.
**Evidence.** `docs/studies/2026-09-28-body-sizes.md` (the table, with the game and clock of every maximum).
**Status.** measured 2026-09-28 (the user's question: units without micro attention lose effectiveness at a scale,
and we throw armies that cannot all engage against a few tier-2 units). The cost measured the same day in the duel
harness (the study's second table): a Stout ball loses nothing to itself at any size (in reach 0.90-0.92 from 6 to
36 a side), but at equal metal six Stouts beat one Fatboy (+0.12) and twenty-five lose to four (-0.54, in reach
0.63); eight lose to two Bulls by 0.18 and twenty-one to five by 0.31. The ceiling is set by what the enemy fields
from 12:00, not by the ball's geometry. The user's rule for the games (2026-09-28): a lead is spent by taking a
second group elsewhere and, later, by tier 2, never by adding to a body of a unit that does not scale; a first-order
cap for the hands by unit range against ball diameter, once the TAS work (`docs/design/2026-09-28-tas-micro.md`)
has said what the body should do instead.
**Would be wrong if.** The pros' small bodies were the map's chokes and not a choice (the same players on Full
Metal Plate would say), or the count at 200 elmos split one intended body into several (the max at 400 would say).

### K-micro-the-front-stops-on-the-nearest-and-blocks-the-rest
**Claim (the user's registered prediction, 2026-09-28, before the TAS result).** "The correct attack pattern for the
TAS situation [player-14's 9:17 push at E3: fifteen Blitzes against three Centurions and two light turrets] is move in
until most of the ball can attack a turret, focus fire it until done, repeat with the second turret, then clean up
the rest. Our current micro tends to have the units in front stop and shoot at whatever is nearest -- both failing
to hit the correct target and blocking our other units from getting in range." Two parts: (1) the winning script is
approach-to-mass-reach, then turret one, turret two, then the mobiles; (2) the fault in `crates/micro` is the front
rank stopping at its first target (H-MICRO-FOCUS chooses among what is in reach, and a stopped front is a wall for
the ranks behind), so the body engages piecemeal on the wrong targets.
**Evidence.** None yet: the prediction is registered so the TAS (`docs/design/2026-09-28-tas-micro.md`, worktree
`tas-micro`) can confirm or refute it against its own best script, and `run/tas_diff.py` can show whether the front
stops in the micro's replay of the scene.
**Status.** scored 2026-09-28 (`docs/studies/2026-09-28-tas-e3.md` on branch `tas-micro`, worktree
`../bar_bots-tas`; 3 seeds x 8 reps, the enemy on its recorded track, a 0.3 s delay on every order). Part (1), the
turret-first pattern, lost on this scene: the near turret dies at a median 6 s and the far at 8 s, but the three
Centurions standing 50-400 from the turrets shred the ball meanwhile (2,015 damage in the first ten seconds against
the turrets' 1,346) and live on in the draws; the position fell with us ahead in 5 of 24 (gathered first: 4 of 24)
against plain attack-move's 21 of 24 and the winning script's 24 of 24 (+0.318, worst +0.216), which kills the
Centurions first at the rock's corner outside turret cover, pulls out, and takes the turrets last. On the position as
it was (five Pawns and two Rocketeers walking in from 920-1,443, the r1250 cut) the pattern fell 0 of 24 and no
fourteen-Blitz script wins every duel (the best 21 of 24, +0.126); a Shellshocker pair with a Rover spotter does
(24 of 24, +0.348), as do three more Blitzes (+0.268). Part (2), the fault: the front does stop on the nearest enemy
(25-31 of 40 front Blitzes first damaged the enemy nearest them; 90-97% of the front's damage to Centurions; still in
57% of its in-reach seconds under the lane), and the ranks behind are somewhat less in reach (0.29-0.34 against
0.42-0.43 on the 900 cut), but they are blocked by the front in only 4-7% of engaged seconds (the pros 8%), and here
the nearest target was the right one. So the fault on this scene is the ball meeting turrets and their screen at once
rather than the screen first out of turret cover, not who the front shoots. Not measured: a turret line with no
mobile screen, where the prediction may hold as written. Candidate rules from the winning scripts (none built):
don't leave a fight you are winning; fight mobile units outside turret reach; after a kill leave the turret's reach;
gather, then go in together; catch separated units; hunt an unseen unit with a fight, not an attack; spotted
artillery against turrets.
**Would be wrong if.** The turrets fall faster to the Centurions being killed first (the turrets' 430 reach against
the Blitz's 180 makes the approach the cost either way), or the front does not stop in the micro's replay (then the
fault is target choice alone).

### K-pro-the-first-soldiers-are-in-his-half-by-two-minutes
**Claim.** Strong players send their first soldiers across the map at once: on Comet Catcher every side of the OS-40+ pool had a soldier in the enemy's half (nearer his start than its own) by a median 2:06 (Armada 1:40, Cortex 2:48, its Rovers coming later), within 1,500 of his start by about 2:00-3:00, and all but one of 80 killed a building of his, the first at a median 3:49 (Armada 2:48). The behaviour is the floor of strong play rather than what separates winners (winners are 40 s earlier into his half, 78 of their 280 army metal there at 3:00 against the losers' 43 of 334). The players' reason (the user, 2026-10-03): the open space of what the opponent may be doing is so broad that pressure is what stops an extremely greedy economic opening. Our player's first soldiers picket the strip instead: first kill of a building of his at 3:10, never (11 min), 9:24, 5:56, 8:20 in player-40 to player-45.
**Evidence.** `card.py` `pressure` over the 80 carded sides of 40 Comet duels at OS 40+ (`run/data/replays/manifest.jsonl`, re-carded 2026-10-03): first soldier in his half 1:38 / 2:06 / 2:56 (p25 / median / p75), first building killed 2:25 / 3:49 / 5:08; winners / losers medians 1:48 / 2:26 and 4:02 / 3:41; army metal in his half at 3:00 78 / 43, at 5:00 142 / 171. Ours from `run/pro_baseline.py` on the arena records (his start from the truth file). The pressure rows and the `army metal in his half` column are in `pro_baseline` and the review's item O6.
**Status.** measured 2026-10-03 on one map's pool; the brief's rules carry it (the players' words first, the numbers as the case); the report tells the player how old its last look into his half is and whether any soldier of ours has been there. To verify: the next games' pressure rows, and whether a human game (no truth file: his start from the first commander sighting is not yet computed) shows the same.
**Would be wrong if.** A pool on another map (Avalanche, Gecko Isle) shows strong players holding their first soldiers home past 3:00, or our games with the first handful across lose more extractors than the pool's loss rate pays for (K-map-comet-catcher-remake-1-8-first-extractor-lost: the pros lose 1.85 a minute and win anyway).

