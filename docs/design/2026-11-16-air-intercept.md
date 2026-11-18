# Air interception: his bombers stopped before they drop

Branch `air-intercept` (worktree `within-reason.air-intercept`). Begun 2026-11-16 from the user's review of the
lobby 3v3 on Tangerine (`run/matches/1794780805-human` in the main tree, lost about 35:00).

**State (2026-11-18):** the wing is built in the micro engine and drilled (section 6; commits 67ba56b6 and
2dd830c4), and flown against raids the game did not show (section 6.4). The player's order is built as ruled, a move
of the hands as rove is: the air patrol (section 7.1, 8e09b893). Its first game (section 7.2) showed that the
wing's loop was wrong outside a raid: it left its place for any aircraft within 4,500 and its return had no leash;
ten of fourteen Falcons were lost over his station. **The loop was re-derived (section 8), ruled on by the user and
rebuilt (sections 8.1 and 8.2): it answers what comes at its place, with the fighters that needs, no further than
a leash, and flies back. On the bench the game's two faults are gone and the raids hold. Its one game (section 8.4,
flown at 24 seconds on his word) had two raids of four Stormbringers at its place: all eight shot down before the
nearest was within 1,650 of it, nothing under the patrol hit, no fighter lost or beyond 2,600 from the place, a lone
bomber 3,100 out left alone. The game itself was lost on the ground. It did not tell 18 seconds from 24 (his
fighters never came into our sight), so `THREAT_SECONDS` is still open, the constant 18.** No bot of `bots/`
declares a wing. Not merged.

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

(As it stood for the first game. Section 8.2 changed what the wing answers and the words that say it: no keep of
4,500, the loop's bounds in its place, and no attack on his buildings on a fighter group's menu.)

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

### 7.2 The first game (air-patrol-1, 2026-11-17)

One realtime 1v1 on Comet Catcher against WRDistilled 0.2.0 (the bot that builds fighters and bombers from 30 metal
a second), with an objective that asked the player for an aircraft plant, twelve fighters in a group of their own
and the air patrol over its energy. Won at 16:55; the patrol was never needed (his three bombers stood at his
station to the end), and it showed what the bench had not.

What worked:
- The player declared `group_F`, marked `energy_field` and named the job in the packet. At 5:32, the second the
  first Falcon was out, Jev rated the patrol over `energy_field` 0.91 and over `home` 0.79 and every other place
  under 0.2; the world pick gave them 0.61 and 0.35, and the hands' draw (H-HANDS-MIN-P) took `home`. The two were
  both "right here". The patrol stayed the group's course for the rest of the game: 43 `change` questions, none
  taken.
- The lane put each new Falcon on the loop as it came out of the plant, one order set a fighter and none repeated;
  from 9:30 to 13:52 all 14 flew within 850 of home.
- The picture's words read as meant: "patrolling the air over home: scrambled at 11 aircraft of his within 4500 of
  it, the nearest at 377 from spot_21 (D2), 4464 from home: of its 14 fighters, 14 flying to meet them".

What failed, both in the lane:
- **What scrambles it.** At 13:52 one of his Stormbringers, milling 4,100 to 4,900 from our home beside his own air
  station (6,000 out), was inside the keep: all 14 Falcons flew 4,000 out in 18 seconds and killed it. Nothing of
  his was coming at the base. A keep of 4,500 is 18 seconds of a bomber's flight, which is what a raid coming
  straight in asks for; as a radius about the place it is also half of this map.
- **The return.** His 11 Falcons came out after the wing (a second scramble at 14:18), and when they turned back
  past 1.15 keeps the lane sent the wing "back on its loop": a patrol order under manoeuvre, given 4,000 from
  home with his fighters 900 away. That is the engine's own chase with no leash, and the wing followed them to his
  station, 5,700 to 6,500 from our home and over his Sentries. Ten of 14 Falcons were lost from 14:34 to 14:44 for
  seven of his; the base had no patrol from 13:54.

The player's debrief named it: "air_patrol ... scrambles far past our half: group_F chased his fighters to E1/E2
into his Sentries and lost 10 of 14 Falcons in one fight"; wished: "A keep radius on air_patrol, so group_F never
chases past our half".

Smaller things seen: the first gate of a new group is asked before the decode has read its places (30 questions for
`group_F` at 5:32, 16 of them patrols; the hands' way for every new actor); the words said "its 1 fighters"; a
patrolling fighter group is still offered attacks on his buildings (`fight_to_<place>`, an air group's move from
before this arc: 26 questions in the game).

Not built with it: a keep other than 4,500 by the player's word; a way for the player to tell a patrol what not to
fly at (any aircraft in the keep scrambles it, a scout included); the patrol for a seat's allies' fields.

Still open beside it, from section 5, on his word: the words for an air party (a radar contact that flies reads
"unidentified, speed unknown" in the picture), and in the brief's air article that an advanced radar earns its
seconds on the flank with a view, that tier-2 fighters answer tier-2 bombers and that a fighter group that only
holds does nothing (the last two are in the move's own text now, not in the article). The bare change of a held
fighter group from hold position to manoeuvre was looked at and left: by the drills it keeps 0 to 2 of 8, and a
fighter that chases from an idle hold lands wherever the chase ended.


## 8. The wing's loop, re-derived (2026-11-17; ruled 2026-11-18, section 8.1 on)

The user's rule for a control loop that fails in a scenario its design did not consider (2026-11-15): "you must
re-evaluate the assumptions you used to design it. Don't patch this". So no rule at the site: what the loop must
hold, what the game broke, which assumptions those were, and what would replace them.

**What the wing is for:** his bombers are behind our fighters' guns before they drop on what the wing keeps. It is
a guard of a place.

**Properties the loop must hold under a fixed order (patrol over P), whatever his aircraft do:**

| | Property | In air-patrol-1 |
|---|---|---|
| 1 | *A leash.* No fighter is ever further from P than a bound the design states | broken: 6,500 from P |
| 2 | *It comes back.* With nothing threatening P every fighter converges on the loop over P from wherever it is, whatever flies near it on the way | broken: the return was an engine patrol under manoeuvre, begun 4,000 out beside his fighters |
| 3 | *Threat, not presence.* It leaves the loop only for aircraft that are coming at what it keeps; an aircraft that is not moves nothing | broken: a bomber milling beside his own station took all 14 |
| 4 | *In time.* Against a raid coming straight in it is behind the bombers before the drop | held in the drills (sections 6.1, 6.4); not asked of it in the game |

**The assumptions that were wrong:**
- *"What flies in the keep is a raid."* Both recorded runs and every made-up raid were bombers flying at the base
  from their first contact to their drop, so the keep, a radius taken from the warning a raid needs (18 seconds of
  a bomber's flight), was used as a test of presence. On this map 4,500 about home is half the map.
- *"When no contact is left the wing is at its place."* In the drills the bombers died or left over our own base.
  The return was never a state of the design: it was the loiter's order given again from wherever the wing was.
- *"The engine's patrol under manoeuvre is a safe idle."* It was chosen so that the wing still answers what the
  radar missed. It is a second control loop inside ours: the engine's own search, 1,000 round the fighter wherever
  the fighter is, with no place in it.
- *The pursuit's end* (a contact is the wing's until 1.15 keeps) was a hysteresis against flicker, not a leash.

**The proposal:**
1. **A threat test in place of the keep's presence test.** A body of contacts is a threat when its course, held
   for about a second, passes within the guarded radius of P and brings it there within the time the conversion
   needs (the 18 seconds the keep stood for), or when it is already inside the guarded radius. A bomber milling
   4,500 out, a scout crossing 3,000 abeam and a fighter wing circling its own station are not threats; the same
   aircraft turning in are, from the second their course holds. The guarded radius is what section 6.4 says the
   wing can keep: under 2,000 (a target 2,080 from P was lost in 8 of 8), to be found by a drill.
2. **A leash.** No fighter is sent to a point, and none follows a target, further from P than a stated bound (the
   meeting point with a raid coming straight in from 4,500 lies 2,400 out; a bound near 3,000, to be drilled). A
   fighter beyond it is in the return whatever is near it.
3. **The return as a state.** A move order straight to P (a fighter on a move order does not search for targets)
   until it is within 1,000 of P; the patrol under manoeuvre only there, where the engine's own search stays
   inside the leash.
4. An option beside these: **in proportion.** Only the fighters a threat needs leave the loop (those that kill it
   in the time it has; the deal already counts them for the attack), the rest stay over P. Fourteen Falcons at one
   Stormbringer left nothing over the base. It makes the wing's state per fighter and not per wing, and is the one
   part of this that changes how a real raid is met, so it is the user's to take or leave.

**Drills before the mechanism, one a property:** this game's two moments cut from its tape as scenes (13:52, the
milling bomber; 14:18 to 14:44, his fighters coming out and turning back): the furthest any fighter gets from P,
the seconds until all are back on the loop, the fighters lost (properties 1 to 3). Made-up beside them: a scout
3,000 abeam, a wing of his fighters flying at the patrol over P (not drilled before: the law is the bombers'
conversion). And sections 6.1 and 6.4 again whole, the two runs at 40 repetitions, for property 4: the threat test
costs up to a second of the warning, and what that takes off 37 of 40 is the price to know before it is paid.

**What stays as it is:** the stern conversion itself (close, turn, trail), the bodies, the deal of targets, the
pass at what the fighter cannot stay behind, and all of the hands' side (section 7.1).

### 8.1 The ruling and the target design (2026-11-18)

The user, on the proposal above: "I agree with the new approach and proposed tests. We can include the "only the
fighters needed" part, and also remove the building attacks for figher only groups." So all four parts, the drills
first, and the hands' attack on his buildings off a fighter group's menu.

**The drills, flown on the old law first** (`run/air_raid.py` on run 2's base with the wing of twelve Falcons, eight
repetitions; `run/air_drill.py --leash`; the place is the block, 5580,6397):

| scene | what flies | furthest a fighter got from the place (mean, most) | seconds with a fighter over 1,500 out | most fighters off the loop at once | fighters lost |
|---|---|---|---|---|---|
| mill | one Whirlwind milling 3,870 to 4,720 north of the place, never coming at it | 2,658, 4,733 | 16.9 | 6.0 (all twelve in the repetitions that saw it inside 4,500) | 0 |
| out | eleven Valiants out to 3,000 from the place on a fight order, called back to their station 5,900 out sixteen seconds on (three repetitions a batch: the bench's engine ends after one such duel) | 4,829, 6,287 | 56.7 | 12 | 3.3 |
| abeam | a scout passing 2,180 west of the place (our anti-air out, so that it lives) | 2,319, 2,699 | 6.5 | 12 | 0 |
| over | a scout straight over the place (our anti-air out) | 1,572, 1,915 | 0.8 | 12 | 0 |
| sweep | eleven Valiants on a fight order at the place | 2,614, 3,114 | 11.6 | 12 | 3.2 (all eleven of his) |

The bench shows both of the game's faults: twelve Falcons leave for one scout or one milling bomber, and after
`out` fighters of ours are found over his station.

**The target design** (the conversion itself, sections 5 and 6, is unchanged; this is what decides who flies it):

1. *What the wing sees:* every contact that flies, wherever it is. Each has a track in the wing's memory: since
   when its course has held at the place, whether the wing has taken it on, when it was last seen (forgotten three
   seconds after).
2. *Coming at the place:* the contact moves, the place is ahead of it, its course passes within `GUARD` (1,500) of
   the place, and it is within `THREAT_SECONDS` (18) of its nearest point to the place at its speed (for a
   Hailstorm flying straight in, 4,500 out: what the keep was). The course *holds* when that has been so for `HOLD`
   (one second) with the heading not more than `STRAIGHT` (15 degrees) off what it was when the second began: a
   bomber milling turns 30 degrees a second and never holds, a raid on its leg does.
3. *Taken on:* a contact whose course holds, or one inside `GUARD` whatever its course. It stays the wing's until
   it has been further than `LEASH` (3,000) from the place and not coming for `RELEASE` (1.5 seconds). A body of
   contacts is a threat while one of its members is taken on.
4. *The fighters a threat needs:* for each of its members the fighters whose fire kills it in `WINDOW` (2 seconds,
   the deal's own measure: six Falcons a Hailstorm, one a scout or a Falcon, four an unidentified contact). The
   threats are served soonest to the place first; a fighter already on a threat stays on it; the rest are the free
   fighters nearest it. Fighters not needed stay on the loop.
5. *The leash:* no fighter is given a point further than `LEASH` from the place; a fighter is given no target
   further than that; and one found further than `LEASH + SLACK` (500) is in the return whatever it was doing.
6. *The return:* a fighter with no threat that is further than `HOME` (1,000) from the place flies a move order to
   it under hold position, and takes up the loop inside `HOME`. The loop is flown under hold position too: the
   engine's own search is off, and what flies near the loop is inside `GUARD` and the wing's by rule 3.
7. *Said once:* scrambled when the first fighter leaves the loop for a threat (with how many went), returned when
   every fighter is back on the loop.

The numbers are the first values; `GUARD` and `LEASH` are drilled (a target 2,080 from the place is outside any
`GUARD` under 2,080: the wing will not leave for a raid on it, where the old law left and saved nothing).

### 8.2 As built, and what the drills gave (2026-11-18)

**Built** as section 8.1 has it (4f0f8b74), in `crates/micro/src/intercept.rs`; the conversion (close, turn, trail, the
bodies, the deal) is untouched. The kind is `GroupKind::Intercept { place }`: the keep is gone from it, from the
bench's `intercept` row and from `run/air_scene.py`. On the hands' side the move's words, the pick's words, the
picture's `doing` and `lane` lines and the patrol's log say threats and the loop's bounds where they said the keep
("its 12 fighters circle there, nothing of his coming at it; 2 aircraft of his in sight are not coming at it";
"scrambled 6 of its fighters at 1 aircraft of his coming at home"; "all back over home after 34 s"); an air group
whose every aircraft is a fighter has no `fight_to_<his place>` on its menu; the player's text says what the patrol
answers and that it "is a guard of its place, not a hunt".

**One departure from 8.1, found by a unit test before the bench.** By rule 3 alone a contact that had flown out
past the leash was still the wing's for the 1.5 seconds of its release, so a fighter turned home at the leash was
chosen for the same leaving body again the next tick, and home again the tick after. A body is a threat only while
one of its members can be flown at (inside the leash, or coming), and a fighter trailing a body takes a target that
can be, or goes home.

**One departure from the drills section 8 named.** The game's two moments were flown as made-up scenes with the
game's distances on run 2's base (`mill`: one bomber milling 3,870 to 4,720 out; `out` and `outaa`: eleven fighters
out to 3,000 from the place and called back to a station 5,900 out, without and with four anti-air turrets by it),
not cut from the game's tape.

**The scenes, the old loop and the new** (twelve Falcons over the block; eight repetitions, `out` and `outaa` three;
`run/air_drill.py --leash`; old: air-L0-\* at d83633d3, new: air-L1-\* on 4f0f8b74's tree):

| scene | furthest a fighter got from the place (mean, most): old | new | most fighters off the loop at once: old | new | fighters lost: old | new |
|---|---|---|---|---|---|---|
| mill | 2,658, 4,733 | 819, 838 | 6.0 | 0 | 0 | 0 |
| abeam (a scout 2,180 west) | 2,319, 2,699 | 964, 1,316 | 12 | 0 | 0 | 0 |
| over (a scout over the place) | 1,572, 1,915 | 1,403, 1,975 | 12 | 4 | 0 | 0 |
| out | 4,829, 6,287 (2,409 out at the end) | 3,135, 3,796 (768 at the end) | 12 | 12 | 3.3 | 2.7 |
| outaa | not flown | 3,003, 3,288 (911 at the end) | | 12 | | 3.0 |
| sweep (eleven Valiants at the place) | 2,614, 3,114 | 2,527, 2,975 | 12 | 12 | 3.2 | 3.6 |

Properties 1 to 3 of section 8 hold on the bench: the milling bomber and the scout abeam move nothing, the scout
overhead takes four fighters (it is a contact of no known type on radar: four by the deal's measure, one once it is
seen), and after `out` every fighter is back on its loop. The most a fighter got from the place, 3,796, is 300
past the leash and its slack: a Falcon at 290 a second needs that to come round.

**The raids of section 6.4 again, and the two runs at 40** (property 4; air-G1-\*, eight repetitions; the runs
air-G1 and air-G2 together; saved, and mean metal lost):

| raid | the old loop (6.4) | the new |
|---|---|---|
| 7 Hailstorms from the north | 8 of 8, 79 | 8 of 8, 473 |
| from the north-west | 8 of 8, 170 | 8 of 8, 614 |
| from the west | 8 of 8, 5 | 8 of 8, 20 |
| from the south-west | 8 of 8, 148 | 15 of 16, 606 |
| the dogleg | 8 of 8, 0 | 8 of 8, 0 |
| two groups at once | 8 of 8, 0 | 8 of 8, 16 |
| the decoy scout, then 7 | 8 of 8, 445 | 8 of 8, 731 |
| 14 Whirlwinds | 8 of 8, 329 | 8 of 8, 557 |
| 12 Hailstorms | 7 of 8, 3,752 | 7 of 8, 7,087 |
| 7 Hailstorms and 8 Valiants | 8 of 8, 382 (2.5 Falcons lost) | 8 of 8, 119 (2.1) |
| a line abreast | 5 of 8 and 7 of 8 | 7 of 8 |
| 2 Liches | 8 of 8, 584 | 8 of 8, 570 |
| 4 Stilettos | 0.8 of 4 dropped | 1.2 of 4 |
| 7 Hailstorms, the base's anti-air out | 2 of 8 | **1 of 24** |
| a fusion 2,080 from the place (9 s of warning) | 0 of 8 | 0 of 8 |
| run 1 as the sensors stood | 37 of 40 | 37 of 40 |
| run 2 | 40 of 40 | 39 of 40 |

The raids and the two runs hold: no cell but one moved by more than an eight-repetition cell swings. The price the
proposal named is there and small: the first hit on the northern raid comes at 10.1 to 10.5 s after contact in the
four batches flown at 18 seconds against 9.1 and 9.6 before, and the metal lost is a little higher in most cells. **The
one cell that fell is the base with no anti-air under the wing, 1 of 24 against 2 of 8** (the same damage before the
first bomb, 88 to 90 % of the bombers' health, and the same bombers dropping, 2.5 to 2.8 of 7: there a half second
is a reactor). In the raid on the fusion 2,080 from the place (flown here with the 9 s of warning the base's own
sensors give, not 6.4's 15) the wing no longer leaves before the bombs: its first hit comes 33 s after contact,
where it came at 7 and cost 1.9 Falcons; nothing was saved either way.

**The bounds, each flown off its first value** (`WITHIN_REASON_INTERCEPT`, eight repetitions, `out` three;
air-Pg25-\*, air-Pl20-\*, air-Pl40-\*, air-Ps24-\*, air-Ps24b-\*):

| bound | scene | at the first value | off it |
|---|---|---|---|
| `GUARD` 1,500, flown at 2,500 | the fusion 2,080 from the place | 0 of 8; 4.0 of 7 dropped; 0.1 Falcons lost | the wing leaves (first hit at 7 s): 0 of 8; 4.4 of 7 dropped; 1.4 Falcons lost |
| | the scout 2,180 abeam | no fighter leaves | 4 leave, to 2,462 out |
| | the milling bomber; the northern raid | none leaves; 8 of 8 | the same |
| `LEASH` 3,000, flown at 2,000 and 4,000 | the northern raid | 8 of 8, first hit 10.1 s | 8 of 8 both; 10.5 and 10.2 s |
| | `out` (furthest mean, most; Falcons lost; his lost) | 3,135, 3,796; 2.7; 5.0 of 11 | 2,000: 1,957, 2,486; 2.3; 2.0. 4,000: 3,706, 3,794; 3.0; 7.0 |
| | the sweep | 2,527, 2,975; 3.6 lost; 11 of 11 | 2,000: 2,492, 2,888; 2.4. 4,000: 2,616, 3,341; 3.2 |
| `THREAT_SECONDS` 18, flown at 24 | the northern raid | 8 of 8, first hit 10.1 s | 8 of 8, 9.8 s |
| | the anti-air out | **1 of 24**, first hit 10.3 s, 6.4 Falcons lost | **6 of 24**, 9.8 s, 5.3 lost |
| | the line abreast | 7 of 8 | 8 of 8 |
| | the milling bomber; the scout abeam; the scout overhead | none; none; 4 | the same |
| | `out` | 3,135, 3,796; 2.7 lost; 5.0 of 11 | 2,797, 2,978; 3.0 lost; 9.0 of 11 |
| | the sweep | 3.6 lost | 2.5 lost |

- **`GUARD` stands at 1,500.** A guard wide enough to send the wing at the raid 2,080 aside saves nothing there and
  costs fighters, and it answers the scout abeam. Where between the place and 2,080 the wing stops saving what is
  bombed is not located (no scene with a target 1,000 or 1,500 aside).
- **`LEASH` stands at 3,000.** Against what comes at the place it does nothing between 2,000 and 4,000 (the wing
  meets a raid 2,300 to 2,600 out by itself); it is what bounds the chase after `out`.
- **`THREAT_SECONDS` is open, 18 or 24.** At 24 the wing leaves as early as the old loop did against a raid first
  seen further out than 18 seconds, and has the half second back: 6 of 24 against 1 of 24 with the anti-air out
  (one-sided p about 0.05), and nothing changed in any scene that is not a raid. Twenty-four seconds is also where
  the leash and not the warning sets the meeting point: a Falcon (289 a second) that leaves when a Hailstorm (230)
  is 23.4 s out meets it 3,000 from the place, so leaving earlier buys nothing. What the bench does not price is
  traffic far out: at 24 s an aircraft of his flying straight at our place for a second is a threat from 5,500 out
  (a bomber) or 7,000 (a fighter), at 18 s from 4,100 and 5,200; in air-patrol-1 his station lay 6,000 from our
  home, so his fighters leaving it our way would have scrambled a fighter each to the leash at 24 and not at 18. The
  constant is 18, as ruled; the switch (`WITHIN_REASON_INTERCEPT=seconds=24` in a game's environment) flies the
  other without a build.

**Still not shown:** the new loop in a game (it has only the bench's scenes behind it, and the first loop failed
outside its drills); the game's two moments from its own tape; a target between the place and 2,080 aside; his
fighters ahead of his bombers; a jammed raid; another map; Highwinds on the loop's scenes.

### 8.3 The loop read for what the drills do not fly (2026-11-18)

The first loop failed in the one game it had, outside every scene it was drilled on. So the rebuilt one was read
by a second instance of the model that was given the code, the four properties and the list of scenes already
flown, and asked for sequences that break a property, flap, stall or report wrongly. It found no path past the
leash and none that keeps a fighter from coming back. What it found, each checked against the code here:

**Two faults, corrected (the commit after 4f0f8b74):**
- *A fighter given another body kept the first one's conversion.* The conversion began (`Close`, the side unset)
  only from the loop or the return. A fighter trailing the last bomber of one body when it died, with a second
  body a threat the same tick, never went by the loop: it was still "trailing", and was sent an attack by id at the
  second body from wherever it was, nose to nose when that body was coming at it; one in its turn kept the turn's
  clock, one closing kept its side of the first track. A fighter now remembers the members of its body, a body is
  the same while one of them is in it, and a fighter given another begins anew.
- *"All back" from a wing with no fighter left.* With every fighter dead nothing was chosen and nothing was on its
  way back, which is the test for the return's word: the log said "all back over home after 40 s, 12 of its
  fighters lost meanwhile" with the raid still on. A wing with no fighter says nothing.
- A branch that could not run (the trail's own way home: a threat has a member to fly at by what makes it a
  threat) is removed.

**Flown again on the corrected loop** (air-L2-\*, air-G3-\*, air-W1-\*; 238 duels): the scenes as in 8.2 (the
milling bomber and the scout abeam take no fighter off the loop, the scout overhead four; after `out` the furthest
Falcon was 2,917 from the place and all were back). The raids: 8 of 8 in twelve of the fifteen, twelve Hailstorms
7 of 8, the anti-air out 0 of 8 and the fusion aside 0 of 8, as before. Run 1 23 of 24, run 2 24 of 24 (on the
rebuilt loop in all, 8.2's with these: 60 of 64 and 63 of 64). Two raids were made for the first fault, a second
wave of four Hailstorms 10 s after the first from the west (`waveW`) and 8 s behind it from the north (`waveN`),
sixteen repetitions each before the correction (air-W0-\*) and after: every reactor kept in all four batches; the
bombers that dropped 1.0 and then 0.8 of 8, and 2.9 and then 1.9; metal lost 242 and then 29, and 605 and then 79.
With the base's anti-air under them these raids lose no reactor either way, so the drills say the correction broke
nothing and lean its way; that it is right rests on the reading of the code and on a unit test that fails without
it.

**Open, the likeliest in a game first; none built, each the user's to weigh:**
1. *A contact missing from one snapshot recalls its fighters.* The bodies are made of this tick's contacts; the
   three seconds a track is remembered keep that it was taken on, not that it is a threat. A body of one (a scout,
   a lone bomber) that is unseen for a tick sends its fighters home that tick, and if they were within 1,000 of the
   place the wing says "back" and then "scrambled" again. A raid of several is recalled only when all of it is
   unseen at once. On the bench single contacts do drop out for a second (the milling bomber 19 times in 8
   flights, a bomber of the northern raid 9 times in 56), a whole raid never did (no second of the northern raid or
   the line abreast with every living bomber unseen after one was seen); what the gaps cost is inside every number
   above, and the first loop judged by the tick's contacts too. The remedy is an addition to the design: a contact
   taken on is carried through a short gap at its last place and velocity, which needs the lane to tell one shot
   down from one out of sight. A jammed raid is where it would matter, and is not flown.
2. *A later raid is not said while an earlier scramble is open* (rule 7 says the scramble once). After "scrambled 4
   of its fighters at 1 aircraft" (a scout) a raid that comes before the wing is back adds no line to the log, and
   a scout that stays inside the leash keeps the scramble open. The picture's `doing` line is right each second;
   the log, which is what the player's report carries, is not.
3. *The body is the unit.* An aircraft the wing has not taken on that flies within 700 of one it has is of its
   body: counted in what the threat needs and dealt as a target while inside the leash (rule 3 as designed: a
   raid's escort is of the raid). A scout the wing follows past his gunships 2,200 out takes fighters at them for
   those seconds, and at a gap that hovers about 700 the fighters over what the scout alone needs would go between
   the loop and the body.
4. *A contact of no known type counts four fighters*, so a scout that leaves sight but not radar inside the leash
   takes three more.
5. *Rare:* a place moved in the middle of a scramble keeps what was taken on at the old one; a stunned fighter is
   counted as flying.
6. *Not verified, the engine's part unknown:* a point off the map. The loop's corners lie 350 from the place, the
   close's move 800 beyond the turn's point, and the leash is a circle that leaves the map wherever the place is
   within 3,000 of an edge; on the bench a scout given a waypoint off the map flew nowhere (the scouts' first
   scenes, air-L0-abeam and air-L0-over as first cut). The lane cannot clamp today: what it knows of the map is the threat grid's size, which the test host sets apart from any
   map. A wing over a place 300 from an edge is the scene that would say.
7. *Outside the loop:* bombers in line astern are one body and the conversion is flown on its last member; not
   drilled.

### 8.4 The second game (air-patrol-2, 2026-11-18)

His word on the report of 8.2 and 8.3: "Good. Let's run the second game with WRDistilled. We can use 24s". Read as
this game flown at 24 seconds, not as the constant's ruling: the code stays 18.

**What was under test.** 375509f3 (the rebuilt loop with the two corrections of 8.3), `WITHIN_REASON_INTERCEPT=seconds=24`
in the arena's environment (read back from the bot process's environment at the start; the picture's words for the
group said "under 24 s away" all game), the launch of air-patrol-1 otherwise: Comet Catcher Remake 1.8, mirror,
seed 3, realtime, `--opponent WRDistilled:0.2.0`, the same objective (an aircraft plant by 7:00, twelve fighters in
a group of their own, on the air patrol over our energy from the first). `run/matches/1795014462-air-patrol-2`.
**Lost at 28:44** (not called; 126 turns; Jev $1.24; the `claude` account +1.0 point of its week, extra usage
+0.00). Read with `run/air_patrol.py` (new: the hands' log joined with the tape, `docs/harness/duels.md`), which
was run on the first game too:

| | air-patrol-1 (the first loop) | air-patrol-2 (the rebuilt loop, 24 s) |
|---|---|---|
| The pick | 5:32, over `home` (the draw; the packet meant `energy_field`), kept to the end | 5:22, over `energy_field` (1150, 5500), kept to the end: one pick, the group's task in all 1,819 calls after it |
| Fighters in the group | 25 over the game, 14 at most | 14 |
| On the loop, no scramble open: the furthest a fighter was from the place | 6,676 (the "return", 14:44) | 1,299; no fighter-second beyond 1,700 in 16,843 |
| Scrambles | 2: a bomber milling 4,489 out, his fighters 4,490 out; nothing of his came within 3,000 of the place | 2: four Stormbringers coming at the place each time, the first seen 4,471 and 4,068 out |
| Fighters sent | all 14, twice | 3 at the first bomber seen, 12 of the 14 once all four were (three a Stormbringer), 2 left circling |
| The furthest a fighter went in a scramble | 4,541, and 6,676 after the log's "back" | 2,571 and 2,433 |
| Fighter-seconds beyond 3,000 from the place | 203 in the scrambles, the chase after them besides | 0 |
| Fighters lost | 10, every one beyond 3,000 | 0 (the 14 went with the side when our commander fell at 28:38) |
| His aircraft down within 3,000 of the place | 0 | 8 of 8 Stormbringers |
| Ours destroyed by his aircraft | a Blitz 5,113 from the place | an extractor 3,634 from the place (16:40, a lone bomber, before the first raid) |

**The two raids, second by second (the tape).** Under the patrol stood 44 solar collectors (53 at the second raid),
four vehicle plants, the aircraft plant, four Nettles and five Sentries.

| | Raid 1 (17:28) | Raid 2 (25:31) |
|---|---|---|
| The lead bomber first in our sight | 4,546 from the place | 4,144 |
| The other three in sight | 3 to 4 s later, 4,580 to 4,640 out | 3 to 4 s later, 4,310 to 4,370 out |
| The fighters' first hit | 6 s after first sight (the lead 3,009 out), with weight from 8 s (286 that second, the lead 2,489 out) | 7 s (400 that second, the lead 2,335 out) |
| The four shot down (health at nothing), after first sight | 10, 12, 15 and 16 s | 10, 13, 15 and 16 s |
| The nearest a bomber came to the place while it flew | 1,932 | 1,678 |
| Hits on anything of ours but the fighters, nearest to the place | 3,583 (the armies' fight) | 2,691 (the armies' fight) |
| Damage to the fighters | 0 | 0 |
| All fighters within 1,000 of the place again | 21 s after first sight | 21 s |

The fourth bomber of each raid was still falling when the lane said "all back"; it struck the ground 1,441 and 854
from the place. The kills are the fighters' by the tape's places (a fighter within 35 to 260 of each bomber as its
health fell, the base's Nettles 1,400 and more away); the tape names no attacker for an aircraft.

**What the game showed.** (1) *In time, in a game:* both raids were stopped before a bomb fell on what the patrol
covers, by twelve Falcons flying the conversion from the loop, with sixteen seconds from first sight to the last
bomber shot down. (2) *The leash and the way back:* no fighter beyond 2,600 from the place, all back over it in
21 s, twice. (3) *Threat, not presence:* from 16:12 to 16:48 one Stormbringer flew 3,100 to 4,500 from the place in
our sight, the scene that took all fourteen out in the first game, and no fighter left the loop. (4) *Only the
fighters needed:* three, then twelve, two circling. (5) *The limit of 6.4 and 8.2, as written:* that lone bomber
destroyed an extractor 3,634 from the place and was not answered; what lies further than the guard from the
place is not the patrol's.

**What it did not show.**
- *18 seconds or 24.* A Stormbringer flies 250 a second, so 18 s is 4,500 out, and the lead bombers were first
  seen 4,546 and 4,144 out: our sight set the start, not the time bound, and both scrambles would have begun in the
  same second at 18. His 32 Falcons were never in our sight or radar while the patrol flew (28,400 fighter-seconds,
  all of them 4,100 or more from the place, nine in ten 5,200 to 7,000). Five times, for two to four seconds, one
  to three of them were on a course to within 1,500 of the place and under 24 s away by the tape's second-to-second
  places, 5,700 to 6,000 out: outside
  18 s's 5,200 for a fighter, inside 24 s's 6,900. With a radar reaching there those are what 24 would have had to
  judge and 18 not; whether so short a course holds the track's second is not known. The price of 24 has a shape
  now and still no number.
- *His fighters coming out and turning back* (the first game's second fault): they did not come. Bench only (8.2).
- *Finding 6 of 8.3* (points off the map): the place lay 644 from the map's edge, the loop flew, and both raids
  came from inland.

**Finding 2 of 8.3, seen.** The log, which is what the player's report carries, said "scrambled 3 of its fighters at
1 aircraft of his" and "all back over energy_field after 19 s" for a raid of four answered by twelve, twice, and
says nowhere what was shot down. The picture's line was right each second ("scrambled at 4 aircraft ... 12 flying to
meet them"), and the player's debrief had it right from there ("14/14 alive all game, scrambled correctly ...
twice").

**The game itself** (the player's debrief, `docs/briefs/inbox/1795014462-air-patrol-2.md`; not reviewed): the army
"never fought as one body", going into his Stouts and artillery "a slice at a time" from 12:54 until some 35 of his
vehicles overran home at 28:18; "the air half was free". Checked on the tape: he had produced 118,600 metal to our
89,700 by 27:00, and our commander fell at 28:38. The patrol's fourteen Falcons are 1,022 metal.
