# Jev (TypeSafe's System One model) as a player of the keyboard

What the pianist games (`docs/design/2026-09-21-pianist.md`, `docs/harness/jev.md`) found about how Jev answers a menu
over a picture of the game. Scope: model jev-1.13.0, the picture and menus of `crates/bot/src/brain/pianist/`, BARb easy
on Quicksilver Remake 1.24.

### K-jev-words-not-numbers
**Claim.** Jev does not count what it has against a plan and does not read a stall off numbers; a count or a stall changes
its answer only when the words beside the option say what the number means ("we have 20 constructors already: far too
many", "energy: STALLING").
**Status.** supported (2026-09-21)
**Evidence.** pianist-smoke-1: the lab chose a constructor at 0.5-0.9 thirty times running against "two constructors first,
then raiders", with `ours.constructors` in the picture rising from 0 to 30 and `soldiers: 0`; the probability drifted from
0.93 to 0.5 with the count and flipped only when metal read "full". Energy stood at 0 from 0:50 to the end with the line
reading "77 coming in, 77 going out: in balance" (the engine caps spending at income once the store is empty). pianist-smoke-2,
with the count in words in the option and the question: the lab switched to raiders at 3:10 with three constructors and to
line units at 6:03. pianist-smoke-3, with the energy state in the generator option's words: energy at 1293 of 1306 by 5:00.
**Would be wrong if.** A picture with the counts as numbers only produced the plan's switch at the right count.
**Used by.** H-HANDS-MENU.

### K-jev-a-lookup-table-counts
**Claim.** Jev follows a table keyed on the picture's own count words ("With 0, 1, 2 or 3 constructors: a construction
vehicle. With 4 or more constructors: a Stout") at 0.94-0.97 either side of the threshold, where the same cap as a
sentence ("constructors until five of them stand, and never a sixth") sits at 0.5 on both sides and an ordered list
("first X, then Y, then Z") loops on its first step at 1.00. The table's key must be the picture's word ("constructors",
not "construction vehicles": 0.69 for the wrong word at six standing), and a paragraph that names another actor's count
leaks into that actor's decompression (`paragraphs_about`: the plant's table naming "constructors" pulled the
constructors' job to help_factory once).
**Status.** supported (2026-09-25, offline against a recorded plant ask and a recorded commander ask, six pictures each)
**Evidence.** hold-1/hold-2 (Comet Catcher, the pianist alone): "first two Rovers" gave 6-9 Rovers, "until we have four
constructors" gave 18 and 46 constructors (armcv at 0.52-0.68 with "We have 5 constructors for 6 extractors" in the ask).
Offline on the recorded ask with the counts edited: the sentence form 0.52 / 0.85 / 0.39 / 0.80 armcv at 7 / 3 / 5 / 2
constructors; the table form 0.04 / 0.94 / 0.04 / 0.96; "NEVER with 5 or more" suppressed armcv at 2-3 constructors too
(0.06-0.07). The commander's opening as an ordered list ("an extractor at spot_45, an extractor at spot_50, one solar,
then the plant"): extractor at 1.00 with two, three and four extractors standing (hold-4: four extractors, the plant at
1:45); as a table keyed on `ours.extractors` and "We have N solar collectors": armsolar 0.76 at two extractors, armvp 0.92
at two extractors and one solar (hold-5 to hold-9: the plant standing at 0:23 every game).
**Would be wrong if.** A table keyed on a count the picture states as a number produced the switch at the wrong count in
a game, or the sentence form reached 0.9 on the same pictures.
**Used by.** run/packets/comet-west-hold-1.md (the plant's and the commander's tables); K-jev-words-not-numbers (refined:
numbers the packet enumerates are read; numbers compared to a threshold in prose are not).

### K-jev-split-vote
**Claim.** A Choice with several near-equivalent options (one per free spot) spreads its probability over them, and a
single alternative wins with a fraction of the total; the remedy is one option for the kind of action and a parallel
question for its parameter (the spot in `where`).
**Status.** supported (2026-09-21)
**Evidence.** pianist-smoke-2: with six `extractor_at_spot_N` options, constructors chose `assist_lab` at 0.20-0.47 over
extractors at 0.14-0.25 each (0.6 between them), 4 extractors at 5:00; pianist-smoke-3 with one `extractor` option and the
spot in `where`: 8 / 14 extractors at 4:00 / 6:00.
**Would be wrong if.** The one-option form chose extractors no more often on the same pictures.
**Used by.** H-HANDS-MENU.

### K-jev-answers-drift
**Claim.** The same state and questions on the same model version do not give the same answers two days apart; the top
choice can flip. Between calls minutes apart the wobble is a few hundredths.
**Status.** supported (2026-09-21)
**Evidence.** `experiments/jev/01`: `escort` 0.72 on 2026-09-19, `defend` 0.58 / `escort` 0.39 on 2026-09-21, jev-1.13.0
both days; scenarios 02 and 03 moved by 0.02-0.03.
**Would be wrong if.** A re-run reproduced the first day's numbers.
**Used by.** H-HANDS-SWITCH.

### K-jev-where-needs-its-premise
**Claim.** A parallel parameter question ("where, if the action needs a place; otherwise answer home") is answered with
the fallback nearly every time, since it cannot see which action the other question chose; a speculative question works
only with its premise stated ("suppose the group sends a scout: where should it look?"), one per kind of action.
**Status.** conjectured (2026-09-21; the per-action form is smoke-6's test)
**Evidence.** pianist-smoke-5: `where` answered `home` on all but a handful of 1,300 asks; scouts sent home, advances to
home, forty one-unit groups. TypeSafe's own fan-out pattern says to state each speculative premise explicitly.
**Would be wrong if.** The per-premise questions still answered the same fallback whatever the chosen action.
**Used by.** H-HANDS-MENU.

### K-jev-instructions-are-standing
**Claim.** The hands read the whole packet afresh at every ask with no memory of the last: a time-bound command in it
("go home now, then build a lab") is matched again each time, so two steps that both fit the moment alternate, and an
instruction written as a state ("the commander stays at home and builds the lab there") holds.
**Status.** supported (2026-09-21)
**Evidence.** pianist-player-1: under "commander: go home now and build a lab at home" the commander, asked every ten
seconds, chose `retreat_home` at 0.76 and 0.81 and `lab` at 0.52 and 0.65 in turn (2:20, 2:30, 2:40, 2:50), abandoning
the started lab each time; the decayed frames were reported as losses and the player diagnosed aircraft.
**Would be wrong if.** The same packet, re-asked, kept the commander on the started lab once it stood at home.
**Used by.** H-HANDS-STARTED; the player's role prompt (`strategist/player.md`: write states, not commands).

### K-hands-abandoned-frames-read-as-losses
**Claim.** A nanoframe its builder walks away from decays and arrives as `UnitDestroyed` with no attacker; counted as a
loss it misleads every reader (the traded line, the fights line, the territory's raided memory, the picture's notes).
**Status.** demonstrated (2026-09-21)
**Evidence.** pianist-player-1: "lost armlab to unseen at home x2", "lost armwin to unseen at home x7" and 500 metal
traded with no enemy within 1,800 of home before 6:04 (the truth file); the player's turns 4 and 5 diagnosed an air
raider and ordered anti-air. Fixed the same day: `Brain::abandoned` (a destroyed unit that was being built at the
last look, with no attacker) is accounted as abandoned everywhere.
**Would be wrong if.** The engine reported an attacker for decayed frames, or never destroyed them.
**Used by.** `brain/briefing.rs` `track_losses`, `pianist/mod.rs` (and `territory.rs` until it was deleted 2026-09-27).

### K-hands-advance-stopped-at-turret-reach
**Claim.** Under the pianist a group advancing with `fight_to` was committed to no turret, so the control lane stepped
every soldier out of the first turret's reach and held it at the edge while its way to the goal crossed one: an attack
on a base could not be carried out whatever Jev or the player said. Committed to everything (as the heuristic waves
are) the advance walks in; whether it should is the odds question the menu words put to Jev.
**Status.** demonstrated (2026-09-21; the fix's effect is pianist-player-3's to show)
**Evidence.** pianist-player-2: the ball of 82 stood 2,255 short of enemy_base from 21:00 to 22:40 with `continue`
(advance) answered on every ask; the record shows 275 `move` and 2 `fight` commands to its units in the stall minute;
the truth file puts a Guardian 1,002 from it and the nearest laser tower at 1,584.
**Would be wrong if.** The ball had walked in under the same commitment, or the flee steps had another source.
**Used by.** H-HANDS-GROUPS (`micro.rs` `note_commitments`).

### K-hands-hold-in-the-base-stepped-out
**Claim.** A pianist group that arrived at its `fight_to` goal was set to Hold, and a hold was priced against no
turret, so the ball that had fought its way into the enemy base stood at `enemy_base` stepping out of the towers'
reach; likewise an engaging group was never committed to the commander, so a ball outweighing the lone enemy
commander many times stepped back from it. Both are the commitment mapping, not Jev and not the player.
**Status.** demonstrated (2026-09-21; the fix's effect is pianist-player-5's to show)
**Evidence.** pianist-player-4: the picture's footwork line read "35 of its 35 soldiers are being held back by
their own footwork" on a group "holding" at enemy_base at 10:33-11:33, and "28 of its 34" on the same group
"attacking party_1 (1 armcom) at spot_19" at 12:09-12:30; the player named the first as the game-2 shape and
worked round it with a sweep of named spots.
**Would be wrong if.** The held-back counts came from the lethal-fight test rather than the unpriced-threat test
(the ball was at full strength and outgunned nothing there: the towers were the only threats).
**Used by.** H-HANDS-GROUPS (`micro.rs` `note_commitments`).

### K-hands-lane-untested-under-the-pianist
**Claim.** The control lane's rules were measured on the heuristic bot's Pawn raids (H-MICRO-LANE: micro-ab2, 3-7-2
against 0-12; H-MICRO-FAN: matt-fan3) and never apart from each other for focus and kite, and never at all under the
pianist, where the groups are Mace balls with different commitments. The lane models a unit as a point that moves at
full speed in any direction at once, and the protocol carries no turn rate or acceleration, so a unit re-stepped every
six frames turns rather than walks (the user, watching: "rapidly changing direction causes pawns to mill about rather
than decisively moving towards or away from a threat").
**Status.** conjecture (2026-09-21); the milling counters (H-HANDS-LANE) are the instrument, a raw-against-laned
batch the test.
**Evidence.** The heuristics ledger's status column for the four lane rules; the user's observation.
A second measurement (2026-09-25 late, hold-lane-on against hold-lane-off, the hold packet, 48 games an arm): every game
past minute 11 and half past minute 20 either way (mean 18.9 against 19.8 minutes, 15 against 17 timeouts, bodies of
34 against 27 soldiers at 20:00 in the games that got there), metal lost per killed 0.66 [0.57-0.76] against 0.68
[0.59-0.77], soldiers lost 4,099 against 4,622 metal a game: the lane neither helps nor hurts the trade under the hands
at 48 games (the floor is about 0.2 of ratio), and its costs in that regime are K-form-a-body-stationed-among-buildings-jams.
**Would be wrong if.** A raw ball traded worse than a laned one in a batch, or the milling counters showed path
close to net displacement under the flee.
**Measured 2026-09-25 late (packet-lane-on / -off, the pianist alone on a fixed packet, 24 games an arm):** the
laned hands traded 1.04 [0.80-1.36] metal lost per killed against 1.16 [0.99-1.41] raw, lost 15.5 soldiers a game
against 22.2, put 3.5% of their damage on their own side against 8.3%, stood 49 from the nearest friend at contact
against 25 with 15% on the line of fire against 45%; the milling counters 2.0 path over net, 1.2 reversals a claim.
Every game was lost either way at 13 minutes (the packet's play, not the lane's). The conjecture's raw-beats-laned
half is refuted at this size; the turn-rate half (a re-stepped unit turns rather than walks) was measured in the
chase and lead work (K-engine-a-short-move-order-brakes-the-unit) and the lane no longer re-steps.
**Used by.** H-HANDS-LANE.
**Amended 2026-09-25.** The four player games' shut-offs were read from the records (`docs/design/2026-09-25-formation-micro.md`):
the milling was real but small (path over net 1.0-1.1 in the samples; two units zigzagging within 50 elmos at
2v1-hard_aggressive 7:04), and the lane's real costs were the braking step (K-engine-a-short-move-order-brakes-the-unit),
focus taking fleeing units back the next tick, a Stop queued behind focus's Attack, and follow without a leash
(K-hands-follow-had-no-leash); most of what the player called "stepping back" was the hands' own fall_back.

### K-hands-follow-had-no-leash
**Claim.** An engaging group re-sent after its party every 2 s while the party stays in sight follows a retreating
party across the map into whatever waits there; the party must be leashed to where the engagement began.
**Status.** measured (2026-09-25) in 2v1-hard_aggressive: group F (11 soldiers) followed a party 2,000 elmos east
9:21-9:46 with the party 300-600 ahead the whole way; K (4) followed one 3,500 elmos 11:04-11:32 and died to Warriors
at 11:33-11:44; the player switched follow off for all at 11:44 ("suspect it re-sends groups after fleeing parties"),
rightly. Fixed: a ground group drawn more than 900 from where it engaged holds and the report says so
(`pianist/groups.rs` `FOLLOW_LEASH`); untested in a game.
**Evidence.** `run/matches/1790259885-2v1-hard_aggressive/00` (the main checkout): `jev-0.jsonl` group tasks and the
record's `cmd` rows (a Fight to the party's new centre every 2 s).
**Would be wrong if.** A leashed group let a party it outweighed escape a fight it was winning more often than it
saved one from a chase; the leash is then the player's to set.
**Used by.** H-HANDS-LANE (follow).

**Status (2026-09-28 late).** The leash and the follow re-send are deleted with the `follow` lane word (the user: a group chasing a Tick or Pawn through our base "just stops randomly and lets the intruder continue for another few seconds"; player-22: 489 leash lines, group_B held at 5:23 drawn 942). The form lane's contact and pursuit own the chase; a chase ends when Jev or the player ends it or the party is unseen 6 s. The 2v1-hard_aggressive cases this claim came from are now Jev's to decide from the picture.

### K-hands-far-places-never-on-the-menu
**Claim.** The picture's places were home, enemy_base, our spots, the ten nearest free spots, the six nearest of
theirs and three passages, so a place in the far half of the map was not on any `where` question and an instruction
naming it did nothing; a deep attack could not be ordered except through `enemy_base`. Named spots and marks fix
the reach; whether Jev picks a far place when told to is the next thing to see.
**Status.** demonstrated (2026-09-21)
**Evidence.** pianist-player-5: the player named spot_36 in four packets from 9:37 and wrote at 10:42 "Only nearby
places appear in the picture, so spot_36 was never on the menu and the ball kept re-picking the empty enemy_base",
and at 13:21 "I wish I could name an unexplored map cell as a destination"; the jev log's places carry no spot_36.
**Would be wrong if.** spot_36 had been unreachable on foot (it is listed as walkable in the map tool).
**Used by.** H-HANDS-NAMED-PLACES.

### K-hands-frame-placed-off-the-ordered-point
**Claim.** The engine places a building's frame where the site search settles, up to about a building's width from
the point ordered (200 elmos for a windmill beside a metal spot), so a started-build test that looks for the frame
within 200 of the ordered point misses it; the builder is then asked again, Jev answers the packet's word
("generator") over `continue`, and the hands order a second frame at the same point, which the engine places beside
the first. Each such answer abandons the last frame.
**Status.** demonstrated (2026-09-22)
**Evidence.** pianist-player-6: build orders to the commander at (3855, 2135) at 0:40, 0:50 and 1:00 created frames
at (4056, 2136), (4056, 2072) and (3992, 2136); the first two decayed at 1:20 and 1:29; the jev log has the
commander "building a armwin at spot_10" and answering `generator` at 0:50 and 1:00 with `continue` on offer. The
user, watching: "we are still leaving wind turbines 80% built and dying in the very early game".
**Would be wrong if.** The decayed frames had been abandoned for a threat (no party was within 800) or the engine
had refused the first site (it created a frame each time).
**Used by.** H-HANDS-STARTED.

### K-hands-spot-read-as-taken-by-its-neighbour
**Claim.** The picture described a metal spot by every building of ours within 350 of it, so a spot 300 from a taken
spot read "our extractor, our lab, our wind generators", the extractor menu (which offers only spots whose words
begin "free") never offered it, and it was never taken: spot_10, 150 from our start, in every player game.
**Status.** demonstrated (2026-09-22; the user, from the replay: "one of its starting mexes is never taken")
**Evidence.** Games 9, 11, 12, 13: spot_10 at (3864, 2088) never had an extractor; spot_12's extractor stands 300
from it; the picture at 1:00 of game 13 reads spot_10 as "our armmex (extractor), our armlab (lab), our armwin ...".
**Would be wrong if.** The engine had refused the site (no refusal was logged, and no order was ever given).
**Used by.** the picture's spot words: the extractor within the spot's radius takes it, the buildings beside it are
said beside it.

### K-hands-ball-chases-lone-raiders
**Claim.** With `engage` the only answer that goes at an enemy party, the hands send the whole group after any raider
at one of our extractors, and a group of ten to twenty-five never catches a lone Fav or Flash while a second raider
kills what the group left; nothing local reacts to a raider at a structure.
**Status.** demonstrated (2026-09-22, the user watching realtime-2: "a fair amount of poor control")
**Evidence.** realtime-2 (`run/matches/1790042330-realtime-2`): 18 engage picks, 11 of them the whole ball (10 to
25 units) after a single Fav, Stump or Beaver, one 2,362 elmos away; 10 units chased one Fav from 4:56 to 5:34
through four re-picks; at 8:27 to 8:38 a Fav sat on our extractor at B3 killing a windmill and an extractor while
the ball of 24 was 2,700 away after three Flashes; the game's fight ledger: a lab, 3 extractors and 4 windmills lost
to Favs, 12 windmills and 3 constructors to raiders in all.
**Would be wrong if.** The chases had been the player's instruction (the packets of realtime-2 said "engage raiders
in sight", with no size).
**Used by.** H-HANDS-DETACH (`send_against`); the player's prompt, "Defence is yours".

### K-hands-lab-ordered-three-ahead
**Claim.** Asked every three seconds while up to two orders waited, and told "constructors 0" while one was being
made and two waited, the hands ordered constructors until the count they were told changed; a whitelist that
changed meanwhile applied only after the waiting orders.
**Status.** demonstrated (2026-09-22, human-3)
**Evidence.** human-3 (`run/matches/1790046088-human-2`): lab asked at 1:06, 1:09, 1:12, each answered corck at
0.95 to 0.98 with the entry reading "constructors 0 (none)"; a fourth at 1:55; the `produce` of 2:13 (corak only)
took effect at 3:01 when the queue had drained; the lab died at 3:03.
**Would be wrong if.** The packet had asked for four constructors (it said "constructors until we have a couple").
**Used by.** H-HANDS-MENU (one order waiting at most; the counts include what is being made).

### K-hands-one-generator-option-was-wind
**Claim.** With one `generator` option chosen by the map's average wind, a packet asking for solar got wind, and
nothing the player wrote could get a generator that costs no energy to build while the store was empty.
**Status.** demonstrated (2026-09-22, human-4)
**Evidence.** human-4 (`run/matches/1790046706-human-2`): packets at 0:12, 1:49 and 2:03 say "solar generator";
the hands built wind at 1:56, 2:01, 2:26, 2:33; the opponent: "I built solar due to the low wind random chance".
**Would be wrong if.** The hands had chosen wind over an offered solar (none was offered).
**Used by.** H-HANDS-MENU (`wind_generator` and `solar_collector` as two options, each with its cost and draw).

### K-hands-idle-gap-per-building
**Claim.** A builder asked only when idle loses about 2.5 s per building to the ask, the answer and the walk to the
next site: a wind generator every 8.5 s where a player queueing a batch gets one every 6.
**Status.** demonstrated (2026-09-22, human-5 against Matt's replay)
**Evidence.** human-5 (`run/matches/1790047251-human-3`): wind generators created 1:10, 1:16, 1:18, 1:38, 1:45, 1:53,
2:02, 2:11, 2:18 (a solar at 1:27), 8.5 s apart on average; Matt's (`run/matches/1789930356-replay-player2-vs-medium`):
0:53, 0:59, 1:08, 1:13, 1:19, 1:24, 1:30 from one batch of orders at 0:53 to 1:04, 6 s apart (armwin build time
1600 at the commander's 300: 5.3 s).
**Would be wrong if.** Our generators had been placed farther apart than his (his batch stood within 200 of each
other; ours beside the builder, one reach away).
**Used by.** H-HANDS-QUEUE.

### K-hands-do-not-count-constructors
**Claim.** A packet's count ("one constructor first, then raiders") is not kept by the hands: asked every few
seconds with the count in words, they chose the constructor again while the first was still being made, and again
after it stood.
**Status.** demonstrated (2026-09-22, human-7; the same in human-3, human-5 and pianist-player-15)
**Evidence.** human-7 (`run/matches/1790049225-human-7`): the lab, allowed [corck, corak] and told "one constructor
first, then raiders (corak) and nothing else", chose corck at 0:54 (0.99), 0:59 (0.69, with "0 and 1 more being
made" in its words) and 1:24 (0.51); the first Grunt came at 2:08 against Matt's 1:12, and three constructors
building windmills emptied the store (energy 15 at 2:00, 0 at 3:00).
**Would be wrong if.** The packet had allowed a second constructor (it did not until the sixth raider).
**Used by.** H-HANDS-PRODUCE (a cap after a colon), the prompt ("the hands cannot count"), the brief's opening
packet lines.

### K-hands-grunts-die-away-from-the-commander
**Claim.** Early raiders of ours fight the human's Pawn raid in ones, twos and fives out at the extractors, 400 to
1,200 from the commander, and trade one for one at best; the commander, kept building at home by the packet and
offered no attack beyond 320, never adds its guns.
**Status.** demonstrated (2026-09-23, human-8; the user: "it let pawns drift too far out and didn't use its
commander to improve exchange rates early on")
**Evidence.** human-8 (`run/matches/1790049731-human-8`): groups A to F formed 1:33 to 3:22 as Grunts left the lab
while the ball held at spot_14; `send_against` two at 2:03, `engage` by a pair at 2:22; eight Grunts destroyed by
3:14 at 380, 447, 525, 539, 602, 715, 771 and 1,231 from the commander, in groups of 1, 1, 2, 2, 5, 5, 5; the
commander's first attack at 3:26.
**Would be wrong if.** The Grunts had died at home beside the commander (none did).
**Used by.** H-HANDS-GROUPS (newcomers walk to the largest group in our half), H-HANDS-COMMANDER-FIGHTS (a busy
party within 500), the brief's fight paragraph.

### K-hands-plant-deterred-by-the-draw-warning
**Claim.** A warning in a factory option's words ("the store empties unless generators come first") outweighs the
player's plan for the hands: with "three solar collectors, then the vehicle plant" as the instruction, two solars
standing, 890 metal banked and the plant affordable, the hands chose extractors and a third solar for a minute and
the plant came at 1:40 (comet-1). The words now state the store's fate over the build (how many seconds short, if
any) and the generator options carry the count of generators standing, since the hands do not count them either
(K-hands-do-not-count-constructors).
**Status.** observed (2026-09-23), one game; the changed words are unmeasured.
**Evidence.** `run/matches/1790051208-comet-1-easy/00/jev-0.jsonl`: the commander's asks 0:24 to 1:40 (plant at
0.08 at 0:24, extractor 0.38 against solar 0.31 at 0:34).
**Would be wrong if.** The plant still came a minute late with the new words.
**Used by.** `menu.rs` `build_draw_words`, the generator words (H-HANDS-MENU).

### K-hands-sequences-are-not-followed
**Claim.** The hands do not follow a sequence written in words: "an extractor at spot_45, then an extractor at spot_50,
then three solar collectors, then the vehicle plant" got four extractors and three solars and the plant at 1:40
(comet-1) and 1:45 (comet-2), with the plant affordable from 0:34 and the option's words saying the store covered
it; "one plant" got three (comet-2). The count of generators standing beside the option (added after comet-1) did
not change it. Each ask is judged on its own, and an extractor or a solar always reads well. The player's list is
now executed by the bot (`queue`, H-HANDS-SCRIPT) and the words are kept for what comes after.
**Status.** observed (2026-09-23), two games; the list is unmeasured.
**Evidence.** `run/matches/1790051208-comet-1-easy` and `1790051789-comet-2-medium`, `jev-0.jsonl`: the commander's
asks 0:24 to 1:45 (comet-2 at 0:43: extractor 0.44, solar 0.31, plant 0.14 with three solars standing).
**Would be wrong if.** A worded sequence were followed under a different phrasing, or the list were not.
**Used by.** the `queue` tool (`mcp.rs`, `menu.rs` `scripted_step`), the player's prompt.

### K-jev-never-is-not-heard
**Claim.** A prohibition in the packet ("never splits", "never sends detachments", "never advances to the place
called shelling") does not take the option off Jev's picks: the option's own words on the menu win, and the
forbidden action is chosen at 0.4-0.8 whenever its words fit the moment. A positive standing instruction is
followed; a negative one is not. The remedy is a removal (the option or place off that actor's menu), not a sentence.
**Status.** supported (2026-09-22, `docs/studies/jev-comet-series.md` §3.1); holds against the standing rules too: two phrasings of "a never-clause removes the choice" left the forbidden detachments at 21-26 % right and the forbidden shelling at 45-48 % against 38 % (`docs/studies/2026-09-22-jev-rules-ab.md`)
**Evidence.** While "never splits / never sends detachments / no detachments" was in force: comet-2 4 detachments in
40 group asks, comet-3 22 in 109, comet-5 7 in 48 (23:14-23:29: the player had just written "forbade detachments",
group_Z1 sent one every five seconds at 0.50-0.75). While "it never advances to the place called shelling" was in
force (comet-1 13:47) group_L advanced to shelling on 7 of 25 asks at 0.58-0.84. "The commander never chases" held
(0 of 113), where the menu's own words agree with the packet.
**Would be wrong if.** The same packets with the option's description rewritten to name the prohibition ("the
instructions forbid this group to split") produced no detachments; then the words, not the negation, were the cause.
**Used by.** nothing yet (the `forbid` proposal, study §8.1).

### K-hands-detachment-folds-back
**Claim.** A detachment born of `send_against` or `split` is asked on the next tick with `join_group_<parent>` on
its menu, and takes it at 0.55-0.75 while its party still stands: the raider is left alone and the order is undone
a second after it was given.
**Status.** demonstrated (2026-09-22, study §4a)
**Evidence.** comet-5: 26 of 58 detachments rejoined within 30 s (group_O and group_P after 1 s at 8:26 and 8:29,
group_Y after 1 s at 10:54); comet-3 15 of 39; comet-4 6 of 16; comet-2 3 of 5. The player's turn at 11:09 in
comet-5 read "10:49 group_A: send 4 soldiers as group_X against party_1; 10:51 group_X: join group_A".
**Would be wrong if.** The detachments that rejoined had already lost their party (the entries at the join read
"attacking party_1 at spot_36, for 1 s" with the party in sight).
**Used by.** nothing yet (study §8.2).

### K-hands-scout-at-the-guess-does-nothing
**Claim.** When a group stands on the enemy base guess and the base is unfound, `scout` with `where_scout` =
enemy_base is the hands' answer every ask and does nothing (a scout to within 600 of the group is dropped), while
the group's entry calls the guess "their_base"; the base is found only when the group happens to walk on.
**Status.** demonstrated (2026-09-22, study §4c)
**Evidence.** comet-4 8:30-10:00: fifteen `scout enemy_base` answers at 0.63-0.81 with `did` empty, the entry
"at their_base (G4)", the base line "not found; presumed at G4"; the real base at H1/H2 was found at 10:22. Across
the series `scout` was chosen 23 times in 1,161 offers and `radar_at` 3 in 1,092; the base was never found in
comet-2, comet-3 and comet-5, and the player's army figure was a third of the truth in every game past minute five.
**Would be wrong if.** The guess had been within 600 of the real base (it was 2,000 off).
**Used by.** nothing yet (study §8.4).

### K-hands-odds-ignore-health
**Claim.** The odds words on a builder's or group's entry price our side by definitions (`force_of` sums defs), so a
commander at 25 % reads "against this unit alone, we outweigh it" against a nine-unit party, and Jev attacks.
**Status.** demonstrated (2026-09-22, study §3.5, §4d)
**Evidence.** comet-4 16:47: health 25 %, eighteen hits that second, "we outweigh it"; attack 0.55, retreat_home
0.13; health 2 % at 16:49 and the game lost. The sweep's `contradicts` Noul flagged the moment at 0.84.
**Would be wrong if.** The pick had gone the same way with the odds line reading "it outweighs us".
**Used by.** nothing yet (study §8.3).

### K-jev-hindsight-contradicts-only
**Claim.** Put back to Jev in hindsight with the packet, the entry, the menu and the play, one question works:
"does the play go against the instructions for this actor" (single-hop over text it can see). Whether an option
was missing, whether the ask was pointless, whether a danger was foreseeable, whether another option fit better:
none separates (saturated, or a death detector, or flat).
**Status.** supported (2026-09-22, `run/jev_sweep.py`, study §7)
**Evidence.** 3,503 decisions, 7.2 M tokens, $0.30, 2.5 min. `contradicts`: 6 of 7 top flags right by hand and in
agreement with the mechanical "never" count. `missing_option`: 2 of 3 wrong (a plant building the one allowed unit
at 0.77). `better_option` at 0.6+ on 57-82 %. `blind` 0.71-0.84 when something died in the next 30 s, 0.07-0.09
otherwise. `pointless_ask` 0.33-0.47 on a holding group told to hold against 0.27-0.41 on the rest.
**Would be wrong if.** A reworded `missing_option` (naming the actor's paragraph only) reached the precision of
`contradicts`.
**Used by.** `run/jev_sweep.py` (the debrief should carry its top ten).

### K-policy-writes-legal-scripts-under-fire
**Claim.** Opus at low effort writes a Lua policy over the hands' picture that is legal every time (no option the
actor was not offered, no parse error), in one script of 100-150 lines at a game's start and amendments of one or
two handlers (3-42 lines) afterwards; the errors it makes are the picture's shape (a field taken for a list, a spot
named in code that the picture did not list), and it does not see a runtime error unless it is told.
**Status.** supported (2026-09-22, `docs/design/2026-09-22-policy-replay.md`)
**Evidence.** Offline replay of comet-5: 1,500 decisions over three runs, 0 illegal, 0 load errors; the one
runtime error came from the role text's own example and stood on 36 turns until the harness fed errors back.
policy-1-easy: 343 runs, 0 errors, 3 illegal orders (spot_25 not a place).
**Would be wrong if.** A medium or a human game produced parse errors or illegal orders at any rate.
**Used by.** H-HANDS-POLICY.

### K-policy-plays-easy-alone
**Claim.** The policy alone, without Jev, beats BARb easy on Comet Catcher: the player's turns read as a player's
(scout, sweep, recall, mass, one attack) and the script carried them.
**Status.** demonstrated once (2026-09-22, policy-1-easy: won in 13.9 min, 12 turns, median 7.1 s a turn)
**Evidence.** `run/matches/1790088367-policy-1-easy`; the ledger row.
**Would be wrong if.** A rerun lost, or medium showed the policy unable to keep up with raids between turns.
**Used by.** H-HANDS-POLICY.

### K-hands-roster-reaches-the-game
**Claim.** The game's build tree, read from the definitions, reaches 190 Armada and 187 Cortex units from the
commander, the largest single build list being 45 (the construction hovercraft) and a constructor bot's 30: far under
Jev's 255 options a question, so the whole build list can stand in the policy's state, while Jev's menu is kept to a
usual list of about fourteen or the player's whitelist (K-jev-split-vote). The game's own language file names every
reachable unit, so the glossary covers the roster with no gaps.
**Status.** counted (2026-09-22, the record header of comet-5; `language/en/units.json` of the checkout at 54199a0d);
its effect on play unmeasured.
**Evidence.** `docs/design/2026-09-22-full-roster.md`; the reachability count in the design's "What stands today".
**Would be wrong if.** Jev's picks over a fourteen-option builder menu were worse than over the old nine (the noop
and better_option Nouls of `run/jev_sweep.py` over a new game against a Comet game), or the policy's option table
made a turn's amendment slower.
**Used by.** H-HANDS-ROSTER.

### K-engine-aircraft-hunt-with-an-empty-queue
**Claim.** An aircraft with an empty command queue attacks the closest valid target within 1000 times its move state
(0 hold position, 1 manoeuvre, 2 roam) on its next slow update, and a fight order makes it bomb the closest valid
target about 500 times its move state ahead on its line, not the party the order was aimed at; a direct attack order
is kept until the target dies or is lost. So a Stop is no hold for an air group, and a fight order picks no target.
**Status.** read from the engine source (2026-09-22, `rts/Sim/Units/CommandAI/AirCAI.cpp`: `AUTO_GENERATE_ATTACK_ORDERS`,
`ExecuteFight`, `ExecuteAttack`); not yet seen in a game.
**Evidence.** `docs/studies/2026-09-22-micro-air-sea-review.md`, findings 3 and 5.
**Would be wrong if.** BAR's gadgets override the move state or the auto-attack for aircraft (its default aircraft
move state was not found in `luarules`), or held bombers on move state 0 still hunt.
**Used by.** H-HANDS-DOMAINS.

### K-hands-presumed-point-reads-as-a-lead
**Claim.** A named point in the picture ("base not found; presumed at G4", an `enemy_base` place) is taken by the
player as a lead however it is qualified: scouts go to it, the army waits for a sighting there, and no other place is
searched while it stands. Wrong by 2,000 elmos, it cost the bombers game its whole eight-minute wait and every
Comet game its scouting.
**Status.** demonstrated (2026-09-22): roster-1-bombers (the flight held at home 13:24-21:12 "for a confirmed
sighting from a Blink" while the Blinks flew to the guess), comet-1 to comet-5, policy-1 to -3, luna/terra-jev.
**Evidence.** the ledger rows of those games; the picture text at that date (`enemy.base`); the overview's own
warning "often wrong by 500 or more" did not move the player off it.
**Would be wrong if.** A player shown an area and a never-looked list scouts no better than one shown a point.
**Used by.** H-HANDS-ENEMY-EVIDENCE.

### K-hands-fillers-outlast-the-list
**Claim.** A builder on one of the bot's filler tasks (helping a factory above all) was never looked at again for its
list: the list's next step was ordered only when the builder was idle or its build in progress 60 % done, and a
guard order never ends. plan-1-bulldogs: the commander helped the vehicle plant from 3:46 to 14:24 (562 s) while the
player gave it lists at 5:30, 7:00, 8:30 and 10:36, none of which started; the picture showed the pending list beside
"helping lab_14835". The player's diagnosis at 9:51 ("a builder helping a factory that has nothing to build counts as
busy for good") was right, except that the factory need not be idle: any helping blocked the list.
**Status.** observed (2026-09-22), one game; fixed the same night (H-HANDS-SCRIPT: fillers give way at once), seen working in hands-1-bulldogs (ten list steps to helping builders).
**Evidence.** `run/matches/1790111510-plan-1-bulldogs/00`, `jev-0.jsonl` `state.actors.commander.doing` 4:00-14:24,
`strategist-0.jsonl` `queue` calls at turns 4-10.
**Would be wrong if.** The list had started and the builder walked back to help on its own.
**Used by.** H-HANDS-SCRIPT (`menu.rs`, the `ready` test).

### K-hands-party-handles-shuffled
**Claim.** Enemy parties were named by distance from home on every picture, so a name in the player's orders pointed
at another party by the next ask, and the `whom` choices carried no distance from the group and named a commander only
by its internal name. plan-1-bulldogs 21:58: the packet said every soldier of group_F attacks the enemy commander now;
at 21:59 the commander had been out of sight for a second and was no party, so the only `whom` choice was the raiders
at E4, and Jev chose attack_unit on it (0.66); at 22:01 both were offered as "3 unidentified at 333 from spot_44 (E4)"
and "1 armcom at spot_17 (H2)" and Jev chose the raiders again (attack_unit 0.74); the player's rewritten packet then
named "party_2", which the naming did not hold.
**Status.** observed (2026-09-22), one game; names held and the words changed the same night (H-HANDS-PARTY-NAMES); hands-1-bulldogs: names held (no same-member party renamed), the commander words not yet seen in play; replayed on plan-1's 22:01 moment the new words put the commander's party at 0.56 against 0.38 and the pick on it, where a commander-first rule sentence left it at 0.40 (`docs/studies/2026-09-22-jev-rules-ab.md`).
**Evidence.** `run/matches/1790111510-plan-1-bulldogs/00`, `jev-0.jsonl` calls 21:58-22:10 (`questions.group_F.whom`,
`answers`, `played`), `strategist-0.jsonl` turns 52-53.
**Would be wrong if.** Jev chose the commander's party with the distance and the commander's words in front of it as
often as it chose the raiders without them; hands-1-bulldogs is the first look.
**Used by.** H-HANDS-PARTY-NAMES (`picture.rs` `enemy_parties`, `menu.rs` `party_words`).

### K-jev-rules-fight-in-sight
**Claim.** A sentence in the standing rules, "an enemy party in sight within 600 of a group that outweighs it is
fought now, whatever the instructions call it", turns a share of the holds beside a base under attack into engage:
on forty recorded moments where a group held with a party in sight within 800 and one of our buildings beside it died
in the next 30 s, the right pick rose from 2 % to 28-32 %, and from 1/25 to 8-9/25 where the odds words said we
outweigh every party in sight; forty raider-at-our-extractor holds went from 2 % to 18-22 %. Two of forty ordinary
decisions changed. Where some party outweighs us the rule hardly moves the answer.
**Status.** measured offline (2026-09-22 night, `run/jev_ab.py`, two runs agreeing); in the rules from this commit;
unmeasured in play.
**Evidence.** `docs/studies/data/jev-ab-2026-09-22-rules-2.jsonl`, variants home/home2 against base.
**Would be wrong if.** Games with the rule showed the same unanswered-raider count on the scorecard (hands-1: 21).
**Used by.** H-HANDS-RULES.

### K-jev-shelling-sentence-was-an-order
**Claim.** The rules' "`shelling` is where a weapon hitting us from out of sight likeliest stands, and advancing onto
it kills it" read to Jev as a standing order to go there: on forty recorded fight_to/move_to answers of shelling
where the packet named places for the group or forbade shelling, the named place was picked 38 % of the time under
the old sentence, 55 % with the last clause cut, and 80 % under "only where the weapon likeliest stands; a group
advances onto it only when the instructions say so, and otherwise goes to the place the instructions name". No
control decision changed.
**Status.** measured offline (2026-09-22 night); in the rules from this commit; unmeasured in play.
**Evidence.** `docs/studies/data/jev-ab-2026-09-22-rules-2.jsonl`, variants shelling/shelling2 against base.
**Would be wrong if.** Groups under fire stopped answering the unseen shooter when the packet did want it.
**Used by.** H-HANDS-RULES, H-HANDS-SHELLED.

### K-jev-hold-words-carry-the-cost
**Claim.** Telling Jev what a party in sight is destroying moves it little when said on the party's line (holds beside
a base under attack 7 to 12-17 % right on 29 recorded moments) and much when said on the hold option's own words
("Holding now leaves what party_85 is killing to die": 25 % alone, 52 % on top of the fight-in-sight rule, from 34 %
with the rule alone; raiders at our extractors 19 to 52 %). No control decision changed beyond the two the rule
alone changes. Jev weighs the words of the option it is about to pick over the words of the party it is not picking.
**Status.** measured offline (2026-09-22 night, `run/jev_ab.py` variants killing/hold_cost/home2_hold_cost); built
the same night (H-HANDS-PARTY-KILLING); seen again live 2026-09-26 (onepass-smoke-3 to -4: "Idle: plant (the metal
store: 1899 of 1900, full)" on the keep world lost to the Blitz the pre-pass rated 0.7 at 0.65 against 0.35 all game;
"plant idle, doing nothing, while the metal store reads 1899 of 1900 stored (full ...)" plus "an idle factory with
metal in the store is a cost, not a course" in the question, and the Blitz was picked at 3:04 and 48 Stouts after).
**Evidence.** `docs/studies/data/jev-ab-2026-09-22-shooting-2.jsonl`; `docs/studies/2026-09-22-jev-rules-ab.md`
second round.
**Would be wrong if.** In play, groups engaged parties that outweigh them because the hold words named a loss; the
odds words stay on the engage line and the replay showed no such flips among the controls.
**Used by.** H-HANDS-PARTY-KILLING.

### K-hands-air-hold-cancels-the-strike
**Claim.** An air group's hold (a move to the group's centre plus move state 0 per unit), re-issued every lane tick
and whenever a followed target leaves sight, cancels the attack orders the same group was given: bombers ordered
onto a commander turn back within a second, circle over the enemy base under its anti-air and never drop.
**Status.** demonstrated (2026-09-22, evidence-1-bombers): 233 Stormbringers built, 228 dead (214 to attackers out
of our sight), the enemy commander never under 93 % health; the record's command lines give the bombers 220 attack
orders and about 4,000 move and move-state orders over the game, 1,161 move-state orders in minutes 30-35 alone;
the player ordered `attack_unit` on the commander eleven times ("the bombers are on top of their commander").
**Evidence.** `run/matches/1790102056-evidence-1-bombers/00/record-0.jsonl` (`cmd` lines), `truth-0.jsonl`
(commander health), the ledger row.
**Would be wrong if.** The move orders came from the player's own packet (they did not: it ordered attack_unit and
nothing else at the end) or the bombers dropped and missed (the commander's health never moved).
**Used by.** H-HANDS-DOMAINS (to fix: an air group with a target keeps its attack order until the target is dead or
lost for a long while; the hold is issued once, not every tick).

### K-hands-menu-verdicts-beat-the-packet
**Claim.** A judgement written into a menu option's words outweighs the player's packet when they disagree. The lab
menu said "We have 3 constructors already: enough for the spots we hold" beside the constructor option (a sentence
from pianist-smoke-1, when the hands built thirty); with the packet saying "two more constructors first, then
Blitzes" and the constructor offered, Jev chose a Blitz 16 asks of 17 (escalate-5, 4:53-8:00), and with only the
constructor allowed and 541-1,357 metal banked it chose "nothing" 12 asks running for 35 s (escalate-6, 5:19-5:52)
until the player relented. The menu's words state observations; the packet judges.
**Status.** supported (2026-09-22 late): two games with the verdict, one without. With the sentence reduced to the counts (escalate-7) the same plant chose the constructor 8 asks of 13 when offered and the packet's asks were met (four constructors by 3:19, six by 6:20); the game was won.
**Evidence.** `run/matches/1790127241-escalate-5-hard-aggressive/00/jev-0.jsonl` and `.../1790129025-escalate-6-hard-aggressive/00/jev-0.jsonl`:
the plant's `played` entries (options, choice, probability) in those windows; `strategist-0.jsonl` the packets.
**Would be wrong if.** With the verdict removed the plant still chose Blitzes over an asked-for constructor at the
same rate, which would put the bias in the model or in the "nothing" option's words rather than in the sentence.
**Used by.** H-HANDS-MENU (the count beside a constructor option is facts only); the tempo-not-rules principle
(docs/README.md: the harness states observations only).

### K-hands-standing-task-outlives-the-packet
**Claim.** A task set under one packet survives the next: the switch margin (H-HANDS-SWITCH, 0.15 over `continue`)
applied to every busy actor whatever the packet's age, so a group's walk or a builder's unstarted build outlived the
packet that should have ended it until the player's words tipped the probability. Escalate-3 (8:31, "the hands sent
it back home"; 16:05, a walk west into beamer fire kept after a rewrite until the packet named it "wrong and
abandoned", which took effect in one turn), escalate-5 (6:53, five Blitzes lost to a walk two packets had ended),
escalate-7 (6:53 the same; 8:19-8:42 two constructors held a refused nano site through three new lists, the build
never started and never displaced). The words that worked each time were a statement that the old task was over:
what the hands lacked was that statement from the harness itself.
**Status.** observed (2026-09-23), four games. The fix (a changed packet asks every older task afresh with no margin;
a newer list displaces an unstarted build; a refused site is kept out) in fixes-1: courses kept by the margin fell from
5-7 % of busy asks (7-10 % for groups) to 4 % (5 %), so the margin was a small part of it; the rest is Jev choosing
`continue` itself, and the `shelling` place still held a group until the player dropped the word (15:33).
**Evidence.** `run/matches/1790124804-escalate-3-hard/00`, `.../1790127241-escalate-5-hard-aggressive/00`,
`.../1790132331-escalate-7-hard-aggressive/00`: the player's notes at the times above; the jev log's `kept` counts;
escalate-7's constructor_5134 picture entries 7:57-8:49 ("about to build a Construction Turret at 330 from spot_28",
the engine's refusal at 8:19, the same order at 8:19 again).
**Would be wrong if.** With the margin lifted on a packet change the groups still kept their walks at the same rate,
which would put the cause in Jev's reading of the packet rather than in the margin.
**Used by.** H-HANDS-SWITCH, H-HANDS-SCRIPT, H-HANDS-REFUSED (amended 2026-09-23).

### K-hands-a-standing-sentence-builds-many
**Claim.** A sentence in the packet about one building ("An Advanced Vehicle Plant goes up at west_yard; free
constructors at home help it") is read by every free builder asked under it as a reason to build one: in fixes-1 the
player asked one tier-2 plant by list (west_yard, 9:34) and two more came off the free menu within 30 s (9:48 p 0.46,
10:18 p 0.29, from constructors whose menus offered the plant among forty options with no word that one was already
under way); escalate-3 and -7 each got four plants the same way. The menu's words carry the count of the type and
nothing about the one being built, so nothing in the ask tells the hands that the sentence is already being obeyed.
**Status.** observed (2026-09-23), four games; with the under-way words a second tier-2 plant was still started 52 s after the first (concurrent-1: the words state, the choice stays the hands' and the packet's). In schedule-1 a constructor queued a second tier-2 plant off the free menu 16 s after the first was ordered by list, and its option carried no under-way words at all: the branch that names an ordered, unstarted factory read the pianist through `self.pianist`, which is taken out of the brain while the menus are built, so it never fired in any game (only the being-built branch did). Fixed 2026-09-23 03:00 (the pianist is passed in); untried since.
**Evidence.** `run/matches/1790137350-fixes-1-hard-aggressive/00/jev-0.jsonl` (the three armavp picks and their options), `strategist-0.jsonl` (the packets at
8:53, 9:22, 9:34), the record's armavp created events (9:54, 10:20, 10:21 by three builders).
**Would be wrong if.** With the under-way words on the option the free builders still picked the plant at the same
rate, which would put the cause in the packet's wording rather than in the option's.
**Used by.** H-HANDS-MENU (amended), H-ECO-YARD-LANE (pending orders' lanes).

### K-hands-shelling-place-led-groups-under-fire
**Claim.** A place whose words recommend an action is taken as an order by every group asked about it: the `shelling`
place said "advancing a group onto it (fight_to) kills it, a group that stays where it is keeps being hit", and
groups walked onto the estimate, under fire from a shooter they could not see, toward a point that moved with each
hit, until the player rewrote the packet or dropped the word: escalate-5 (12:29, 386 hits in 20 s at spot_23),
escalate-7 (12:29, "the hands kept picking the vanishing `shelling` place; its own text recommends advancing onto
it"), fixes-1 (15:33, group_T stood at spot_8 under fire for two minutes and moved the turn the word was dropped).
The place is right to exist (the user, pianist-player-6: the army had no way to push out to what shelled it); the
recommendation in its words was wrong, since whether to go, to see it first or to leave its reach is the packet's.
**Status.** observed (2026-09-23), three games; with the observation-only words (schedule-1-hard-aggressive) the place stood in 230 of 612 calls and a group advanced onto it once, on the player's own route.
**Evidence.** The player's notes at the times above; `crates/bot/src/brain/pianist/picture.rs` (the old words, in
git before ce6a358's successor).
**Would be wrong if.** With observation-only words the groups still walked onto the estimate at the same rate, which
would put the cause in the packet's own sentences about shelling.
**Used by.** H-HANDS-SHELLED (amended); the tempo-not-rules principle (the harness states observations only).

### K-hands-review-periods-hid-the-turn
**Claim.** Before 2026-09-23 an actor reached a Jev call only on its review period (10 s busy, 5 s holding) or when
an enemy party first came within 600 of it; a new packet marked nothing due, and the events of the ticks between
calls were not seen (the under-fire set was the current tick's events alone, one tick in ten). So the player's
retreat in fixes-1 reached the group six seconds after it was written (the user, watching). What withholding
saved, measured on fixes-1 and escalate-7: 40-43 calls a minute, 11-12k input tokens a call of which 3.5-4.6k is the
picture and about 800-900 each question; one of four or five groups asked per call; 8.3-8.6M input tokens a game,
$0.35-0.36 at Jev's price. Asking every group every call would about double that; latency was 244 ms under 9k
tokens and 326 ms over 15k, the largest call 29k of a 64k window.
**Status.** measured (2026-09-23) from `run/matches/1790137350-fixes-1-hard-aggressive/00/jev-0.jsonl` and escalate-7's; the event-aware scheduler
(H-HANDS-SCHEDULE) verified in schedule-1-hard-aggressive (won 18.9): a group's next ask after losing a unit came 0.9 s later (median, max 2.9)
against 5.7 s (max 9.8) before, every group but one was asked on the call after each of 25 packets (82 of 83; before 39 of 169), and the cost did
not move (612 calls, 8.05M input tokens, $0.34 against 686, 8.64M, $0.36: fewer calls, each a little larger). The packet call was the new peak:
28-41 questions, up to 47k of the 64k window with 13 actors; since 2026-09-23 04:00 the builders go on the call after the groups' (untried).
**Evidence.** The jev logs' `usage`, `questions` and `ms` per call; the player's notes in fixes-1.
**Would be wrong if.** The scheduler's extra calls doubled the tokens a game without the front-line groups answering
within two seconds of a hit or a packet.
**Used by.** H-HANDS-SCHEDULE.

### K-hands-cancel-left-the-build-running
**Claim.** Cancelling a builder's list (`queue` with `null`) removed the list and nothing else: a build the builder
had started ran on to completion, and the player had no order that dropped it (escalate-1: "a way to cancel a
constructor's queued plant that actually stops it, rather than the hands finishing it at spot_7"; comet-5, 24:11: the
commander finished a stale solar list).
**Status.** observed (2026-09-23), two games; the `stop` step (a list beginning with `stop`, or the bare word, stops
the builder and drops its task before the list) used once in held-1 (2:58): the builder was idle within 2 s and its
extractor frame was destroyed; the list's next step, a turret, was then not built because a Pawn within the started-alarm
reach took the builder off its list for the menu (H-HANDS-SCRIPT's threatened rule).
**Evidence.** `pianist/mod.rs`, the list loop: `scripts.remove` alone on `None`; the two debriefs in `docs/briefs/inbox`.
**Would be wrong if.** The player used `stop` where `null` was meant and lost frames it wanted finished.
**Used by.** H-HANDS-SCRIPT.

### K-hands-shelling-named-a-seen-shooter
**Claim.** Hits from out of sight were laid to an unknown shooter even when a unit of the weapon's type had just been
seen within the weapon's reach of the hits: quick-1 (Quicksilver, 4:56) woke the player for "shelling" by the enemy
commander, seen 7 s before at the very place the estimate named (the player: "the 'shelling' wake was stale");
escalate-2 had two alarms (9:06, 10:06) that put enemy turrets far out of range of the group they said was hit. The
weapon's definition name carries the shooter's unit type, and the brain remembers enemy buildings, soldiers (90 s)
and the commander with their places, so the hits can be laid to the nearest such unit within reach.
**Status.** observed (2026-09-23), two games; the attribution in play in shell-1: 469 of 845 shelling calls named a seen unit, most of them Rovers (range 180) seen 0 s before, which says the shelling place fires for any out-of-sight hit, a raider's included; a range floor is the open candidate.
**Evidence.** quick-1's strategist log, the turn at 4:50 and the jev call at 4:52 (`commander: seen at shelling (F6)
0:07 ago`); escalate-2's debrief.
**Would be wrong if.** The hits were laid to a seen unit of the type that was not the shooter (two of the type about),
and a group walked to the wrong one.
**Used by.** H-HANDS-SHELLED.

### K-hands-packet-asked-over-a-started-frame
**Claim.** The event-aware scheduler's packet rule (every builder not on a list is asked at the next call) reached
builders standing on a started build, which H-HANDS-STARTED keeps off the menu until it is done: shell-1's tier-2
plant builder, its list ended (`armavp avp_yard, assist`), was put on the call by the packets of 18:19 and 21:28 at
0 % of a metal-starved frame, `busy` false because the task predated the packet, and Jev chose `assist_lab` both
times (0.41 and above the `continue` at 0 %: "nothing wasted"); the frames decayed and the game was lost at 33:27
with a 4:1 economy lead and no tier 2. Four more frames went the same way to the player's `stop` and to new lists.
**Status.** observed (2026-09-23), one game, six frames; the fix (a started, unthreatened build stays off the
packet's and the events' call) verified in started-1: over 52 packet calls the only builders asked while building were
under fire.
**Evidence.** `run/matches/1790172269-shell-1-hard-aggressive/00`: the jev log's `played` at 1100 s and 1288 s
(`constructor_28184`, `assist_lab`, source jev, busy false), the record's `created` and `destroyed` events for the
six `armavp` frames, the player's notes at 20:02 and 24:57.
**Would be wrong if.** A builder on a frame that will never finish (metal starved for minutes) needed the packet to
move it, and the player had no other way: it has `queue` with `stop`.
**Used by.** H-HANDS-STARTED, H-HANDS-SCHEDULE.

### K-hands-a-call-can-overrun-the-window
**Claim.** A Jev call's size is the questions': about 1,400 characters a question at the median and up to 2,900 for a
group with many places and parties in reach, so a call asking 45 actors' worth of questions ran to 135k characters
and 62k tokens, past the 64k window: shell-1 lost two calls to `max_tokens_exceeded` (11:34, 22:28) and sent 69 calls
over 40k tokens and 20 over 50k; the game's Jev bill was $1.00 against $0.18-0.36 for the shorter games.
**Status.** measured (2026-09-23) from shell-1's jev log; the question budget (80k characters a call, the rest
deferred half a second) verified in started-1: 1,370 calls, none failed, the largest 47k tokens.
**Evidence.** `run/matches/1790172269-shell-1-hard-aggressive/00/jev-0.jsonl`: the two `error` entries and the
`usage` and `questions` of the calls around them.
**Would be wrong if.** Deferred actors waited more than a call or two, or the budget cut a group's questions while an
enemy stood on it (groups are kept first).
**Used by.** H-HANDS-SCHEDULE.

### K-hands-place-lists-were-the-bill
**Claim.** In a 30-minute hard_aggressive game (1,350-1,370 calls, 24-25M input tokens, about a dollar) the characters
sent went: 23 % to the two lists of all 55 places under every group question (`where`, used in 8 % of the group
plays, and `where_scout`, used twice a game), 21 % to the places block (54 entries of 168 characters every call,
about one a call named in the instructions), 18 % to the builder questions (24 options of 200-950 characters, a fifth
of them the economy sentences repeated in each), 15 % to the actors (12 points of it actors not asked), 9 % to the
group decisions themselves, 4.5 % to the enemy block and 3.3 % to the rules; 17 % of the tokens went to calls asking
only holding groups with nothing near, and 4-6 % to asks whose question, entry and instructions differed from the
previous only in numbers (the factory, 90-98 % the same answer). Exact repeats were 7-18 a game: every entry carries
a clock or a distance.
**Status.** measured (2026-09-23) on started-1 and shell-1 (`scratchpad/jev_waste.py`); the diet (H-HANDS-DIET)
verified in diet-1 (normal: 10.3k tokens a call, won) and diet-2 (lean: 9.25k a call, $0.015 a minute against $0.035,
won 15.5): the place lists and the `where_scout` question were the bill they measured as; what remains is the
builder options (34 % at lean), the places block (22 %: the asked builders' reach and the spots the enemy block
names) and the `where` lists the instructions fill (31 places).
**Evidence.** `run/matches/1790174701-started-1-hard-aggressive/00/jev-0.jsonl`,
`run/matches/1790172269-shell-1-hard-aggressive/00/jev-0.jsonl`: `usage`, `state`, `questions`, `played` per call.
**Would be wrong if.** The pruned `where` list left out the place the instructions or the fight called for, a brief
actor entry lost a join or a detachment its words carried, or the replayed factory answers flattened a mix.
**Used by.** H-HANDS-DIET, H-HANDS-SCHEDULE.

### K-arena-player-turns-were-held-without-penalty
**Claim.** The arena's player games up to pace-1 (2026-09-23) were played with the game held during every turn and
no think penalty, so the player answered a frozen picture and its orders landed at the frame it was woken on; in the
eleven real-time games on disk the player was inside a turn 27-52 % of the game's time, 55 % of the critical events
(an own extractor, soldier or commander destroyed) fell inside a turn, and the next turn came a median 6 s (p90 20 s)
after them. The arena results overstate a live game's by that latency. The penalty existed but held back only the
commander mode's outputs, so it would not have applied to the player's packet.
**Status.** measured (2026-09-23) on the real-time records (`run/matches/*human-*`, `*realtime-*`) and the arena
code. The corrected penalty (every turn output held back, default 1 with `--player`) verified live in penalty-1
(turn 1's instructions reached the hands at frame 705, due 535) and played in penalty-1..4 (Jev medium/low, Lua
medium/low, hard_aggressive): 0-4 at 25-30 min, 300-770 game seconds held per game, turn lengths unchanged by the
penalty (medium 7.6 s, low 6.0 s median). One game each, and all four debriefs blame unscouted pushes into tier-2
artillery, so the penalty's own weight at this tier is not separated yet; the Lua side's opening suffered most
(penalty-3: 4 extractors at 5:00, "amendments land a turn late" on top of the runtime's own one-turn lag).
**Evidence.** `crates/bot/src/strategist/shared.rs` `hold_for_turn` before 2026-09-23; the `batch.json` of every
player game (`"think_penalty": null`); the scratchpad measurement over the real-time strategist and record logs.
**Would be wrong if.** The penalised games' results matched the unpenalised ones: then the latency did not matter at
this tier.
**Used by.** the arena's `--player` default; H-HANDS-SCHEDULE (the hands act inside the latency).

### K-hands-moho-over-a-frame-was-refused
**Claim.** A tier-2 extractor ordered at a spot where a tier-2 frame is already going up, or where the tier-2
extractor already stands, is refused by the site search (the frame is not a basic extractor of ours to upgrade, and
the engine will not put a second frame on it), and the hands retried such spots every two seconds: the list and the
hands both named spots by the player's words, not by what stood there.
**Status.** measured (2026-09-23, pace-1-hard-aggressive, 17:31-26:47): 18 `no_site` events, all `armmoho`, 23
"armmoho order ... never started"; over a frame at spot_45 (18:05, the frame 17:31-19:08), spot_50 (19:09, 19:40,
20:36; the frame 18:28-20:42), spot_54 (25:24); over a finished moho at spot_36 (five times 23:25-23:34) and spot_30
(24:17, 24:37, 25:47); the player's lists named spot_36 twice and spot_50 after its upgrade, `queue` answered "done
in order" each time. Eight mohos stood in nine minutes from two advanced constructors, one frame at a time. The
three winning games at this tier built no moho (0 `no_site`). Fixed in `hands.rs` (H-HANDS-UPGRADE: repair the
frame, else the nearest extractor still to upgrade): upgrade-2 ordered three moho frames, 0 refused sites, four "help
finish the armmoho already under way" plays.
**Evidence.** `run/matches/1790188079-pace-1-hard-aggressive/00/record-0.jsonl` (`no_site` events), `bot.log`,
`jev-0.jsonl` (the `where` answers); the Opus review of the game (2026-09-23 evening).
**Would be wrong if.** A build order at a same-type frame's position were accepted and continued the frame: then the
repair route is a detour, not a need.
**Used by.** H-HANDS-UPGRADE.

### K-hands-lane-from-the-centre-counted-behind-the-plant
**Claim.** The exit lane measured from a factory's centre (a strip of half-width 96 from the centre out through the
front) contained points up to 96 elmos behind the factory, so a unit wedged against its back wall counted as
standing in the lane, and the `yard` wake and the factory's `yard` words said the lane was blocked while units left
through the front every 10-25 s.
**Status.** measured (2026-09-23, pace-1): constructor_9823 at (1400-1423, 3600-3631), between a solar and the north
wall of the plant at (1424, 3694) with a 12x12 footprint, from 23:46 until the plant died at 29:56 (364 stuck
seconds, 124 failed moves); three `yard` wakes (10:32, 24:28, 25:30) said "nothing of ours stands in the lane";
the plant built a Stout every 10-25 s throughout. The lane now starts at the front face (`yards.rs` `lane_at`);
upgrade-2: no yard wake, and a constructor stuck at yard2 said so in its `doing`.
**Evidence.** The record's `move_failed` events and the strategist log's wake prompts; the Opus review.
**Would be wrong if.** The engine's `SendToEmptySpot` sent units out of any side: then a lane behind is a lane too.
**Used by.** H-ECO-YARD-LANE, H-HANDS-STUCK-WORDS.

### K-hands-a-diverted-list-step-was-lost
**Claim.** A list step was popped from the builder's list when it was ordered; when the hands then diverted the
builder (a retreat home under fire, an attack), the step's task was replaced and the list resumed at the following
step, so the diverted step was never done.
**Status.** measured (2026-09-23, pace-1): constructor_16966 on `extractor spot_7` at 18:22, sent home by the hands at
18:40, its list going on to `armllt spot_7`; no extractor at spot_7 for the rest of the game. Fixed (`Pianist::list_steps`,
the step back at the front when the builder is diverted from the task it became): upgrade-2 returned 14 steps
(`extractor spot_22`, `armllt spot_59`, `armllt spot_38`, `assist` eleven times).
**Evidence.** `run/matches/1790188079-pace-1-hard-aggressive/00/jev-0.jsonl` (`did` lines of constructor_16966); the
Opus review.
**Would be wrong if.** The player meant a diversion to cancel the step: then the step should be dropped and said.
**Used by.** H-HANDS-SCRIPT.

### K-hands-needs-player-was-noise
**Claim.** Jev's `needs_player` Noul carried no information: a bell around 0.77 (0.72 to 0.83), 99 % of calls at or
above 0.5, with the 0.8 wake threshold inside its mode; its crossings selected noise, not events.
**Status.** measured (2026-09-23, upgrade-2, penalty-1, penalty-2): medians 0.77, 0.76, 0.77; at or above 0.8 on 19,
25 and 28 % of calls; within 10 s of a loss of ours 24 % (upgrade-2) and 35 % (penalty-2) crossed, against 10 and
6 % with no loss in 30 s; the wakes it produced (10, 10, 16 per game) came 2 to 4 s after the orders landed. The
study of 2026-09-22 (`docs/studies/jev-comet-series.md` h) had it at 0.6 or more on 78-96 % of calls. Retired.
**Evidence.** `run/wake_read.py` on the three matches (the histogram and the loss split).
**Would be wrong if.** A sharper question (one naming what the instructions cover) gave a bimodal answer; untried.
**Used by.** H-HANDS-LOSS-WAKE (its replacement).

### K-player-idle-after-the-flight-costs-more-than-the-flight
**Claim.** Under the think penalty the player is blind while its orders are on their way (41-44 % of game time), but
a loss of ours reached the next turn as late while the player was idle as during a flight, because a unit loss woke
nothing: only extractor losses, enemies near an extractor, a group starting an attack, shelling and the timer did.
**Status.** measured (2026-09-23): upgrade-2 (medium) a loss of ours to the next turn start 8.6 s median, p90 20.7,
9.3 s during a flight against 7.6 idle; penalty-2 (low) 5.1 s, 5.1 against 5.0; penalty-1 7.0, 8.5 against 6.0. Of
84 flights in upgrade-2, 45 had a loss inside, 9 woke a turn within 1 s of landing, 11 waited 7 to 37 s for the
timer. The player chose 40 to 60 s waits on 30 of 85 turns, one at 19:34 during the collapse (35 s asleep, nine
losses in the next flight).
**Evidence.** `run/wake_read.py run/matches/1790198489-upgrade-2-hard-aggressive/00` (and penalty-1, penalty-2).
**Would be wrong if.** The turns woken sooner did not change what the hands did: check the wake-1 game's losses
per flight and the turns' orders against upgrade-2.
**Check (2026-09-23, wake-1, won 18.5).** The schedule moved as designed: a loss of ours to the next turn 5.4 s median
(p90 10.9); flights with a loss inside were followed within 2.2 s median and never over 5 s; timer-only turns 4 of
100; 5.4 turns a minute at the same $0.11 a turn. Whether the sooner turns won the game is one game's evidence: the
economy and one committed push did, and group_C was still wiped at 11:34 with the wakes on time.
**Used by.** H-HANDS-LOSS-WAKE, H-WAKE-FLIGHT-REVIEW, H-WAKE-HOT-FLOOR.

### K-hands-carried-on-while-losing-in-flight
**Claim.** A group losing a member while the player's orders were on their way was asked again within about a second
and chose to carry on (continue, engage, attack, advance) almost every time; nothing told Jev that a fight going wrong
with the player out was its to end.
**Status.** measured (2026-09-23): upgrade-2, 30 such group moments, 20 asked before the landing, next ask 1.1 s
median; choices continue 2, engage 2, walk 2, attack 5, retreat 1, fall back home 1, never asked again 4. penalty-2:
41 moments, 31 before the landing, advance 8, continue 7, hold 5, engage 3, attack 6, retreat 1.
**Evidence.** the same script's "hands' next choice" table.
**Would be wrong if.** With the `player` line, the `losses` line and the fall-back rule Jev still continues: then the
rule must be mechanical (a losing group with the player out falls back without asking).
**Check (2026-09-23, wake-1).** It still continues: at the 24 in-flight loss moments the hands chose continue 9, attack 6,
walk 3, fall_back 1 (`fall_back` was offered on 393 asks and chosen 9 times in the game). The words did not hold; the
mechanical rule is the next step, for the user's decision.
**Used by.** H-HANDS-FALL-BACK.

### K-hands-the-bundle-at-the-wipe
**Claim.** At the decisive Jev call of wake-1 (11:23, group C's wipe) the fight-relevant content was under a fifth of
the bundle: of 12.6k characters of state, places were 6.3k (38 entries) and unasked actors 3.4k (15 of 16); the
`where` question repeated the places (1.9k) and the engage option restated the parties (in a 2.0k `do` question).
At 18:15 (28k tokens, 29 questions) seven `where` questions repeated the same place list, about 20k of 71k characters.
The lean diet was on: its "places in play" kept every spot named anywhere in the state text, and the enemy section
named 29 never-looked spots.
**Status.** measured (2026-09-23, `run/matches/1790212198-wake-1`, the section sizes by script). The cuts (H-HANDS-DIET,
2026-09-23) took the replayed in-flight moments from 9.8k to 8.0k tokens (an approximation of the cuts on recorded
states; the real bundle is measured in wake-2).
**Evidence.** the jev log's `state` and `questions` at frames 20370 and 32850.
**Would be wrong if.** The cut state answered worse on the control moments: check wake-2's ordinary decisions against
wake-1's with `run/jev_ab.py --control`.
**Check (2026-09-23, wake-2).** In the game: 7,861 input tokens a call median (10,296), 25 place entries (40), 10.1k
characters of state (14.3k); $0.31 for 24.8 min against $0.35 for 18.5. The bare-name `where` options were answered.
**Used by.** H-HANDS-DIET.

### K-hands-precedence-wording
**Claim.** Jev follows the packet over the picture unless the precedence is stated where the question is, and the
statement needs its boundary: without one, it falls back from fights we are winning.
**Status.** measured (2026-09-23, `run/jev_inflight_ab.py` on wake-1's 21 in-flight loss moments, three asks each):
as recorded, 1 of 21 fell back (the packet's "keeps on it" over "it outweighs us, killing 8 of our Blitz now", engage
0.45 to fall_back 0.34 at 11:23). With the precedence in the rules and the `do` question but no boundary, 18 of 21 fell
back, among them a 24-group that outweighed its party and had lost one, and a 45-group that outweighed its party
heavily and had lost five. With the boundary ("a few losses, or a noticeable share, against a party it outweighs are
the cost of fighting, even while that party is killing some of them"), 9 of 21: the wipe at 11:23 (0.97), the losses
to unseen fire at 13:44 (0.98) and 14:19 (0.47) fall back; the winning fights at 13:35, 15:04, 18:01 and 18:07 keep on;
still over-corrected at 13:23 (outweighs heavily, lost 5 of 45, killing 11 of ours: fall_back 0.53) and 12:02 (0.37 to
0.49), and at 18:15 the ball that had lost 16 of 39 fell back from their commander in sight, which the packet said to
attack. With the state cuts the same 9 of 21 at 8.0k tokens against 9.9k.
**Evidence.** the script's tables in this session; `docs/harness/jev.md`, the docs re-read ("literal reading").
**Would be wrong if.** In the check game the hands fall back from fights the player meant them to press (the 18:15
shape): then the packet needs a named way to override the rule ("even when losing a large share"), and the rules a
sentence that honours it.
**Check (2026-09-23, wake-2, lost 24.8).** Over-corrected live: 27 of 42 in-flight loss moments fell back (wake-1: 1
of 24) and 203 of 900 group choices were a fall-back or retreat, 27 of them with "we outweigh it" and no losses line.
Two harness defects of the option itself did much of it: a one-second hold at the front became "where it last
held", so `fall_back` arrived at once and the group stood to be asked again; and `retreat` and `fall_back` were both
offered to a group already walking back, which alternated them every ask (group_L, 17:09-17:39, at 0.92-0.96 each).
Fixed (a station is 15 s of holding; the option only when a party within reach outweighs the group or a tenth of its
metal went in 30 s, to a station farther from the party; nothing offered twice to a group already walking back): the
prompt's own share of the over-correction is measured in wake-3.
**Check (2026-09-23, wake-3, lost 20.8; the Opus review `docs/studies/2026-09-23-wake-3-jev-regressions.md`).** The
prompt's own share: at the replayed in-flight moments the precedence matched 17 of 18 recorded choices, and the words
"killing N of our Blitz" on a group's line (a party's kills anywhere) made groups that outweighed their party retreat
in 7 of 51 asks against 0 of 69 without the clause. The rest was mechanical: the station clock restarted on every
`hold` re-pick (fall_back offered on 2 of 724 asks, retreat taken instead, 73 of 724), and the player's packets
(60 in 17 minutes) sent walking-back groups forward every ask while the rule sent them back (61 swings). Fixed: the
hold clock kept, the group's line scoped to its own soldiers, an instruction after the last loss stands, the guide on
rewrites. Next check: wake-4.
**Used by.** H-HANDS-FALL-BACK.

### K-hands-commander-odds-by-metal
**Claim.** The force odds weigh a unit by its metal, and a commander's 2700 metal is the base it can build, not
the fighter it is: by the square law (sqrt of damage a second times health) the Armada commander is worth about
400 metal of tier-1 soldiers. So its own line read "against this unit alone, we outweigh it heavily" against two
Stouts, three Warriors and a Hammer (1390 metal, every one outreaching its D-gun), and the hands, told by the player
that it walks back from soldiers bigger than a raider, followed the line: an extractor toward the party at 98%,
`attack party_108` at 50%, `retreat_home` at 10%, dead four seconds after the first hit. The same weight against
five Pawns (270 metal, all inside its D-gun) reads the same words, and there they are true.
**Status.** demonstrated (wake-4, 2026-09-24, one death that lost a game led 26k to 2.7k); the fix replayed (commander-ab, 165 recorded asks): on the 27 the new odds call outweighed the hands take the commander away on 67% with the new line and the rule sentence together (30% recorded; 30% with the line alone, 44% with the sentence alone), on the 117 raider asks 5% (4% recorded); wake-4's fatal ask went from `extractor` 0.70 to `retreat_home` 0.68.
**Evidence.** `run/matches/1790221479-wake-4/00/jev-0.jsonl` f=38175-38370 (the commander's entry and answers);
`combat.rs` `power` before 2026-09-24 (metal times the matchup factor); `crates/bot/data/units.json` (armcom 187.5
dps, 3700 health; armstump 80.8, 1800, 225 metal; armwar 183, 1590, 270).
**Would be wrong if.** Jev had read the player's sentence over the line (it did not: 0.46 extractor, 0.01
retreat_home) or the death had come from something the line could not show (it named the party and its distance
for eight seconds).
**Used by.** H-HANDS-COMMANDER-WORTH; H-HANDS-SCHEDULE (the builder alarm).

### K-hands-the-chase-could-not-end
**Claim.** A picked fight against a raider was a fight-to-point at the party's last position, re-issued every one or two seconds, so a chaser ran to where the raider had been and stopped; with an enemy within 700 the flank form put the line at the frontmost unit's depth, so two of four Rovers got fight points 400-500 short of the Pawn every tick; and the flee rule fired on the Rovers. Four Rovers (168) never closed on one Pawn (87) from 350-600 away in 28 s, and against a Tick (132) they ran at full speed and still trailed it by 175-380 the whole way. Nothing the deciders chose could be seen through.
**Status.** measured (worlds-2, 2026-09-26, `record-0.jsonl` frames 4800-5640 and 7140-8400; the player's own note in nouls-1 at 4:03: "Blitzes cannot catch Ticks", and at 7:22 "the whole 17-strong guard was chasing one Tick to spot_7 in the far corner"). The hunt (H-HANDS-THREATS: attack by id re-issued each second, hunters raw, only units faster than the quarry) is the fix; the raid scenario measures it.
**Evidence.** `run/matches/1790309898-worlds-2/00/record-0.jsonl` (the `cmd` lines' fight points per Rover against the `s` samples' positions); `bot.log` line 76 (H-MICRO-FLEE=10 H-MICRO-FORM=12 in minute 2-3).
**Would be wrong if.** The Rovers had been slowed by terrain rather than orders (the samples show 43 elmo/s along a straight chase where they ran 160 elmo/s toward the Tick two minutes later on the same ground).
**Used by.** H-HANDS-THREATS; the micro agent's hunt primitive and raid scenario.

### K-hands-a-lone-unit-under-turrets-reads-as-a-lone-unit
**Claim.** A party is the enemy's mobile units within 400 of each other, and its odds and lines weighed those units alone: a Centurion standing among an Overwatch, a beamer and two light turrets read "party_99 (1 armwar, at nest) met with 4635 metal: we outweigh it heavily", and the worlds question, which prefers a fight the line calls won (K-jev-one-question-over-joined-worlds-coordinates), sent the whole group at it. The player's words for the same nest ("never goes to spot_20", "ignores lone units") reached the executor's raider rules, which the worlds candidates did not consult.
**Status.** demonstrated (worlds-2, 2026-09-26 morning, 18:06-18:43: group_C from 43 soldiers and 8.6k metal to 17, 16 Stouts in 15 s, the game's lead gone; the same shape at 13:42-14:11 against the D3/E3 nest, 9 Blitzes in a minute). Fixed the same morning: the covering turrets' metal is the party's in every odds line, the lines name them, no fight candidate against a party the rules ignore or one at a never place, and a chase that reaches a never place ends there. Not yet seen live.
**Evidence.** `run/matches/1790309898-worlds-2/00/jev-0.jsonl` f=32580-33690 (the `standing` lines' `lines` for group_C and the `worlds.pick` answers); `strategist-0.jsonl` the notes at 18:21, 18:28, 18:43 and the `standing` calls at 18:21 (`raiders_lone: ignore`) and 18:28 (`never: spot_20 spot_23 spot_18 spot_15 spot_16`).
**Would be wrong if.** The turrets had been in the line and Jev had taken the fight anyway (the offline replay can test the wording once a game records turrets with a party), or the losses had come from the mobile army rather than the nest (the fire log: armllt and armhlt hits on Stouts 10.1k and 5.5k, the nest).
**Used by.** H-HANDS-TURRETS-WITH-A-PARTY; H-HANDS-WORLDS.

### K-hands-a-tool-refusal-arrived-a-turn-late
**Claim.** The `standing` tool answered "checked against the picture at their next second (a refused rule is said in the report's `done` lines)" to every set, so the player learned a refusal one turn later from the picture, or not at all, and re-sent orders it believed had lagged: 15 sets refused in worlds-2, most for a `spot_N` the picture's short list did not carry (the picture lists our extractors' spots, a few free ones, theirs and the ones the instructions name; the tool's `station: spot_27` named none of those) or a party whose name had gone stale by the next second. A set lands when the turn ends, about 13 s of think penalty after the call (15:12 to 15:25), which the player also read as lag.
**Status.** measured (worlds-2, 2026-09-26 morning: the player's notes at 15:24, 17:07, 19:43, 20:27; the `done` lines "standing: refused: group_C: spot_27 is not a place in the picture"). Fixed the same morning: the check runs in the tool with the turn's marks and every walkable spot as places, the answer names the actor refused and why while the others' rules go through, and the picture lists the spots the tool's rules name. The think penalty stands: it is the game's real-time cost of a turn.
**Evidence.** `run/matches/1790309898-worlds-2/00/strategist-0.jsonl` (the `standing` calls and the turns' `done` lines); `mcp.rs` the `standing` arm before 2026-09-26.
**Would be wrong if.** The player keeps naming stale parties (the check answers at once now, but the name is stale by the time the rule fires: a party's name lives while a member of it is in sight).
**Used by.** H-HANDS-STANDING.

### K-hands-the-commander-was-never-found-by-its-handle
**Claim.** `unit_by_handle` split the handle at its last underscore with `?` before matching the word `commander`, so every lookup of the commander by handle returned None (the constructors' `constructor_N` handles split fine): the first-list skip of the default's builds never ran for the commander (bank-1, 2v1-medium, 2v1-hard, 2v1-hard_aggressive: four solars before the plant each time, the list's three on the default's one), and a `stop` at the head of a commander list never stopped its build in progress or dropped its task.
**Status.** measured (2026-09-24): in 2v1-hard_aggressive the install log line ("first list for ...: the hands had ordered [...]") appeared for five constructors and never for the commander, while its narration showed two extractors and a solar ordered before the list landed at 0:25. Fixed in `yards.rs` the same day and checked live in 2v1b-medium: the list landed at 0:16 with two extractors ordered, the line read "the hands had ordered [armmex, armmex], its first 2 steps skipped", three solars followed and the plant stood at 0:51 (done 1:10; the pool 1:07).
**Evidence.** `run/matches/1790259885-2v1-hard_aggressive/00/bot.log` lines 67-93; the four openings in `docs/experiments.md`.
**Would be wrong if.** The line still fails to appear for the commander after the fix (another condition in the install chain).
**Used by.** H-HANDS-SCRIPT (the first-list skip, `stop`); the `remove` tool and the `reclaim` step on the commander's handle.

### K-hands-a-family-question-frees-the-course
**Claim.** Asked the group's `do` as one flat choice, Jev keeps the recorded course at the moments the detectors flag
(a party in sight while a building of ours dies beside the group; a raider at our extractor within 1,200): `continue`
holds 0.5 to 0.9 outright and the fight options together hold 0.16 to 0.24, so no merging of synonyms could have won
them (the family with the most mass was the right one in 0 to 10% of those moments). Asked in two levels in the same
request, a `kind` question over families (stay, fight, go, back, the singletons as they are) and one refinement per
family with the original options and words, the pick is a wanted option at 51 to 59% of the same moments (P(wanted)
0.39 to 0.49), and 62% at the never-split moments (28% flat: the fight family's refinement prefers `engage` to a
detachment). Putting the hold option's cost clause on `continue` in the flat layout moves almost nothing (9 to 11%):
the structure does it, not the words. The cost: 18% of ordinary decisions (recorded at 0.6 or more, unflagged)
change, ten of fourteen from `continue` to `engage`.
**Status.** retired 2026-09-25 (the two-level menu deleted with the heuristic bot, `docs/design/2026-09-25-one-decider.md`: it freed the course too far in family-1 and was never measured better). Before: measured offline (2026-09-24, `run/jev_ab.py --variants base,continue_cost,two_level`, 40 moments a
detector from six games and 40 controls, two repeats: `docs/studies/data/jev-ab-two-level-2026-09-24.jsonl`); built
in the hands the same night (H-HANDS-TWO-LEVEL, `pianist/family.rs`), the check game family-1 (hard_aggressive) is the
live test: the detectors' rates on its log against the six games'. Family-1 (2026-09-24 night, LOST at 31:17):
raider_ignored 3.0% of group asks (46 of 1,529) against 16-29% in the six flat games, hold_beside_attack 0.2%
against 1.6-3.7%, never_split 1.8% against 0-2.2%; a busy group kept its course at 7% of its asks (60 of 802)
against 81-88% (fight 53% at a median composed probability of 0.55, go 30%, back 8%); the switch margin held 36
picks. The freedom overshot: the player wrote at 16:29 that the hands pulled every group after Pawn raids whatever
the packet said, and at 20:01 that the home groups splintered into ones and fours chasing Pawns. The game was
lost to a Razorback that killed ~20 Stouts in the open at 27:52 after a 12.7k-against-2k army lead had sat at home
(18:04). The claim's mechanism holds live; its cost in ordinary decisions is far above the offline 18%, because the
controls were the confident picks and a live game asks every ten seconds under raids. The raw family masses were
not logged (fixed: `raw` on the call), so the fight mass behind the picks is unmeasured.
**Evidence.** The ledger row jev-ab-two-level; ../jev_experiments battery F (a synonym takes half the leader's mass;
flips only under a 0.75 lead) and battery D (batching is free), which made the one-request layout possible.
**Would be wrong if.** The live composition (the family's mass, then its refinement) behaved differently from the
offline replay, or the 18% of changed ordinary decisions cost more games than the freed moments win.
**Used by.** H-HANDS-TWO-LEVEL (the request in `mod.rs`, the composition in `family.rs`, the margin in `hands.rs`).

### K-jev-a-packet-decompresses-to-standing-orders
**Claim.** Put to Jev once with one extraction question per (actor, rule) over a fixed vocabulary, the player's
prose packet comes back as standing orders cheaply (three hundredths of a cent a packet), stably (identical
paragraphs give identical rules 94% of the time) and literally: a question in the packet's own idiom is answered at
0.97-0.99, one off it hedges (0.33-0.44 with the words present), and a disjunctive question is answered for its
false part. At two thirds of the recorded moments where the hands held while a raider stood at our extractor, the
packet in force had a raider order the vocabulary caught.
**Status.** retired 2026-09-27 evening. The claim held for what it measured (cheap, stable, literal on its own idiom) and the stage still lost: over the eleven games with people (`docs/studies/2026-09-27-jev-posing/README.md`, the decompression audit) it produced a handful of rules the `standing` tool carried anyway, and it hurt through switch-offs the tool could not make over a packet rule, pruning read from another actor's sentence or with its condition dropped ("never chase single units" became `raiders_lone ignore` on every group and stopped eleven Stouts in front of a Bull), one-seat packets read into every seat, and a re-read of the same packet every quiet second (a bug: up to 15 % of a game's Jev tokens). The literalness was the problem as much as the strength: routes, conditions and "every group" had no rule to land in, and a question that could not take them was answered with what it could. Before: measured offline (2026-09-25, `run/decompress.py`, since deleted, 293 packets of four games); built and run live in standing-1 (ledger row: the bill $0.17 against $0.59, 49% of plays by the executor, raider_ignored 3 moments, and the faults the player's notes name, three fixed after).
**Evidence.** `docs/studies/2026-09-25-decompression.md`; ledger row decompress-1.
**Would be wrong if.** The same rules executed in code answered those moments no better than the per-ask Jev read,
or the player's packets drifted off the vocabulary's idiom so that the hedged share grew past a third.
**Used by.** the design `docs/design/2026-09-25-menus-from-scratch.md` §9d (not built).

### K-jev-one-question-over-joined-worlds-coordinates
**Claim.** The groups asked in one call, put to Jev as one Choice over the worlds their candidate actions make, each
world with its computed consequence line, never send two groups after one lone raider (the per-actor form did in
every one of 37 recorded collision calls) and answer a raider ignored by the per-actor form 60-71% of the time
(0% recorded, 51-59% under the two-level layout), from one question in place of four per group. The pick diffuses as
the worlds multiply (p(top) 0.63 under 8 worlds, 0.23 at 24 and more) because our worlds are near-equivalent, and the
gain-only lines tilt toward fighting as battery I warned; a Score per world with keep-favouring level words tilts the
other way (controls 77%, raiders 29%).
**Status.** measured offline (2026-09-25, `run/worlds_ab.py`, 205 calls of three games, three forms); built in the
hands the same night as the groups' one decider (H-HANDS-WORLDS, `pianist/worlds.rs`: world 1 the rules, the rest one
group's deviation, capped at 8). worlds-smoke-1 (the pianist alone on standing-2's packet, easy): 80 questions of 2 to
4 worlds, the rules' world picked 33 times and a deviation 47 (w2 41), no actor played by two deciders within 20 s
(smoke-pianist-1 under the executor: 29 such flips), 80 asks saved on one-candidate groups, the game reached the
20-minute cap where the executor's had lost at 17:07 (one game each: noise). The live test on the player's brief is
worlds-1.
**Evidence.** `docs/studies/2026-09-25-joint-worlds.md`; ledger row worlds-ab; ../jev_experiments battery J.
**Would be wrong if.** Live, the world form's picks lost the packet's courses more than the two-level layout did, or
its diffuse picks over many worlds played worse than the per-actor asks on the same moments.
**Used by.** H-HANDS-WORLDS (`docs/design/2026-09-25-one-decider.md` §4).

### K-jev-a-noul-per-kind-of-action-is-a-coin-flip
**Claim.** Asked, beside the picture and the player's instructions, whether a kind of action ("a metal extractor at a
free spot", "a factory making a constructor", "a radar") is something to put hands or metal into this second, Jev
answers 0.44-0.60 for every kind every time: the noul carries no signal, and used as a gate at 0.5 it prunes the one
state the instructions ask for as readily as any other. A noul over a concrete state ("is this what plant_4919
should do now, rather than standing idle? The move: plant_4919 makes a Construction Vehicle ...") reads the
instructions' table against the counts and ranks (the threat form's state nouls, threats-smoke-1..5).
**Status.** observed (2026-09-26, onepass-smoke-1: 292 kind nouls, mean 0.50, 158 at or above 0.5, with
`dim.constructor` at 0.49 and 0.44 while the packet's table said "with 0 constructors: a construction vehicle").
**Evidence.** `run/matches/1790320258-onepass-smoke-1`, the `worlds_gate` lines' `flags`; the ledger row.
**Would be wrong if.** Kind nouls worded as facts about the instructions ("do the instructions call for a
constructor before any soldier?") came back sharp; battery J's dimension nouls (`j5_dimensions.py`, 20/20) asked
whether an attribute affected the answer, a different question.
**Used by.** H-HANDS-ONE-PASS (the pre-pass asks a noul per open state, none per kind).

### K-jev-rule-defaults-answered-before-the-pick
**Claim.** Under the one pass as built on 2026-09-26, the packet's standing sentences ("a raider at one of our
extractors is met by a small detachment") were read into rules once, at 0:02, and fired as world 1's defaults the
second a party appeared, before the pre-pass asked anything; the pick then saw a hunt already under way and kept it.
So the pianist-alone games measured the rules' defence, not Jev's: the pick never chose a response. Put the question
to Jev with no defaults (`--no-rules`), a lone Pawn met by nobody opens nothing when no group exists and the
commander's attack state is not offered beyond 320; a Flea killing an extractor 1,000 from a group of Blitzes opens
the whole-group attack at 0.56-0.80, worded "it outruns this group: a chase drives it off, a kill needs faster
hunters", and the pick keeps world 1 every time (nine of nine).
**Status.** observed (2026-09-27, onepass-hard-4 read against its log: the 5:04 hunt was the rule default at frame
9120, the gate rated the party 0.43; onepass-norules-hard-1: plays by pick 54, by rule 0, group plays 0, 12 threat
asks, 9 opened).
**Evidence.** `run/matches/1790322526-onepass-hard-4` (the `pass` lines' `played` with source `rule`),
`run/matches/1790347711-onepass-norules-hard-1`; the ledger rows; the H-HANDS-ONE-PASS row.
**Would be wrong if.** A threat state opened for a party the group can catch (a Pawn against Blitzes) were kept at
world 1 too: not yet seen without rules, since no group existed when the Pawns came.
**Used by.** H-HANDS-ONE-PASS (the user's ruling, 2026-09-27: a response is a decision, not a default; what Jev
decides wrongly is prompt work, not a rule).

### K-jev-a-response-opens-by-the-party-noul-not-its-own
**Claim.** Whether Jev sends a builder at a raider depends on which question opens the move. Asked as the builder's
own state ("is this what the commander should do now, rather than building a solar? The move: commander attacks
party_1 (1 Pawn, 446 away, 12 s of walking, killing our extractor now) ...") the noul sits at 0.31-0.43, under the
0.5 flag, and the pick never sees the world. Asked as a state of the party's threat slot, the party's noul ("does
party_2 need answering this second by someone other than what stands? Yes when it is killing something of ours that
nothing there can stop") opens every response at 0.71-0.80 whatever the state's own noul (0.32), and the pick then
takes the attack at 0.49 and a one-Blitz hunt at 0.50 over world 1. The move is the same; the actor-centred question
weighs it against the builder's course, the party-centred one against the loss.
**Status.** observed (2026-09-27, onepass-norules-hard-2: five offers as a builder state, none opened;
onepass-norules-hard-3: two threat-slot offers, both opened and picked).
**Evidence.** `run/matches/1790348526-onepass-norules-hard-2` and `run/matches/1790348777-onepass-norules-hard-3`,
the `worlds_gate` lines' `flags` and the `plan` lines; the ledger rows.
**Would be wrong if.** A party-centred opening sent builders at every passing scout: in -3 two Pawns passing 400-650
from the commander and killing nothing rated 0.24-0.44 and opened nothing.
**Used by.** H-HANDS-COMMANDER-FIGHTS, H-HANDS-ONE-PASS (the builder's attack lives in the threat slot).

### K-jev-a-job-sentence-is-not-a-per-second-signal
**Claim.** A standing-job sentence in the packet ("constructors: their standing job is taking free metal spots on
our strip, nearest home first (spot_36, spot_54, spot_28, ...)") does not reach the pass as a per-second signal the
way the plant's table does. Asked as the move ("is this what constructor_X should do now, rather than idle? The
move: builds a metal extractor at spot_28 (14 s of walking): our 5th") the noul sits at 0.2-0.35; asked as a fact
about the instructions ("do the instructions make this its next step now?") it sits at 0.4-0.7 for an idle
constructor near the spot, 0.2-0.35 for a busy or distant one, and reads the commander's "standing job is helping
the plant" as the constructors' job too (0.41 mean, 40 of 97 above 0.5); the forbidden spots read 0.03-0.07 under
every wording. And when the instructed spot is in the pick, the pick declines it: 120 of 120 worlds at a mean
probability of 0.06 with 400-1,000 metal in store and two extractors (onepass-norules-hard-6). The plant's table
("with 0, 1, 2 or 3 constructors: a construction vehicle; with 4 or more, while our soldiers are a handful: a Blitz")
is followed every game because each row is a count the state's words carry.
**Status.** observed (2026-09-27, onepass-norules-hard-5 and -6; the offline replay of -5's moments, four wordings,
three runs each: `told_battery`, results in the ledger rows). Open beside it: the game's noul on a request sits up
to 0.3 from every replay of the same request (same model, state, questions, instructions): unexplained.
**Evidence.** `run/matches/1790392527-onepass-norules-hard-5`, `run/matches/1790392915-onepass-norules-hard-6`
(the `worlds_gate` lines' `.told` flags), the ledger rows.
**Would be wrong if.** The job sentence were followed after all under some wording of the state or the question:
four wordings tried. The table form is the counter-case, not the refutation: written as a table of counts ("with 4
extractors: an extractor at spot_28", `run/packets/comet-west-hold-table-1.md`) the same job is followed in the
table's order with no rules (onepass-norules-hard-7: 10 extractors at 8:00, as hard-4 had with rules; the strip
extractor's move noul 0.23 mean, 22 of 694 above 0.5, enough because each row names a count the state's words
carry). A condition stated positively only ("one solar when the energy line reads STALLING") still fails: the
commander built seven solars at "spending faster than it comes in"; the constructors' row that names the other case
("with energy banking or in balance: no solar, ever") got none.
**Used by.** H-HANDS-ONE-PASS (`WITHIN_REASON_TOLD` off by default); the user's split of 2026-09-27: the player
owns the build order, Jev follows instructions and answers reflexively; what the player has to write: tables of
counts, each row naming its case and the other case.

### K-hands-a-fixed-reach-hides-the-question
**Claim.** A fixed reach on a threat state (offer the hunt within 1,200 of the group, the builder's attack within 320,
500 or 600 of the builder) hides the response the user wants rather than sparing Jev a bad one: each game's worst
loss came at a party just outside the reach, with no state against it on the gate. The distance belongs in the words
(the walk or drive in seconds, what the party is killing, the metal sent against the party's) and the pick weighs it.
**Evidence.** onepass-player-2 (the ball 1,380 and 1,475 from the block that killed the plant and the commander, no
whole-group state under the 1,200 reach; the user: "a flat 1200 limit seems wrong"); onepass-norules-hard-1 (a Pawn
550 from the commander killed a Sentry and an extractor for a quarter minute, attack never offered);
`run/matches/1790397807-onepass-player-3` (a Pawn 1,000 from the commander hit a constructor at spot_45 from 2:36 to
3:04 with no attack state until a Tick came within 300, then offered 0.38 and taken 0.41; the Blitz ball stationed
at spot_38 had no hunt state against the Ticks 2,400 north eating six extractors 4:51-6:55, only the whole-group
state at 0.09-0.19, declined). 2026-09-27: the whole-group attack and the way back lost their reach after player-2,
the hunt and the builder's attack after player-3 (the builder's within the raider reach, 1,200).
**Would be wrong if.** Jev took far states indiscriminately once offered (the whole ball walking 2,400 after every
Tick, the commander leaving the plant for every Pawn on the strip): the next player game is the check, the hunt's
drive in seconds and "killing extractor" in its words the lever if it declines them all.
**Checked (onepass-player-4, `run/matches/1790430054-onepass-player-4`).** Not indiscriminate: the commander's
attack was on the gate 518 times at 600-1,200 away and 197 within 600, taken 7 times (4:50, 4:55, 5:59 against lone
Ticks, 26:25-28:31 in the last stand); hunts were offered 1,186 times beyond 1,200 and 422 within, taken 54 times (41
by the player's detachment rules, 13 by the pick). No walk back was re-advanced (0 fall-back-then-advance pairs
within 3 s, against every fourth second in player-3).
**Used by.** H-HANDS-THREATS, H-HANDS-COMMANDER-FIGHTS.

### K-hands-a-raider-answer-is-its-drive-off-time
**Claim.** The measure that ranks the answers to a raider is the time until the raider is driven off: the seconds
each answer takes to reach it, from its position and speed. A slower chaser still drives it off (the raider leaves
when the chaser arrives or stands and dies), so the chasers are not only those that outrun it. The words carry that
time for every answer and the fixed rules say the soonest wins, the commander last.
**Evidence.** `run/matches/1790430054-onepass-player-4`, 5:30-6:24: one Tick at spot_54 killing an extractor; the
hunt filter took only members faster than 132, so the two Rovers (168) were the hunters offered (0.16-0.33) and the
seven Blitzes (101) never; the commander's attack (1,163 away, 31 s of walking) was taken at 0.30 and it chased the
Tick round the base. The user, watching: the Blitz should definitely be offered as a chaser; the math to present is
how long unit y takes to drive unit x off. Built 2026-09-27, untested in a game.
**Would be wrong if.** Jev still took the commander over a Blitz whose words say it drives the Tick off sooner, or
sent slow chasers after every raider so that the group melted into detachments: the next player game is the check.
**Used by.** H-HANDS-THREATS, H-HANDS-COMMANDER-FIGHTS, `crates/bot/src/brain/pianist/rules.md`.

### K-hands-extractors-stop-at-the-named-spots
**Claim.** Under the prose pipeline the extractor count stops where the player's naming stops, not where Jev's
judgment does: the hands offer a builder the free spots the instructions name and an unnamed spot only when none is
named, and a builder on a `queue` list gets no extractor state at all while the list runs. Jev takes a named free
spot when it is on the gate.
**Evidence.** `run/matches/1790397807-onepass-player-3`: 40 spots on our half; the constructors' instructions named
the strip's 12 and the home 3 and said "never build east of spot_38 or spot_43"; 16 held at 14:00 and 20:00; from
16:00 to 24:00 the constructors' slots offered an extractor state in 133 of 703 slot-seconds, always spot_36, taken;
constructor time went 33% turrets, 13% assisting, 11% solars (all from lists), 24% extractors (17 points from the
pick, 7 from lists). `run/matches/1790430054-onepass-player-4`: 17 at 10:00 and 15-17 to 16:00 with 7-8
constructors on solar/assist/turret lists; 22-25 from 18:00 once the player set `job: expand` (8:40). The games that
reached 25-30 by 9:00-10:00 (penalty-4-lua-low, held-1-hard-aggressive) had the bot's own economy engine taking every
free spot; the strong players' median is 23 at 12:00 (`run/replays/recheck.py --floors 40`).
**Would be wrong if.** A player naming 25 spots in order, with `job: expand` or short lists, still stalled at 16-17
with free named spots on the gate declined: then the pick, not the naming, is the cap.
**Used by.** H-PLAYER-EXPANSION-PACE (the prompt teaches the player; the user's ruling that the count is the player's).

### K-hands-an-idle-lab-under-a-full-store-is-declined
**Claim.** When the instructions make production conditional in prose ("Maces while the exit lane is clear and energy
is in balance", "Pawns only as a raider response"), the pick reads the condition as unmet and leaves the lab idle
with the metal store full, for minutes: the lab's build state stays on the gate and is not taken.
**Evidence.** `run/matches/1790445610-models-medium-sonnet5`: from 7:00 to 15:00 the lab slot was open in all 481
seconds, the store full in 374, five lab plays; the pick took "nothing changes" at 0.4-0.7 over "lab_28531 makes a
Pawn" at about 0.15 (the Opus reviewer's reading of the worlds question, verified on the plays and the store).
`run/matches/1790445701-models-medium-gpt56-terra`: 4:29-8:31, 241 seconds open, 190 full, no lab play in minutes
5-8. Opus games do not show it: their packets say "Blitzes, without pause".
**Would be wrong if.** A sentence in the hands' fixed rules ("a full store overrides a condition on production unless
production is forbidden outright") did not open the lab's state: then the noul, not the reading, is the cap.
**Used by.** H-HANDS-RULES (the full-store sentence, 2026-09-27). The rerun added a case the sentence does not cover: models-medium2-opus55-low's packet made production unconditional ("Stouts continuously. Never idle") and four plants still stood idle under a full store from 21:00, see K-hands-the-pick-dilutes-over-single-change-worlds.

### K-hands-the-pick-dilutes-over-single-change-worlds
**Claim.** The one pass offers the pick a base world and one world per deviation ("as w1, and: X"), so with many idle
actors the question carries ten or more worlds each changing one thing, and "nothing changes" wins most seconds on
a plurality even when every deviation is wanted: an idle lab under a full store, its state rated 0.8 in the pre-pass
and the packet saying "never idle", gets a build in one second of ten.
**Evidence.** `run/matches/1790447448-models-medium2-opus55-low` (the lab's states never default: a lab builds only
by the pick): from 21:00 four plants idle, the store full 26% of the game; a lab world stood in 512 full-store
questions of a median 11 worlds and was picked 45 times; the pick's median for w1 0.31, for the best lab world
0.17, with the lab states' own nouls at 0.79-0.85 (22:02: w1 0.37 against w4 0.16, the rest under 0.08). The
dilution follows the count of worlds: models-medium2-sonnet5-nothink, a median 4 worlds, a lab world picked in 81
of 154 full-store questions; onepass-player-5, 6 worlds, 26 of 201; models-medium2-gpt6-astra, 6 worlds, 28 of 103
(`scratchpad lab_pick.py`, kept in the session; the method: the questions whose lines hold "As w1, and: plant_N makes",
the store at 95% of capacity from the record's samples, the answer's choice). With the full-store sentence in the
rules (onepass-player-7, 398a80a): plant_22773 was picked 13 times a minute until 17:29 and 0 times in minutes 17:30
to 19:00 while offered 35 and 30 times, the store full from 18:20; a lab world was picked in 7 of 174 full-store
questions of a median 5 worlds. The sentence did not lift the lab world's share. Each unit on a lab's list is its
own world, so a list multiplies the dilution: onepass-player-8 (cap 7), 28:00-29:40, four plants with two units each
put eight lab worlds in every question (78 questions, a median 9 worlds, w1's median 0.32 against the best lab
world's 0.15) and a lab world was picked 9 times in 100 s with 2,000-2,300 metal banked; the player narrowed each
plant to one unit at 29:25 and noted at 29:35 "plants started building after single-unit lists".
**Would be wrong if.** The full-store sentence in the rules (H-HANDS-RULES, 2026-09-27) lifted the lab world's share
above w1's in questions of ten worlds: then the reading, not the question's shape, was the cap. Or if a base that
takes a lab's best-rated state when the lab is idle and the store full (a default made from the pre-pass noul, no
pick needed) did not empty the store: then the labs are idle for a reason the pick sees and the pass does not.
**Used by.** (candidate: an idle lab under a full store takes its best pre-pass state in the base, the pick free to
change it; or the pick asked per actor kind so labs are not weighed against group courses.)

### K-player-other-models-read-the-prompt-literally
**Claim.** The player prompt (4,700 words of doctrine whose counters, constructor counts and the meaning of a full
bank are carried by case handles and inference) is followed by Opus 5.5 and Opus 5 and read literally by the others:
they keep the rules they can quote and miss the strategy behind them.
**Evidence.** Seven Opus reviews of the models-medium bundle (2026-09-27), each verified on the record: Luna and
GPT-6 astra built only Hammers (speed 46) against Blitzes (101) and killed three enemy units in a game ("raiders
against raiders" not acted on); Luna and Terra kept three constructors against "four to six"; Terra read a banking
store as being ahead; Sonnet 5 called the small tools one by one for four turns against "one `orders` call"; astra
wrote counts for the hands ("with twenty or more Maces, split eight"); Terra, Opus 5, astra and Opus 5.5 low each
read an order still in flight under the think penalty as an order that failed, and re-issued or reversed it.
**Would be wrong if.** A shorter rule-shaped section (the counters, the constructor count, the bank, the landing of
orders, as flat statements) did not move these models' play: then the models, not the prose, are the cap.
**Used by.** H-PLAYER-RULES-SECTION (2026-09-27); the report's landing line (H-HANDS-PLAYER-WAKE). The rerun on 158e488 (models-medium2-*, before either) showed the same shapes: Luna 35 soldiers and 0 extractors at 12:00, Terra's energy empty 58% of the game, astra all Maces against raiders, Sonnet 5 calling the small tools one by one.

### K-hands-the-base-fired-ahead-of-the-pick
**Claim.** Under the one pass as first built, a standing rule's default went in force the second its slot appeared,
before the question was asked, and the pick landed on top of it in the same tick (lockstep: the ask and the pick have
the same frame, 1,237 of 1,237 picks in onepass-player-8). A picked state that did not replace the default (a hunt
over an engagement of the same party) left two current states in one slot; the base took the first, fell back to the
other when the hunters died, and the group was ordered both ways in turn without any pick choosing the second.
**Status.** observed (2026-09-27), onepass-player-8 7:13-7:38: party_22 (6 Pawns) appeared 7:13; `raiders_party
whole_group` made the whole-group attack the default and it fired by rule the same second (fight orders 7:16, 7:18);
the pick took the four-Blitz hunt at 7:22 (0.28 against 0.23); from 7:23 the slot held `hunt_group_A` and
`whole_group_A` both current; 7:31 the hunt state gone, base the engagement; 7:32 the party split, nothing current,
the default fired again at 7:35 and the pick split four off again at 7:35 and 7:38. The user (viewer): "some of the
tanks hesitate and turn back, leaving the one tank that actually completed the initial attack movement to die alone."
The calls themselves: gate 278 ms median (p90 316) over ~53 KB and 50 questions, pick 238 ms (p90 274) over ~33 KB.
**Evidence.** `run/matches/1790454920-onepass-player-8/00/jev-0.jsonl` (the `pass` lines' slots with `current`
flags, the `plan` lines, the `call` lines' `ms`), `record-0.jsonl` (the `cmd` lines 7:13-7:38).
**Would be wrong if.** The engagement's task were cleared by the hunt in the code (it was not: `start_hunt` set the
hunt and left the task), or the picks had chosen the whole-group state on the seconds it re-fired (they chose w1).
The same shape with one answer: a threat slot whose only state beside `leave` was the rule's default was not "open", so the default fired by rule while the pick sent the group elsewhere the same second (bluegecko-2v1-great-divide 4:57-4:58: attack by rule, hold and fall back by pick; fixed the same day: a threat whose default is due is an open decision).
**Used by.** H-HANDS-ONE-PASS (the pass holds open slots for the pick, 2026-09-27; a hunt ends the engagement it
overrides; a threat's due default is a decision); the bar-review skill, item H7.

### K-hands-home-is-no-way-out-with-the-party-at-it
**Claim.** The builders' retreat rule sent a commander home whatever stood there: onepass-player-8 30:00 ("walking to
home, 1804 to go" into the Razorback razing the base, home on its own never list), bluegecko-3v1-comet-catcher-2
15:41-15:51 (commander_t1 at 55% walked home into thirteen Stouts and died before the player's list to build south
landed; the player: "the retreat rule walked it home into 13 Stouts"). A builder on a list with a build queued ahead
had no retreat as its default (18:23: commander_t2 "on a list with 'assist' so the hands could not move it" from
eleven enemies at its home; its plant died, it fled at 2%). Unarmed parties frightened builders (19:50: "constructors
kept fleeing unarmed enemy air constructors").
**Status.** observed (2026-09-27), two games, the player's notes and the pictures.
**Evidence.** `run/matches/1790454920-onepass-player-8/00` (30:00 prompt), `run/matches/1790469236-bluegecko-3v1-comet-catcher-2/00`
(notes 15:41, 15:56, 18:23, 19:50).
**Would be wrong if.** The commander at 15:41 had a way out the rule could not see (it did: the player's own list sent
it south to build turrets, and it walked).
**Used by.** H-HANDS-STEP-AWAY (amended 2026-09-27).

### K-hands-idle-plants-wait-for-the-store
**Claim.** A factory's only default state was standing idle, and the hands' rules told Jev an idle factory "builds
now" only while the store is full, so with the store low the plants stood idle and the income went to the builders:
in bluegecko-3v1-comet-catcher-5 to 5:07 each of the three vehicle plants was idle in about half of the samples
(117 of 236, 101 of 233, 126 of 239), the store under 150 metal in nine of ten of those, and every lab play was
the pick's (14, 17, 14 in five minutes). In the game a queued unit only slows while metal is short; an idle plant
wastes its build power and the metal a unit would have drawn goes to whatever else is building. The user,
2026-09-27: waiting for the bank is wrong.
**Evidence.** `run/matches/1790475073-bluegecko-3v1-comet-catcher-5/00` records (`own` idle flags of the plants
against `m[0]`) and jev logs (`played` entries of kind lab, all source plan).
**Status.** Fixed in code 2026-09-27 (H-HANDS-LAB-DEFAULT), unmeasured. Exploited by [[H-HANDS-LAB-DEFAULT]].

### K-hands-lists-landing-together-skip-each-others-spots
**Claim.** When the player's lists for several constructors land in one turn, each list's extractor step finds its
spot held by another constructor's not-yet-started hands' order, skips it as "not free", and that order is then
dropped by the other constructor's own list, so nobody builds the spot and the constructors go on to the turrets
written after it (bluegecko-3v1-comet-catcher-8, 6:20: "the bot counted those steps as done and skipped them, so
several constructors went straight to their turrets with no extractor built"; the count stood at 27 for 30 s and
five constructors were re-listed). Two more small refusals of the same night: `produce` refused a plant still going
up as "not a factory in the picture" (1:05, three games), and the constructors built energy converters unasked
whenever energy banked (5:52), which the OS-48 players say a metal-heavy map has no use for.
**Evidence.** `run/matches/1790478204-bluegecko-3v1-comet-catcher-8/00/strategist-0.jsonl` (turns at 1:05, 5:52,
6:20).
**Status.** Fixed 2026-09-27 (a spot held by an unstarted order of a builder with a list waiting is free to another
list; the unit cards carry nanoframes so `produce` knows a factory going up; converters out of the usual menu, by
name in a list only). Exploited by [[H-HANDS-LISTS]].


### K-hands-a-group-was-a-point-to-the-deciders
**Claim.** Until 2026-09-27 evening a group was its members' centroid to Jev and the player: "at", "N to go",
"stalled", `enemies_near` and the odds were all measured or priced from that point. On a column strung 2,000-2,900
long, the head was already in B3 under two Beamers and five Welders while the line read "advancing to spot_26,
1953 to go" with no enemies near (game 3, 8:35); the Pounder ball had no `enemies_near` line while Bulls stood
669-936 from its nearest Pounder (game 9, 15:44-16:05), and 11-13 Pounders stood inside Bull reach under "we
outweigh it heavily"; "health: full on average" was said at 22 of 39 lost; a group split between the spot_19
plateau and the shore below never got its centre within 300 of its goal, so its fall-back walk stayed the base
world for two minutes and the player's stations were shown as the default 59 times and never played, and every
newcomer was sent to the centre, in deep water (Cape Violet 9:40-11:17, `topology-review.md`). A group's walk
states went to home and non-spot places only, so a raid route named in the packet put nothing on the menu
(game 10, 4:30-9:02).
**Evidence.** `docs/studies/2026-09-27-jev-posing/README.md` (I1, the costliest mechanism, in every game);
`run/matches/1790481450-bluegecko-3v1-cape-violet/00/topology-review.md`; `run/matches/1790479031-bluegecko-3v1-comet-catcher-8/00/posing-review.md`.
**Status.** observed in eleven games with people; fixed 2026-09-27 evening by [[H-HANDS-GROUP-BODY]] (the check:
the next arena games' pictures at the moments of contact).
**Would be wrong if.** The pictures on the new code still price a strung-out group whole, or a body's arrival by
its members lets a group hold with its tail under fire more often than the centre did.

### K-hands-the-block-left-sight-unsaid
**Claim.** The enemy section said only what was in sight this second and what buildings were remembered per cell:
a party that left sight vanished from the picture, so after 19:30 of game 3 nothing said where his block had gone,
and the switch to Bulls in game 9 was invisible ("factories_seen: none, ever" at 15:49 with 37k of his seen dead).
The deciders were asked to weigh a fight with no memory of the army they had just seen.
**Evidence.** `docs/studies/2026-09-27-jev-posing/README.md` (I5); `run/matches/1790479031-bluegecko-3v1-comet-catcher-8/00/posing-review.md`.
**Status.** fixed 2026-09-27 evening by [[H-HANDS-ENEMY-MEMORY]].

### K-hands-the-odds-were-a-metal-price
**Claim.** Until 2026-09-27 evening the odds line was the square-law metal verdict with, since d265666, reach
beside it: a Bull (tier 2, 3,500 health, reach 460) against Stouts was "an even fight" (game 9: 13 Stouts, 2,925,
against 3 Bulls, 2,850); a party of radar blips was priced as Pawns for speed ("it outruns this group at 87
against 75" of Welders at 48, Cape Violet 10:20); the enemy commander was metal (eleven Incisors into its D-gun,
thirteen Brutes into its death blast, game 7); a mixed party's aircraft were chased by tanks (game 3 14:21); and
a group under fire from out of sight read "we outweigh it heavily" of the blips it could see (game 3 19:30).
**Evidence.** `docs/studies/2026-09-27-jev-posing/README.md` (I4); the games' `posing-review.md`.
**Status.** fixed 2026-09-27 evening by [[H-HANDS-ODDS-WHAT-SHOOTS]]; what stays open is the matchup table's
tier-2 and turret rows (the metal verdict itself is still tier-1-trained).

### K-hands-no-state-named-a-building
**Claim.** Until 2026-09-27 evening a group could be offered only keep, walks to passages and marks, splits, its
station, fall back, retreat, join, a one-soldier scout, and hunts or attacks on mobile parties: no state named a
building or an unlooked spot, so a group at its station with nothing mobile in sight held for minutes with 22-27
of his unguarded buildings within 3,000 (game 10, 6:00-9:00, his truth file), the packet's raid route landed
nowhere, and four Shellshockers were never used in the nest fight (game 9); a group under fire from out of sight
could neither close on the shooter nor step out of its reach (game 3 19:30); the fall-back point was pruned into
the Bulls' path (game 9); `raiders_lone ignore` stopped eleven Stouts under a Bull (comet-catcher-3 21:52).
**Evidence.** `docs/studies/2026-09-27-jev-posing/README.md` (A1, A2, X2, X5; the user's 5:58 read of game 10);
`run/matches/1790471259-bluegecko-3v1-comet-catcher-3/00/decompression-review.md`.
**Status.** fixed 2026-09-27 evening by [[H-HANDS-GROUP-STATES]] (the check: the raid, sweep and gather states
picked in the next arena games, and what they cost).

### K-hands-one-shooter-for-the-side
**Claim.** Until 2026-09-28 the fire from out of sight was one estimate for the whole side: the weapon with most hits
in the last 90 s and one likeliest place, said to every group as its `unseen_shooter` and used by every group's
`pull_out` and `close_on_shooter` states. A Beamer's laser hits many times for little, so it won the vote over a
Gauntlet's few heavy shells; at player-9 21:15 four groups across the map, group_M at B2 included, were offered
"pulls out of the shooter's reach (490)" from a Beamer at E4, while group_G at D4 stood 1,105-1,231 from a Gauntlet
its own party line named with its 1,220 reach, and its pick flipped on 13 seconds between a 42-elmo pull-out and its
station from 21:17 to 21:34 (about 1,300 metal at D4).
**Evidence.** `run/matches/1790513490-player-9-posing/00/review.md` (findings 5 and 6), the `jev-0.jsonl` states
21:15-21:26 (`pull_out_*` words for group_C, group_G, group_M, group_T all with reach 490), the record's `destroyed`
events 21:14-21:26 against the Gauntlet at (4320, 1712).
**Status.** fixed 2026-09-28 by [[H-HANDS-SHELLED]] (amended: per-group estimates from the hits on the group's own
members, the weapon by damage); the check is the next game's `unseen_shooter` and `pull_out` words under a Gauntlet.

### K-jev-follows-a-prose-route-from-the-picture
**Claim.** Jev follows a route written in the packet's prose ("spot_49, then spot_46, ..., in that order") and answers
a fork on it from the group's role sentence, when the picture says which stops were reached and the nothing-changes
world's line names the cost of standing at a reached stop. At the recorded arrivals of player-9's 6:00 loop and
its scout's 3:53 arrival it named the next stop as the best move 24 of 24 times with reached lines (21 of 24 without,
under the 0.5 flag) and picked it 24 of 24 with the cost sentence (2 of 24 without: "holding for 1 s since it
arrived" read as a course); at the 3:58 commander sighting it kept the scout on its route 4 of 4 at 0.82-0.90 where
the station-list mechanic retreated it 2 of 4. The state Jev needs is words: the program counter, not the program.
**Evidence.** `docs/studies/2026-09-28-route-in-prose.md`, `docs/studies/data/route-ab-2026-09-28.jsonl`,
`run/route_ab.py`.
**Status.** supported offline (2026-09-28, seven moments, four repeats, one game); built the same day as
[[H-HANDS-ROUTE-FACTS]] (`docs/design/2026-09-28-routes-in-prose.md` §4.1-4.3); the live check is the loop's halts,
the E3 task changes per minute and the base found by minute N.

### K-hands-call-over-65k-tokens-fails
**Claim.** A Jev (jev-1.13.0) call whose state runs past about 65,000 input tokens is refused with HTTP 400
`max_tokens_exceeded`, and the hands then keep their last course for the second: nothing is re-decided.
**Status.** demonstrated (2026-09-28, player-10-routes)
**Evidence.** `run/matches/1790523133-player-10-routes/00/jev-0.jsonl`: 30 `error` rows 19:33-26:31 (19 in minute 20),
the largest answered call 65,587 tokens in and 7,820 out at 19:26 with 22 actors and 10 groups in the picture; 372 of
2,407 calls were over 40,000 tokens in. The F4 push fell apart in those minutes with the hands answering nothing.
**Would be wrong if.** The limit were on the answer alone, or the model's window grew.
**Used by.** [[H-HANDS-CALL-BUDGET]] (`pianist/diet.rs` `shed`).

### K-player-a-case-keeps-its-condition
**Claim.** A rule in the brief loses its condition the moment it is written: "tier 2 when the first heavy unit of
his is seen or by 12:00" was a 3v1 rule filed under the 3v1 chat and read as general, while "tier 2 in 4 of 58
sides" was a duel count with no reason attached; the player teched in duels and did not punish his tech. A case
carries its condition by construction (the scenario, the clock, the report's numbers, the move, the outcome, the
because), and two cases of the same shape with opposite right answers name the variable. The same two rules of the
brief (pause the other plants at tier 2; three to five Lashers, never more) were said again to us by the person in
game 10 and had not been followed, which a counted case may fix and a rule did not.
**Status.** proposed (the user, 2026-09-28: the brief should hold the lessons as situation-specific examples across
maps, since the player has years of experience to gain from tens or hundreds of thousands of tokens); unmeasured.
**Evidence.** `docs/briefs/player.md` before 2026-09-28 (rules under their game's heading); player-9, player-10,
bluegecko-3v1-comet-catcher-9 and -10 (`docs/experiments.md`); the user's discussion of 2026-09-28.
**Would be wrong if.** The player copied a case's surface (a spot number, a clock) where the situation only rhymed, or
the longer brief cost more in turn time than the cases gained; the kill test is the recorded turns at the cases'
moments (player-9 6:00 and 12:00, player-10 3:58) replayed under both briefs.
**Used by.** [[H-PLAYER-CASES-ACROSS-MAPS]].

### K-player-the-eco-line-never-said-stalling
**Claim.** The player's prompt keyed its solar rule on a word its own report never carried: "a solar collector
whenever the energy store is empty or the energy line reads STALLING" (`crates/bot/src/strategist/player.md`), while
the report's eco line gave energy as store, capacity and a net figure only ("energy 1/1300 (+0)": the engine caps
spending at income once the store is empty, so a stall reads as balance). STALLING lived in the hands' picture
(`picture.rs` `flow_words`). In player-12-post-human the 1:45 turn read "energy 1/1300 (+0)" behind an assisted
constructor and ordered four solars; the stall ended at 1:46 when the constructor rolled out and the metal those
solars took (620 at an income of 8) kept the store at 0 to 3:10. The score line and the eco line also disagreed on
the counts (extractors 2 against 3 at 1:45, "labs 1" at 1:00 of a plant standing at 1:12): the eco counts took
frames as standing.
**Evidence.** `run/matches/1790535786-player-12-post-human/00/strategist-0.jsonl` (the 1:45 turn), the record's
samples, `00/review.md` (Z3, the candidate harness note).
**Status.** observed 2026-09-28; fixed the same day: the eco line says "STALLING: the energy store is empty and
everything that needs it builds slowly" under the hands' condition (store under 5% with usage at 90% of income or
more), and its counts are of standing units. Exploited by [[H-PLAYER-ECO-WORDS]].

### K-player-the-brief-outgrew-one-argument
**Claim.** The player's system prompt (the player prompt and the brief, 133 KB together on 2026-09-28) was handed to
the Claude CLI as one `--system-prompt` argument, and Linux refuses a single argument over 131,072 bytes: player-13's
session failed to start ("Argument list too long (os error 7)"), the player never turned, and the game was lost at
9:03 on the hands' defaults (twenty constructor bots, Jev's calls failing on 17,558 questions a minute). Every
game before it ran under the limit (101 KB the day before).
**Evidence.** `run/matches/1790549874-player-13-gecko-opening/00/bot.log` line 4; `wc -c docs/briefs/player.md
crates/bot/src/strategist/player.md`; `getconf ARG_MAX` is the total, MAX_ARG_STRLEN the per-argument limit.
**Status.** fixed 2026-09-28: the prompt is written to `system-prompt.md` in the strategist's cwd and passed with
`--system-prompt-file` (`crates/bot/src/strategist/mod.rs`, `Launch::spawn`); the Codex path already wrote
`AGENTS.md`. A second lesson under it: a game whose player never started runs at full speed and is called in
minutes, and the hands without a packet make constructors without end (H-HANDS-LAB-DEFAULT with no allowance).

### K-hands-a-produce-list-naming-a-unit-twice
**Claim.** A `produce` list that names a unit twice (`armflash:1, armfav:5, armcv:1, armfav:10, armcv:1, armfav`,
the brief's Rover-mass line) was read per name: the count made was held per (builder, name), each entry's cap was
compared with it alone, and the plant's default was the first entry whose unit any entry still permitted. So at five
Rovers made the second Rover entry still permitted Rovers ("5 more allowed" in the picture beside "all 5 allowed
made: no more"), the default stayed on the first entry, and player-14's plant made nine Rovers (1:39 to 2:23) and no
constructor until the player replaced the list at 2:15 ("con late (plant skipped armcv)"); the constructor came at
2:32 against thebluegecko's 1:40 and the fourth extractor at 3:10.
**Evidence.** `run/matches/1790550188-player-14-gecko-opening/00/record-0.jsonl` (the plant's created events),
`jev-0.jsonl` at 1:58 to 2:00 (the plant's `allowed` words), `run/hands_window.py ... 1:40 2:20 plant_13472` (the
rule playing armfav each time with Jev's armcv at 0.53 to 0.65 against the base).
**Status.** fixed 2026-09-28: an entry's cap is cumulative over the earlier capped entries of the same name
(`entry_cap`, `entry_permits` in `pianist/mod.rs`; the plan's `permits` and default, the picture's words), with a
test. Exploited by [[H-HANDS-LAB-DEFAULT]].

### K-hands-the-all-list-bound-the-constructors
**Claim.** The player's opening `produce {"all": ["armflash:1", "armfav:5", "armcv:1", ...]}`, written for the plant,
was every constructor's allowance as well: `allowed_units` resolves an actor's own list, then `all_builders` for a
builder, then `all`, and the plan offered a builder only the buildings its allowance named. A list of plant units
names no building, so from 2:40 to 9:40 of player-14 no constructor was offered a turret, a solar or a nano turret
(the Jev log holds no such state; the first came at 9:45, five seconds after `produce {"all": null}`), the standing
order for a turret beside each outer extractor from 2:40 had nothing to act on, and every turret before 9:40 came
from an explicit `queue` list (the first queued 5:26). Thirteen extractors died 4:29-7:25, twelve with no turret
near. The same shape in player-15 (the same opening list): every turret to 8:00 from a list. The player prompt said
`produce` restricts "a lab, or every lab", so the player could not know.
**Evidence.** `crates/bot/src/brain/pianist/picture.rs` `allowed_units`; `plan.rs` the builder's `offered`; the
reviewer's `scratchpad/review-p14/turret_offer.py` over `jev-0.jsonl` of both games (turret plays by source: all
`list`); `00/review.md` finding 1.
**Status.** fixed 2026-09-28: a builder's allowance that names nothing it can build leaves it on the usual menu
(the rule H-HANDS-PRODUCE had registered for builders and the code had not run); the prompt says `all` reaches
builders only where it names a building. Exploited by [[H-HANDS-PRODUCE]].

### K-hands-an-open-entry-ahead-of-a-count-never-ends
**Claim.** A `produce` list's default was the first entry in order that still permitted its unit, and an entry with
no count always permits: `armstump, armflash, armart:4, armcv:2`, written three times from 13:28 of player-16, kept
the plant on the Stout in 13 of 14 plays (the Shellshocker offered at 0.34-0.52 and not picked) and the game ended
with 197 Stouts and 161 Blitzes made and no Shellshocker or Janus, the player having listed them seven times.
**Evidence.** `run/matches/1790553987-player-16-gecko-opening/00` (the `produce` calls in strategist-0.jsonl, the
created events, `run/hands_window.py ... 13:30 15:00 plant_13472`); `00/review.md` finding 4.
**Status.** fixed 2026-09-28: the default takes the counted entries first, in order, then the open ones; the brief's
words on `produce` say an open entry is the filler after the counts. Exploited by [[H-HANDS-LAB-DEFAULT]].

### K-hands-a-scout-body-offered-an-armed-fight
**Claim.** The Rover body of the brief's opening reached his base in player-16 (4:02) and player-17 (4:53) and was
killed there by the pass's own offer: with the odds words not saying "it outweighs us" (thirteen to seventeen Rovers
against "2 armpw"), the threat slot offered "attack party_N with the whole group", the pick took it (p 0.49 in
player-16, 0.36-0.37 in player-17, confidence 0.14-0.22), and the Rovers (105 health, 35 a second) fought Pawns
(370 health, 90 a second) beside a light turret and his commander: five lost for nothing at 4:06-4:14 of player-16
(the hunt of his four constructors called off one second before), eleven lost for nothing at 4:53-5:28 of player-17
(engagement #0: 341 for 0, 544 of ours on the spot against 324 and a 170 turret). thebluegecko's Rovers trade 377
for 620 at the enemy's base in minute 3 by killing what is unguarded and never standing in a reach, which is what
the rove lane does and the pass does not.
**Evidence.** `run/hands_window.py <player-16> 3:28 4:20 group_A`, `<player-17> 4:40 5:40 group_A`; the reviews;
`run/analyze_match.py <player-17> --engagement 0`.
**Status.** fixed 2026-09-28: a scout body is offered neither a hunt nor a whole-group attack on an armed party, and
the slot names the rove lane (H-HANDS-SCOUTS-FIGHT-NOTHING-ARMED). Open: the odds words priced "2 armpw" while 324
of soldiers and a turret stood on the spot (the parties near the target and its cover are not in the group's odds).
**Amended 2026-09-28 (the engagement plan's replay):** the fix as built did not cover player-17's body: its group_A at
4:53 was fourteen Rovers and one Blitz (seven and the Blitz at 5:03), and the check wanted every armed member a scout.
Now scouts more than half the armed metal; `run/plan_replay.py` on player-17 at 4:53, 5:03 and 5:13 says "a scout
body". The odds fault is answered for a body with a plan: the battlefield prices each element with its cover and the
whole position (H-HANDS-ENGAGEMENT-PLAN); the threat slot's own odds words are unchanged.

### K-jev-plans-the-screen-first-from-the-geometry-words
**Claim.** Given the battlefield of a position in words (each element's cover, where our reach reaches it from outside
every other reach, the odds with the cover priced in) and candidate orders of operations worded with where each phase
is fought from and what else reaches us there, Jev's choice ranks the order the TAS found winning (the screen first
from outside turret cover, the turrets last) above the turrets first and the nearest first.
**Evidence.** `docs/studies/2026-09-28-engagement-plan.md`: player-14 E3 at 9:16 and 9:17 (the fight's first planned
seconds; nothing of E3 was in sight at the design's 9:05): screen_first 0.79 / 0.76 and 0.80 / 0.81 (with / without
the examples block), statics_first and nearest_first 0.01-0.03; the same order at player-15 9:24 and 9:27
(screen_first 0.57-0.78, gather_first second at 0.19-0.38). One ask per arm, jev-1.13.0. It would be wrong if a game's
`engagement` lines picked a walk-in with a stand-off candidate offered, or if the pick flipped between asks of the
same picture.
**Status.** conjectured (offline replay only; no game yet). Exploited by [[H-HANDS-ENGAGEMENT-PLAN]].

### K-jev-the-examples-block-moved-no-plan
**Claim.** The examples block of the plan question (the E3 and F3 scenes, the Stout bench, the pros' Rovers, one line
each) changes no pick on the recorded moments: Jev's ranking comes from the candidates' own words.
**Evidence.** `docs/studies/2026-09-28-engagement-plan.md`: seven moments asked with and without it; the top plan is
the same in all seven; the largest shifts are the decline at player-16 16:10 (0.21 with, 0.11 without) and
one_at_a_time at 12:51 (0.79 against 0.66). One ask per arm: Jev's variance between asks is not separated from the
block's effect.
**Status.** conjectured. The block stays in the live question (the design's) until a game says otherwise; it is the
first thing to cut if the question's size matters.

### K-hands-the-plan-odds-do-not-see-reach
**Claim.** The plan's phase odds are `combat.rs`'s (metal, matchups, turrets at three times their metal) and do not
price reach, so a position whose statics or soldiers out-range the body reads "we outweigh it" in every phase and the
decline is not taken: the design's F3 test (hold and shell, or decline, above the walk in) fails.
**Evidence.** `docs/studies/2026-09-28-engagement-plan.md`: player-15 F3 at 9:24 and 9:27, the decline 0.01-0.03 of
five (no artillery in group_B, so no shell candidate; one Rocketeer at 475 in sight, not the four the design names);
player-16 16:10, an Overwatch (620) and three Sentries against 18 Blitzes and 8 Stouts (reach 180-350): the odds 1.8,
the decline 0.21 with the examples and 0.11 without, one_at_a_time taken. It would be wrong if the same moments'
decline rose above the walk-ins with reach said as a cost and the odds unchanged.
**Status.** conjectured. Open: price the approach under an out-ranging reach into the phase words (the odds words of
`picture.rs` `odds_words` already say "it outranges us", the plan's phases do not).

### K-hands-a-low-confidence-pick-turned-an-engaged-army
**Claim.** The world pick's confidence is logged and was not a gate: at 21:29 of player-17 the army (40 units) met
a Fatboy, seven Maces, a Hound, two Sharpshooters and turrets at D1/E1, and at 21:34 the pick chose "group_B attacks
party_25 (2 armstump) with the whole group", a party 1,300 behind the front, at confidence 0.03 (the two picks before
it, "attack party_51 (2 armham)" and "advance to spot_8", at 0.15 and 0.09); the army turned, went 40 to 23 units,
and engagements #16, #17 and #22 cost 6,900 for 1,284; the ratio fell from 3.0x to 1.1x by 22:00.
**Evidence.** `run/hands_window.py run/matches/1790557341-player-17-gecko-opening/00 21:30 21:38 group_B`; the
review's finding 4; `run/analyze_match.py ... --engagement 16`.
**Status.** fixed 2026-09-28: a pick under 0.25 confidence holds every fighting group's slot at its base
(H-HANDS-ENGAGED-PICK-BAR). The bar is a guess from this one game: the picks that moved engaged groups well were
not surveyed for their confidence, and the survey (`{"t":"plan"}` lines with `changed` naming a group in Engage,
against the trade in the next minute) is the check.

### K-hands-a-scouts-nibble-opened-a-hunt-through-ignore
**Claim.** `raiders_lone: ignore` yields to self-defence, a party hitting the group (comet-catcher-3 21:52: eleven
Stouts stood under a Bull killing them). In player-18 the player set `ignore` and `no_chase` on group_B for its
launch at 10:56 and a Flea (party_32) hitting the tail at 11:06 was enough: the pass offered "2 of group_B hunt
party_32 (1 armflea)" and the pick took it, the group met a Rover at E6 and reacted, and at 11:20 the player wrote
"group_B ignored the prose route (kept reacting to his Rover)" and forced a station. The launch, one of five that
game, never reached its route.
**Evidence.** `run/hands_window.py run/matches/1790559*-player-18-planner/00 11:00 11:22 group_B`; the player's
notes at 10:56 and 11:20 (`run/commander_turns.py --notes`).
**Status.** fixed 2026-09-28: a party of scouts alone is not the self-defence that overrides `ignore` (a Bull is).
Exploited by [[H-HANDS-STANDING]].

### K-hands-the-bar-held-the-groups-slot-and-not-the-threat-states
**Claim.** The confidence bar of H-HANDS-ENGAGED-PICK-BAR (a pick under 0.25 moves no fighting group) held slots
named `group_X` and nothing else, and the states that move a group at a party come from the threat slots, named
after the party (`party_142.whole_group_U`, `party_107.leave`): in player-18, the first game with the bar, an
engaged group was changed 22 times under 0.25, among them "group_U: attack party_142 (2 armstump) with the whole
group" at 34:09 at confidence 0.06 and "group_B: leaves party_107 and holds" at 27:03 at 0.13, the two shapes of
player-17's fault (K-hands-a-low-confidence-pick-turned-an-engaged-army).
**Evidence.** `run/matches/1790560788-player-18-planner/00/jev-0.jsonl`, the `plan` rows at f=61470 and f=48690;
the review's hands section (`00/review.md`, the bar check, which takes each group's task from the call line before
the pick); `pianist/mod.rs` at cc03951, `after_pick`'s `held`.
**Status.** fixed 2026-09-28: the hold is keyed on the actor of the picked state, and a `leave` is held when a
group engages that party. Not yet seen in a game.

### K-hands-a-plan-walked-into-a-never-place
**Claim.** The engagement planner read the packet's paragraph and standing orders into the question but not the
`never` rule into its gate: in player-18 the player put spot_23 under `never` at 21:04, and from 21:33 group_B's
plans (the E4 position won at 21:33, engagement #16, 3,654 of his for 1,670) went on to the E2 nest north of it,
engagement #18 costing 675 of ours for 280. The chase (`groups.rs`) and the threat pass both stop 400 short of a
`never` place; the planner did not.
**Evidence.** `run/matches/1790560788-player-18-planner/00/review.md` (the planner section: "then the plans went
north"); `run/analyze_match.py ... --engagement 18`; the player's 21:04 `standing` call in `strategist-0.jsonl`.
**Status.** fixed 2026-09-28: `engagement_of` declines a position with an element within `NEVER_REACH` of a
`never` place of the group (unit test on the E3 scene). Not yet seen in a game.

### K-jev-the-pre-pass-outgrew-the-context-by-its-questions
**Claim.** The pre-pass request grows with the actors' options, not the state: at 15:52 of player-19 it carried
322 noul questions over 22 actors (104 of them a group's advance to a spot, about 30 a constructor: the buildings
at every spot), 181,000 of its 215,000 characters, with the state at 30,000 (the 160,000-character shedding of
player-10 never fired). Jev answered HTTP 400 `max_tokens_exceeded` on 44 calls in 14:50-16:13, every actor
keeping its course each time; the largest call that passed carried 65,771 input tokens, and answered in 0.6 s.
**Evidence.** `run/matches/1790563943-player-19-planner/00/jev-0.jsonl` (the `error` rows at f=26700-29040, the
`call` rows' `usage` and `questions`); `00/bot.log`.
**Status.** fixed 2026-09-28 in the client (`crates/jev` `REQUEST_CHARS` 150,000: a request past it goes as
batches of whole questions over the same state, the answers merged, the call row's `batches`). Open: whether a
group needs an advance to every spot and a constructor every building offered every second (the size), and
whether Jev's context limit is 65,536 input tokens (the largest that passed was just above).

### K-player-the-report-counted-his-spots-and-never-named-them
**Claim.** The score line told the player how many spots the opponent held and never which: in player-18 it read
"the opponent is known to hold 12" while the picture had his base at G1-G2 since 4:24, and the player sent the
whole army at "his southern strip" (spot_79, 75, 70, 77, 72, 65, 60, 57) at 10:56, 12:17, 16:14 and 24:34, where
by the truth file not one extractor or turret of his stood at any of those times; his 17-19 extractors were within
2,500 of his start or on the D1-E4 column (spot_1, 13, 21, 23, 32, 31 at 12:17), several of them in our picture.
**Evidence.** `run/matches/1790560788-player-18-planner/00/review.md` (F1, the root); `run/commander_turns.py
run/matches/1790560788-player-18-planner/00 --told` at 10:11-10:56; the truth file's extractor positions.
**Status.** fixed 2026-09-28: the line names each held spot by number and grid, nearest our start first
(H-PLAYER-SPOTS-HELD). Not yet seen in a game.

### K-player-the-rover-body-walked-onto-his-commander
**Claim.** The brief's Rover body goes to his start's end of the strip by a route ending in a station; player-19's
player ended the route at `their_corner`, a mark set at 0:00 on his commander's start, and the ten Rovers crossed
strung out and four died to his commander at 4:00, 4:01, 4:02 and 4:03 (the record's `destroyed` rows, killer
armcom); the player then marked `his_base` at 4:01 and waited for "his lab found" (notes 7:37, 8:00) with the army
at C7-D7 at 2.6x, while the lab stood 480 from the commander (truth file: armlab at (6944,336) from 0:58, the
commander's start (6899,681)) and was first seen at 20:51. The 2:00-9:00 window at 1.8-3.3x ended at 10:00 at 1.4x.
**Evidence.** `run/matches/1790563943-player-19-planner/00/review.md` (E8, the root); `run/minutes.py` on the match
(the windows, `look_min` 3.9 against `fac_min` 20.9); the player's 3:15 `standing` call in `strategist-0.jsonl`.
**Status.** brief amended 2026-09-28 (the station one of his outer spots, never a mark on his commander; a
commander seen is a base found). Not yet seen in a game.

### K-player-the-brief-put-the-rovers-on-an-evasive-lane
**Claim.** In player-20 and player-21 the Rover body arrived at his base whole and did no damage because the brief told the player to put it on `lane rove` on arrival, and the player did exactly that: the tool text and the brief both say a roving unit "never stands inside the reach of anything that can shoot it", so the player had the information and followed the instruction; the fault was the instruction. Under `rove` at a base BARb's Pawns patrol, every Rover steps off within a second of a Pawn's approach and the tour kills nothing (player-21: `lane group_A: rove` at 3:38, the record holds no enemy death from 3:15 to 4:30 while the report said "destroyed 21"; player-20 the same at F8 from 2:55, 420 killed against the line's 620). The brief justified the lane with "which is what his Rovers do (377 for 620 in minute 3)", and his records show no such thing (K-open-comet-thebluegecko-rover-mass, reopened).
**Evidence.** `run/matches/1790568930-player-21-leash/00/strategist-0.jsonl` (the lane call and its reply at 3:38), `00/review.md`; `run/matches/1790566750-player-20-live/00/review.md`; the brief's Rover paragraph before commit da69668's successor; the tool text in `crates/bot/src/strategist/mcp.rs` (`lane`); the count over his 14 Comet records (a scratch pass over `record-<team>.jsonl`, 2026-09-28: Rovers made, alive and lost by minute).
**Status.** observed (2026-09-28, the user watching player-21's replay: "evasive mode just sent our rovers ping-ponging around the map in a particle cloud rather than actually using it to damage the enemy"). The brief now commits the body (H-PLAYER-OPENING-SNIPPETS amended); first game player-22-commit.
**Would be wrong if.** A committed Rover body at his base dies to Pawns and turrets for less than the rove lane's tour killed. Player-22 measured it: eleven Rovers in eight seconds at 4:03-4:10 for two of his. The claim stands (the player had the information and followed the brief), and the lesson moved up a level: neither lane can use a Rover pile at a defended base (the user, 2026-09-28: the build needs specific micro the hands do not have).

### K-hands-the-follow-leash-fired-on-the-first-tick
**Claim.** `engage_group` anchored the follow leash at the centre of every member of the group, and the leash
(`tick_groups`, `FOLLOW_LEASH` 900) measures the body's place, which excludes the stream still joining from the
yard; a group whose joiners stood at home while its body engaged 900 or more away held on the first tick after the
engagement ("drawn N from where it engaged"), the pick played "attack party_N with the whole group" again the next
second, and the engine got a Fight and a Stop every half second: player-20 unit 15785 at frames 27540/27555/27570/
27585 (fight, stop, fight, stop), 422 stop orders in minute 9 (five extractors lost at C5-C7 under them) and 1,026
in minute 15, through engagement #13 (15:12-15:55 at B8: 2,525 lost for 0 with 665 of soldiers, 510 of turrets and
the commander on the spot against 7 Maces, 5 Centurions and 4 Rocketeers). The player saw it twice (notes 9:30,
15:30); its `lane group_B: focus, kite, march` was refused (`focus` retired) and `lane raw` at 15:40 ended the stops.
**Evidence.** `run/matches/1790566750-player-20-live/00/record-0.jsonl` `cmd` rows (stops per minute, unit
15785's sequence); `groups.rs` `engage_group` (`from`) against the leash at `GroupTask::Engage`; the review's H5.
**Status.** fixed 2026-09-28: the anchor is the body's place. Not yet seen in a game.

### K-player-the-prompt-offered-a-retired-lane-word
**Claim.** The player's prompt described `focus` among the footwork rules `lane` keeps, while the tool's schema
accepts `flee`, `fan`, `kite`, `form`, `march`, `follow` (FOCUS retired with H-MICRO-FOCUS): player-20's 15:30
`lane group_B: ["focus", "kite", "march"]`, its answer to the stop storm, was refused whole, and the storm ran ten
more seconds until `lane raw` at 15:40.
**Evidence.** `run/matches/1790566750-player-20-live/00/strategist-0.jsonl` (the 15:30 tool_call, "lane: REFUSED");
`crates/bot/src/strategist/mcp.rs` (the lane schema); `strategist/player.md` line 74 before this fix.
**Status.** fixed 2026-09-28: the prompt's sentence names `form` in `focus`'s place.


### K-hands-a-plurality-over-near-duplicates-picks-nothing
**Claim.** A Choice's probabilities sum to one, so worlds that share a change split its support while the one world
with unique content keeps its share whole: with a median 96 joint worlds a pick, "nothing changes" won 142 of 315 picks
at a median 0.33 while some change held a median 0.46 summed over its worlds (105 of the 142). It is not a
safe-fallback preference: framed without its sentence, as a plan, or last, w1's share on the idle-factory seconds
stayed 0.10-0.12 and the builds' 0.46-0.54, and the order Jev gives the near-duplicates is chance.
**Evidence.** player-26-joint, `docs/studies/2026-09-29-pick-framing.md` §1-§3 (the 10:44 pick re-asked 13 ways; 44
picks under five framings).
**Status.** answered 2026-09-29 by the pick in two stages (H-HANDS-ONE-PASS amended): offline a build on 29 of the 92
idle-factory seconds against 9, w1 8 against 44 (§4). Recheck in player-27: w1's share of picks, builds while the
store is full, the candidate's sampled probability (the mass under 0.05 was 0.26 of a median pick).

### K-hands-one-change-a-second-starved-the-merge
**Claim.** With every world one actor's change from world 1, a plan that needs several actors to move in one second
(ten fragments joining one body, a raid on two flanks) can only land one change a pick, each pick contested by every
plant's build and every raider's hunt; the merge of a fragmented army never catches up with the splitting.
**Evidence.** player-25-nodrops: 664 picks with a merge world on the board, a merge chosen 33 times (5%), "nothing
changes" 323, a plant's build 99, a hunt 86, a builder's step 73; 50 hunt splits in 19 minutes; the player wrote
"join group_K1" for ten groups at 14:08 and one of them had that merge on the board. The single-change form was the
one-pass note's (2026-09-26, mine), reversing the product of the one-decider design (2026-09-25, the user's) on the
bare-Choice diffusion of the joint-worlds study; battery J with consequences holds 0.95-1.00 to 255 worlds.
**Status.** fixed 2026-09-29: the joint worlds restored (H-HANDS-ONE-PASS amended, one-pass note §4). Recheck in the
next player game: worlds a pick, joint worlds picked, merges picked against splits, the pick's latency and tokens.

### K-hands-the-bare-merge-lost-to-a-failed-hunt
**Claim.** Jev knew ten groups of one Blitz stood on one spot (each entry: "at spot_76 (C8)", "1 Blitz", "split_from:
group_A, 37 s ago"; group_A's: "7 groups of 7 soldiers split from it ... still out") and the player had written "every one
of our small groups joins group_A" (6:59, 7:18 "no hunts, no detachments", 7:37, 7:47), and still rated the merge worlds
0.00-0.03 in the pick, six of them a second, against 0.44 for one Blitz hunting a Rover whose words said "its last hunt
of it ended 1 s ago: no hunters". The merge's line said only "group_H merges into group_G and takes its task. Left to
nobody: party_10 ... killing 2 of our Solar Collector": nothing about the body it makes, and the raider marked unanswered;
the hunt's line was the one that named the raider being driven off.
**Evidence.** player-24-hunts `jev-0.jsonl` at 13380 (7:26): the worlds w6, w8-w12 merges, `worlds.pick` w2 0.44, w8-w12
0.00-0.03; the noul `group_H.join_group_G` 0.49, `group_G.join_group_H` 0.14; the `instructions` in the same call. One
merge picked all game, at 8:26 (group_A into group_F, 0.2).
**Status.** words fixed 2026-09-29 (H-HANDS-GROUPS amended: the merge state says the one body and what it is doing).
Open: the merge target is the nearest group only, so "join group_A" was on the board for two of the ten groups; and Jev
picked a hunt the second after the player wrote "no hunts", which is Jev's reading of the instructions, not the words.

### K-hands-a-hunter-held-its-groups-name
**Claim.** With a hunt kept inside its group, the pass's one-actor-one-threat rule (`compose`'s `taken` set) marked the whole group as sent the moment one member hunted: in player-23-clean at 3:53-4:17 one Rover of group_A hunting a Tick 1,800 east (party_6) left the six Blitzes and the second Rover at home unofferable against the Tick killing the base's extractors (party_7), though Jev rated that hunt 0.47-0.56 and "needs answering" 0.72 every second; the only mover offered was the commander (picked at 0.45, cannot catch a Tick), and the extractors at spot_67 and spot_68 died at 4:06 and 4:17. The design's detachment-inside-the-group (threat-response §2) was the cause; the user: a hunt is a group of its own.
**Evidence.** `run/matches/1790652620-player-23-clean/00/jev-0.jsonl` (the worlds at 3:58: no hunt of party_7 among w1-w5; the `party_7.hunt_group_A` question at 0.54), `record-0.jsonl` (the extractor deaths).
**Status.** fixed 2026-09-29: a hunt splits off as its own group (H-MICRO-HUNT amended).

### K-hands-the-gate-re-asked-every-actor-every-second
**Claim.** The pre-pass's cost grows as asks a minute x actors x options, and the ask-on-change that compares one signature of the whole picture stops skipping once there are about ten actors: something differs every second, and every option of every actor is asked again. Most of what is asked buys nothing: a busy builder's options are 57% of the gate's characters for a handful of moves, and nearly half the characters are options of an actor whose own `change` noul came back under 0.5 in the same call.
**Evidence.** `run/matches/1790740188-player-29-hard/00/jev-0.jsonl`, read in `docs/studies/2026-09-30-jev-load.md`: gates 26 a minute at 4:00 and 58-60 from 20:00; 47 questions and 9k tokens a gate at 4:00, 614 and 112k at 25:00 in 2.9 batches sent one after another (340, 831, 1,240 ms for one, two, three); the gate 88.4M of 120.7M tokens and 1,066 of 1,675 s of calls; builders 258,089 option questions for 194 moves from the pick; 212,714 option questions (47% of the characters) under a `change` or `answer` below 0.5. Replayed on the recorded gates, asking a busy actor only on its own change sends 14% of the builders' questions, 44% of the labs', 85% of the groups' (40M tokens for 88M), and removes no call: a threat is in sight nearly every late second (gates 1,627 to 1,613, picks 1,437 to 1,428).
**Status.** conjectured as a fix (H-HANDS-ONE-PASS amended 2026-09-30: `plan::settle`, the batches sent at once); unmeasured in a game. Open: the two pick calls, 27-30 s of a late game minute, which only fewer group changes would cut (1,107 of 1,437 picks changed something).

### K-jev-reads-a-prohibition-asked-per-option
**Claim.** Jev does not apply a prohibition in the player's instructions ("never hunts and never sends detachments") when it rates or picks an option whose own words argue for it ("we outweigh it", "can catch it"), and a general sentence telling it to ("a change the instructions forbid is never the best one") only leans every change toward "nothing changes". Asked about one option at a time whether the instructions forbid it for the group that would make it, it reads the packet's own terms, and with that reading said on the option's line the pick follows it.
**Evidence.** `docs/studies/2026-09-29-pick-framing.md` §8, player-29-hard's recorded picks replayed: 181 picks offering a hunt by a few soldiers of the ball under packets forbidding it, 92 early picks under packets ordering such hunts. The sentence in the pick's question: stage-one mass on the forbidden hunts 0.40 to 0.39, stage two 0.76 to 0.65 with the joins falling alike (0.74 to 0.66). The gate's hunt noul as asked: 0.37 forbidden against 0.32 ordered. The per-option question: the ball's hunts a median 0.84 (p10 0.72, n = 351), the early hunts 0.53 (p90 0.68, n = 841; a Rover on a Tick at an extractor 0.33, a Blitz on a Tick while Rovers were the named hunters 0.60). The sentence on the line: mass 0.40 to 0.08, the top option a forbidden hunt 100 to 3 of 181, chosen over world 1 172 to 10; the unmarked hunts of the early set 0.47 to 0.49.
**Status.** conjectured as a fix (H-HANDS-ONE-PASS amended 2026-09-30: `plan::FORBIDDEN` 0.7, hunts only); unmeasured in a game, and the bar of 0.7 not replayed.

### K-player-the-first-seconds-were-the-default-instructions
**Claim.** The player's first orders landed seconds into the game (about 0:09 in the arena under the think penalty, ten to thirty seconds in a realtime game), and until then the hands asked Jev under the default "no player is connected" instructions and played the answer. Whatever that first pick was opened the game, whatever the player then ordered.
**Evidence.** Every player game's `jev-0.jsonl` has a gate at 0:02 with `instructions` starting "Standing instructions (no player is connected" and the player's first list in force at 0:08-0:09. Through player-28 the pick was the extractor at spot_67, the list's own first step; in players 29 and 30 it was a solar (`docs/studies/2026-09-29-pick-framing.md` §9), a fourth before the plant against the brief's "never a fourth", the plant at 1:18 for 1:08.
**Status.** fixed 2026-10-01 (H-PLAYER-OPENING-TURN): the opening turn is taken before the game and the hands wait for it. Three arena smokes, 2026-10-01 (opening-smoke-1, -2-realtime, -3-two-seats): the shim connected at `init` with the game's own spot list; lockstep, one seat: the opening turn at frame 0 in 9.9 s of wall, the list in force at the first pass (0:02), M M M S S S and the plant at 1:08; realtime (no placing phase in the arena, so the turn ran 14.6 s into a running game): the hands waited and first played at 0:10, the plant at 1:15; two seats: one turn naming both commanders, the plants at 1:08 and 1:10. No game with people yet.

### K-hands-a-produce-list-was-an-allowance-not-a-sequence
**Claim.** A factory's `produce` list with counts read to the player as an order ("one constructor, three Rovers, a constructor, then Blitzes") and to the code as an allowance: every unit named stayed on offer while its total count lasted, and Jev picked among them each time, rating the constructor highest in the first minutes.
**Evidence.** player-30-hard, the plant's list `armcv:1, armfav:3, armcv:1, armflash:4, armcv:1, armflash:4, armcv:1, armflash`: made a constructor at 1:38, a Rover 1:50, a Blitz 1:55, a constructor 2:02, a Rover 2:16, a constructor 2:23 (the gate's nouls at each pick: armcv 0.61-0.85, armfav 0.42-0.62, armflash 0.40-0.73); opening-smoke-1: constructors at 1:29 and 1:44. The brief's Comet line said "the hands build the counted entries in order, each to its count, the open entry last as filler", true only while the plant's default (the first permitted entry) played, deleted 2026-09-28.
**Status.** fixed 2026-10-01 (H-HANDS-PRODUCE amended: `sequence_next`, the counted entries made in order by code). player-31-hard: told `armcv:1, armfav:3, armcv:1, armflash:6, ...` the plant made a constructor at 1:28, three Rovers 1:43-1:50, a constructor at 1:54, Blitzes from 2:14; 167 factory plays from the list and 59 from the pick. The player read the seconds between the last counted unit and the pick as "plant idle (its counted list ran out)" (8:53, 14:41), and a list of counted entries only (`armcv:1`, 21:51) left the first plant making nothing from 22:05 to the end.

### K-hands-an-unarmed-party-taking-a-building-apart-reads-as-harmless
**Claim.** A resurrection bot (or any constructor of his) taking one of our buildings apart is shown to Jev as a party that "cannot hit us", by its definition name alone and with no word of what it is doing; Jev rates answering it at or under the bar and the pick keeps world 1, though the player's instructions order the answer. (The user, watching player-31: "at 6:51 an enemy Lazarus came by and started deconstructing our mexes with no response".)
**Evidence.** player-31-hard, his Lazarus (unit 13091), `jev-0.jsonl` and `record-0.jsonl`: seen at 6:20; the extractor at spot_54 fell from 100% at 6:23 to 4% at 6:33 and was gone; the one at spot_62 from 100% at 6:40 to 6% at 6:50 and was gone; the Lazarus died at 7:39. The player's turn at 6:25 (in force 6:33) told the home picket to "attack on sight, all together, any Pawn, Tick, scout car or resurrection bot at ... spot_62, spot_54, passage_1" and the commander likewise. Every second from 6:21 to 6:52: `party_9.answer` 0.41-0.51 on "Yes when it is killing or about to kill something of ours", the party written "party_9 (1 armrectr, at spot_62)"; the hunt by the picket 0.23-0.55, its line "it cannot hit us: nothing there shoots at what this group is", the hunter on offer one Rover ("31 metal: it outweighs us"), the commander 0.13-0.32 ("it outruns everything here", while it stood still ten seconds at each extractor). The one second the worlds opened (6:39): stage one sampled the picket's whole-group attack, stage two kept world 1 at 0.54 to 0.46. Neither the rules nor the state say "Lazarus", what an `armrectr` is, or that the extractor's health was falling 10 points a second (the bot's own samples have it); our own units are written "Blitz (armflash)", his by definition name only.
**Replay (2026-10-01, `run/jev_taking_ab.py`, `docs/studies/data/taking-apart-2026-10-01.jsonl`, $0.03).** The 19 recorded gates of 6:21-6:52 asked again, party_9's questions only. As recorded: the answer noul a median 0.43, at 0.5 or over in 4 of 19 seconds. The packet's unit names made internal and nothing else: 0.43, 5 of 19: the name was not the cause here. With the party written "taking apart our Metal Extractor (armmex), 66% left and gone in about 8 s" and the odds "it is unarmed and cannot fight back": 0.80, 15 of 19 (0.77-0.86 on every second of a reclaim; the seconds it walks between the two extractors and after the second stay at 0.31-0.62). The moves' own nouls hardly move (the picket's hunt 0.46 to 0.47, its whole-group attack 0.26 to 0.34, the commander 0.23 to 0.26), but a threat's moves all enter the pick once the answer clears the bar. The 6:39 worlds with the words of 6:39, 6:41, 6:44 and 6:48: under the recorded words stage one led with the picket's whole-group attack (0.27-0.34) and world 1 beat the one-Rover hunt 0.66-0.73 to 0.27-0.34; under the new words stage one led with the hunt (0.41-0.53) and the hunt beat world 1 0.69-0.83, the whole-group attack 0.61-0.85 (0.58-0.64 before). The picket's hunt read as forbidden at 0.67-0.71 throughout: the packet said "all together".
**Status.** fixed 2026-10-01 in words (H-HANDS-PARTY-KILLING amended: `track_takings`, `Party.harming`, `Party.unarmed`; the role text asks for internal unit names in the packet, on the user's word, with no effect measured in this replay); unmeasured in a game. Not covered: the seconds before a reclaim starts (the first extractor went 13 s after first sight, the picket 13 s away).

### K-hands-no-group-could-stand-with-a-builder
**Claim.** A group could be told to stand at a place, never to stay with a builder: a constructor walking from spot to spot had no soldier beside it unless the player moved a station after it turn by turn, and a raider at it was answered from wherever the groups stood. (The user, watching player-31: "at 3:17 a pawn starts harassing one of our constructors, while Blitz units are free and could have stood guard".)
**Evidence.** player-31-hard, `strategist-0.jsonl`, `jev-0.jsonl`, `record-0.jsonl`. The packet from 0:10 expected raids "from the east along the south (passage_3 at D8) and from the north-east" and stood every Blitz and Rover at spot_69; at 2:18 the player queued constructor_28188 to spot_62 and spot_54 on the west side with nothing about a guard; at 3:08 two Blitzes fresh from the plant, 900 from spot_62, joined group_C at spot_69 as the packet said. 3:12: a Pawn seen at A6, 395 from the constructor, group_C 1,950 away. The hands answered in the first second (the whole group at 3:13, one-Blitz hunts at 3:14 and 3:16). 3:16: the extractor frame at spot_62 dead; 3:18-3:22: the constructor from 100% to 63%; 3:29: the Pawn dead with a Rover. The player's turn at 3:13 ordered nothing; at 3:38 it split the pickets east and west and at 3:55 put them back at spot_69.
**Status.** an option since 2026-10-01 (H-HANDS-ESCORT: the pick chooses it, the code follows the builder and decides no fight). player-32-hard: the player wrote no escort; the state was offered 346 times to groups whose paragraph named the commander for another reason and rated a median 0.05. Where the pickets stand stays the player's judgement.

### K-hands-a-list-sent-its-builder-back-at-the-raider
**Claim.** A builder on the player's list that the pick sent home from a raider was ordered back to its step by the list the next second, and a list whose step had just died under a raider went on to the next step past it: the retreat the hands picked lasted a second.
**Evidence.** player-31-hard, the `played` rows of `jev-0.jsonl`: 92 plays "go home; its list step ... waits for it" on listed builders, 83 of them followed within three seconds by a list step for the same builder, over 17 builders. 3:17: constructor_28188's frame at spot_62 dead, the list ordered "extractor spot_54" with the Pawn 75 from the constructor and 340 from spot_54; 3:19 the pick sent it home; 3:20 the list ordered spot_54 again; the player cancelled the list that turn ("its list kept turning it back toward the Pawn"). 3:42-3:47: constructor_4919 sent home twice and ordered to `armllt spot_76` by its list three times in five seconds, a Pawn at spot_76. The 30 s hold after a retreat (ca5f797, 2026-09-26) was deleted with the defaults on 2026-09-28 (f2cf462).
**Status.** fixed 2026-10-01 (H-HANDS-SCRIPT amended: the pick's way out holds the list while the builder is still threatened, `list_is_held`). player-32-hard: 22 ways out on listed builders, none undone by the list within three seconds; the commander's 12 attacks on a raider all were (not covered); and the hold did not end for a builder ever hit (K-hands-a-unit-hit-once-was-under-fire-for-the-game). The 3:17 step toward the Pawn, ordered before the pick had answered, is not covered: a wait judged in code was built and removed the same day on the user's word.

### K-hands-a-unit-hit-once-was-under-fire-for-the-game
**Claim.** The hands' record of units under fire (`Pianist.hits`, "our units hit since the last call") was cleared after each ask until the one pass replaced the menus (cf66874, 2026-09-25) and never after: from then a unit hit once counted as under fire for the rest of the game. A builder once shot was "threatened" for good: on a list it stayed in the pass with its ways out instead of off it, it never had its next step queued behind a build at 60%, a started build never shut its other states, and a group with any member ever hit kept its shooter-out-of-sight states.
**Evidence.** The code: `schedule.rs` inserts, nothing removed (`git log -S"hits.clear()"`: added 0d7f1cd, gone with `hands.rs` in cf66874). player-32-hard, where the list hold made it visible: the hold ends when the builder is no longer threatened, and builders showed as held for 641 builder-seconds; the commander idle at home from 33:57 to the end with the nearest enemy party 2,679 away, constructor_15017 idle from 18:52 to 19:36 with the nearest 2,062 away, constructor_26002 idle at home at 35:06 with the nearest 827 away (`jev-0.jsonl` pictures against `record-0.jsonl` positions).
**Status.** fixed 2026-10-01: a unit is under fire for three seconds after its last hit (`Pianist::under_fire`, `UNDER_FIRE_FRAMES`). Every game from 2026-09-25 to player-32 ran with the stale record; what it cost them is not measured.

### K-player-a-second-opening-list-added-an-extractor
**Claim.** A step the old list had queued behind the build in progress survived the list's replacement: the builder built it, and then the new list's steps. Queued steps are ordered when the build before is 60% done, so every build has such a window, and a replacement landing in it added a step neither list meant.
**Evidence.** player-32-hard: the pre-game list (`extractor` x3, `armsolar` x3, `armvp`) played extractors at 0:02 and 0:08 and queued its third (spot_68) at 0:17, frame 510; the 0:10 turn's list (`extractor spot_74`, `extractor spot_73`, `armsolar` x3, `armvp`: the second and third extractors by name) came into force at 0:17, half a second later. The commander built spot_68, the new list skipped spot_74 as standing and played spot_73 at 0:33: four extractors, a walk of about 990 between the last two with nothing begun from 0:29 to 0:55, the plant at 1:34 against 1:08 in player-31. The player's note at 1:10: "4 extractors but plant not started, 1093 metal banked". The 0:10 report had shown both steps already issued; the player could not see the third. In the code the queued task's clock restarted when it became the task (`promote`), so the rule that an unstarted build gives way to a list set after it was ordered (`play_lists`) never applied to it. The first reading of this game (that the new list "starts from its own first step" and built again what the first had built) was wrong: the new list named exactly the extractors still to come.
**Status.** fixed 2026-10-01 (H-HANDS-SCRIPT amended: `list_replaced`, `take_queued`); a unit test, not yet seen in a game.

### K-hands-a-first-list-lost-steps-to-the-builders-earlier-builds
**Claim.** The first-list skip of 2026-09-24 matched a builder's first list against the first things the hands had ever ordered that builder to build, by kind and without the place, so a constructor's first list in mid-game lost its leading steps whenever its kinds matched, and the report said the hands had built them.
**Evidence.** player-32-hard, eight firings ("its list arrived after the hands had built its first N steps"), set against what the pick had ordered each constructor before: four right by coincidence (the list's first step was the build under way: constructor_15017 spot_69, constructor_26737 spot_62, constructor_25508 `armmoho spot_63`, constructor_17024 a solar), four wrong, five steps never built: constructor_14458 6:18 `extractor spot_50` (it had built at spot_68), constructor_3166 6:42 `extractor spot_61` (spot_56), constructor_23638 13:14 `extractor spot_52` and `armllt spot_52` (spot_19), constructor_22685 38:46 `extractor spot_76` (spot_68). player-31-hard: six firings, not checked one by one.
**Status.** retired 2026-10-01 with `Pianist::ordered`: its reason (the hands' default opening running before the first list) went with the opening turn of 2026-09-30. Not covered any more: a player whose first turn sends no list for the commander and who sends one later gets the list's steps on top of what the hands built meanwhile.

### K-jev-a-forbid-noul-on-every-arm-ranks-but-does-not-separate
**Claim.** The hunt's second question ("Read the player's `instructions` alone: do they forbid this move…", `plan::FORBIDDEN`) asked of every arm a gate offers orders the arms by how far they are from the packet: what a sentence of the packet forbids rates highest, what it orders lowest, and what it does not speak of sits between and overlaps the forbidden. No bar separates "the packet says never" from "the packet does not ask for this". The pick already plays little that the noul rates high, so a mark on every arm would touch few plays.
**Evidence.** Offline (the user, 2026-10-01: "for each offered world arm, let's add a separate nonce that asks 'did the instructions forbid this' offline"), `run/jev_forbid_ab.py` on player-32-hard: every gate of 12:00-22:00 and every fifth elsewhere, 805 calls, 90,638 arms, 14.2M tokens, $0.60; the played arms and the labelled window in `docs/studies/data/forbid-every-arm-2026-10-01.jsonl`. The hunts' replayed noul fell on the game's side of 0.7 in 90% (median difference 0.02). Under the 18:53 packet (group_X4 "never turns back west or south", "never splits detachments", "does not wait for soldiers still on the road"; group_P4 "never joins group_X4", "advancing as one"), by a hand reading of the packet: forbidden in its words, group_P4 joining group_X4 0.97, group_X4's splits 0.94, its scout 0.93, group_P4's splits 0.91, group_X4's fall back home 0.89, its walks west 0.84-0.88, its hunts 0.85, its gatherings 0.82-0.85; ordered, the one-soldier groups joining group_P4 0.17-0.27, group_P4's advances onto its named spots 0.34-0.58, group_X4's advance to spot_18 (its next stop) 0.55 and to spot_8 0.42; not spoken of, shelling from a standoff 0.40-0.45, closing on the shooter 0.79, the sweep 0.84, pulling out 0.80-0.89; and against the packet's order, group_X4's raid on spot_18 ("kills the armllt, armclaw, armbeamer and armhlt there") 0.82. Over all arms the median is 0.73 and 60% are at 0.7 or over (32% at 0.8, 18% at 0.85, 8% at 0.9); over the 647 played arms the median is 0.43 and 10.2% are at 0.7 or over (2.6%, 1.2%, 0.2%). Of the 66 played arms at 0.7 or over, read against the actor's paragraph: about eleven break a sentence one can point to (a hunt of one armpw by a ball that "never chases a lone armpw", walks back to a place the packet had just abandoned, a fall back home under "never falls back home"), about five are what the packet ordered or allowed, and the rest are moves it does not speak of (most of the 24 joins, eleven of them by groups the packet does not name, and eight closings on a shooter). One play that broke a sentence sat under the bar: group_X4's fall back home at 19:04 under "never turns back west or south", 0.69.
**Status.** measured offline on one game; nothing built. The reading of the 66 is one reader's, against paragraphs cut at 420 characters.

### K-hands-the-ball-at-the-nest-was-strung-out-and-changing-course-not-split
**Claim.** In player-32-hard the ball lost its fights at his F3 nest without the pick splitting it: it reached the nest as a column, and the pick changed its course every few seconds among moves the packet did not order. The player read this as "they split detachments and feed them in".
**Evidence.** player-32-hard's Jev log, every played group arm of 12:00-22:00: 114 joins, 38 hunts, 32 whole-group attacks, one split; of the hunts twelve are of one soldier and eight of four or more. From 18:20 to 20:20 group_X4 (the ball) was never the source of a hunt or a split. Its eighteen plays: four advances to `shelling`, four shellings from a standoff, three whole-group attacks, a fall back home (19:04), an advance to spot_37, a walk to spot_32, three more advances to `shelling`, and the join into group_P4 (20:19); never the advance to spot_18 the packets of 18:16 to 19:13 ordered. The gate's own words at the lost fights: "for the 11 of its 35 soldiers in the fight", the tail 958 to 1,416 behind the front. The one detachment near there, 16 of group_P4 at 20:22 (the player's note of 20:28 names it), was a hunt under a packet saying group_P4 "goes in NOW with the soldiers it has", "does not wait for its tail", when group_P4 was 62 soldiers with its tail 4,956 behind its front, every new unit of the plants having joined it from home; the game rated that hunt 0.34 forbidden.
**Status.** observed 2026-10-01. It corrects this session's first reading of the game ("the hands split it and fed it in"), which was the player's note taken as the finding.

### K-player-a-raid-went-where-the-scout-had-seen-nothing
**Claim.** The player's report said which metal spots had never been seen and nothing about what had been seen: not that a place was seen empty, not when, and not that no factory of his was on record in so many words that mattered. A player holding a lead sent it at ground a scout had already shown to be empty, and took the place of his commander's last sighting for his base.
**Evidence.** player-33-hard (the user's review: "we succesfully scouted their base at 2:57, but in a critical move-out at 7:43 we march the army to the wrong corner of the map"). 2:55: the roving Rover found his commander, a radar and a light turret at spot_25 (H3); the player's note at 3:03: "his base is north-east"; the truth: his commander began at G1 (6899, 681), his lab stood at H1, and the three spots around them (two in G1, one in H2) were "never within sight" in every report from 3:13 to 23:51, each of which also said "its factories standing as far as we know: none seen standing". 5:52: "Spending the lead on his southern strip (79,75,70,77,72,65)", with the reason "His base is in the north-east at H3, so the raid hits his far end"; the reports said he was known to hold two extractors, both in H3, and that every spot of his box but those three had been in sight; in truth nothing of his stood south of row 3 all game. The group of 20 was at H8 at 7:54 with one Pawn killed; five extractors of ours died in the minute before 7:58. His lab entered the report at 23:51, 33 s before his commander died. The brief's player-18 passage names the same route as the mistake; whether it planted the route or failed to stop it the trace does not say (the turn recorded no reasoning).
**Status.** observed 2026-10-01; answered with information (H-SCOUT-GLANCE: the report's `scouting` block), not yet measured in a game.

### K-hands-a-list-step-was-taken-as-done-by-type-alone
**Claim.** A list step reaching a builder whose current build order was of the same type was reported as "the build already under way: taken as done" and dropped, whatever place the step named: the test (H-HANDS-STARTED's keep of a build ordered again, extended to list steps on 2026-09-24 so a list's solar would not re-issue the pick's) compared the unit type only. A builder walking to an extractor the pick had ordered lost every `extractor spot_N` step of a new list, one each second, until a step of another type came up or the build began.
**Evidence.** Found by the player of fable-2-medium (its note at 2:46: constructor_28188's list "skipped extractor 62 and 54 as 'already under way' though it had not arrived"). The report's lines set against the builder's last build order in the Jev log, four games: fable-2-medium 16 such steps, 6 of them naming a place other than the build under way; player-33-hard 9 and 6; player-32-hard 10 and 7 (at 24:08-24:12 constructor_27366's `armmoho spot_67, spot_74, spot_68, spot_62, spot_54`: the first was its build, the other four went in four seconds); player-31-hard 8 and 4. About 23 steps of 43 lost (the match of step to order is by the place's name in the order's words, so a few may be the same site under another name).
**Status.** fixed 2026-10-01 (H-HANDS-SCRIPT amended: `at_the_steps_site`); a unit test, not yet seen in a game.

### K-player-the-same-step-on-three-lists-made-two-plants
**Claim.** A list could not say that a step on several builders' lists is one building, so a player putting several constructors on one factory by giving each the step got several factories; and a list sent again could not say which of its steps were done, so the bot guessed.
**Evidence.** player-33-hard: the player's note at 11:58, "Starting the Advanced Vehicle Plant at avp_yard (south of home) with three constructors on it"; the record: `armavp` frames created at 12:17 by constructor_27527 and at 12:18 by constructor_19417, the first finished at 14:18, the second taken apart by its own builder at 17:37 on the player's `remove` ("AVP reclaim gave 1.7k bank"). The help of a factory frame of the same type within 900 (H-HANDS-STARTED across builders) did not apply: neither frame stood when the other builder was ordered.
**Status.** answered 2026-10-01 with ids on list steps (H-HANDS-SCRIPT, `docs/design/2026-10-01-list-ids.md`); unit-tested, not yet seen in a game.

### K-hands-the-only-way-to-decline-a-fight-was-to-walk-home
**Claim.** Among a group's answers to an enemy party the hands offered one way back, "falls back to home from party_N", with the base wired in as its destination and a plain move as its order; the pick chose it whenever the words argued against the fight (the odds counted for the few soldiers in reach of a strung-out group; "it outranges us ... goes in as one body or not at all"), and large groups walked toward the base. The group's ordinary walks to named places were asked beside it and almost never reached the pick; with the special answer taken away the pick leaves the group on its course, it does not choose a walk.
**Evidence.** player-33-hard (the user's review: "repeated orders for large army groups to return home ... it never should have retreated fully to the base when there was no threat there to repel"). `threats.rs`: `let to = "home".to_string()`. The log: 38 plays of `party_N.back_group_X` (9 of the group's own "falls back to our base"), 24 walks home by groups of six or more, about 5,000 soldier-seconds, four reaching the base (the main group of 37 at 14:14 from 1,707 to 260 from home in 47 s; of 46, six such orders in 30 s from 16:26, alternating with attacks). The player ordered a group home once (nine damaged Blitzes) and wrote "the fall back home is abandoned", "is not going home". In those 38 seconds: the group's own menu was asked in 36, about thirty walks and advances each; a line with one of its own moves stood in the pick in 6. Offline (`run/jev_back_ab.py`, `docs/studies/data/back-home-2026-10-01.jsonl`, $0.03): the recorded mass on the fall back to home a median 0.57; with every world holding it removed, stage two leaves the group on its course in 27 of 38 (attacking a party 10, hunting 9, holding 4, walking 4), sends a detachment in 6, attacks with the whole group in 3, takes the group's own fall back to our base in 1 and a pull out of a shooter's reach in 1; a walk to a named place in none (2% of stage one's mass). Over the game the group's own ways back were asked and left: "falls back to our base" 1,931 times (median 0.13, played 9), a walk to home 799 (0.11, never), "falls back to where it last held" 263 (0.20, never). The packets named no place for a group to go to when it declines a fight.
**Why the walks did not reach the pick (found 2026-10-01 after the first report, which gave the walks' words and the packets as the reason).** A group held by a party's answer in world 1 (attacking it or hunting it) gets no world of its own moves: `plan.rs` `compose` skips its slot (`taken`) and `resolve` sets it back to keep. Counted over the 38 picks by the group's task that second (`groups` in the log): attacking a party 14, a move of its own in the pick 0; hunting 10, 2 (both a merge); holding 9, 8; walking or advancing 5, 0. So for a group in a fight the party's "falls back to home" was the one answer that left it, whatever the packet named. Read in the code and not seen in a game: a place a group's paragraph names is a stop of its route (`diet::named_in_order`), so a fall-back place written there in prose would be offered as an advance when the paragraph says "advance", and world 1 would say "the route's next stop <that place> not ordered" of a group holding short of it.
**Status.** observed 2026-10-01. The user's ruling: no hard-coded retreat mechanics; a way back whose only destination is home is a mistake in the hands' harness. **Removed the same day** (the user: "we should remove the home-wired answers and instruct opus to routinely communicate a fallback point"): the party's `back_` answer and the group's own `retreat`; `home` is a walk like any other, on a far group's menu when its own paragraph names it. The instruction to the player is not written: how its fall-back place reaches the hands (the two findings above) is the user's to decide. Unmeasured in a game.

### K-jev-steps-back-to-a-place-the-packet-names
**Claim.** Given a sentence in the packet naming a place for a group to step back to, and a plain walk to that place on the menu whose words carry the fight's odds, Jev takes the walk in the seconds it had taken the hands' walk home, and mostly leaves it alone in fights the group is winning. A group in a losing fight is opened by its own `change` question, not only by the party's `answer`. With no place named it seldom walks to a place nobody gave it and about half the time falls back onto another group of ours.
**Evidence.** player-33-hard, offline (`run/jev_fallback_gate_ab.py`, `run/jev_fallback_ab.py`; `docs/studies/data/fallback-gate-2026-10-01.jsonl`, `fallback-place-2026-10-01.jsonl`). At the gate, 36 fall-backs to home: the named walk at 0.5 or over and among the group's two best moves in 35 (median 0.73; asked as an answer to the party, 32); the same walk with the packet as recorded, 4; falling back onto our largest other group, 17; the group's `change` 0.5 or over in 27 of the 27 seconds it was asked (median 0.86). In the pick, 38 fall-backs: the named walk is stage one's top change in 32 and beats "nothing changes" in 37. Controls, 38 seconds where a group attacked a party it outweighed: through the gate in 10, picked in 2.
**Status.** measured offline 2026-10-01; one game; the place is chosen by a rule standing in for the player and the lines are hand-written; the pick replay adds one world to a recorded pick. Unbuilt: `docs/design/2026-10-01-hands-rebuild.md`.
**Would be wrong if.** In a game on the rebuilt menu, groups whose packets name a fall-back place still stand in losing fights, or leave winning ones for it more than the controls' rate.

### K-jev-its-own-best-rated-move-carries-a-route
**Claim.** A group keeps a prose route's momentum when world 1's line names the move Jev's own gate rated best for the idle group ("a stop it has reached, with its next move, the advance to spot_46, not ordered"), as well as when code reads the next stop from the packet's order; a list of the unreached places in no order does not do it, and the wording matters.
**Evidence.** player-9-posing's 24 arrivals, four repeats, the two-stage pick (`run/route_ab.py --two-stage`, variants B, E, F, G, H, J, K; `docs/studies/data/route-ab-2026-10-01*.jsonl`): the next leg picked 24 of 24 with code's "next stop" (E), 7 with no cost line (B), 10 with the unordered list (F), 16 with a first wording of the best-rated move (H), 24 with the wording above (J), and 24 with a fall-back place also named in the prose and on the menu (K). At the 3:58 fork K's scout took the fall-back its new sentence ordered.
**Status.** measured offline 2026-10-01; one game, seven moments. Unbuilt.
**Would be wrong if.** On a menu with many more moves the gate's best-rated move at an arrival is often not the next stop (here it was, 24 of 24).

### K-jev-half-the-bill-is-the-picture-sent-again
**Claim.** In a player game 44% of Jev's input tokens are the state, sent whole in the gate and in both pick calls of every second (16,000 characters, of which the rules text 4,500); a group's walks and advances are another 16% and are played about 65 times; a third of the picks play nothing. The cost of asking one actor or party, everything counted, is 4,200 to 6,000 tokens through the whole game: what grows is how many are asked.
**Evidence.** player-33-hard (37.4M tokens, $1.57) and player-32-hard (74.1M, $3.11), `run/jev_budget.py`; the rates measured against the service on 2026-10-01 (state 0.34 tokens a character, gate questions 0.28, an empty request 280); `docs/studies/2026-10-01-jev-token-budget.md` §1.
**Status.** measured 2026-10-01 on two games. Seven changes that leave the decisions alone are counted at $0.73 and $1.43 for the two games; unbuilt.
**Would be wrong if.** A game on another map or tier puts the tokens elsewhere (many more builders off lists, say).

### K-jev-a-pick-split-by-component-costs-less-and-changes-more
**Claim.** The opened actors' changes fall into components that do not touch (a median 2 a pick, a median 1 candidate each). Asking each component its own two-stage pick, every candidate's second stage at once and all of it in one request, costs 58% of the game's two pick calls, removes one round trip, and changes 1.58 actors a pick against 1.23.
**Evidence.** player-33-hard, 120 picks of 3 worlds or more (`run/jev_split_ab.py`, `docs/studies/data/split-pick-2026-10-01.jsonl`): 9,700 tokens a pick against 16,700; where a component had a choice (133), the split's best change was the joint pick's most-weighted in 96; both changed a component in 119 cases, to the same change in 92; the split alone in 71, the game alone in 29.
**Status.** measured offline 2026-10-01, one game; compared with the joint pick's own probabilities, not with a better answer. Built 2026-10-02 (the `split` layer, d4bacb7) and measured in player-38-split: on 158 audited picks the split's one request cost 58% of the joint pick's two calls (10,206 tokens against 17,451) and took 1.12 moves a pick against 0.89; the extra moves were groups' (walk, join, send, attack); two spends of metal in one pick came once in the 158, and the joint pick took the same two. The game was lost (30:30) on an unconverted lead; whether the extra group moves cost anything is not separated (group picks within 10 s of the last: 64% against player-36's 59%).
**Would be wrong if.** In a game the split's extra changes are the ones the joint pick was right to decline (two groups leaving one area; the spending case did not show in player-38).

### K-jev-the-rules-text-steers
**Claim.** The hands' standing rules text is 1,019 tokens in every call (12% of a game's bill) and it moves answers: with it taken out, or cut to what the words mean, about one in five of the moves the gate rated 0.5 or over as an actor's best falls under 0.5; the pick's choices move less (stage one the same choice in 49 of 60 against 53 re-asked unchanged, stage two 56 against 58). Cutting each question to its actor and its move saves a quarter of a gate and moves about as much.
**Evidence.** player-33-hard, 50 gates and 120 picks asked again under four arms (`run/jev_trim_ab.py`, `docs/studies/data/trim-2026-10-01.jsonl`): a strong move still 0.5 or over in 66 of 72 re-asked unchanged, 53 with no rules, 48 with short rules, 53 with bare questions.
**Status.** measured offline 2026-10-01, one game. Not a verdict on whether the moves it lifts are good ones.

### K-hands-the-rebuilt-base-costs-ten-times-the-old-hands-with-builders-off-lists
**Claim.** With every builder off a list (a fixed packet, no player), the rebuilt hands' base version with the `news` layer costs about ten times what the old hands cost on the same seed, and almost all of it is builders: a builder's menu is every type it can make where the base layout has it, an extractor at every free named spot, and every type that stands at a place (a defence, a radar) at every named place, 150 to 500 moves a builder each time it is asked, where the old menu had about twenty. The budget study's model of the rebuilt menu ($3 for a game like player-33) was made on player games, whose builders are on lists, and does not hold here.
**Evidence.** Three games on Comet Catcher, the west start, BARb hard_aggressive, seed 1, twelve minutes, the same packet but for group_A's paragraph (`docs/experiments.md`, rebuild-smoke-1, rebuild-smoke-2, rebuild-baseline-old-1; `run/layers_read.py`). The old hands (e690e5e): 3.4M input tokens, $0.14. The rebuild with every type at every named place: 41.4M, $1.74, 216,000 gate questions of which 199,000 a build (92%). With only the placed types at named places: 33.5M, $1.41, 172,000 questions of which 135,000 a build (78%) and 24,000 a walk to a named place. The `news` layer held back 53 to 59% of what the base would have asked.
**Status.** measured 2026-10-02, one game an arm. The design's remedy is the `fuse` layer (the packet decoded once: which buildings and places are whose); unbuilt. Unmeasured in a player game, where the groups carry the bill.
**Would be wrong if.** A player game on the base costs under twice player-33's $1.57, or the `fuse` layer leaves the builders' share above half.

### K-hands-a-closed-actor-could-not-answer-a-party
**Claim.** With enemy parties as questions and the answers on the actors' own menus, an actor the `news` layer has closed must still be asked its moves aimed at a party; closed whole, a holding group beside a raid is no answer to it for up to twenty seconds, however high the party's `answer`.
**Evidence.** rebuild-smoke-1, 4:39-4:47: a Pawn (party_5) shot a constructor for eight seconds about 1,000 from group_A (five Blitzes holding by a pick of the second before); the party's `answer` read 0.60 to 0.80 every second, group_A's menu was closed, and nothing aimed at the party was a question. Over the game group_A was moved by three picks in twelve minutes and six extractors died; in rebuild-smoke-2 twelve died (fifteen were built). The old hands on the same seed lost three.
**Status.** observed 2026-10-02; fixed the same day (the layer skips an actor's `change` and its own moves only; `compose.rs` test "a closed actor's answers to a party are still asked"). Whether the extractors' losses were this or the raids' own variance is not shown by one game an arm.
**Would be wrong if.** With the fix a group holding near a raid still sends nothing while the party's `answer` is high and the detachment is not marked forbidden.

### K-jev-the-forbidden-mark-reads-an-ordered-detachment-as-forbidden-beside-a-never
**Claim.** The forbidden noul ("Read the player's `instructions` alone: do they forbid this move for the group that would make it?") rates a detachment the packet orders as forbidden when the same paragraph forbids other detachments: "met by one soldier sent from group_A; that is the only detachment group_A ever sends" puts the one-soldier detachment at a median 0.85, over the mark's bar of 0.7, and the line then says the instructions forbid it.
**Evidence.** rebuild-smoke-1 and -2 (the packet `run/packets/comet-west-rebuild-1.md`): `send_1` 0.7 or over in 11 of 11 and 64 of 83 asks (median 0.86 and 0.85; `send_2`, `_4`, `_8`, which the packet does forbid, 0.87 to 0.90). The old hands on the old packet's sentence ("(send_against 1) ... the only detachment group_A ever sends, and it never splits otherwise"; rebuild-baseline-old-1): the hunt of one soldier 0.7 or over in 24 of 25 asks, one hunt played in twelve minutes. Four `send_1` were played in rebuild-smoke-2, none in rebuild-smoke-1.
**Status.** observed 2026-10-02 on one packet under both the old and the rebuilt hands; the mark's bar (0.7, the user's) was measured on packets that forbade every detachment. Not replayed with other wordings. Counted 2026-10-02 on player-38 (160 marks read blind, budget study §10): the mark stood at 0.7 or over on 8 of the 13 detachments the instructions order, on 6 of 37 they are silent on, on 56 of 62 they forbid in words.
**Would be wrong if.** A packet that orders one size and says nothing of the others is also marked, or one that says "one soldier, never more" is not.

### K-jev-a-partys-answer-carries-low-rated-moves-and-they-move-armies
**Claim.** When a party's `answer` opens the moves aimed at it, the moves that go to the pick are mostly ones Jev itself rated under 0.5, and the pick's second stage takes them: an army on a course is turned onto one raider, or flips between attacking a party and walking from it.
**Evidence.** player-34-rebuild-base: 122 of 291 picks that sent a group at a party were of a move rated under 0.5; at 12:56 the main army of 45, advancing and closed by the `news` layer, was turned to shell one Pawn (the move 0.35, the Pawn's `answer` 0.53) and walked from C6 to B3 a minute before the fight that decided the game. player-35-rebuild-layers: 130 of 281, 105 of them carried by the party's `answer` alone and 47 of those on a body of twenty or more; 16:03-16:23 nine picks at three Hounds under a nest (six rated 0.34 to 0.48), ten Stouts lost; 28:46-28:58 eleven picks in thirteen seconds between attacking a Razorback (0.26 to 0.40, the `answer` 0.71 to 0.74) and walking from it (0.51 to 0.66). Counts from the logs (`worlds_gate.flags` beside each `plan` line's played move); the cases from the games' reviews.
**Status.** observed 2026-10-02 in two games. The rule is the rebuild note's §4 ("a move aimed at a party is also a candidate when that party's `answer` is 0.5 or over") as built in §14 (the two best-rated moves aimed at the party go whatever their rating); the old party menus had the same rule over the three nearest groups. Not changed then: the user's to rule.
**Ruled and built 2026-10-03 (the user, after player-49):** a party's `answer` opens a move in place of the actor's
`change`, and the move's own rating must reach 0.5 (`compose.rs` `candidates`; the top-two rule deleted). Chose because:
in player-49 23 of group_O's 74 picks at a party were moves under 0.5 (18 of them shells; the army sent to shell one
Blitz 3,900 away at 0.28-0.39 four times), and words alone did not stop it (the walking distance added to the shell
move lowers a far shell 0.04 on average, 61 moves replayed, and what took its place in the top two at 14:03 was a
forbidden detachment of the same army). Rejected because: the top-two rule was built so a raid always had an answer
(K-jev-a-response-opens-by-the-party-noul-not-its-own: a builder's attack on a raider rated 0.31-0.43). The cost to
watch: in the same game the bar would also have held back 51 of the small groups' 124 picks at a party (28 attacks,
18 detachments) and both of the builders'. Unmeasured in a game.
**Would be wrong if.** With party-aimed moves held to their own 0.5, raids on extractors go unanswered (the reason the rule exists: K-jev-a-response-opens-by-the-party-noul-not-its-own), or the armies' turns come as often from moves rated 0.5 and over.

### K-jev-reads-a-packet-once-well-enough-to-fuse-for-the-actors-it-names
**Claim.** Asked once a packet about the instructions alone, Jev's clear "no" (under 0.2) to "do they name this place, this building, this join for this actor" is right often enough to take the move off the menu until the next packet: of the moves fused off and asked anyway, under 1% were rated 0.5 or over. It holds for actors the packet speaks of. For an actor born after the packet (a detachment) the same "no" is the packet's silence, and fusing on it takes away the walk back and the join back that the actor needs.
**Evidence.** rebuild-smoke-5 and -6 (a fixed packet): 3,263 fused moves audited, 6 faults. player-35-rebuild-layers: 14,291 audited, 111 faults (0.8%: 50 joins, 37 walks, 17 follows, 7 advances; 34 rated 0.6 or over), none of 102 audited forbidden marks; 48% of the base's questions not sent; on groups' menus 76% of walks, 46% of advances, 85% of joins and 97% of follows fused. Of 91 detachments 81 were born with a step-back reading under 0.5 and 73 with the join back fused; group_Y6 and group_Z6 (28:32-28:55) lost six of eight with moves fused that the audit rated 0.55 to 0.68. A join was read both ways ("group_S6 joins group_V4" at 0.91 and 0.93), and at 23:25 the army of 43 joined its detachment. Places in a "never goes to" list read as the group's (265 of 265 at 0.8 or over) and are then asked each second.
**Status.** measured 2026-10-02, one player game and two packet games. Fixed after player-35 and unmeasured since: an actor that appeared after the packet landed and that the packet does not name is not fused; the join question is one-way.
**Would be wrong if.** A player game's audit puts the faults over 2%, or groups the packet names stand without the move they need.

### K-jev-told-to-attack-read-as-on-it
**Claim.** Jev answers the question it is put: told that a group is "on" a party, it says the party needs no other answer, whatever that group is doing on the ground. The words of who stands against a party must be measured (in reach, how far, idle, outrun), not read off the group's task.
**Evidence.** player-35-rebuild-layers, 7:47-8:28 (the user's review: a Tick killing extractors at 8:09 beside nine Blitzes): the party's question read "by someone other than what stands (group_W on it)?" and world 1 "party_17 ... met by group_W" while all nine of group_W stood with no order 1,000 to 1,500 from the Tick; the `answer` was 0.30 to 0.47 in eight asks from 7:38 to 8:06 and the `openers` layer held every other group's attack on it. Replayed offline on the game's own requests, three asks each: 7:57, 8:03, 8:06 with "group_W on it" 0.44, 0.49, 0.51; with "group_W is told to attack it, but nothing of it has the party in reach (the nearest of its 9 soldiers is near, about 10 s of walking away, every one of them standing with no order); the party outruns it ..." 0.81, 0.86, 0.83. At 7:47 to 7:55, the Tick driving and killing nothing yet: 0.30 against 0.43 to 0.54. A Rover's attack on it, 355 away and faster than it: 0.50 to 0.56 with the old words, 0.66 to 0.68 with the new.
**Status.** measured offline 2026-10-02 on one moment of one game; H-HANDS-STANDING built on it. In a packet game (rebuild-smoke-8-standing): the `answer` averaged 0.68 over 74 asks whose question said "told to attack it, but nothing of it has the party in reach" (80% at 0.5 or over) and 0.40 over 16 that said a group had it in reach. player-36-standing (with the player): 0.59 over 277 (70% at 0.5 or over) against 0.41 over 71.
**Would be wrong if.** In a game with the standing words, a party killing our buildings with nothing in reach of it keeps an `answer` under 0.5.

### K-hands-an-attack-was-one-order-to-where-the-party-stood
**Claim.** A group's attack on a party was one fight order to the place the party stood at the pick; the party moved, the group arrived and stood with no order, and its task went on reading "attacking" for as long as anything of ours saw the party. A `join` gave the joiners one move order to the other group's centre and never that group's order.
**Evidence.** player-35, 7:23-8:28, from `record-0.jsonl`: group_W's two Blitzes got their last order at 7:42 (back to the fight point of 7:23, spot_47), the seven that joined at 7:46 a move to where the two had stood; from 7:53 all nine carried the idle flag and no command went to any until 8:26, the task's `to` following the Tick from (3489, 4704) to (2109, 4588). The Rover sent at 8:09 reached its fight point at 8:14 (144 from the Tick, which it hit from health 81 to 68) and stood there while the Tick drove 600 off and killed spot_61 at 8:28. Group-seconds with an attack task, every soldier idle and over 400 from the party (`engage` rows of the Jev log against the record's samples): player-30 0 of 872, player-31 8 of 857, player-32 6 of 809, player-33 28 of 532; player-34-rebuild-base 98 of 1,060 (9%); player-35 193 of 1,522 (13%). The upkeep code was the same in all six; the old hands offered the attack again each second to the group already on it, the rebuilt menu does not.
**Status.** observed 2026-10-02; H-HANDS-ATTACK-FOLLOWS and H-HANDS-JOINERS built on it; rebuild-smoke-8-standing: 0 of 104 such group-seconds (rebuild-smoke-5 14 of 99, -6 4 of 88), one packet game; player-36-standing 22 of 988 (2%). Why the rebuilt hands show it two to ten times as often is not established.
**Would be wrong if.** With the upkeep the share stays where it was: then the idle seconds are something else (the lane's let-go, a party out of sight of the group but in sight of a building).

### K-hands-the-same-layer-keeps-an-answer-the-picture-has-outdated
**Claim.** The `same` layer lets a noul's last answer stand for 20 s while the question's words are the same. The words of "should the commander build a solar now, rather than stand idle" are the same before and after the count the packet's table keys on has changed, so an answer given with one extractor standing is played with two.
**Evidence.** rebuild-smoke-7-standing: the packet says "With 2 extractors and 0 solar collectors: a solar collector". At 0:12 (one extractor, the commander idle) the solar was rated 0.28 and the next extractor was picked; at 0:22 (two extractors, idle again) the gate did not send 47 questions, the solar's 0.28 stood, spot_36 was asked fresh at 0.55 and picked; the commander walked out to five more extractors, began the plant at 2:36 and lost it at 3:24, and the game was lost at 8:33 without a soldier made. rebuild-smoke-5 on the same start and packet asked the solar fresh at 0:19 (0.80) and had the plant at 0:32. The layer's audit in player-35: 26 faults of 1,105 (2.4%).
**Status.** observed 2026-10-02, one game; fixed the same day on the user's word: an answer stands only while the counts it was asked over stand (`layers::counts`: our buildings by kind, the army's size word, the economy's lines without their numbers). Unmeasured in a game since.
**Would be wrong if.** The stale answer and the fresh one agree when the state between them changed (the audit's 2.4% says they mostly do; the opening is where one wrong answer costs the game).

### K-jev-the-course-in-each-question-is-what-the-move-is-weighed-against
**Claim.** A move's question must carry the course it would replace in its own words ("rather than group_W attacks party_17: ..."). With "rather than what it does now", or with the course's words moved into the actor's entry and referred to, Jev rates fewer moves at 0.5 or over: the repetition is a fifth of the gate's question text and it is not free to cut.
**Evidence.** `run/jev_words_ab.py` on 40 gates of player-36 (4,063 own-move answers, `docs/studies/data/words-ab-2026-10-02.jsonl`): of 56 actors whose best own move was at 0.5 or over as recorded, asked again unchanged 48 kept a move there; with the course cut 29; with the course in the entry 33. Mean change of an answer 0.018, 0.045, 0.035; answers crossing 0.5: 1.0%, 2.5%, 2.2%. Tokens 89-90% of the gate's. The budget study's §3 found the same of a shorter cut on the old menus (53 of 72 against 66).
**Status.** measured 2026-10-02, offline, one game's gates. The evening's replay on player-40 (`docs/studies/2026-10-02-jev-question-cuts.md` §2) splits it by kind: a builder's answers barely move without the framing (crossings 14 to 16-21 of 14,112), a group's move two to three times as much as asking again (20 to 34-53 of 3,818) and an idle group's move loses its rating when the question no longer says it stands idle; the fixed 141 characters can go from builders' questions and not from groups'; built the same evening as the diet's `builder_words` (H-HANDS-DIET), unmeasured in a game.
**Would be wrong if.** A form that keeps the course's words but says them once per actor (a question type with shared instructions, should the API gain one) moves the answers as little as asking again does.


### K-jev-answers-move-with-the-order-of-the-pictures-sections
**Claim.** The same request with the picture's sections in another order gets other answers, far beyond what asking again does; a line added that bears on nothing does not. An offline replay must send the sections in the bot's order (its JSON maps are sorted by key), or its "as played" comparison is off.
**Evidence.** `run/jev_context_ab.py`, 100 gates each of player-39 and player-38 (`docs/studies/data/context-ab-2026-10-02/`): sections reversed, 2.91% of 24,059 move answers on the other side of 0.5 against 0.82% asked again, openers 15.4% against 4.1%, forbidden marks 34.8% against 6.1% (376 of 1,211 fell under 0.7); 148 picks, second stage 10.1% another choice against 3.0%. One irrelevant line added: 0.92% against 0.82%. With `instructions` and `rules` moved to the end (the replay scripts' order until 2026-10-02) the answers as played came back in 132 of 11,682 crossings against 86 in the bot's order.
**Status.** measured 2026-10-02, offline, two games' gates; 2026-10-03 on human-12's 32 gates of a named attack, `instructions` after `rules` moved nothing against the control (K-hands-an-anti-air-soldier-is-not-artillery). Five more orders each moved 2.0 to 2.8% of move answers; scored against blind verdicts on 309 questions the orders disagreed on, the bot's order was right in 196, rules-picture-instructions in 206 (p 0.39), the reverse in 181, picture-instructions-rules in 163 (it says yes half again as often); on 160 forbidden marks of player-38 the orders with the actors last ranked forbidden over allowed at 0.74-0.75 against the bot's 0.83. The bot's order stays.
**Would be wrong if.** A larger blind sample puts another order clearly ahead of the bot's, or a reordering is found that moves the answers no more than asking again does.

### K-jev-small-picture-cuts-cost-a-coin-and-larger-ones-cost-answers
**Claim.** What a blind reader never uses can be cut from the picture at the cost of answers near the bar landing either way: `enemy.buildings_seen`, the scouting lines (`never_looked`, `looked_long_ago`, `start_box`), the rules' sentences for kinds of actor not asked about, and a fight's facts said a second time in a question. What a reader does use cannot: the other actors' entries, the "Given ..." opening of a question, the move's odds in the forbidden question, the lists and the actor's description in the decode's questions.
**Evidence.** The budget study §10 (`docs/studies/2026-10-01-jev-token-budget.md`): the four cuts together, 7.0% of a gate's tokens and 10.3% of a pick's, put 1.32% of move answers on the other side of 0.5 against 0.83% asked again and 0.89% for a placebo line; blind Opus reviewers sided with the cut in 36 of 72 decided crossings and used a removed part in 2 of 133 questions. "Given" cut: wrong in 46 of 67. Picture cut to the asked actors: wrong in 29 of 42. The short forbidden question: 10 of 29 forbidden detachments marked against 22. The short decode: 14 of 90 newly fused readings were things the instructions say.
**Status.** measured 2026-10-02, offline; the four cuts built the same day (H-HANDS-DIET `only_used`), unmeasured in a game.
**Would be wrong if.** In a game with the four cuts built, the layers' audits or a review show answers the full picture would not have given.

### K-jev-a-yes-near-the-bar-is-mostly-wrong-to-a-blind-reader
**Claim.** Of the gate's answers that a small change of the request carries across 0.5, a careful reader of the same picture, instructions and rules says no to about four in five: Jev's marginal yeses (walks, follows, joins and detachments rated just over the bar) are moves the instructions do not call for.
**Evidence.** Seven blind review packets of 2026-10-02 (`docs/studies/data/context-ab-2026-10-02/*.verdicts.jsonl`): of 263 decided crossings the reviewers said yes to 56; on controls far from the bar they agreed with Jev in 152 of 160. Whichever arm said yes at a crossing was wrong in most (`no given`: 7 of 42 added yeses right; `asked actors` 2 of 27; `no roving` 3 of 23).
**Status.** conjectured 2026-10-02: one reviewer a packet, a model's reading and not a person's; the sample is crossings, not all answers near the bar.
**Would be wrong if.** A person's reading of the same packets says yes to half or more, or a sample of answers between 0.5 and 0.6 that did not cross reads yes as often as answers over 0.7.

### K-jev-the-lab-afford-flag-reopened-every-constructor
**Claim.** The cheapest way to cut a game's gate text is in who is reopened, not in the words: the `news` layer's store flag carried whether an idle lab could afford its cheapest unit, that part flipped twice every lab cycle, and it reopened every constructor each time. A fifth of player-40's gate text was constructors reopened this way, 46% of their openers said yes, and 0.1% of the 75 moves a yes asked reached the bar.
**Evidence.** Player-40's Jev log (`docs/studies/2026-10-02-jev-question-cuts.md` §1): 112,237 questions of 466,274 from builders open on the store flag, 117 at 0.5, 73 played; the flag's three parts reconstructed at the gate frames, the afford part 149 flips of 231; 8 to 9 constructors reopened at once at 13:00, 13:41, 13:42.
**Status.** measured 2026-10-02 on one game; the afford part moved to the labs alone the same evening (H-HANDS-LAYERS `news`), unmeasured in a game.
**Would be wrong if.** A game with the change shows constructors idling over a store they could spend, or the moves they used to find on the store's news (73 a game) not found at their own news or the 20-second re-ask.

### K-jev-a-place-question-halves-the-place-moves
**Claim.** One noul per place ("does any of ours have a reason to go to, fight at or build at spot_43 now, rather than carry on; those who could: ...") ranks the places well but says yes leniently: at a bar of 0.3 it keeps 44% of the places and 53% of the place-aimed move questions while keeping 24 of 25 places with a recorded move at 0.5 and 69 of 83 where an idle actor had a move at 0.3. Stricter wordings lower every place with the hits and rank worse; a per-gate top-N rule is worse than the bar. The pre-gate is a halving of 79% of the gate's text, net about a quarter after its own cost, not the fivefold cut its 2.5% hit rate suggests.
**Evidence.** `run/jev_cut_ab.py` on 24 gates of player-40, six wordings (`docs/studies/2026-10-02-jev-question-cuts.md` §3, `docs/studies/data/question-cuts-2026-10-02/`): 1,324 place questions for 16,356 move questions; the first wording's ranking 0.896 (share of no-hit places rated below a hit), the strict one 0.817, the compact one 0.868 at 80% of the characters; the control asked again kept a move at 0.5 at 21 of the 25 places.
**Status.** measured 2026-10-02, offline, one game's gates; 25 hits is a small count. Built the same evening as the `places` layer at 0.3 (H-HANDS-LAYERS), unmeasured in a game.
**Would be wrong if.** A place question that carries less (the place alone, no list of who could go) or more (the instructions' sentence about the place) separates the hits from the rest at 0.95 or better, or a game with the layer on shows moves at the bar lost at blanked places beyond the audit's share.

### K-player-a-lobby-sets-the-faction-at-the-start
**Claim.** In a game with people the factions are set by the lobby at the start, so the pre-game turn cannot name a unit by its internal name; with lists that take internal names only, the player writes prose and the hands play the prose until the lists come twenty seconds into the game, and the lists then build over what the hands already built.
**Evidence.** human-10 (2026-10-02, Gecko Isle, three seats; `run/matches/1790987321-human-10`): the turn-0 text, "We don't know the factions yet, and those orders need the units' internal names ... What I wished I could order: a `produce` or list entry that works for either faction, such as 'solar' or 'a constructor'"; the three Jev logs: a wind on every seat at 0:02 and a lab at 0:10/0:14 on t2 and t3 by Jev's pick (t3's gate: armlab 0.56, armwin 0.52, the nearest spot 0.39) against prose that put the factory third; the lists at 0:17-0:25, a second lab at 1:08/1:14, teal's reclaimed at the player's 1:15 turn; 5/8/6 extractors by 5:00.
**Status.** observed 2026-10-02 (the user: "the openings across the three seats were pretty bad"); role words for lists and `produce` built the same night (H-PLAYER-OPENING-TURN); the hands acting on placeholder prose is open: a seat with no list holding still would be a gate (the no-live-heuristics ruling), the alternative is the player saying "do nothing until the lists" in its prose.
**Would be wrong if.** A game with people on a lobby with factions set beforehand shows the pre-game lists landing at 0:00 as the arena smokes did, with no prose phase.

### K-jev-the-step-back-rule-and-the-players-commit-pull-a-column-apart
**Claim.** Under a fortified party the gate reverses a group's course on alternate seconds: with the advance in force the rules' step-back sentence (over an odds line that counts the soldiers within 800 alone) makes the walk back the move, and with the walk in force the player's "attack NOW" makes the advance the move; `change` says yes either way and the pick takes what is offered. Giving the odds line the whole group's weight against the party and its turrets, and making the step-back sentence yield when the instructions speak to that very fight, stops the walk back being offered while the group advances (7 of 15 moments to 1) and keeps the advance offered while it walks (11 of 13), at the control's drift elsewhere; a sentence that a recent course stands does nothing.
**Evidence.** human-10, group_O_t1 at the D4 outpost, 10:14-10:41: fourteen course changes in 27 s, `change` 0.76-0.91 whichever the course, the opposite move 0.53-0.74; 10,109 metal lost within 800 of the outpost for 1,380. `run/jev_thrash_ab.py` on the 32 gate calls of two windows and 12 other group gates (`docs/studies/2026-10-02-thrash-words.md`, `docs/studies/data/thrash-words-2026-10-02/`).
**Status.** measured 2026-10-02 offline on one game's moments; the pair built the same night (H-HANDS-GROUP-BODY, `picture.rs` `group_odds`, `rules.md`); the user, 2026-10-03, after reviewing player-44 and human-11's replays: "The thrash hasn't been a huge problem in this game and in player-44 ... I would call that fix a success" (player-44: 57 reversals of 643 course changes, human-11: 30 of 178, none under turrets). The whole-group odds alone did nothing, the exception alone half (4 of 15).
**Would be wrong if.** A group that is truly outweighed, with instructions that name its fight, keeps advancing under the pair (the exception reads as "never step back"); or a game with the pair shows the thrash at another kind of place (unseen fire, as at 19:23).

### K-hands-a-newborn-constructors-first-pick-is-the-lab
**Claim.** A constructor born idle is rated best at helping its lab, over the spot the instructions name and over the job a listless constructor was given, and the move's own words carry the argument ("it adds its build power to whatever the factory makes"). The player catches each one with a list a turn or two late; the economy is set back by that much every time.
**Evidence.** human-11 (`run/matches/1790995126-human-11`): four constructors in a row, 2:41 (help lab 0.55 against the listed spot it stood on, 0.36), 5:45 (0.51; "work their lists", no list), 6:14 (0.62), 6:51 (0.53 against the instruction's wreck fields 0.42); re-asked in the long question form 6:14 and 6:51 put the wreck fields first (0.60, 0.61) and the lab third or fourth, 2:41 and 5:45 kept the lab first (the scratchpad's `newborn_ab.py`, $0.004).
**Status.** observed 2026-10-03 in one game; the help move's words now state the fact, the short builder form is off at normal (H-HANDS-DIET), the brief tells the player to name the listless constructor's job and to stage its list before it stands (H-HANDS-LISTS); unmeasured in a game.
**Would be wrong if.** A game with the new words shows newborn constructors going to their listed spots at the first gate, or still to the lab: then the pull is in the picture's economy lines, not the words.

### K-jev-a-losses-line-without-killers-steps-a-raid-back-from-nothing
**Claim.** The rules' step-back on "it is losing this fight" fires on any losses of the last 30 s whatever killed them: a raid that has just killed a turret and lost three soldiers to it reads as losing the fight with the unarmed constructor left standing, and walks home.
**Evidence.** human-11, group_J at spot_7, 6:37: `losses` "lost 3 of its 12 soldiers (162 metal, a large share: it is losing this fight)" (the three to a Sentry dead since 6:32), `enemies_near` "party_6 (1 armck) ... it is unarmed and cannot fight back ... it is hitting this group now: 2 of our Pawn" (the constructor reclaiming them), `change` 0.81, `go_spot_8` 0.78; the player's 8:15 text still had the Pawns "standing idle".
**Status.** fixed in the words 2026-10-03 (H-HANDS-PARTY-KILLING: killers kept, the verdict from the losses to the party and the turrets alone, "taking apart" for an unarmed party); unmeasured in a game.
**Would be wrong if.** With the killers said, a raid still walks back from an unarmed unit at the bar (then the pull is elsewhere in the entry), or a group under a real party no longer steps back because its killers are out of sight (the "something unseen" losses carry no verdict by design: a rule to revisit if it costs a group).

### K-hands-an-anti-air-soldier-is-not-artillery
**Claim.** The shell move's "long-reach soldiers" were chosen by reach alone, so an anti-air bot (Crossbow, armjeth, reach 760 against aircraft and none against the ground) counted as artillery: the move was offered and played against ground parties with nothing that could fire, and its standoff stopped the group short of a fight it outweighed. The odds model was never fooled (it weighs by `can_hit`); the move's eligibility was. Dropping the move does not stop the attack-walk churn around it.
**Evidence.** human-12 (`run/matches/1790998669-human-12`): group_T (13 Pawns, 6 Crossbows) "shells party_8 from their reach with 6 long-reach soldiers" at 8:28-9:21 against four Centurions and lost 13 Pawns at the standoff 9:22-9:32; group_G1 (28 Maces and Crossbows, 3.3k) the same against two Centurions at 12:34-13:20 while they killed the lab, five solars and the nano. Asked again offline without the shell question (`run/jev_shell_ab.py`, 57 gates of the two windows, `docs/studies/data/shell-thrash-2026-10-03/`): the top move still changes 17 times in 38 gates for group_G1 (18 as recorded) and 14 in 17 for group_T (14), the shell's share going to walks; the whole-group odds appended to every question's course words change nothing (attack median 0.46 either way); one sentence appended to the instructions naming the attack halves the switches after 12:43 (12 to 7, walks at the bar 6 to 1) although the player's own instruct already said the same in its middle. `change` answers 0.50-0.68 at every gate of a named attack the group outweighs 6 to 1, four seconds into it.
**Status.** The filter fixed 2026-10-03 (`combat.rs` `artillery`, used by `menu.rs` and `execute.rs`). The sentence's effect is its wording, not its position (the user, after the Jev experiments' "put the relevant fields last": "we should try making it last"): on the same 32 gates the player's own attack sentence repeated as the last line (`tail`), its whole paragraph said twice at the end (`dup`) and the `instructions` section moved after `rules` (`last`) all sit on the control (walks at the bar 5-6 of 32, attack median 0.42-0.45, 10-12 changes of the top move at the bar) where the appended sentence had 1, 0.48 and 7; the player's paragraph was already the last of the packet. What differs is "now", "keeps attacking", and the absence of the player's parenthetical "(the Centurions at home, spot_13, spot_11)", which names the place the walks went to. The churn around it is open: the engine is `change` at the bar during a named fight and a walk to the player's own fallback place (spot_11, which the player also named as where the Centurions were) rated 0.5-0.6 as the alternative. Also found and not built: the picture's whole-group clause counts anti-air soldiers' metal in the group's weight.
**Would be wrong if.** A game after the fix shows a group with real artillery (Lugers, Hammers do not reach 600) no longer offered the shell move; or the churn vanishes in the next game with the fix alone (then the shell standoff was the engine after all).

### K-hands-a-new-group-rewrote-the-allowance-and-reset-the-sequence
**Claim.** A `produce` list with `group: "new"` restarted its counted sequence at every soldier that founded or re-founded the fresh group, because the hands marked the founding by rewriting the player's allowance (`group` to `None`), the pianist reads any changed allowance as a new list and clears the factory's `produced` counts, and in lockstep with the think penalty the turn's landing (`shared.rs` `apply_delayed`, the whole `allowed` map from the turn's end) put `new` back, so the next soldier founded another group and reset the counts again. The hands must never write to the player's allowance; the factory's own group is the hands' own record, keyed by the `produce` call.
**Evidence.** player-45 (`run/matches/1791000152-player-45`): one `produce` at 24:06 (`armacv:2, armbull:3, armmart:2, armbull, armmart`) made seven Advanced Construction Vehicles and six Bulls, never a Luger; the player's report's `allowed` line fell back to "Advanced Construction Vehicle (2 more allowed), Bull (3 more allowed)" at 25:36 and 28:23, each within seconds of a player turn (25:26/25:36, 28:14/28:23) during which a Bull finished and was grouped (25:31, 28:15). No other writer of the map exists (`grep allowed.lock()`).
**Status.** Fixed 2026-10-03 (`groups.rs` `group_for_newcomer(named, call, rally)`, `rally: HashMap<UnitId, (u64, String)>`, the picture's `output_joins` words); test `a_newcomer_joins_the_named_group_else_its_factorys_own_and_new_starts_a_fresh_one`. Unmeasured in a game. The same clobber is the likeliest cause of human-11's group renaming churn under standing orders (open there).
**Would be wrong if.** A plant with a `new` list still restarts its sequence in the next game (then another allowance change is in play: the `call` or the role-word resolution), or `new` lists stop founding a fresh group on a second `produce` call.

### K-jev-a-now-in-a-route-holds-until-the-next-packet
**Claim.** Jev reads a packet's "now" as true at every ask, and the entry's `reached` register does not undo it: a
group between stops is sent back to the stop the packet names "now" although its entry says it reached it. Saying the
stop was reached in the move's words, in the rather-than clause, in a new entry line or in the route sentence does not
make the next stop the best-rated move; taking the "now" out of the route sentence lowers the passed stop and leaves
the next stop under the bar. Standing at a reached stop, the next leg is picked (the routes study's 24 of 24).
**Status.** measured offline on player-49's recorded gates, 2026-10-03, one ask an arm (Jev asked again moves a
rating about 0.015). group_O 12:48, 430 past spot_23 after a fight: the advance to spot_23 0.69-0.71 recorded; 0.70 /
0.54 / 0.66 / 0.69 / 0.46 with the reached fact in the move, the move as "goes back to", the rather-than clause, an
entry `route` line, the packet's sentence marked "(reached 12:20, done)"; the next stop spot_18 0.26-0.39 throughout;
over 41 gates the 197 reached-place moves shifted 0.02 or less on average. The route sentence without "now", and as
"the place on the way after the last one in its `reached`, never back": spot_23 over the 7 gates that asked it
0.51 -> 0.39 and 0.35 on average, spot_18 never over 0.5 between stops, 0.51-0.55 at the stop (12:20-12:21).
**Evidence.** `run/matches/1791038139-player-49/00/jev-0.jsonl` replayed with `run/jev_ab.py`'s `requests_of` and
`ask`; the scripts were scratch and are not kept. The user's reading (2026-10-03): "Jev operates mostly statelessly,
so if you say to do something now it will do it always until the next frame."
**Would be wrong if.** A game played under the prompt's paragraph still shows a group sent back to a passed stop with
no "now" in its packet, at ratings over the bar.
**Used by.** the player prompt's "nearly stateless" paragraph; [[H-PLAYER-NO-NOW-IN-A-WAY]].

### K-jev-the-cost-of-leaving-a-fight-under-fire-said-on-the-shell-move-stops-the-switch
**Claim.** A group attacking a party and hit this second rates the shell on that same party near the bar, and the
switch goes to the pick about a third of the time; one sentence on the shell move saying what the switch does to a
group under fire takes every such shell under the bar and moves nothing else.
**Status.** measured offline 2026-10-03, one ask an arm, on the recorded gates of two games. player-49 (20 gates): the
shell's mean rating 0.46, 8 at 0.5 or over, 7 going to the pick; with "; group_X is under his fire now, and this takes
every soldier out of the fight it is in: they turn and walk to the standoff and the screen point, shooting less while
they walk and bunching where they arrive" 0.32, none; with "; this ends the attack group_X is making under fire: a group
that changes between attacking and shelling walks instead of shooting, and loses soldiers for it" 0.32, none.
player-46-pressure (22 gates): 0.39, 7 and 4; 0.28 and 0.31, none. The groups' other shell moves and every other
question moved by Jev's ask-again noise (0.013-0.015 mean absolute). Not built; not in a game. The other half of the
flip (shelling to attack, 16 of group_O's 103 picks in player-49) is not tested.
**Evidence.** `jev-0.jsonl` of `run/matches/1791038139-player-49/00` and `1791006737-player-46-pressure/00`, replayed
with `run/jev_ab.py`'s `requests_of` and `ask` (the script was scratch); the gates are those whose `pass` line has the
group's course as an attack on the party, the event "group_X hit", and `shell_<that party>` among the questions.
**Would be wrong if.** In a game with the sentence the attack / shell alternation on one party goes on at the old rate
(player-49: 34 of group_O's picks), or a shell that was right under fire (a turret outranging the screen) is no longer
taken.
**Used by.** [[H-HANDS-PARTY-MOVES-AT-THE-BAR]] (the first wording, built 2026-10-03 on the shell move of a group attacking that party with a member under fire; first game player-50).

### K-hands-a-partys-turrets-were-counted-against-a-group-out-of-their-reach
**Claim.** The turrets covering a party's place were part of that party's odds for every group with a soldier within
800 of it, on the group's course, its `change` question, its entry and every walk ("stepping back from"), wherever the
group stood; a group out of the turrets' reach was told of a fight under them and took its fall-back.
**Status.** observed in player-50 and replayed offline, 2026-10-03; fixed the same day (`picture.rs`
`party_where_we_stand`: the turrets count in the group's own fight only when one reaches a soldier of it; the entry
says where they stand; the moves that go to the party keep them). player-50 6:17: group_D, 7 Blitzes at (5012, 611),
"near party_3 (1 armflash, under 2 turrets: 2 armllt): an even fight, and it outranges us"; the Sentries stood at E3,
about 1,000 from the group (truth), the one at F1 razed at 6:14-6:16. The walk to the fall-back place rated 0.68 as
recorded, 0.52-0.53 with the odds word alone changed to a win, 0.39 with the turrets off the course, the `change`
question and the entry (four asks an arm); no forward move rose (the attack on his undefended extractor 0.31-0.34),
and the advance on the unseen shooter's place stood at 0.49-0.50. The same seconds' fire on the group (eight units hit
at 6:16) was said to be a Sentry's; no live Sentry was within 980, his Shellshocker and two Janus were 740-930 away
out of sight (the attribution not traced in the code).
**Evidence.** `jev-0.jsonl`, `truth-0.jsonl` and the record of `run/matches/1791042814-player-50/00`; replays with
`run/jev_ab.py`'s `requests_of` and `ask` (scratch scripts).
**Would be wrong if.** A group out of a nest's reach still takes its fall-back from a party standing under the nest
with the words as fixed, at the old rate.
**Used by.** [[H-HANDS-TURRETS-WHERE-THEY-REACH]]; the odds' narrow-win word ([[H-HANDS-ODDS-NARROW]]).

### K-jev-the-next-place-of-a-way-said-on-the-move-keeps-a-body-on-it
**Claim.** At an arrival the pick is between changes described one line each, and a line that names a party at its
end beat the line of the way's next stop: player-56 at 6:11, ten Blitzes at spot_53 on a way written "spot_53,
spot_57, spot_60, spot_51 ... it never sends soldiers after an armfav" were turned to spot_59, 2,200 behind them on
our side, where one Rover stood at an extractor (the pick: spot_59 at 0.65 over spot_57, whose own rating was the
higher, 0.67 against 0.57). The lines said "near ... nothing of his was there when last looked at" and "some way
off ... party_6 (1 armfav) stands at it: we outweigh it heavily": neither said which place the way names next, that
the other is not on it, which way it lies, or what the party is worth. Replayed on that pick, four asks an arm, the
share for the turn back: as recorded 0.61-0.68; the direction said 0.54-0.63; the worth said 0.46-0.51; both
0.52-0.58; "the next place of its way" on the way's stop 0.36-0.41; that and "not on its way" on the other
0.11-0.16; all four 0.04-0.08. This is also the between-stops resume left open on 2026-10-03 afternoon ("last
visited" words did not move it: K-jev-a-now-in-a-route-holds-until-the-next-packet): the words that work are on the
move, not in the actor's entry.
**Status.** demonstrated offline on one pick (2026-10-03 night, `run/matches/1791062625-player-56-experience`,
frame 11,130; the user saw the turn in the running game); built as words, unmeasured in play.
**Would be wrong if.** Other arrivals do not follow (one pick replayed), or the way read from the paragraph is wrong
where a paragraph names places that are not stops (a fall-back place counts as its last).
**Ruling (the user, 2026-10-03 night).** The way words were built by code reading the group's paragraph and were
removed: "Code parsing the group's paragraph is the recurring anti-pattern." Jev had the reading itself: the decode
of the packet in force read spot_57 for group_C at 0.96 and spot_59 at 0.22, over the fuse's cut of 0.2 by a hair
(0.06, 0.52, 0.26, 0.22, 0.07 on five near-identical packets), and nothing used it above the cut. Strengthening the
pick's own instruction did nothing (0.55-0.66 with three different sentences). The cut stays, for pruning, and
goes higher.
**Used by.** [[H-HANDS-WAY-WORDS]] (the direction and the worth only).

