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
**Used by.** `brain/briefing.rs` `track_losses`, `territory.rs`, `pianist/mod.rs`.

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
**Would be wrong if.** A raw ball traded worse than a laned one in a batch, or the milling counters showed path
close to net displacement under the flee.
**Used by.** H-HANDS-LANE.

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
the same night (H-HANDS-PARTY-KILLING), unmeasured in play.
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

