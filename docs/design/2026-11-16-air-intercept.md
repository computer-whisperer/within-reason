# Air interception: his bombers stopped before they drop

Branch `air-intercept` (worktree `within-reason.air-intercept`). Begun 2026-11-16 from the user's review of the
lobby 3v3 on Tangerine (`run/matches/1794780805-human` in the main tree, lost about 35:00).

## 1. The ask, in his words

2026-11-16: "Probably the biggest mistake I see us make in that game is the damage we take from thebluegecko's
bombing run at 19:09 and later at 21:07. In discussing the matter with him later, the main point is that bombing runs
like that have to be intercepted with fighters before they reach the base -- the static defences just engage too late
to stop the actual bomb drop events. Review what happened that game, as well as how we control air units in one or
more seats, and let's figure out how to improve the bot."

After the review (section 2), the same day: "Agreed on the patrol being something the player orders with the engine
choosing targets. This is an acute timing-sensitive vulnerability, so we can treat this like rove for now in terms of
having the player hand the necessary resources over to the micro engine for control. Reactor spacing is not something
to focus on now, since this was entirely defendable if we had the right control setup. Proceed with the drill runs to
see what kind of micro behavior we need to be ready. One contribution I would have there is to consider the added
benefit of if the player had built a t2 radar in a practical spot."

So, ruled:
- **The player orders the patrol; the engine chooses its targets.** No entry in `docs/code-decisions.md` for that.
- **Fighters may be handed to the micro engine as a roving group is** (H-MICRO-ROVE's shape: the player declares the
  group and what it is for, the lane runs its members). "For now": his word, to be looked at again.
- **Reactor spacing is not this arc's.** The blasts of our own reactors did most of the damage in both runs (below);
  he rules the runs "entirely defendable if we had the right control setup".
- **Drills before any build,** and a tier-2 radar "in a practical spot" among their arms.

## 2. What the game showed (the tape, format 6, and the seats' transcripts)

| | Run 1 | Run 2 |
|---|---|---|
| Bombers (thebluegecko, Hailstorms, 310 metal, 1,520 health, 248 a second) | 5 | 7 |
| Way | his plant 18:14, the west edge, the south edge | 19:54, the north edge, 16 s waiting in the north-east corner, the east edge |
| Our first contact | in sight 18:53, 3,050 from the block | on radar 20:52 (two of seven), all 20:56, in sight 20:58 |
| First building dead | 19:09 (15 s) | 21:09 (17 s) |
| Our loss within 1,000 of the block | 14,540 metal | 43,807 metal |
| of it named to the bombs | 4,390 | 1,045 (the first fusion, 3,350, unattributed) |
| His loss | all 5, after the drop | all 7, six after the drop |
| Anti-air's damage before the first building died | 1,850 of 7,600 | 8,400 of 10,640 |

- A Condor radar plane crossed the block about twenty seconds before each run and was shot down.
- The block: three fusions 96 to 128 apart (a fusion's blast reaches 240), and in run 2 an advanced fusion (blast 640,
  10,600) among the labs and the anti-air built after run 1: a fusion's blast finished it, and its blast took the rest.
- **No fighter of ours in the whole game.** Seat t3 built an aircraft plant at 12:55 for bombers ("armfig fighters only
  once his aircraft have been seen"), lost 15 of 17 bombers over his base at 15:31, wrote "he has tier-2 air
  superiority. Switching to home anti-air", and took the plant apart at 16:27 for an advanced bot lab. The bombed
  seat (t5) wrote nothing of aircraft before run 1 and built turrets and Archangels inside the block after it. His
  eleven Nighthawks flew a patrol over his own half all game and never came with the bombers.
- Our radars: three of tier 1 (2,100), all in the middle of our ground; he flew the edges.

How the hands fly aircraft today (read 2026-11-16):
- An air group told to hold gets hold position and a move to its centre (`pianist/groups.rs` `hold_orders`, the
  domains design's decision 4, made so that held bombers do not fly off): the engine's own search is then off. A
  fighter that is idle or on a patrol or fight leg attacks the nearest enemy aircraft in sight or on radar within
  1,000 elmos times its move state (1 manoeuvre, 2 roam), and none on hold position (`CAirCAI`, engine 2026.07.04).
- No patrol in the protocol (until this branch).
- An attack on a party is one `Attack` at its dearest unit for every member, re-picked when it dies.
- A party is `air` only when every member is identified: radar contacts read "unidentified, speed unknown", though a
  contact's place, height and velocity are in the snapshot (`EnemyUnit::pos`, `vel`).
- The pass runs each second and Jev answers in about half a second: the bombers of run 2 were on `group_aa_t5`'s
  menu from 20:52.0 with a ground fight's words ("about 60 s of walking"), and the picks were "walk to F7" four times.
- WRDistilled sends its fighters when his aircraft are within 1,500 of a building of ours (`AIR_DEFEND`): six seconds.

## 3. The drills

A run is cut from the tape and flown again in the duel bench (`docs/harness/duels.md`, "Air drills"):
`run/air_scene.py` writes the scenario (ours as it stood: everything within 1,200 of the target, the anti-air within
3,000, the radars, everything within 1,200 of the bombers' way; his bombers where they were) and the script (his own
orders to them from the tape, twelve seconds late so that our fighters are up); `--fighters`, `--add` and `--arm`
add the defence under test. `run/air_drill.py` reads the batches.

What the drill is not: the units are given whole (no damaged building, nothing under construction: run 1's
advanced fusion at 81 % and run 2's fourth Chainsaw at 44 % are left out), our mobile anti-air stands where it stood,
nobody of ours gives an order but the fighters, and the game is test-31836 where the lobby's was its own version.

## 4. What the drills measured (2026-11-16, commit 46f77307, eight repetitions an arm unless said)

"Saved" is a repetition that kept every reactor: the outcome is a cliff, since one fusion lost beside the advanced
fusion takes the block (run 2: 0 to 4,000 metal lost, or 39,800). The fighters stand or loiter by the block; the
warning is the time from our first contact to the first harm.

**The baselines are the game** (`air-A-run1-none`, `air-A-run2-none`): run 1 saved 0 of 8, 13,284 lost, 17 s of
warning from 3,530 out; run 2 saved 1 of 8, 35,231 lost, 17 s from 3,180 out.

**The control, twelve Falcons (876 metal), the sensors as they stood** (`air-A-*`):

| The fighters | Run 1 saved | first hit after contact | Run 2 saved | first hit |
|---|---|---|---|---|
| none | 0 of 8 | | 1 of 8 | |
| landed on hold position, all attack one bomber 1.5 s after contact (the hands today) | 0 | 8.0 s | 1 | 10.9 s |
| landed on roam, no order (the engine's search, 2,000) | 2 | 11.0 | 0 (11 of the 12 died on the ground in the block's blast) | 14.7 |
| the engine's patrol on roam, 900 round the block | 1 | 6.1 | 1 | 12.1 |
| landed, a fight order at the contact 0.5 s after it | 0 | 7.1 | 4 | 10.0 |
| landed, each its own bomber 0.5 s after contact | 0 | 7.0 | 1 | 10.1 |
| up on the patrol, a fight order at the contact 0.5 s after it | 5 | 6.0 | 4 | 8.6 |
| up on the patrol, each its own bomber 0.5 s after contact | 8 (nothing lost) | 5.8 | 2 | 9.0 |

The same two scrambles 1.5 s after contact instead of 0.5 (the hands' pace against the lane's, `air-B-*-spread15`,
`-pspread15`): run 1 up 8 of 8, landed 1; run 2 up 2, landed 0. A second's delay costs less than being on the ground.

**The fighters' number and tier, up and scrambled, the sensors as they stood** (`air-C-*`; fight order / own bomber):

| | Run 1 saved | Run 2 saved |
|---|---|---|
| 8 Falcons (584 metal) | 5 / 6 | 6 / 1 |
| 12 Falcons (876) | 5 / 8 | 4 / 2 |
| 16 Falcons (1,168) | 7 / 8 | 6 / 2 |
| 24 Falcons (1,752) | 8 / 8 | 6 / 8 |
| 8 Highwinds (1,200, tier 2) | 8 / 8 | 8 / 8 (86 and 34 metal lost) |

**The engine's own patrol with no scramble of ours** (`air-D-*`): 24 Falcons run 1 7 of 8, run 2 3; 8 Highwinds run 1
1 (five of them died in the fusions' blast), run 2 6; on a loop 1,500 out twelve Falcons saved 0 and 0 and eight
Highwinds 1 and 0 (in run 2 neither fired a shot: the search is 2,000 from the leg ahead of the fighter, and the
fighter was on the far side). Eight Highwinds held as the hands hold today: 2 and 5.

**An advanced radar (3,500), twelve Falcons** (`air-B-*`, `air-E-*`):

| The radar added | Run 1: contact from | saved (up, own bomber) | Run 2: contact from | saved (up, own bomber) |
|---|---|---|---|---|
| none | 3,530 (17 s) | 8 of 8 | 3,180 (17 s) | 2 of 8 |
| at the block | 3,540 (17 s) | 8 | 3,200 (17 s) | 3 |
| on the flank he came by | beside t4's radar: 3,560, nothing gained; by the south-west coast gun (2660, 6900): 4,720 (24 s, and the drill counts no contact sooner) | 1 | by the east shore (6400, 5300): 4,660 (25 to 27 s) | 8 (32 metal lost); landed and scrambled 8 and 7; the engine's patrol alone 4 |

- At the block a tier-2 radar sees no further along either way than what we had: the east naval radar and the
  sight of our western units reached as far.
- On the flank it is seven to nine seconds more, **if it has a view**: the site beside t4's own radar stood 2,700 to
  3,500 from the bombers the whole way and never had them on radar, and a third site (3760, 5800) had them in one
  repetition of two. Radar is blocked by ground; the site is the decision.
- **In run 1 the earlier contact made the scramble worse** (1 of 8 where the later contact gave 8). The trace: the
  fighters flew out, met the bombers 2,500 from the block twelve seconds after contact, each fired its two missiles in
  the pass, turned wide, came out 1,100 to 1,400 behind, and gained 40 to 80 a second: their next shots fell as
  the bombs did, twelve seconds later. With the later contact the pass was at 2,200 and the turn brought them back
  in reach in five seconds.

**The mechanism under all of it.** A Falcon's missile is 125 to aircraft every 0.9 s at 680; a head-on pass at a
closing 540 a second is two missiles, about 260 a fighter. A Hailstorm has 1,520: six Falcons to a bomber for a kill
in the pass, so twelve Falcons kill two of seven, and then turn for five to eleven seconds. A Highwind's missile is
750 every 1.4 s: one Highwind to a bomber in the pass, which is why eight of them stop seven bombers every time
wherever they start. What decides a run is the salvo weight that reaches the bombers before they let go: the
fighters' type and number, and how many passes the control gets out of the warning.

**Not measured, or not clean:** a start from astern (`air-F-*`: the fighters given beside the bombers shot them on
the ground; what it did show is that fighters trailing a bomber gain 40 a second and only those within 680 fire);
his fighters with the bombers (they were never with them in the game); mixed anti-air and fighters where the block's
own anti-air is thinner (run 1's block had five Nettles and a few Crossbows, run 2's three Chainsaws and two
Arbalests); bombers at their full speed at an early contact (in the drill they are still gaining speed 4,700 out,
which flatters the early-radar arms); Hailstorms flown apart instead of abreast. The one bomber seen at its start
(`contact_after`, `docs/harness/duels.md`) is unexplained.

## 5. What the micro engine needs (proposed, not built)

An **intercept group**, declared by the player as a roving group is (his ruling, section 1): the side's fighters, a
place to keep, and the word that the lane runs them.

1. **Up, by the place it keeps.** A loiter of a few hundred elmos, not a landing (a landed wing answered three
   to four seconds later and died in the block's blast) and not a wide patrol (the 1,500 loop never engaged).
2. **Scrambled at the first contact that flies**, in sight or on radar: a contact's height and velocity are in the
   snapshot, so a blip at 248 a second above the ground is an aircraft before its type is known. The engine's own
   search (1,000 or 2,000) threw away the first five seconds of seventeen.
3. **A merge point chosen for the second pass,** not the earliest one: near enough to the place that the turn
   brings the fighters back in reach before the drop. This is the control law to find in the bench; the scripts could
   only fly at the contact.
4. **Targets dealt by salvo weight:** to each bomber the fighters that kill it in the pass (six Falcons, one
   Highwind), the nearest bombers to the place first; none on a bomber already dead or already covered.
5. **Back to the loiter** when nothing that flies is left on contact, and never after it over his ground.

Beside the lane, for the player and the hands: the patrol as the player's order for a wing the lane does not run
(`Command::Patrol` is in the protocol on this branch); the hold of a fighter group must not be hold position; the
words for an air party; and in the texts, that tier-2 fighters are the answer to tier-2 bombers (eight Highwinds
against twenty-four Falcons), that a tier-2 radar earns its seconds only on the flank with a view, and that fighters
over our own block do not meet his.
