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
**Status.** measured offline (2026-09-25, `run/decompress.py`, 293 packets of four games); built and run live in standing-1 (ledger row: the bill $0.17 against $0.59, 49% of plays by the executor, raider_ignored 3 moments, and the faults the player's notes name, three fixed after).
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
**Used by.** H-HANDS-ONE-PASS (the pass holds open slots for the pick, 2026-09-27; a hunt ends the engagement it
overrides); the bar-review skill, item H7.
