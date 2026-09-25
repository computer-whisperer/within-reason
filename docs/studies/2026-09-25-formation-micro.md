# Formation micro in the engine: the lane driving one side of a duel

Part B of `docs/design/2026-09-25-formation-micro.md`, 2026-09-25, by Claude (Fable 5.1). The duel harness runs the
bot's control lane (`crates/micro`) for one army (`duel --lane`), so a fight can be fought with the lane off (the
director's plain attack-move, the harness as it was), with the old lane (`old`: flee, fan, kite; focus while it
existed) and with the formation (`on`: the old lane and H-MICRO-FORM), on the same sites and seeds, and read with
the fire instrument (muzzled seconds by cause, friendly fire and its victims, the exchange) and the shape instrument
(nearest-friend spacing and the friend-on-the-line share at contact, `run/replays/shapes.py`'s definitions, so the
numbers sit beside the pro survey's: `docs/knowledge/formations.md`). Every arm is 8 duels unless said; `margin` is
the first army's value left minus the other's; `k/l` is metal killed per metal lost over the arm; `nn` the median
nearest-friend distance at contact and 10 s later; `fol` the share of units in reach with a friend within a hull of
their line of fire; `ff` friendly fire over all damage done. Comparisons are `run/duel_ab.py <off> <arm>`.

## What voided the first day's batches, and the checks that stand

Two things had to be found before any number below meant anything.

1. **The lane took the other site's units.** A match fights two sites at once and the team's snapshot holds every
   unit of the team, so the lane on one site treated the other duel's units of its team as its own, uncommitted, and
   fled them from everything. The tell was a null arm: the lane present with every rule switched off lost 0.27 of
   margin to the plain order (form2-blitz-old, 7 losses of 8 in fights twice as long) while the lane's own debug
   count showed no claim on its own army. Fixed with `View::mine`; the null arm then matched the plain order
   (Blitz +0.152 +- 0.049 against +0.154 +- 0.047; Stout +0.026 +- 0.048 against +0.034 +- 0.057). Every lane arm
   before the fix is void (the ledger names them); the retirements taken on them were undone and re-measured.
2. **Single-rule ablations had never worked.** The lane checked `enabled` only for H-MICRO-LANE and H-MICRO-FORM;
   `WITHIN_REASON_DISABLE=H-MICRO-FOCUS` ran the whole lane. Each rule is gated by its ID now, in the bot and the
   harness.

Smaller: a stray sweep bomb wiped one army in 4.9 s with nothing dealt by either side (one row of ~250;
`duel_ab.py` drops such rows), and two runs of the same `off` arm differ by more than their standard errors
(Blitz +0.154 +- 0.047 and +0.210 +- 0.050): read the between-batch spread, not the within-batch error, as the noise.

## What the smoke duels taught (one site, 1-2 duels each, `WITHIN_REASON_MICRO_DEBUG` naming what held each muzzled unit)

11 Blitzes against 22 Pawns. With the lane off the Blitzes win (+0.26) firing 6.6 shots per in-reach second, 0%
muzzled. The first formation re-ordered a unit between standing, stepping back and closing every few frames: 1.8
shots per in-reach second, 24% muzzled, cause "clear". One Stop per stand: 2.4. The old lane alone: 24-33% muzzled,
21 of 36 muzzled seconds under H-MICRO-FLEE, which fled seven to ten Blitzes at contact (their fire on the ball,
2,100-2,450 a second, is "lethal" for every 730-hp unit in it) in a fight the Blitzes win. Strength odds (damage a
second times health: 22 Pawns out-shoot 11 Blitzes and do not out-fight them) and an "already under the guns" test
took the flee to 0% muzzled in both arms. So: an order costs shots, a unit that walks does not fire, and the
formation's contact behaviour became one Stop, held while anything is within reach + 100.

## The batches (8 duels an arm, `--parallel 2 --sites 2`, Quicksilver Remake 1.24, speed 50)

The first army is the lane's side; `off` arms were fought once (twice for the Blitz pack, the spread between the two
is the noise floor). `old` is the lane as committed on 2026-09-25 before focus was retired (focus, the amended
lethal flee, kite, fan); `on` is the final lane (H-MICRO-FORM, the amended flee, kite, fan; no focus). Batches
form-blitz-off, form2-blitz-off, form4-blitz-old, form4-blitz-on, form4-blitz-on-nofocus, form8-blitz-on,
form-stout-off, form4-stout-old, form4-stout-on, form4-stout-on-nofocus, form8-stout-on, form5-stoutmix-off/-old,
form8-stoutmix-on, form5-mix-off/-old, form8-mix-on.

**Blitz pack: 11 Blitzes against 22 Pawns (1,200 metal a side)**

| Arm | Margin | W-L | Killed per lost | Shots per in-reach s | Muzzled | Friendly fire | Nearest friend | Friend on the line |
|---|---|---|---|---|---|---|---|---|
| off (twice) | +0.154 +- 0.047, +0.210 +- 0.050 | 7-1, 8-0 | 1.16, 1.24 | 6.6 | 0%, 1% | 10.7%, 9.0% | 43, 39 | 28%, 26% |
| old (focus, lethal flee amended, kite, fan) | -0.001 +- 0.058 | 4-4 | 0.98 | | 1% | 10.8% | 41 | 28% |
| old + H-MICRO-FORM (form4-blitz-on) | +0.098 +- 0.021 | 8-0 | 1.09 | | 0% | 11.2% | 58 | 26% |
| the same without focus (form4-blitz-on-nofocus) | +0.308 +- 0.023 | 8-0 | 1.42 | | 0% | 10.7% | 59 | 28% |
| **on**, final (form8-blitz-on) | **+0.322 +- 0.024** | 8-0 | **1.45** | 6.5 | 1% | 7.8% | 59 | 23% |

**Stout mirror: 12 against 12 (2,700 metal a side)**

| Arm | Margin | W-L | Killed per lost | Shots per in-reach s (0.83 possible) | Muzzled | Friendly fire | Nearest friend | Friend on the line |
|---|---|---|---|---|---|---|---|---|
| off | +0.034 +- 0.057 | 5-3 | 1.04 | 0.83 | 0% | 0.1% | 40 | 24% |
| old | -0.053 +- 0.058 | 3-5 | 0.94 | | 0% | 0.2% | 40 | 27% |
| old + form (form4-stout-on) | -0.023 +- 0.046 | 4-4 | 0.98 | | 0% | 0.0% | 57 | 18% |
| the same without focus | -0.071 +- 0.037 | 3-5 | 0.93 | | 1% | 0.0% | 58 | 22% |
| **on**, final (form8-stout-on) | +0.040 +- 0.036 | 5-3 | 1.04 | 0.83 | 0% | 0.0% | **60** | **19%** |

**Stout body against a Cortex bot mix: 12 Stouts against 20 Grunts and 13 Thuds (2,700 against 2,680)**

| Arm | Margin | W-L | Killed per lost | Shots per in-reach s | Muzzled | Friendly fire | Nearest friend | Friend on the line |
|---|---|---|---|---|---|---|---|---|
| off | -0.001 +- 0.050 | 4-4 | 0.99 | 0.77 | 0% | 0.5% | 42 | 31% |
| old | +0.052 +- 0.044 | 5-3 | 1.05 | | 0% | 0.2% | 40 | 31% |
| on, a Stop at contact and a stand-keep of 100 (form5-stoutmix-on) | -0.086 +- 0.063 | 3-5 | 0.90 | 0.66 | 6% (line 52, edge 19, clear 76 s) | 0.8% | 62 | 25% |
| **on**, final | +0.032 +- 0.044 | 5-3 | 1.03 | 0.78 | 0.4% | 0.6% | 61 | 23% |

**The user's case: 8 Blitzes and 7 Stouts against 14 Pawns, 6 Stouts and 2 Janus (2,455 against 2,586)**

| Arm | Margin | W-L | Killed per lost | Muzzled | Friendly fire | of it the Stouts' shells on the Blitzes | Nearest friend | Friend on the line |
|---|---|---|---|---|---|---|---|---|
| off | -0.182 +- 0.056 | 2-6 | 0.86 | 0% | 4.7% (5,110 hp) | 4,291 | 43 | 37% |
| old | -0.193 +- 0.091 | 3-5 | 0.84 | 0% | 3.3% (3,536) | 3,015 | 45 | 34% |
| on, final | -0.160 +- 0.038 | 2-6 | 0.88 | 0% | 6.0% (6,711) | 4,898 | 51 | 43% |

The victim table names the user's observation exactly (the Stouts' shells on the Blitzes are 84-86% of the force's
friendly fire in every arm) and the formation does not cure it: the raiders form their own body, but both bodies
close on the same enemies and the faster Blitzes end in front of the Stouts anyway. Keeping the raider body out of
the line body's arcs (a flank slot for it, or the Stouts holding while Blitzes are ahead) is not built.

## The two scenarios (Comet Catcher Remake 1.8, the recorded places, `--reps 8 --parallel 2`)

Side 0 is ours; the lane drives it. The units stand where the record left them, so the spacing at contact is the
record's (42-43) unless the lane moves them.

| Scenario, arm | Margin | W-L | Killed per lost | Shots per in-reach s | Muzzled (line / edge / clear s) | Nearest friend | Friend on the line |
|---|---|---|---|---|---|---|---|
| 2v1b-hard 15:10 (33 against 12, won live), off | +0.805 +- 0.005 | 8-0 | 1.79 | 0.68 | 2% (29 / 4 / 14 of 2,013) | 42 | 24% |
| old | +0.758 +- 0.009 | 8-0 | 1.45 | 0.46 | 16% (52 / 39 / 311 of 2,536) | 42 | 21% |
| on | +0.755 +- 0.012 | 8-0 | 1.45 | 0.42 | 19% (106 / 42 / 373 of 2,680) | 42 | 19% |
| on, kite margin 80 (form9-scen1510-on) | **+0.803 +- 0.008** | 8-0 | 1.77 | 0.64 | 3% (15 / 10 / 37 of 2,156) | 42 | **13%** |
| bank-1 27:34 (25 against 18 with two Fatboys at 700, lost live), off | -0.619 +- 0.019 | 0-8 | 0.68 | 0.41 | 29% (229 / 89 / 411 of 2,494) | 43 | - |
| old | -0.620 +- 0.029 | 0-8 | 0.68 | 0.39 | 32% (247 / 50 / 435 of 2,272) | 42 | - |
| on | -0.750 +- 0.012 | 0-8 | 0.45 | 0.25 | 49% (232 / 91 / 469 of 1,607) | 64 | 36% |

The won fight stays won in every arm (8-0), but both lane arms fired less (0.42-0.46 shots per in-reach second
against 0.68) and kept less (0.76 against 0.81): a debug repetition put 33 of 42 muzzled seconds under H-MICRO-KITE,
Stouts (reach 350) kiting Warriors (325) on the rule's 20-elmo margin, stepping back every reload instead of
firing. With the margin at 80 the fight is fought as the plain order fights it (+0.803, 1.77 killed per lost, 0.64
shots a second, 3% muzzled) with the friend-on-the-line share at contact down from 24% to 13%. The out-ranged fight is lost in every arm, and the
formation loses it worse: spaced and standing at 0.92 of a Stout's reach, our units are shelled from 700 while the
plain ball at least charges in and trades 0.68 (the design's open case; a rule for an out-ranged body is not built).

## The live games

`target/release/arena --matches 1 --parallel 1 --speed 50 --map "Comet Catcher Remake 1.8" --corner nw --mirror
--side armada --profile hard_aggressive --max-minutes 25 --label form-1` (and form-2 with the final binary): the
heuristic bot with no LLM. Both were lost early, called by the referee at 10.8 and 14.6 game minutes, before the bot
had an army: form-1 88 soldier-seconds with an enemy in reach (Rocketeers and Warriors), 2 deaths, 240 metal lost
for 258 killed; form-2 228 soldier-seconds, 12 deaths, 2,314 lost for 1,264 killed (exchange 1.83), 49 lane claims,
0 deaths to turrets or the commander. There were no fights to read the formation in: the heuristic bot against
hard_aggressive on this map dies to Pawn and Flea raids at minute 10-15, whatever its footwork. Against bank-1 and
the 2v1b games (25-33 minutes of the Opus player with Jev, 3,385-5,558 Stout in-reach seconds) these games are not
comparable, and the muzzled shares (2-3% of 88-228 seconds) say nothing. Four medium games were run for fights:
form-3-medium (the lane with the formation, before the kite margin) ran to the 25-minute timeout undecided with
2,902 soldier-seconds in reach, exchange 1.10 (10,144 lost for 9,183 killed), 12% muzzled (Rocketeers 173 of 1,514 s,
Hammers 139 of 736, Warriors 20 of 518; causes clear 56%, edge 25%, a friend on the line 19%), 71 soldiers lost, 5
of them to turrets or the commander, 535 lane claims at 1.17 reversals each; form-4-medium (the final lane),
form-5-medium-noform (`--disable H-MICRO-FORM`) and form-6-medium-nolane (`--disable H-MICRO-LANE`) were all lost
at 11-12 minutes with 225, 138 and 2 soldier-seconds in reach. One game in four reached a fight; the heuristic bot on
this map is decided by its opening and its outposts, not its footwork, and a live reading of the formation needs
the player games (the pianist with Jev) that the brief left out. The step-speed table in form-3: Pawns after a
lane step at a median of 0 elmos/s (n = 96) against 86 on a fight order, Warriors 26 against 44: after the fix the
short Moves left are the formation's own slot orders to units already at their slot (a Move to where you stand),
not flee steps; the flee's steps now go 250 and were too few to measure (46-49 claims a game).

## Part C: why the players switched the lane off (from the records, not the notes)

The `lane` calls and their clocks come from `strategist-0.jsonl`; what the lane did comes from the record's `cmd`
rows (a Move within 130 of the unit is a lane step), its `rules` decision rows, the `lane` milling field, and the
group tasks per second in `jev-0.jsonl`. The scratch scripts are not kept; the tables they printed are summarised.

| Game, clock, setting | The player's reading | What the record shows | Verdict |
|---|---|---|---|
| 2v1-hard_aggressive 7:13, flee and kite off for all | "flee footwork pulled groups out of Pawn fights" | In the minute before, groups left fights by the hands' own `move_to->home` (five fall-back tasks, Jev's rule for an outweighed group); the lane fled 14 times, on single units and on group D, which stayed in its fight at 21-96 elmos from the Pawns. Two D units at 7:04-7:12 were re-stepped every 6-9 frames to points zigzagging within 50 elmos | Mostly the hands' fall_back. The lane's part: the chooser's zigzag (fixed: a cell within 1.2 of the least threat is kept) and the lethal flee judged by damage a second (fixed: strength, and never from inside the guns) |
| 2v1-hard_aggressive 11:44, follow off for all | "suspect it re-sends groups after fleeing parties" | Right. Group F (11) followed a party 2,000 elmos east 9:21-9:46, a Fight at the party's new centre every 2 s with the party 300-600 ahead; K (4) followed one 3,500 elmos 11:04-11:32 and died to Warriors 11:33-11:44 | The lane's: no leash. Fixed: a group drawn 900 from where it engaged holds and says so (K-hands-follow-had-no-leash) |
| 2v1b-hard 17:23, small groups to raw | "so they join instead of stepping back" | Groups of 1-3 (S, T, U, V, X) stepped back by the hands' fall_back (a party always outweighs a pair); the lane then fled them on their walk home, at half speed: after a lane step Stouts moved 28-30 elmos/s, Blitzes 25-40, against 48-55 after a far move and 67-100 after a fight (three games, n = 62-550 a cell) | The stepping back was Jev's. The lane's part: the braking step (fixed: the order goes 2.5x further than the cell, re-issued after 64 not 32; K-engine-a-short-move-order-brakes-the-unit) |
| 2v1b-hard_aggressive 5:51, A and D raw | "groups only moved after lane raw" | The lane fired no rule and gave no order to A or D in the two minutes before; they held because Jev chose hold; the same turn's `engage` orders moved them at 5:52 | Misreading |
| bank-1 8:53, I and J raw | (none) | J's six Blitzes at 44 elmos from Pawns were fled twelve times a second and taken back by focus the next tick; J never left its fight | The lane's: chatter between a fresh flee claim and focus (fixed: a claim stands its first second; focus retired) |
| bank-1 14:13, L and Q raw | (none) | Q walked home under flee steps at 39 elmos/s (the braking step). L, holding (standing order Stop), got `Attack` then `Stop` from focus every second, the Stop cancelling the Attack in the same batch | The lane's (both fixed; focus retired) |
| 2v1b-hard 10:09-10:29, 12:16, 18:26-18:36, 23:59; 2v1b-hard_aggressive 14:13; bank-1 3:46, 9:05, 20:46, 28:36 | various | Where the lane was active (2v1b-hard 12:16: 107 flee firings, 276 steps in a minute in which the hands flipped H between fall back and advance nine times), its part was the braking step under the hands' own reversals; at 5:51-type shut-offs (3:46, 9:05, 14:13) the lane had done nothing to the group in the minute before | Precautionary or misread; the braking step where active |

The milling instrument's own numbers were small (path over net 1.0-1.1 in most samples): the lane's real costs were
the braking step, the chatter and the follow, not milling in circles.

## What it says

- **The formation does what the survey asked of it.** Spacing at contact goes from ours (40-43) to the pros' (57-64
  against their 67-70), the friend-on-the-line share from 24-31% to 13-23% (the pros' 22%), in every pairing and in
  the won scenario, without losing shots (0.83 per in-reach second for Stouts in both arms, 6.5 against 6.6 for
  Blitzes).
- **It wins where a ball loses its shots to its own hulls: the Blitz pack.** +0.32 against +0.15 / +0.21 over three
  batches of the formation (+0.28, +0.31, +0.32) and two of the plain order; 1.45 metal killed per metal lost
  against 1.16-1.24; friendly fire 7.8% against 9-10.7%. In the Stout mirror and the Stout body against the bot mix
  it neither wins nor loses (within a sigma), and in the won scenario it keeps the fight as it was.
- **It does not solve the user's case**, the Stouts' shells on the Blitzes in front: separate bodies do not keep the
  faster raiders out of the line's arcs, and the mixed force trades the same (-0.16 against -0.18) with more
  friendly fire (6.0% against 4.7%). And against artillery that out-ranges the body (bank-1 27:34) standing spaced
  at reach is worse than the charging ball (-0.75 against -0.62).
- **The old lane was costing fights, and the harness says exactly where.** Focus cost the Blitz pack 0.21 of margin;
  the lethal flee as written fled won fights at contact (7-10 Blitzes at once); a 100-elmo step re-issued every 32
  held units in the engine's braking zone at half speed; the kite's 20-elmo margin had Stouts stepping back from
  Warriors every reload (0.46 shots a second against 0.68). Every order costs shots: a stand is now no order at all.
- **Method.** Two batches of the same `off` arm differ by 0.06 with standard errors of 0.05: read the between-batch
  spread as the noise. A null arm (the lane present, every rule off) is the check that the harness measures the
  rules and not the plumbing; it found the two-site contamination that voided the first day's lane arms.

## The rear behind a firing front (2026-09-25, later)

`docs/design/2026-09-25-queued-rear.md`; the instrument is `run/queued.py` and the harness's `engaged_s` /
`queued_s` / `blocked_s`. The records: queued (behind a firing friend that can reach, the unit cannot) 25% / 15% /
10% / 14% of engaged soldier-seconds in bank-1, 2v1b-hard, 2v1b-hard_aggressive, family-1 (geometric 37 / 24 / 23 /
27%; the pros 20%); blocked (the friend within 40 of the unit's line to its enemy) 14 / 10 / 4 / 9% (geometric 22 /
16 / 11 / 17%; the pros 8%); nearly all of it units walking up in their first ten seconds of engagement, a
standing jam 1% or less. Two clash cuts (bank-1 13:50: 45 against 15; family-1 11:05: 37 against 13), 8 reps an
arm, the lane on ours:

| Cut, arm | Margin | Killed per lost | Friendly fire | Queued | Blocked | Nearest friend | Friend on line |
|---|---|---|---|---|---|---|---|
| bank-1 13:50, plain order | +0.693 +- 0.016 | 0.67 | 40.3% | 13% | 9% | 31 | 6% |
| null (lane present, rules off) | +0.706 +- 0.012 | 0.71 | 39.9% | 14% | 9% | 31 | 3% |
| formation, no flank | +0.804 +- 0.007 | 1.05 | 17.3% | 16% | 8% | 36 | 11% |
| **formation with H-MICRO-FORM-FLANK** | **+0.840 +- 0.006** | **1.30** | **4.4%** | 21% | 10% | 38 | 5% |
| family-1 11:05, plain order | +0.658 +- 0.016 | 1.17 | 18.2% | 11% | 8% | 26 | 77% |
| formation, no flank | +0.578 +- 0.022 | 0.95 | 21.6% | 7% | 6% | 29 | 77% |
| **with the flank** | **+0.713 +- 0.011** | **1.40** | **3.2%** | 17% | 10% | 32 | 62% |

The other pairings with the flank against the plain order: Blitz pack +0.287 +- 0.035 (+0.154), Stout mirror
+0.023 +- 0.036 (+0.034), the mixed force -0.072 +- 0.066 (-0.182 +- 0.056; the Stouts' shells on the Blitzes 3,522 hp
against 4,291), Stouts against the bot mix +0.132 +- 0.042 (-0.001 +- 0.050). Ranks of twelve on the march instead:
bank-1 +0.799, family-1 -0.050; dropped. The live medium game form-7 (the flank on): 2,146 engaged soldier-seconds,
queued 19%, blocked 6%, exchange 1.34, one game.

What it says: the blocked share does not move in the harness (6-10% in every arm, the pros' 8%), because a body
reaches the enemy within seconds whatever the rule; the front stopping costs the rear its line of fire, not its
reach, and the rear's shells on the front were 40% of the damage done in the bank-1 cut. Filing the rear to the
ends of the front's line takes that to 4% and the fight from +0.69 to +0.84. The raider body's flank slot (the
Stouts on the Blitzes) is the same rule: the Stouts and Janus put 33,500 hp into the Blitzes in the plain cut and
2,300 with the flank.

## The arena A/B (2026-09-25, lane-ab)

24 games an arm, the heuristic bot with no LLM, Armada mirrored against BARb medium on Comet Catcher Remake 1.8,
corners alternating, the same seeds in every arm (`--seed-base 1`), 25 minutes, speed 50, six parallel. `new` is
the tree at 3c83cde (H-MICRO-FORM and -FLANK, the flee, kite and follow fixes, focus gone); `old` is the bot built
at 44dfd19 (the lane as it was on 2026-09-24) run through `--bot`, the shim and protocol being unchanged between
them; `off` is `--disable H-MICRO-LANE`. Two of the old arm's matches aborted on a port collision and were replayed
alone on their seeds and corners. `run/micro_ledger.py`, `run/fire.py`, `run/queued.py` and `run/replays/shapes.py`
over every record (both corners).

| | new | old | off | the pros |
|---|---|---|---|---|
| W-L-timeout | 1-13-10 | 0-13-11 | 0-14-10 | |
| metal lost per metal killed by minute 25 [bootstrap 95%] | **0.79 [0.65-0.95]** | 0.84 [0.73-0.93] | 0.94 [0.83-1.05] | |
| the same by minute 10 | 0.67 [0.40-1.09] | 0.62 [0.37-0.97] | 0.96 [0.68-1.43] | |
| soldiers lost a game: to turrets or the commander / elsewhere | 6.2 / 36.2 | 5.4 / 37.5 | 5.2 / 45.1 | |
| soldier-seconds under fire a game | 1,741 | 1,731 | 1,655 | |
| in-reach seconds; shots per in-reach s | 51,475; 1.27 | 51,404; 1.23 | 55,688; 1.31 | |
| muzzled (by a friend on the line) | 6.4% (737 s) | 6.1% (1,399) | 11.5% (3,143) | |
| friendly fire | 3.6% | 4.6% | 4.6% | |
| engaged s; queued; blocked | 61,514; 19%; 6% | 48,238; 17%; 7% | 56,272; 14%; 6% | 20%; 8% |
| fight sides; nearest friend at contact | 402; **51** | 368; 40 | 429; 40 | 67-70 |
| friend on the line at contact | **23%** of 1,900 | 32% of 1,676 | 33% of 2,038 | 22% |
| body at contact, median | 7 | 7 | 6 | 6 |

The noise floor: per game the metal lost has a standard deviation of 5,000-6,500 on a mean of 6,000, so 24 games
resolve about 0.15 of the lost-per-killed ratio; the new and off intervals just touch, new and old overlap. Wins
are one in 72: the heuristic bot does not beat medium on this map, whatever the lane does, and a claim about wins
would need the player games. What moved, and in the direction the harness said: the trade (0.79 against 0.94), the
deaths in unit fights (36 against 45), the muzzling by friends (737 against 3,143 s), the spacing and the line of
fire at contact (the pros' numbers, reached live for the first time). What did not: deaths to turrets or the
commander and the time under fire, the flee's own measures (K-micro-a-tick-is-a-tower-margin, amended); the
blocked share (6-7% in every arm, the pros' 8%). Three quarters of the friendly fire in every arm is the
Rocketeers' rockets landing on the Hammers and Warriors in front of them, which no rule here addresses.

## The flank alone (2026-09-25, flank-ab), and the Rocketeers' friendly fire

`--ab-disable H-MICRO-FORM-FLANK`, 48 matches interleaved in blocks of four, the settings of lane-ab, every seed
and corner in both arms.

| | with the flank (A) | without (B) |
|---|---|---|
| W-L-timeout | 0-17-7 | 0-16-8 |
| metal lost per killed by minute 25 [bootstrap 95%] | **0.79 [0.68-0.91]** | 0.87 [0.74-1.04] |
| by minute 10 | 0.57 [0.34-0.88] | 0.52 [0.35-0.71] |
| lost / killed a game | 4,896 / 6,232 | 4,073 / 4,684 |
| deaths a game: turrets+commander / elsewhere | 4.9 / 29.5 | 2.9 / 25.1 |
| muzzled (by a friend on the line, s) | 6.1% (736) | 7.8% (1,336) |
| friendly fire | 4.0% | 4.3% |
| queued / blocked | 18% / 6% | 15% / 6% |
| nearest friend at contact (fight sides) | 55 (308) | 54 (304) |
| friend on the line at contact | **26%** of 1,528 | 31% of 1,485 |

Noise: per-game sd 4,000-8,000 on means of 4,000-6,000; 24 games resolve about 0.15 of ratio and the intervals
overlap. What the rule alone shows in the arena is the line of fire at contact (26% against 31%, the pros' 22%)
and the muzzling by friends (halved); the trade moved its way within the noise; wins none in 48.

**The Rocketeers' friendly fire** (K-form-rocketeers-hit-the-friends-in-front): three quarters of all friendly fire
in every lane-ab arm is Rocketeer rockets on Hammers, Warriors and Rocketeers. At the 258-324 friendly-fire
unit-seconds an arm the Rocketeer's nearest enemy is at 404-431 (its reach 475; the front's 325-380), a friend
stands within 40 of its line in 563-1,035 friend-events and within 60 of the enemy in 3-13: the rocket meets the
front on its path; it is not splash. The cut lane-ab-new 05 21:10 (24 Rocketeers, 33 Hammers, 17 Warriors against
21) replays it: friendly fire 8.7% of damage done with the lane off, 9.8% on, the same victims both ways, the fight
won 8-0 either way (+0.76 / +0.74). No fix built; the shape says the long-reach body needs its own line of fire
past the line body (a slot beside, not behind), or the mix fewer Rocketeers behind Hammers.
