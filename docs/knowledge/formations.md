# Formations

How forces are shaped in fights: spacing, grouping, type mix, commanded lines. Measured on the 40 Comet Catcher Remake
1.8 duels with both players at OS 40+ (693 fights) and three of our games on the same map (bank-1, 2v1b-hard,
2v1b-hard_aggressive; 60 fights), `run/replays/shapes.py --orders --boot`, study
`docs/studies/2026-09-24-pro-fight-shapes.md` (definitions of fight, body, core, friend on the line there). The records
cannot show shots or friendly fire for the pros; every muzzling number here is geometry. Scope: that map, game
test-31383, tier 1.

### K-form-pros-order-one-point-per-unit
**Claim.** High-OS players give a group its own point per unit on a straight line spaced about two hulls apart, and we
give a group one point. The spacing, not a line abreast, is what arrives: the pros' cores at contact are blobs more
often than lines.
**Status.** measured (2026-09-24). Last group order (3+ of the fight's participants, one frame) in the 30 s before
contact: pros one point per unit in 753 of 1,070 sides (70%), one shared point in 223 (21%); targets' straightness
(minor over major principal extent) median 0.06 and spacing median 64 elmos (833 orders); 326 of 755 straight lines at
60+ degrees to the travel, 242 at 30-60, 187 under 30. Us: one shared point in 34 of 44, one point per unit in 2.
At contact the pros' cores (3+) are line / blob / stream 27 / 57 / 16% of 731.
**Would be wrong if.** The per-unit points came from something other than a drawn formation (a widget spreading plain
clicks), which the record cannot tell; or the same count on another map or pool showed shared points as the norm.
**Used by.** H-MICRO-FORM (one slot per unit, 64 apart, across the heading).

### K-form-pros-stand-two-hulls-apart
**Claim.** The pros' soldiers stand about two hulls from their nearest friend at contact; ours stand at about one.
**Status.** measured (2026-09-24), nearest-friend distance in the body at t0 and t0+10: Stout (hull 32) pros median 70
elmos, 37% within 48 (446 unit-snapshots), us 40, 79% (799); Blitz pros 67, 38% (4,425), us 25, 78% (435). Incisor 48,
Grunt 40, Brute 54, Pounder 115 for the pros.
**Would be wrong if.** A rerun with our new records showed our spacing at 60+ without a spacing change in the bot (the
three games being unrepresentative), or the pros' spacing on another map at one hull.
**Used by.** H-MICRO-FORM (the slot spacing).

### K-form-friend-on-the-line-twice-ours
**Claim.** A friend stands on the line of fire of about twice as many of our units in reach as of the pros'; part of the
gap is that we fight in bigger bodies, the rest that we pack them tighter (at the same size we are still 18 points
higher).
**Status.** measured (2026-09-24). Units with an enemy in reach and a friendly soldier within 24 elmos of the segment to
it: at contact pros 542 of 2,429 (22%), us 60 of 140 (43%); 10 s in 31% against 54%. Stouts 24% of 228 against 53% of
247. By body size (t0 and t0+10): 3-6 soldiers pros 17% (1,520), 7-12 30% (1,594), 13+ 41% (1,463); us 13+ 59% of
227 (36 bodies). Median body at contact: pros 6 soldiers (961), us 13 (34). Among the pros the share did not predict
the trade: a third or more on the line traded 4.0 points better than less, interval -3.6 .. +11.5 (141 / 141 sides).
**Would be wrong if.** A shots-based count (run/fire.py's muzzled seconds, our records) did not fall when our bodies
were spaced and cut to the pros' size.
**Amended 2026-09-25 (lane-ab, 24 games an arm, the heuristic bot against medium on Comet Catcher).** It fell.
With H-MICRO-FORM the friend-on-the-line share at contact is 23% of 1,900 units in reach (402 fight sides; the pros'
22%) against 32-33% for the old lane and no lane; the nearest friend 51 against 40 (the pros 67-70); muzzled seconds
by a friend on the line 737 of 51,475 in-reach seconds against 1,399 (old) and 3,143 (none). Supported, live.
**Used by.** H-MICRO-FORM (ranks of six, bodies of at most 12); the duel harness's shape instrument measures it (`fol_in`, `fol_blocked`).

### K-form-pros-fight-with-pure-bodies
**Claim.** The pros rarely fight with raiders and line units in one body, and when they do there is no fixed layering
to copy; we fight mixed most of the time.
**Status.** measured (2026-09-24). Cores of 3+ holding both short-range (reach <= 250) and long-range (>= 300)
soldiers: pros 73 of 731 (10%), 71 of 212 (33%) when the side owned 2+ of each on the map; us 20 of 33 (61%), 18 of
23 (78%). In the pros' mixed cores the short-range median stood ahead (> +50 elmos) in 38%, beside in 29%, behind in
33% (73). Mean front rank (1 front, 0 back) is 0.45-0.60 for every common type. The pros' mixed cores did not trade
differently from pure ones: +3.4 points, interval -2.8 .. +8.0 (70 / 634 sides). The pool's armies are raider-heavy
(Incisor, Blitz, Grunt), so a Stout-and-Blitz army at 16+ is thinly sampled.
**Would be wrong if.** A pool with more tier-1 line armies showed mixed bodies with the raiders consistently in front or
behind.
**Used by.** H-MICRO-FORM (raiders and line units under one order are two bodies).

### K-form-a-line-at-contact-trades-better
**Claim.** Among the pros a force that meets the enemy as a line (across at least twice along) trades a few points better
than a blob and more than a stream; front width relative to the enemy's does not matter by itself.
**Status.** measured (2026-09-24), weak. Metal-weighted exchange share: line 52% (192 sides), blob 46% (402), stream
43% (110); line - blob +5.2, interval +0.8 .. +10.1 over resampled games; line - stream +8.1, -0.1 .. +18.4. The
eventual winners' lines traded 60% (102), the losers' 40% (90). Wider core than the enemy's 51% against 49% (165
pairs). Observational: a stream may be a chase or a retreat rather than a choice of shape.
**Would be wrong if.** Our arena games with the micro engine forming lines at contact traded no better than its blobs
over 24+ games.
**Used by.** H-MICRO-FORM (one rank across the heading).

### K-form-pros-raiders-fight-at-reach
**Claim.** The pros' short-range units hold the edge of their reach in a fight; ours close in.
**Status.** measured (2026-09-24). Distance to the nearest enemy combatant over own reach, unit-seconds with one within
reach + 100: short-range pros median 0.94 (86k), us 0.86 (2.9k). Line units 0.87 against 0.88.
**Would be wrong if.** The difference vanished when counted only for Blitzes against the same enemy types.
**Used by.** H-MICRO-FORM (a unit closes on the nearest enemy to 0.92 of its reach and stands there; the raider step-back tried first cost shots and was dropped, form-smoke).

### K-form-a-fifth-of-an-engaged-unit-waits-behind-a-friend
**Claim.** In a fight, a soldier with no enemy in reach stands behind a friend that has one and is shooting for
about a fifth of its engaged time among the pros and a quarter to a third among us; the share grows with the ball
(ours 10% alone to 29% with six or more friends within 120; the pros' 13% to 25%). The front stops when it has a
target and the rest, sent to the same point, are held behind it.
**Status.** measured (2026-09-25), `run/queued.py` (engaged: four or more armed enemies within 700; queued: no enemy
within reach + 20, a friend within 300 in reach, nearer its enemy, and firing this second; geometric: without the
shot). Ours, shots / geometry: bank-1 25% / 37% of 9,418 engaged soldier-seconds, 2v1b-hard 15% / 24% of 4,494,
2v1b-hard_aggressive 10% / 23% of 1,293, family-1 14% / 27% of 4,797; the pros on Comet Catcher at OS 40+ (40
games, both sides) 20% geometric of 243,795. Reproduced in the duel harness: the bank-1 13:50 cut (45 against 15)
13% queued under the plain attack-move, 17% under H-MICRO-FORM's ranks of six (the rear rank's slot is out of
reach behind a standing front); family-1 11:05 (37 against 13) 12% and 8%.
**Would be wrong if.** A body whose rear filed to the ends of its front (H-MICRO-FORM-FLANK) still spent a fifth
of its engaged time queued in the harness, or the pros' share on another map fell well under 20%.
**Amended the same day (the blocked refinement).** Queued with the firing friend within 40 of the unit's line to
its nearest enemy (physically in the way) is *blocked*: the pros 8% geometric; ours 22% / 16% / 11% / 17%
geometric (bank-1, 2v1b-hard, 2v1b-hard_aggressive, family-1), 14% / 10% / 4% / 9% with the shot. Nearly all of
our blocked seconds are units *moving* (bank-1: 1,245 of 1,335; family-1 403 of 409), most in the first ten seconds
of their engagement: the walk-up of the rear behind a front that has stopped, not a standing jam (90 still seconds of
9,418 engaged in bank-1). In the harness the blocked share is 6-10% in every arm, the ball's included (the bodies
reach the enemy within seconds), and the live medium game form-7 6%. The cost of the walk-up is friendly fire, the
rear firing into the front: 37-40% of the damage done in the bank-1 13:50 cut under the plain order, 18% in
family-1 11:05; H-MICRO-FORM-FLANK takes those to 4% and 3% and the fight from +0.69 to +0.84 and +0.66 to +0.71.
**Used by.** H-MICRO-FORM (the line case, since 2026-09-28; before, H-MICRO-FORM-FLANK); the harness's `engaged_s` / `queued_s` / `blocked_s` columns.

### K-form-the-rear-at-the-ends-not-in-the-back
**Claim.** When a body's front stops with targets in reach, the units behind it should file to the ends of the
front's line rather than close on the same point: the walk into the front's back costs the fight its friendly fire
(the rear's shells land on the front) and its exchange, and the ends of the line are where the rear can reach.
**Status.** measured (2026-09-25) in the duel harness, 8 repetitions an arm on the same sites and seeds, with a null
arm: bank-1 13:50 (45 against 15) plain +0.693 +- 0.016, the formation without the rule +0.804, with it
+0.840 +- 0.006, killed per lost 0.67 / 1.05 / 1.30, friendly fire 40% / 17% / 4%; family-1 11:05 (37 against 13)
+0.658 / +0.578 / +0.713, friendly fire 18% / 22% / 3%; the user's mixed force (8 Blitz, 7 Stout against 14 Pawn,
6 Stout, 2 Janus) -0.182 +- 0.056 without, -0.072 +- 0.066 with; 12 Stouts against 20 Grunts and 13 Thuds -0.001
without, +0.132 +- 0.042 with; the Blitz pack and the Stout mirror unchanged (+0.287 against +0.154 / +0.32 with the
formation alone; +0.023 against +0.034). Not measured in a live game with fights (form-7: exchange 1.34, one game).
**Would be wrong if.** An arena A/B of H-MICRO-FORM-FLANK over 24 games traded no better, or a live game with the
rule showed the ends of the line standing out of reach while the front lost (the wide line's known cost).
**Amended 2026-09-25 (lane-ab).** The whole final lane (the flank in it) against the old lane and no lane, 24 games
an arm: metal lost per metal killed 0.79 [0.65-0.95] against 0.84 [0.73-0.93] and 0.94 [0.83-1.05]; friendly fire
3.6% against 4.6% and 4.6%; blocked 6% against 7% and 6%. The rule is not separated from the rest of the lane in
the arena (the arms are whole binaries); the trade moved the way the harness said, within intervals that overlap.
**Amended 2026-09-25 (flank-ab, the rule alone).** `--ab-disable H-MICRO-FORM-FLANK`, 24 games an arm on the same
seeds and corners: with the flank 0.79 [0.68-0.91] lost per killed, without 0.87 [0.74-1.04]; friend on the line
at contact 26% against 31% (the pros' 22%); muzzled by a friend on the line 736 against 1,336 s; friendly fire 4.0%
against 4.3%; blocked 6% in both; wins none. Supported on the line of fire and the muzzling; the trade within the
noise (about 0.15 of ratio at 24 games).
**Used by.** H-MICRO-FORM (since 2026-09-28 the line case of the body's shape: a rank at the body's reach along a party wider than it is deep, its ends curled round; H-MICRO-FORM-FLANK folded into it).

### K-form-rocketeers-hit-the-friends-in-front
**Claim.** Three quarters of our friendly fire in heuristic games on Comet Catcher is the Rocketeers' rockets on
the Hammers, Warriors and other Rocketeers in front of them: the Rocketeer (reach 475) fires from behind the
front at the enemy the front (Hammers 380, Warriors 325) is fighting, and the rocket meets a friend on its low path.
It is not splash at the target, and the formation does not change it.
**Status.** measured (2026-09-25). lane-ab (72 games): the Rocketeer's share of friendly fire 74-76% in every arm
(new 50,475 of 60,000 hp; off 67,956 of 84,000), its victims Warriors 16,807 / Hammers 16,326 / Rocketeers 7,616
(new). At the 258-324 friendly-fire unit-seconds an arm: the nearest enemy at a median 404-431 elmos; a friend
within 40 of the line to it and nearer in 563-1,035 friend-events (Hammers 233-411, Rocketeers 186-430, Warriors
128-193); a friend within 60 of the enemy in 3-13. The cut lane-ab-new 05 21:10 (74 against 21) replayed 8 times:
friendly fire 8.7% of damage done with the lane off, 9.8% with it on, the same victim table both ways.
**Evidence.** `run/fire.py`'s `xf` over `run/matches/1790299846-lane-ab-new`, `-old`, `-off`;
`docs/data/scenarios-2026-09-25/lane-ab-new-05-2110-truth.json`; batches rock-2110-off/-on.
**Would be wrong if.** The engine's `avoidFriendly` test for the Rocketeer's missile were shown to refuse these
shots (it does not: the damage is booked with the Rocketeer as attacker), or a cut with the Rocketeers in front
of the Hammers showed the same share.
**Used by.** (none; a candidate for a slot rule that keeps the long-reach body's line clear of the line body, or
for the unit mix)

### K-form-a-body-stationed-among-buildings-jams
**Claim.** A body under H-MICRO-FORM stationed beside a factory and its solars (the hold packet's group_A at spot_45,
40-80 Stouts by the mid game) is held in slots on and between buildings: three times the engine's move failures and
stuck seconds of the same packet with the lane off, with no gain in the trade. The transform snaps slots to reachable
terrain, not to ground clear of buildings.
**Status.** measured (2026-09-25 late), hold-lane-on against hold-lane-off, 48 games an arm.
**Evidence.** Move failures 28 against 8.6 a game; stuck seconds (`run/floor.py` stuck_s) 573 against 163; metal lost
per killed 0.66 [0.57-0.76] against 0.68 [0.59-0.77]; nearest friend at contact 41 against 40; friend on the line 31%
against 41%; friendly fire 8.1% against 6.7% of damage (Stout on Stout 208k against 149k hp); muzzled 10.3% against 7.6%
(by a friend 2,542 against 2,984 s, at the reach's edge 1,009 against 637, clear line 3,344 against 2,536); speed share
over far-goal seconds 0.63 against 0.65 with 10% against 2% of soldier-seconds under two orders (speed 0.41 under two).
The raids' regime (packet-lane-on/off, Blitz skirmishes in the open) had shown the opposite on friendly fire (3.5%
against 8.3%) and spacing (49 against 25): the lane's shape effects belong to bodies in the open, not to a block parked
on a base.
**Would be wrong if.** The same body stationed in the open (spot_36) showed the same move failures and stuck seconds
with the lane on.
**Used by.** H-MICRO-FORM (a slot on a building is a defect to fix: clear-ground snapping, or a smaller footprint at a
station).

### K-form-a-concave-walked-beats-area-fire
**Claim.** A body of tier-1 tanks meeting a few area-fire units (Fatboys, Bulls) should walk to slots on a concave at
its own reach from them, spread wider than two hulls, with Move orders: that takes the fight from a loss to a win at
equal metal, and most of it is the walking (the shells miss units that keep moving to their slots) rather than the
spacing's width. The same concave walked with Fight orders (which stop at the first thing in reach) loses as the ball
does.
**Status.** measured (2026-09-28) in the duel harness only, Mithril Mountain v2.0.1 flat sites, equal metal, 8 duels
an arm (`docs/studies/2026-09-28-body-shape.md`): 25 Stouts against 4 Fatboys +0.462 (attack-move -0.515, the lane of
899d64a -0.335, the concave with a Fight at 64 -0.429; with a Move at 65, 110, 240: +0.424, +0.427, +0.437); 21 Stouts
against 5 Bulls +0.366 at 130 (-0.329, -0.385, -0.352; a Move at 65 +0.154, at 98 +0.323, at 160 +0.369); 6 against
1 Fatboy +0.648 (+0.142); 8 against 2 Bulls +0.189 ± 0.241 (-0.186). The Fatboys' damage per counted shot 2,074 under
the Fight, 1,246-1,379 under the Move. In reach while engaged against the Fatboys 0.72-0.75 against 0.63-0.68; muzzled
0.2-0.3 of in-reach seconds under the Move (walking units in reach fire less), 0.00 under a Fight.
**Would be wrong if.** A live game's tier-2 fights under the lane (read with `run/fire.py`) showed the Stouts walking
under the Move losing their shots without the area fire missing them (the Fatboys' or Bulls' damage per shot the same
as against a ball), or the slots on sloped ground unreachable (the duel's sites are flat and its view does not snap).
**Used by.** H-MICRO-FORM-SPACING (the march's spacing); H-MICRO-FORM's contact concave and H-MICRO-FORM-ARC until
2026-10-28, when the ring went on the user's ruling (`docs/design/2026-10-28-contact-cleanup.md`): the claim stands as
measured, and the cleanup's bench reprices it against ranks held through contact.

### K-form-a-per-unit-step-out-of-a-turret-does-not-pay
**Claim.** Stepping a unit out of a turret's reach whenever it stands inside it with nothing of its own in reach (the
TAS's "never idle under a turret", as a per-unit reflex) does not improve a fight: it pulls units walking to their
slots back out and drags the fight; the TAS's gain from leaving the turret's reach came from the body's order of
targets and its gathering, which are body-level decisions.
**Status.** measured (2026-09-28) in the duel harness, six arms of 24 duels (E3 900 and r1250, F3 r1500; the recorded
track and the reacting director): as first built E3 900 on the track +0.056 against +0.217 without (the fight 35 s
against 22 s); for idle units only +0.146 / +0.217 there, +0.161 / +0.230 on F3's track, and within 0.04 on the other
four (`docs/studies/2026-09-28-body-shape.md`). H-MICRO-STEP-OUT retired the same day.
**Would be wrong if.** A body-level version (a body whose target is dead gathering outside the turrets' reach before
the next target, the TAS's v5) gained on the same arms.
**Used by.** none (H-MICRO-STEP-OUT, retired).

### K-micro-the-contact-followed-a-tick-into-his-base
**Claim.** The lane's contact rule (H-MICRO-FORM) forms on the nearest armed mobile enemy within 700 of the body's
centre and moves each unit's slot on when the target has walked 96 from it, with no bound to the body's own goal:
in player-21 six Rovers whose group task read `fight_to spot_61` and then `spot_64` (C6, D7, stations in our half)
got a fresh Fight point every 1-2 s from 5:41 trailing party_11, a lone Tick, along row 2 (the points from
(1144, 2280) at 5:40 to (5976, 1000) at 6:40, 4,700 elmos), and all six died at (5741-5983, 911-1047) at 6:40-6:46
to Rocketeers, Centurions and Pawns for one Lazarus. Neither the player nor a pick ordered the chase.
**Evidence.** `run/matches/1790568930-player-21-leash/00/record-0.jsonl` `cmd` rows for units 25079, 26776, 1971,
19953, 5134, 24632 (the fight points by ten-second bucket); the review's finding 3; `crates/micro/src/lib.rs` `form`
at 1ffc8e9 (`engaged` = any foe within `HORIZON`, `keep` the target most claims name).
**Status.** fixed 2026-09-28 (H-MICRO-FORM-PURSUIT: a target that has run 600 from first contact and is out of
reach is let go for 30 s; a lane test in `rove.rs` shows the let-go and the return to the station); the cause gone
2026-10-28 with the contact target itself (`docs/design/2026-10-28-contact-cleanup.md`: a block halts in its ranks for
what is in reach and follows nothing; the `rove.rs` test keeps the case). Not yet measured: the harness has no fleeing-target scenario, and the E3/F3 arms hold
static or standing targets, so they cannot show a regression from it either way.


### K-micro-the-release-doubled-the-hosts-stop
**Claim.** The lane's `release` re-sent a soldier's standing order when its claim ended, and when the host's order that
ended the claim arrived in the same tick, that order was already in the host's list: every group-wide stop to a formed
body went out twice (player-22, minute 22: 171 of 173 duplicated orders were stops, 110 stops for 55 units in one row
at 22:21). Fixed: a release re-sends nothing the host gave this tick (`crates/micro/src/lib.rs` `release`; the test
`a_hosts_stop_to_a_formed_body_goes_out_once` fails without the condition).
**Evidence.** `run/matches/1790646080-player-22-commit/00/record-0.jsonl` (the `cmd` rows of minute 22); the review
agent's report on the game.
**Status.** fixed 2026-09-28. It cost nothing in play by itself; it doubled every order storm and the counts.

### K-form-the-form-up-rank-assumed-flat-ground-and-a-compact-body
**Claim.** The march's form-up block (ranks 300 ahead of the body's centre, the same slot indices as at the
destination) breaks a body that is strung out or standing on broken ground: units ahead of the block are sent back to
it, points land on ground the units cannot climb, the units with a short way reach their points and go on alone, and
the rest arrive one by one.
**Status.** observed in one game (the user watching the replay, 2026-10-03), measured from its record. player-49
12:33 (f 22590), group_O, 25 units with a chained pair of fight orders: the body lay over about 760 elmos with its
centre (4412, 2349) in a crater (x 4290-4520, z 2200-2400, rim to height 322 on a floor of 237); 8 of the 25 form-up
points on rim cells of slope 57-104 against the tank classes' limit of 28 (the snap used the Pawn's field, limit 105);
four Stouts at x 4580-4735 given form-up points 290-360 behind them; the tail given points 640-800 away. 11 of 25
ever reached their form-up point (12:33-12:45), the destination points were reached singly 12:43-12:49, two of the
group died at 12:37 and 12:39. The chain was sent three times in four seconds (the hands' shell / attack / shell at
12:33, 12:35, 12:36), each from a moved centre.
**Evidence.** `run/matches/1791038139-player-49/00/record-0.jsonl` (`cmd` rows at f 22590, the `s` rows after),
`terrain-0.bin`; `crates/micro/src/lib.rs` `form` (the march), `crates/bot/src/brain/routes.rs` `survey` (the home
field is the raider's class).
**Would be wrong if.** A game after the 2026-10-03 fixes shows a body still arriving one by one from a march with no
re-orders in it (then the cause is the strung-out body itself or the centre-based contact test, both left as they are).
**Used by.** H-MICRO-FORM (the 2026-10-03 amendment). Open: "engaged" is judged from the body's centre, so a strung-out
body whose head is in contact still marches; a strung-out body is not gathered before it forms.

### K-group-an-anchor-placed-by-far-members-sawtooths-the-march
**Claim** (2026-10-25, from migration-2-2 and the D21 drills). A formation whose anchor is placed or re-aimed from the
centroid of every member, far joiners included, and whose members hold the anchor only after they have once stood in a
slot, walks a sawtooth whenever (a) a third or so of the members are far behind and (b) something marks the group dirty
on a clock: the anchor is set down hundreds behind the head, the head walks back onto slots behind it, the anchor runs
on at full pace with nobody holding it, and the next dirty tick repeats it. In a game the clock is the ground rebuild on
any structure or wreck changing anywhere (every 5 s, 82% of five-second windows), and the far members are the joiners
every pick makes. The pace type's lead of 300 masks the repetition (it rides ahead of its slots) but not the first
set-down; a faster type with a learned lead of 0 shows the full sawtooth.
**Evidence.** migration-2-2 (`run/matches/1792944804-migration-2-2`): group_B 7:06-7:42 under one order, the anchor
set down 300-640 behind the head every fifth second, members ordered 300-430 backwards, 800 elmos in 35 s at speed 101;
3,611 marching group-seconds at formed 0.26, 215 backward snaps, 192 within 5 s of a footprint change. Drills (ledger
d21-*): twelve Blitzes with six Pawns 1,800 behind and a structure churned every 5 s snapped at 5, 10 and 15 s (anchor
back 798, arrival 23 s against 14 without churn); Pawn-only arms placed the anchor 350-440 behind the head at the
declaration (formed 0 for the walk) and snapped once at a pull over 400; churn on a formed body did nothing.
**Would be wrong if.** Under D22 (the anchor on the densest cluster, the drag from everyone within 300, no set-down on a
rebuild) the same arms still show the snaps, or a game shows the sawtooth with the anchor placed on its body.
**Used by.** H-MICRO-GROUP (the D22 amendment); design D21, D22.

### K-group-a-placement-that-counts-joiners-or-ignores-the-fleeing-puts-the-anchor-with-the-stragglers
**Claim** (2026-10-25 evening, from migration-3-1 and the D23 drills). A formation whose anchor is placed from the
positions of every member (joiners included) or of every member not fleeing is put with whoever is densest or still
standing, not with the body that marched: with a feeder joining fresh units at the lab, the lab's cluster outnumbers a
small front and a re-declared order (a follow step, a party that moved, a Hold) re-aims the anchor to the lab; with
members strung out along a road none within 400 of another, the tie goes to an arbitrary member; with the body fleeing
after a fight, a new order's set-down goes to the two or three stragglers not fleeing, and the body, done fleeing, is
"away" from slots a map behind it. The placement has to read the body the group had (members once formed, where they
stand now, fleeing or away included), everyone only when nobody has formed yet, ties toward the goal, and a new order
must move nothing while a body member is within reach of the anchor.
**Evidence.** migration-3-1 (`run/matches/1792955667-migration-3-1`): 16:44 and 16:47 the anchor of a one-member march
re-aimed onto the four fresh units at the lab (back 913); 5:55 a Hold on five Blitzes strung out resolved to the one at
the lab, the two at the enemy's edge marked away, the march restarted from the lab at 6:00 and reached the front at
6:30; 20:18 and 20:26 set-downs with 20 of 23 fleeing went 1,500-1,850 to the stragglers, nobody formed 20:26-20:49.
Drills (ledger d23-*): six Pawns walking 800 with eight joining 1,800 behind at 4 s and the goal re-declared at 5 s:
under D22 anchor back 2,130, backtrack 0.18, the front arriving at 29 s; under D23 anchor back 37, backtrack 0.01, the
front at 7 s as without the re-declaration; the join alone moved nothing on either (membership is not an order). Six
Pawns 440 apart: D22 anchor back 65 and backtrack 0.12; D23 the anchor at the front member, 0 and 0, the front
arriving at 13 s as a plain walk (the rest joiners the whole way, formed share 0.08: what the host must decide to
wait for).
**Would be wrong if.** A game under D23 shows the anchor set down away from its formed members on a new order, or a
formed body re-aimed onto a joiners' cluster; or the strung-out walk's forward placement loses more than it gains (a
host that wanted the group gathered gets a front that walks on alone).
**Used by.** H-MICRO-GROUP (the D23 amendment); design D23.

### K-group-ranks-laid-in-the-travel-frame-rotate-at-every-bend
**Claim** (2026-10-31, the user's reading of apex-v0409-comet-sym0-opus-1 and the code). With the slots laid in the
frame of the heading, and the heading recomputed every tick toward the next way point, every bend of the way rotates
the whole grid about the anchor: a rear slot of a 300-deep block moves about 100 for a 20-degree bend, each member
walks a different vector to its slot, and two interleaved blocks rotating about two anchors cross each other's paths.
That is the shuffle at trajectory changes, and it is worst with another formation's ranks interspersed. Units move at
one speed in every direction, so a block that keeps its facing and translates through a bend loses nothing on the
march; facing matters at contact, where the close re-aims it. Holding the facing between corners is not free on a
corridor circuit: the skewed block wants more lateral room than the planner priced for its half-width across the
heading.
**Evidence.** The code before the change (`group.rs` `tick`: `heading` set every tick, `form::slots(anchor, heading,
..)`, the re-deal only at corners of 20 degrees or more). The drills (ledger face0 / face1, 3 reps a circuit arm, 8 a
pair arm): the plain circuit 177 / 179 / 181 s at formed 0.95 before, 182 / 184 / 186 at 0.93 with the facing held
to 60 degrees, 179 / 182 / 184 at 20 degrees, 182 / 185 / 186 at 120; the fed circuit 179 / 180 / 181 and
180 / 181 / 183 at formed 0.82 both, its join median 59 to 51 s; the pair drill's second block arrived 49 → 46 s (walk)
and 56 → 51 (follow) with overlap 34.5 → 32.1 and 0 → 0.4 s, the attack taken 8 of 8 against 7 of 8.
**Would be wrong if.** A game under H-MICRO-FACING showed the interleaved shuffle unchanged, or the circuit's three per
cent grew into bodies cut off at walls in play (the room check is the mitigation to build then).
**Used by.** H-MICRO-FACING.

### K-group-companies-over-the-cap-march-superposed
**Claim** (2026-10-31, from apex-v0409-comet-sym0-opus-1). Two companies of one army that together exceed the cap
walk the same route from the same place superposed for the whole march: the merge cannot take them, each company's
way is planned over a ground that knows nothing of other formations, and `keep_off` parts them only at the way's end.
A third company sent to an end two others hold never settles: its placed end moves about and its members stand in the
others' ranks. The planner has to see the others: a standing formation's cells dear enough to walk round, a marching
one's dear enough to lean off, and a re-plan when a standing one comes onto the way before its end; the end itself
stays one rule's (`keep_off`): two placements of it disagreed and moved it at every re-plan (foot1 / foot2).
**Evidence.** The record's `groups` (`run/matches/1793417501-apex-v0409-comet-sym0-opus-1/00`): 1,746 company-pair-
seconds of two marching companies of group_G within 190, every pair over the cap of 24; two companies of 22 within
22-206 from 15:00 to 21:47, 327 s unbroken from 16:20; marching companies of 15+ formed 0.60 with another within 190
against 0.67 with none within 600; at spot_46 13:45-14:15 a company of 24 beside two arrived ones swung 8-21 of 24
formed with 6-16 exceptions at an end 240-415 from the spot. `ground.rs` before the change: `way_found` over the
ground's costs alone; `lib.rs` giving `keep_off` the footprints after the plan.
**Would be wrong if.** The pair drills under H-MICRO-GROUND-FORMATIONS showed the second block's arrival later or its
overlap longer than under `keep_off` alone (ledger pair2 / foot1), or a game showed companies walking through each
other's standing footprints with the price in force.
**Used by.** H-MICRO-GROUND-FORMATIONS.

### K-group-a-disk-of-room-is-no-stand-among-our-buildings
**Claim** (2026-10-31, from barb-ladder3-20-1). A body's standing place cannot be judged by the clearance about one
cell. The clearance (`room`, the distance to the nearest blocked cell) is the right price for a corridor the body walks
through; at a destination the question is whether the body's slots land on open ground, and a formation standing at a
building has the building inside its hull, where a disk of its half-width is empty by definition. Judged by the disk,
every defended extractor, factory and turret cluster of ours sends a body told to stand there to the nearest open field
within the search's reach, and the hands, told by the lane that the walk ended, call the place reached. Judged by the
slots, the body stands beside or around the building, within its own depth of the point.
**Evidence.** The record and Jev log of `run/matches/1793456996-barb-ladder3-20-1/00`: group_W1 (12 Blitzes) at
14:18-14:51 under the packet "With no raider at our extractors it stands at spot_63"; the lane's end recomputed from
`terrain-0.bin` and the structures standing (an extractor and two Sentries at spot_63): room at the spot 36 with them,
285 without; the nearest cell with 224 of room (2952, 4296), 480 from spot_63 and 233 from spot_59, the anchor's place
to the elmo; the walk's way 0-1 elmo, arrived at once; the register alternating "reached spot_59" / "reached spot_63"
every second; 23 picks of go_spot_63 at 0.57-0.75. group_Q3 at the same spots 22:53-23:15 (way 332 → 35, arrived 441
off): 12 picks. The unit tests under H-MICRO-STAND. Drill (`duel --drill walk --ours armflash*12 --order move --lane on --post armmex*1+armllt*2 --distance 1500`, ledger stand0 / stand1): the old stand stopped the body 270-400 short of the extractor in 3 of 3 and the lane called it arrived with no Blitz within 300; the new one ends 46-55 from it with 10-11 of 12 within 300 at 14-15 s.
**Would be wrong if.** A body under H-MICRO-STAND stood on or jammed against a building it was sent to (the slots'
snapping failing to part it), or the circuit and pair drills (ledger stand1-*) lost formed share or arrival against the
face1 / foot1 rows.
**Used by.** H-MICRO-STAND, H-HANDS-ARRIVAL.

### K-group-a-follow-point-that-is-a-member-swings-with-the-stragglers
**Claim** (2026-10-31, from barb-ladder4-20-1). A group that follows another cannot be aimed at a member of the ward.
The hands' body place (`Group::body`: the member at the 60th percentile of distance from the centroid) names a different
member whenever the ward is loose, and a declared ward is loose by construction: its newcomers are members with slots
from the moment they join, so a Flash walking in from the far side of the base is the 60th-percentile member of five
Bulls. A follower aimed at it is aimed at the straggler, and the lane, told a route end that moved more than 48, re-aims,
re-plans and re-deals the body; the follower mills and never settles beside its ward. The ward's place for a follower is
the ward's formation, its anchor, and following a formation is the lane's business as placing one off another is.
**Evidence.** The Jev log and record of `run/matches/1793479422-barb-ladder4-20-1/00`, 17:53-19:33, under the packets
"group_V2 (our two Bulls) leads it: [seven groups] follow group_V2 or join it, side by side as one body": the follow
chain V2 ← U2, N2; U2 ← D, J2, T2, O2; N2 ← W2 (33 soldiers behind two to five Bulls fresh from the Bull plant, which
held at the plant 18:04-18:14 with a spread of 1,000-1,370: a Flash joining from (606, 4020) and a Bull "away"); U2's
follow target at 18:28 (1677, 5901) is the Flash's position, at 18:29 (2262, 6049) a Bull 600 off; U2's way left 0 →
456 → 364 → 536 → 324 → 135 → 86 → 438 in eight seconds and its centroid moved 330 elmo in the twenty seconds 18:20 to
18:40; whole game 2,927 follow group-seconds, 580 of 2,830 consecutive follow-point moves over 150, 54 over 400, the
largest 1,604. The lane's test `a_follower_keeps_to_its_wards_anchor_and_stands_beside_it` (a larger follower stands
beside its ward, not on it, and keeps within 700 on the march), the hands' `a_following_group_names_its_ward_to_the_lane`;
drills ledger ward0 / ward1.
**Would be wrong if.** A follower under the fix lagged its ward further on the pair-follow drill than the hands' stepped
point did (face1 / stand1: anchors 486-509 apart, overlap under 1 s, arrival 49-51 s), or a follower twice its ward's
size stood on it.
**Used by.** H-HANDS-FOLLOW, the contact cleanup's §5.3.
