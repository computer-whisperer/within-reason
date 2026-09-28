# The engagement plan on recorded moments (2026-09-28)

Validation of `docs/design/2026-09-28-engagement-plan.md` before any game: the bot's own code (`bot --plan-replay`,
`crates/bot/src/brain/pianist/replay.rs`, wrapped by `run/plan_replay.py`) run on the main tree's records (read-only):
the brain rebuilt from the record's header and terrain, his buildings remembered as `track_enemy_buildings` keeps
them, our units and his in sight at the clock's sample, the groups, places, party names, the packet and the standing
orders from the Jev log's last call at or before the clock. Jev `jev-1.13.0` (the `jev-latest` alias), one ask per row
and arm; 28 calls in all (two of them an earlier cut of the candidates, below). The candidates as of commit "step 3"
(`engagement.rs` `candidates`).

## The named moments

None of the design's clocks had a position in reach; each was scanned forward a second at a time (`--dump`, no call)
to the first second the gate plans the fight's body.

| Moment (design) | What the replay says at the clock | First planned second, and why later |
|---|---|---|
| player-14 9:05, group_C before E3 | not planned: nearest position (three turrets elsewhere) 1,443 from its front | 9:16: E3's first Centurion came into sight at 9:15, the near turret at 9:16 (the far one remembered); nothing of E3 was known at 9:05 |
| player-15 9:20, before F3 | not planned: group_B 1,265 from two remembered turrets | 9:24 (party_11 joins the turrets, 165 from the front); 9:27 with party_15 (a Rocketeer, a Centurion) and a third turret |
| player-16 11:05, the F2 push | not planned: group_E 4,922 from the nearest position | 12:27 (group_E, 21 Blitzes and 2 Stouts advancing to his lab, two Sentries at F1-F2); 12:51 with a Blitz screening two Sentries at spot_6 |
| player-16 16:00, the F2 push | not planned: group_B 1,815 from the nearest position | 16:10 (group_B, 18 Blitzes and 8 Stouts, four turrets at his lab: an Overwatch reach 620 and three Sentries); group_A at 16:07 is a scout body |
| player-17 4:50, the Rovers | not planned: no position known | 4:53 on: not planned, a scout body (14 Rovers and a Blitz at 4:53, 7 and the Blitz at 5:03; the scout check by every armed member missed both, now scouts over half the armed metal) |

## The rankings

Probabilities from the Choice's answer, highest first (the taken plan: top when above 0.4 and above the decline's).
"TAS verdict": the screen first from its stand-off above the statics first and the nearest first (the E3 study:
24/24, 5/24, 21/24); "F3 test": hold-and-shell or decline above the walk in.

| Moment | Body / position | Candidates | With the examples | Without | Against the verdict |
|---|---|---|---|---|---|
| player-14 9:16 group_C | 15 Blitzes (strung out 807) / 2 Centurions (a screen under the near Sentry), 2 Sentries | screen_first, statics_first, nearest_first, gather_first, decline | screen_first 0.79, gather_first 0.14, decline 0.04, statics_first 0.01, nearest_first 0.01 | screen_first 0.76, gather_first 0.15, statics_first 0.03, nearest_first 0.03, decline 0.03 | met: screen first (and gather-then-screen) above both walk-ins |
| player-14 9:17 group_C (the TAS clock) | 14 Blitzes / the same (the third Centurion in sight from 9:18) | as 9:16 | screen_first 0.80, gather_first 0.13, decline 0.04, nearest_first 0.02, statics_first 0.01 | screen_first 0.81, gather_first 0.11, statics_first 0.03, nearest_first 0.03, decline 0.02 | met |
| player-15 9:24 group_B | 14 Blitzes, 7 Rovers, 5 Stouts (tail 4,120 behind) / 4 constructors and a Pawn, 2 remembered Sentries | screen_first, statics_first, nearest_first, gather_first, decline | screen_first 0.57, gather_first 0.38, nearest_first 0.03, statics_first 0.01, decline 0.01 | screen_first 0.61, gather_first 0.32, nearest_first 0.05, statics_first 0.01, decline 0.01 | F3 test not met: no shell candidate (no artillery in the body), decline last; the four Rocketeers of the design's F3 scene were not in sight (one Rocketeer at 9:27) and every phase read "we outweigh it" |
| player-15 9:27 group_B | the same / a screen of 4 constructors and 2 Pawns, a Centurion-Rocketeer party, 3 Sentries | as 9:24 | screen_first 0.73, gather_first 0.23, decline 0.03, nearest_first 0.01, statics_first 0.00 | screen_first 0.78, gather_first 0.19, statics_first 0.01, nearest_first 0.01, decline 0.01 | F3 test not met, as 9:24 |
| player-16 12:27 group_E | 21 Blitzes, 2 Stouts (tail 1,366) / 2 Sentries 430 apart, neither covering the other | one_at_a_time, statics_first, nearest_first, decline | one_at_a_time 0.91, nearest_first 0.04, statics_first 0.03, decline 0.02 | one_at_a_time 0.88, nearest_first 0.06, statics_first 0.04, decline 0.02 | no TAS verdict here; the stand-off plan ranks first |
| player-16 12:51 group_E | 14 Blitzes / a Blitz screening 2 Sentries (the screen has no stand-off) | one_at_a_time, statics_first, nearest_first, decline | one_at_a_time 0.79, decline 0.11, nearest_first 0.07, statics_first 0.03 | one_at_a_time 0.66, decline 0.17, nearest_first 0.12, statics_first 0.05 | as 12:27 |
| player-16 16:10 group_B | 18 Blitzes, 8 Stouts / an Overwatch (reach 620) and 3 Sentries at his lab | one_at_a_time, statics_first, nearest_first, decline | one_at_a_time 0.59, decline 0.21, nearest_first 0.12, statics_first 0.08 | one_at_a_time 0.54, statics_first 0.19, nearest_first 0.16, decline 0.11 | the decline second with the examples (0.21), fourth without (0.11): the Overwatch out-ranges the body and the odds read "we outweigh it (1.8)" |

Before `one_at_a_time` existed (the first cut: a position of statics alone had only the walk-ins and the decline),
the same player-16 moments ranked nearest_first 0.59 / 0.64 at 12:27, 0.63 / 0.63 at 12:51 and 0.52 / 0.61 at 16:10
(with / without the examples), the decline 0.06 / 0.04, 0.16 / 0.16 and 0.33 / 0.11. That cut's player-14 and
player-15 rankings match the table's within 0.06.

## What it says

- **The E3 verdict is met, and not by the examples.** At both E3 seconds screen_first takes 0.76-0.81 and the two
  walk-ins 0.01-0.03 each; without the examples block the numbers move by 0.01-0.03. Jev reads the geometry the
  candidates' words carry ("from its stand-off ... nothing else reaches us there" against "also in reach of us there:
  turret_8265") and the odds priced with that cover (1.4 walking in against 2.6 from the stand-off).
- **The examples block is worth nothing measurable on these seven moments.** Every top pick is the same with and
  without it; the largest shift is the decline at 16:10 (0.21 with, 0.11 without) and one_at_a_time at 12:51 (0.79
  against 0.66). One ask per arm: Jev's own variance between asks is not measured here.
- **The F3 test fails as posed.** No F3 moment offers a shell (no artillery in group_B) and the decline ranks last
  (0.01-0.03): the words price a body of 2,882 metal against 1,433 as "we outweigh it" in every phase. The out-ranging
  the design names (four Rocketeers at 475 under a turret) was not in sight at 9:24-9:27; what the reach words say
  of a Rocketeer party (its reach 475 beside our 180) did not move the ranking. Whether the plan declines F3 needs
  the out-ranging priced into the phase odds or said as a cost; the odds are `combat.rs`'s and do not see reach.
- **An out-ranging static is not declined either.** At 16:10 the Overwatch (620 against our 350 at most) put the
  decline at 0.21 / 0.11: the same fault as F3.
- **The gate, not the ranking, is where player-17 is answered**: the Rover body is a scout body and gets no plan.

Next (not done here): the one arena game the design names, read with `run/analyze_match.py --engagement`, the
`engagement` lines of the Jev log and `run/fire.py`.

## The re-ask gate after player-18 (2026-09-28 evening)

player-18 (`run/matches/1790560788-player-18-planner/00`, the main tree's first game with the planner) logged 72
`engagement` lines: 70 before 36:00, 47 of them declines, 20 flips of the taken plan between consecutive asks of one
group; group_H asked 8 times in minute 17 while its nearest position flipped between two turrets and two parties. The
gate now asks again only on a material change, after a hold, and not while the phase asked under runs
(H-HANDS-ENGAGEMENT-PLAN, amended).

**On player-18's log** (a scratch pass over the recorded asks, the gate's rule re-implemented in Python: parties' units
from the call line at the ask, deaths from the record's `enemy_destroyed`, the running phase from the call line's group
task; the recorded answer stands in for what each kept ask would have got, and asks the old gate never made cannot be
added): 24 of the 70 asks to 35:59 (26 of 72 to the end), 16 declines, 7 flips, at most 2 asks in any group-minute
(was 8). Reasons of the 26: 12 first asks, 7 an element dead, 6 a new party, 1 a new static, none stale.

**The rankings, re-asked after the change** (the replay does not go through the gate; the candidates are the same;
one ask per arm, with / without the examples):

| Moment | Before | After |
|---|---|---|
| player-14 9:16 | screen_first 0.79 / 0.76 | screen_first 0.80 / 0.79, gather_first 0.15 / 0.11, the walk-ins 0.01-0.03 |
| player-14 9:17 | screen_first 0.80 / 0.81 | screen_first 0.78 / 0.79, gather_first 0.15 / 0.11, the walk-ins 0.01-0.03 |
| player-15 9:24 | screen_first 0.57 / 0.61 | screen_first 0.55 / 0.59, gather_first 0.41 / 0.34, decline 0.01 |
| player-15 9:27 | screen_first 0.73 / 0.78 | screen_first 0.70 / 0.71, gather_first 0.27 / 0.25, decline 0.01-0.02 |

The order is the same at every moment; the largest shift is 0.07 (9:27 without the examples), within one ask's noise
for all we know (Jev's variance between asks is not measured).

