# Air interception: his bombers stopped before they drop

Branch `air-intercept` (worktree `within-reason.air-intercept`). Begun 2026-11-16 from the user's review of the
lobby 3v3 on Tangerine (`run/matches/1794780805-human` in the main tree, lost about 35:00).

**State (2026-11-17):** the wing is built in the micro engine and drilled (section 6; commits 67ba56b6 and
2dd830c4), and flown against raids the game did not show (section 6.4). The player's order is built as ruled, a move
of the hands as rove is: the air patrol (section 7). No bot of `bots/` declares a wing. Not merged.

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

## 5. What the micro engine needs (the proposal of 2026-11-16; section 6 is what was built from it)

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

## 6. The wing as built (2026-11-17)

The user, 2026-11-17: "Agreed, let's build the intercept group in the micro engine and drill against the two runs."

`crates/micro/src/intercept.rs`, H-MICRO-INTERCEPT: a declared group of kind `Intercept { place, keep }`. The law is
written out at the head of the module and in the heuristic's row; in short:

- **Loiter.** With nothing flying within the keep (4,500 by default) the fighters patrol a 700 loop round the
  place, each beginning at the next corner.
- **Scramble** at the first contact that flies in the keep: an aircraft by its type, a radar contact by its height
  and speed (the snapshot carries both: K-snapshot-a-radar-contact-carries-its-height-and-its-velocity).
- **The stern conversion.** The proposal's point 3 ("a merge point chosen for the second pass") was the wrong
  frame: there is no second pass to be had (K-engine-a-fighter-sent-at-a-plane-flying-at-it-passes-it-and-is-left-behind).
  What the fighter wants is to be behind the bombers, where the engine holds it and it fires every reload
  (K-engine-a-fighter-behind-its-target-is-held-there-and-fires-every-reload). So each fighter closes on a point
  beside the bombers' track and ahead of them, turns before they pass, and rolls out behind the rearmost. The turn
  is watched while it is flown: a fighter that would come out ahead of them (they slow and turn for their run just
  where the wing meets them) flies on, or across the track, until they have passed.
- **Targets** dealt from the rear of the body forward, each bomber covered by the fighters that kill it in two
  seconds and no more (six Falcons, two Highwinds), a fighter given only what is in front of it.
- **Back to the loop** when nothing that flies is left within 1.15 keeps of the place.

### 6.1 What it does against the two runs

Saved = no reactor lost. The scripted arms are section 4's (eight repetitions); the wing's cells are the batches
air-J-\* and air-K-\* together (commit 67ba56b6 and the same code before its clean-up).

| twelve Falcons | run 1 (five Hailstorms) | run 2 (seven Hailstorms) |
|---|---|---|
| none (the game) | 0 of 8, 13,284 lost | 1 of 8, 35,231 lost |
| held as the hands hold an air group | 0 of 8 | 1 of 8 |
| up, scrambled by script, a bomber each | 8 of 8 | 2 of 8 |
| **the wing, sensors as they stood** | **37 of 40**, 1,123 lost a run | **40 of 40**, 239 lost a run |
| up, scrambled by script, advanced radar on the flank | 1 of 8 | 8 of 8 |
| **the wing, advanced radar on the flank** | **40 of 40**, 42 lost a run | **40 of 40**, 99 lost a run |

Other wings, sensors as they stood (eight repetitions unless said; a cell of eight swings by two):

| wing | metal | run 1 | run 2 |
|---|---|---|---|
| 8 Falcons | 584 | 4 of 8 | 8 of 8 |
| 16 Falcons | 1,168 | 23 of 24 (5 of 8 in the first eight) | 8 of 8 |
| 24 Falcons | 1,752 | 7 of 8 | 8 of 8 |
| 4 Highwinds | 600 | 3 of 8 | 8 of 8 |
| 8 Highwinds | 1,200 | 8 of 8 | 8 of 8 |

In run 1 as the sensors stood the wing's three lost repetitions of 40 each lost about 13,000 (two, two and four
bombers dropped); in 23 of the 37 kept, one to three bombers still dropped something and no reactor went. "Saved" is
a threshold on a thin margin there: the count of bombers that drop (1.0 to 1.3 of 5 a run) is the steadier number.

No fighter was lost in 146 of the 160 twelve-Falcon repetitions. All the losses are in run 1 as the sensors stood
(1.5 a run, in 13 of its 37 kept repetitions too, so not the fusions' blast alone); what kills them was not traced
(a dying Hailstorm's blast and our own flak's bursts are the candidates).

### 6.2 How it was found (the instrument)

`run/air_drill.py --places` reads where the fighters were, second by second, from the merge to the bombs. A Falcon
behind the rearmost bomber and in reach lands 0.5 to 0.7 hits a second; anywhere else, next to none.

| twelve Falcons | in firing position | still inbound | ahead, flying their way | wide | far behind | hits a fighter-second |
|---|---|---|---|---|---|---|
| scripted scramble, run 2 | 12 % | 13 % | 0 % | 6 % | 69 % | 0.22 |
| scripted scramble, run 1 | 34 % | 16 % | 3 % | 40 % | 7 % | 0.48 |
| the wing's first builds (air-I7, air-I9, air-I10) | 20 to 43 % | 13 to 27 % | 21 to 60 % | 2 to 8 % | 1 to 18 % | 0.25 to 0.46 |
| the wing as committed (air-K), run 1 | 53 % | 26 % | 5 % | 13 % | 4 % | 0.38 |
| the wing as committed (air-K), run 2 | 56 % | 23 % | 3 % | 8 % | 9 % | 0.51 |

Three things the drills corrected, in the order found:
1. The attack order given at the turn's first tick flew the nose-to-nose pass again (air-I1). The attack is given
   only once the fighter is round on their heading with a contact ahead of it.
2. A third of the wing never began its turn (air-I11): a fighter reaching its turn point at an angle swung onto the
   bombers' heading chasing the point, and the trigger, which read its heading, never held. Reaching the point
   begins the turn now, whatever the heading. This was the "ahead" column.
3. The turn's cost is four seconds of the bombers' flight, not the three it takes to come round, because the plane
   leaves it slow (K-engine-a-fighters-turn-costs-four-seconds-of-the-bombers-flight); and to a point dead astern
   the engine turns a plane either way, so the turn is given by a point abeam toward the track.

### 6.3 Limits

- **Not in a game.** The bench gives the wing whole fighters, up on their loop twelve seconds before his bombers
  move, against bombers flying his recorded orders. In a game the wing must exist, be up, and be the player's to
  hand over.
- **His fighters with the bombers** are not drilled: the wing flies the same conversion on whatever flies. He sent
  none with these runs; his eleven Nighthawks patrolled his own half.
- **The keep is a radius.** A scout or his patrol crossing it scrambles the wing; nothing yet weighs what the
  contact is or where it is going.
- **Inbound shots.** The scripted pass lands 1.2 to 1.6 hits a fighter-second on the way in, the wing 0.4 to 0.5
  (it closes on a point beside the track, not on the bombers). Not pursued: the wing's place behind them is worth
  more, and the two were not combined.
- **Highwinds** fly the Falcon's turn numbers; four of them kept run 2 and not run 1.
- **Constants** (`TURN_SECONDS` 4.0, `TURN_ACROSS` 550, `TRAIL` 300, `CLOSE_ACROSS` 200) were set on these two
  runs and Falcons; between builds an eight-repetition cell moved by two for no change that mattered, so they are
  placed, not tuned.
- The bench's own: section 3's, and the early sighting it guards with `contact_after` (the wing does not read
  that guard; the bombers' first place is outside its keep in both runs).

### 6.4 Other raids: how far the law carries (2026-11-17)

The user, on the results above: "my only worry is that what you have built is too specific to this scenario. What is
your take on its robustness to other flight paths, targets, bomber control, and other parameters?" Asked of the
bench: `run/air_raid.py` puts made-up raids against run 2's base (sensors and anti-air as they stood; twelve Falcons
at the block; eight repetitions; "none" is the same raid with no fighters). Saved, and mean metal lost:

| raid | the wing | none |
|---|---|---|
| 7 Hailstorms straight in from the north | 8 of 8, 79 | 4 of 8, 21,378 |
| from the north-west | 8 of 8, 170 | 8 of 8, 2,416 |
| from the west | 8 of 8, 5 | 8 of 8, 0 |
| from the south-west (13 s of warning) | 8 of 8, 148 | 7 of 8, 2,143 |
| from the west with a 50-degree dogleg 2,700 out | 8 of 8, 0 | 8 of 8, 290 |
| 4 from the north and 4 from the west at once | 8 of 8, 0 | 7 of 8, 6,737 |
| a scout across the keep, then 7 from the north | 8 of 8, 445 | 3 of 8, 22,960 |
| 14 Whirlwinds (tier 1) from the north-west | 8 of 8, 329 | 5 of 8, 14,000 |
| 12 Hailstorms from the north | 7 of 8, 3,752 | 0 of 8, 40,082 |
| 7 Hailstorms and 8 Valiants with them | 8 of 8, 382 (2.5 Falcons lost) | 8 of 8, 1,221 |
| 7 Hailstorms in a line abreast 1,500 wide | 5 of 8 and 7 of 8 | 1 of 8, 35,277 |
| 2 Liches (295 a second to the Falcon's 289) | 8 of 8, 584 | 8 of 8, 1,236 |
| 4 Stilettos (300) | 0.8 of 4 dropped | 1.5 of 4 dropped |
| 7 Hailstorms, **the base's anti-air taken out** | **2 of 8**, 26,500 (5.6 Falcons lost) | 0 of 8, 34,942 |
| 5 Hailstorms, the anti-air taken out | 5 of 8, 13,100 | 0 of 8, 34,937 |
| 7 Hailstorms at **a fusion 2,080 from the wing's place**, 15 s of warning | **0 of 8** (4 of 7 dropped) | 0 of 8 (4.2 of 7) |

What carries: the law reads the bombers' place and velocity each tick and nothing of the map, the target or the
route, and it held from four directions, through a dogleg, against two groups at once, a decoy, twice the bombers of
half the health, and more bombers than the runs had.

What it leans on, measured:
- **The base's own anti-air.** Twelve Falcons shoot down about four Hailstorms in the time they have and cripple
  the rest; the anti-air finishes those. With none, seven Hailstorms were stopped in 2 of 8 and five in 5 of 8.
- **Being where the bombs go.** From contact the wing needs five to ten seconds to its first shot and four more
  for the turn. A target 2,000 from its place costs seven seconds of flight: with fifteen seconds of warning and no
  anti-air there, three of seven were shot down before the drop.
- **Being faster than the bombers.** A Falcon cannot stay behind a Liche or a Stiletto.

Two faults these raids found, both corrected (2dd830c4):
- A body the fighter cannot stay behind is now met nose to nose (`CATCH`): the wing's damage before the first bomb
  went from 42 to 81 % of two Liches' health and from 29 to 53 % of four Stilettos'.
- The turn's point was a half turn's whatever the fighter's course, so a wing that came across the bombers' track
  (the target away from its place) crossed in front of them and came round wide. The point is now solved for the
  heading the fighter comes with (`turn_point`): its seconds behind the bombers in reach went from 18 to 54 % in
  that raid. The two recorded runs after it: run 1 kept 13 of 16 and 23 of 24, run 2 16 of 16 (run 1 as the sensors
  stood, every batch since the law's commit: 79 of 88).

Not flown: his fighters ahead of his bombers as a sweep, gunships, a jammed raid (nothing on radar), another map's
ground, Highwinds or another faction's fighters on these raids, and anything in a game.

## 7. The player's order: the hands' air patrol (2026-11-17)

The user's rulings (section 1): the patrol is "something the player orders with the engine choosing targets", and
"we can treat this like rove for now in terms of having the player hand the necessary resources over to the micro
engine for control". Two ways to do that were put to him:

- **A: a move of the hands, as `rove` is**: a group of fighters is offered the patrol over a place; the player
  names the group, the job and the place in the packet, Jev picks it, and any later pick takes the group back.
- B: the hold of every fighter group is a wing at its hold point (no new move, but the hand-over is then implied by
  a hold and not said).

**His ruling, 2026-11-17: "I agree with A for how the player and hands manage this."** And after the raids of
section 6.4: "Good. Let's proceed with A." B is not taken.

### 7.1 As built

- **The move.** An air group whose every aircraft is a fighter (it hits what flies harder than the ground: the
  lane's own test, `micro::intercept::fighter`, so no bomber, gunship or stun drone, and no mixed group) has
  `air_patrol_<place>` for every named place that is a point: the marks, and the spots, passages, cells' names and
  `home` the packet names. The place the group stands at is among them (a walk there is not on a menu, a patrol
  there is the common case). The move carries the place's reading, so under the `fuse` layer the patrols at places
  the packet does not name for the group go off its menu as its walks there do. Its words: "group_F patrols the air
  over home (...): its fighters circle there in the air on their own and fly at every aircraft of his that comes
  within 4500 of it, turning in behind his bombers to shoot them down before they drop, and come back to it when
  none is left; a pick of yours brings it back".
- **The task.** The pick sets `GroupTask::AirPatrol { at, place }` and orders nothing. From the next tick the brain
  declares the group to the lane as `GroupKind::Intercept { place: at, keep: 4500 }` and the lane flies its
  fighters (section 6). The hands keep up no order for it; its menu keeps `stay`, `hold`, the walks and the patrols
  over the other places, and has no move at a party; a party near it, a hit and a sighting are no news that asks it
  again. Any other pick is the way back, with that pick's own orders.
- **The deal.** The group's place is the patrol's point, so new fighters come to it and are the lane's once inside
  the gate, and a fighter out on a scramble stays the group's (the leaving gate is 30 s of its own flight, 8,600
  for a Falcon, against 5,200 for the keep's edge). What the slots took that is no fighter is sent to the place on
  hold position and said in the group's words.
- **The words.** `doing`: "patrolling the air over home: its 12 fighters circle there, nothing of his flying within
  4500 of it", or "... scrambled at 7 aircraft of his within 4500 of it, the nearest at spot_12 (D3), 2900 from
  home: of its 12 fighters, 3 flying to meet them, 5 turning in behind them, 4 attacking". `lane`: what the lane
  does with it. `air_patrol`: the log, "12:04 scrambled at 7 aircraft of his at D3, 3900 from home" and "12:38 back
  over home after 34 s: nothing of his flies within its keep, 2 of its fighters lost meanwhile"; both also go into
  what the hands did, for the player's report.
- **The player's text** (`crates/bot/src/strategist/player.md`, the list of the hands' moves): the move, how to ask
  for it (the group, the job and the place, as for roving; a mark where no place names the thing to keep), that a
  fighter ordered straight at a bomber is left behind and a fighter group on a hold flies at nothing, and what
  section 6.4 measured: it keeps what it circles, it helps the anti-air under it and does not replace it, and a
  fighter stays behind only what is slower than it. Not read by him yet.
- **One correction to the lane from it:** a wing whose place the host moves flew on round the old one (a patrol
  never goes idle, and the loop was sent again only to an idle fighter); the loop now carries its place.
- **The ledgers:** `docs/code-decisions.md` §2 (the hand-over, by his words), §3 (who is offered it, and no move at
  a party) and §5 (the lane's row); `docs/heuristics.md` H-MICRO-INTERCEPT; `docs/harness/record-format.md` (the
  group's `air_patrol` task and the pass line's `air_patrol` events).

Not built with it: a keep other than 4,500 by the player's word; a way for the player to tell a patrol what not to
fly at (any aircraft in the keep scrambles it, a scout included); the patrol for a seat's allies' fields.

Still open beside it, from section 5, on his word: the words for an air party (a radar contact that flies reads
"unidentified, speed unknown" in the picture), and in the brief's air article that an advanced radar earns its
seconds on the flank with a view, that tier-2 fighters answer tier-2 bombers and that a fighter group that only
holds does nothing (the last two are in the move's own text now, not in the article). The bare change of a held
fighter group from hold position to manoeuvre was looked at and left: by the drills it keeps 0 to 2 of 8, and a
fighter that chases from an idle hold lands wherever the chase ended.

