# Jev on the keyboard: the Comet Catcher series read decision by decision (2026-09-22)

Five player games on Comet Catcher Remake 1.8 (comet-1 to comet-5, `docs/experiments.md`), Opus writing the packets
and Jev (TypeSafe System One, jev-1.13.0) answering every menu. What the hands did well, what they did badly, where
the harness fought itself, what the player never knew, which options were dead weight and which were missing. Two
instruments, both new and both in `run/`:

- `run/jev_audit.py <match> [...]`: the mechanical sweep over `jev-<ai>.jsonl`, the record, the transcript and the
  truth file. Accounting per kind of actor, the option table (offered / chosen / played), no-op asks, flip-flops,
  detachments that fold back, choices the hands could not carry out, unsure answers, the three global Nouls binned
  against the next minute, the player's view against the truth, move failures.
- `run/jev_sweep.py <match> [...]`: a Jev classifier in hindsight. Every unscripted decision goes back to Jev with the
  packet in force, the actor's entry, the menu with its words, what was chosen and played, and what of ours died in
  the next 30 s; five yes/no questions. `--summarize` lists the strongest flags. Written to test the user's idea
  that a Jev sweep could replace Opus readers for finding the noteworthy moments; section 7 says how far it can.

Everything below is from the logs. Where a number is quoted, the section of the audit that prints it is named.

**Short answer.** Jev follows the packet's positive instructions and the produce list closely and cheaply (2,512
calls, 3,573 decisions, 18.7 M input tokens, about $0.78 and 200 ms a call for the whole series). It does not hear
"never": with "never splits" or "never sends detachments" in force it sent detachments on 4 of 40, 22 of 109 and
7 of 48 asks; with "never advances to shelling" in force it advanced to shelling 7 times in 25 asks. It never
scouts or builds radar on its own (scout 23 of 1,161 offers, radar 3 of 1,092), so the enemy base was never found
in three of the five games and the player planned every attack on a third of the enemy's real army. The harness's
own worst habits: a detachment is asked one second after it is born with "join the parent" on its menu and takes it
(26 of 58 detachments in comet-5 folded back within 30 s); a scout order to the guessed base does nothing when the
ball already stands on the guess (fifteen asks in a row in comet-4); the odds words price a 25 % commander as a
full one, and it attacked and died; half of all asks change nothing. Of the five hindsight questions only
"does the play contradict the instructions" is worth keeping: its top flags were right 6 of 7 times checked, it
agrees with the mechanical check, and it costs $0.30 and three minutes for the whole series.

## 1. The series in numbers (`--section summary,kinds`)

| game | result | Jev calls | decisions | scripted | packets | player turns | Jev input tokens |
|---|---|---|---|---|---|---|---|
| comet-1 easy | won 17.5 min | 555 | 821 | 0 | 19 | 33 | 4.49 M |
| comet-2 medium | lost 8.6 | 150 | 181 | 0 | 10 | 18 | 0.89 M |
| comet-3 medium | lost 17.0 | 537 | 743 | 10 | 16 | 21 | 3.59 M |
| comet-4 medium | lost 16.9 | 471 | 681 | 13 | 20 | 30 | 3.62 M |
| comet-5 medium | lost 27.7 | 799 | 1,147 | 47 | 32 | 51 | 6.05 M |

About 30 calls a minute, median 190-215 ms, no retries, no rate limits. The plant is never idle (0-1 % of
plant-seconds, 5 % in comet-3), nor are constructors (0-2 %) or the commander (1-2 %): whatever else, the hands keep
the builders busy.

## 2. What Jev did well

- **The produce list is law.** With "raider tanks and nothing else" in force the plant chose Blitz on 238 of 240
  asks (comet-1); with Stouts allowed, 30 of 31 (comet-4); nothing outside the list was ever built. `produce` is the
  one lever whose effect is exact.
- **Holds are stable.** A group with nothing in sight holds at 0.7-0.99; the switch margin (H-HANDS-SWITCH) had to
  hold a busy actor's course only 5-59 times a game, and only 1-5 % of played choices differed from the chosen one.
- **A raider at an extractor gets a detachment at once.** `send_against` at 0.97 on the first ask after the party
  appears (comet-5 3:53); the three raider episodes of comet-5 were answered within 4 s median. The instruction
  "a detachment of two or four against a single raider" was carried out the way it was written.
- **New standing work is taken up.** Turrets beside extractors were in no packet of comet-1 (turret_at played 0
  times, none built) and were played 24 times in comet-4 and 25 in comet-5 once the packets asked (22 and 30
  turrets built, the rest from lists). A radar was built once in comet-3 and once in comet-5, both from menu picks.
- **Scripted lists end the opening wobble.** comet-1's commander alternated solar and assist_lab nine times between
  2:10 and 4:48 at 0.6-0.9 each way (the two standing states both fit); under a list (comet-4, comet-5) the opening
  ran to the second and the alternation began only when the list ran out (comet-5 2:03-2:11).
- **The answers are cheap and fast enough to ask everything every few seconds**, which is what makes the rest of
  this study possible.

## 3. What Jev did badly

1. **"Never" is not heard** (`docs/knowledge/jev.md` K-jev-never-is-not-heard). The packet's prohibitions do not
   remove an option from Jev's picks; the option's own words on the menu win. While "never splits / never sends
   detachments / no detachments" was in force: comet-2 4 detachments in 40 asks, comet-3 22 in 109, comet-5 7 in 48
   (at 23:14 the player wrote "forbade detachments" and group_Z1 sent one at 23:14, 23:19, 23:24 and 23:29, at
   0.50-0.75). While "it never advances to the place called shelling" was in force (comet-1, 13:47 packet) group_L
   chose fight_to with `where` = shelling on 7 of 25 asks at 0.58-0.84. The commander's "never chases" held (0 of
   113 asks), which is the one case where the option's own words agree with the packet.
2. **No initiative on information.** `scout` was chosen 23 times in 1,161 offers, 20 of them in comet-4 under a
   packet that said "send one scout"; 0 of 312 in comet-3, 1 of 390 in comet-5. `radar_at` 3 of 1,092, and 2 of
   287 in comet-5 with "build a radar at spot_43 once we have a few extractors" in the packet. The extractor option
   outweighs both every time ("words not numbers": "once we have a few" is a count).
3. **"Nearly empty" reads as "save".** The plant's `nothing` option says "only when metal is short"; at metal
   "0 of 1200 (nearly empty); 6.0 in, 5.0 out" the plant chose nothing 36 of 79 asks in comet-3 (1:44-2:20 and
   again around 2:56) and 13 of 91 in comet-4, at 0.57-0.68 with confidence 0.15-0.36. The store is meant to be
   empty in an opening; the words say short. The idle cost was small (the ask repeats every 3 s and the unit
   being made continues), the asks were wasted.
4. **Place answers are diffuse.** With 30-44 places on `where`, the top probability was under 0.35 in 128 answers
   in comet-1 and 149 in comet-4; when it mattered (the answer was then played) 35 and 18 times, mostly a
   constructor's extractor spot at 0.21-0.32. K-jev-split-vote, unchanged, now for places.
5. **The dying commander attacked.** comet-4 16:47: health 25 %, "hit this second by armwar" eighteen times over,
   "party_1 (4 armham, 2 armrock, 3 armwar) 216 away: against this unit alone, we outweigh it"; attack 0.55,
   retreat_home 0.13; health 2 % at 16:49, game over. The odds words are the harness's (section 4d), the pick is
   Jev's: the health words were on the same entry.
6. **Low confidence is common and means little.** 6-21 % of unscripted answers sit below confidence 0.4 (235 of
   1,100 in comet-5), mostly two-option lab asks and the commander's opening; the played choice was as often right
   as wrong in those. Confidence is not a signal to gate on.

## 4. The harness fighting itself

a. **Detachments fold back** (`--section detach`). A group born of `send_against` or `split` is asked on the next
   tick with `join_group_<parent>` on its menu and takes it at 0.55-0.75: comet-5 26 of 58 detachments rejoined
   within 30 s (8:26 group_O after 1 s, 8:29 group_P after 1 s, 10:54 group_Y after 1 s), comet-3 15 of 39,
   comet-4 6 of 16, comet-2 3 of 5. The raider is left alone, the order cost a call, and the player read
   "send 4 soldiers as group_X against party_1" followed by "group_X: join group_A" in its next turn (comet-5
   10:49-10:55 shows two such pairs in six seconds).
b. **The forbidden option stays on the menu.** `send_against` under "never splits" (3a); `shelling` on `where`
   under "never advances to shelling" (comet-1 13:51-14:19); the `shelling` mark vanishes when the hits stop and
   the group holds where it is, which woke the player five times in comet-5 ("group_U1 was walking to shelling,
   which is no longer a marked place") until it "banned the vanishing shelling destination" at 21:13.
c. **The scout that does nothing.** comet-4 8:30-10:00: the ball stood on the enemy_base guess (G4), its own entry
   read "at their_base (G4)", the base line read "not found; presumed at G4", the packet said "advances onto
   their_base and stays there killing every building". Asked every 5 s it answered `scout enemy_base` fifteen times
   (0.63-0.81) and the hands did nothing each time: a scout to within 600 of the group is dropped. The real base was
   at H1/H2, 2,000 away, found at 10:22 when the ball walked on. The picture named a guess "their_base".
d. **Odds ignore health.** `odds_words` (`pianist/picture.rs`) prices our side with `force_of`, which sums
   definitions; a commander at 25 % counts as 2,700 metal of fighter. Section 3.5 is the result.
e. **"Nothing (only when metal is short)" beside an economy line that says "nearly empty"** in an opening where the
   store should be empty (3.3).
f. **Half the asks change nothing** (`--section noop`). A holding group is asked every 5 s and answers hold on
   36-67 % of group asks (318 of 642 in comet-5); a busy builder answers continue on 65-70 % of builder asks. Each is
   a chance to flip (K-jev-answers-drift) and a line in the player's "what your hands did" list: 40 of 71 lines in
   comet-5's 11:09 turn, most of them "hold".
g. **The player's list against its own packet.** comet-5's packet said a constructor never walks east of our strip;
   the player's own `queue` lists sent constructor_13472 to spot_61 and spot_38 (both east), where it spent 906 s
   east of x 2000. Jev did as it was told twice over.
h. **`needs_player` is saturated** (`--section globals`). The Noul is at 0.6 or more on 78-96 % of calls in every
   game (728 of 799 in comet-5); the wake needs 0.8 for three seconds and fired 14 times in five games. The player
   changed something on nine of the ten such turns read (comet-3, comet-5), but it changed something on every one
   of its 153 turns, so the wake's value is unmeasurable from these games. `attack_coming` rises with the metal lost in the next minute (comet-4:
   380 metal at p < 0.3, 940 at 0.3-0.6, 1,850 above 0.6) but crossed 0.6 on 3, 0, 27, 6 and 1 calls per game, so
   no threshold on it would ever fire; `base_in_danger` separates weakly (230 / 500 / 520 in comet-4).
i. **The commander boxed in by its own solar** (comet-4, 153 move failures 6:59-10:51, "helping lab build" in its
   entry throughout, "told no" in all 30 prompts; `--section view,movefail`). Known from the user's question; the
   audit confirms the player was never told.

## 5. What the player never knew (`--section view`)

1. **The enemy army.** The picture's "seen in the last three minutes and not seen to die" figure was a third of the
   truth in every game past minute five. The truth file against the player's prompt:

   | game, turn | the player read | the truth had | what it did next |
   |---|---|---|---|
   | comet-3 12:30 | 17 soldiers, 1,934 metal | 34, 3,955 | switched to Stouts "and fight massed near home" |
   | comet-4 13:13 | 14, 2,190 | 35, 4,045 | the Stout ball east at their commander |
   | comet-5 15:36 | 13, 1,950 | 45, 7,135 | committed east; eleven Stouts into nine Janus at 17:07 |
   | comet-5 17:07 | 14, 2,940 | 52, 8,355 | |
   | comet-5 23:06 | 19, 3,730 | 58, 11,240 | 24 Shellshockers without a screen |

   The clause "it may have much more" stood beside every figure. The number is what the player planned on.
2. **The enemy base.** Never found in comet-2, comet-3 and comet-5 (27.7 minutes of play against a base whose
   position was a guess); found at 11:56 in comet-1 and 10:22 in comet-4, after the ball walked into it. The enemy
   commander was never seen in three games. The truth file has BARb's plant standing at 1:00 in comet-5; the
   player's prompt at 11:09 read "its factories seen: none".
3. **The commander stuck** (4i).
4. **Which extractors died, and to what.** The prompt gives "extractors lost in the last 3 min: 2" and a "raided
   lately" grid list; the debriefs of comet-3 and comet-5 both asked for the per-spot list.
5. **Enemy types and ranges at contact.** The picture's `in_sight` has the types; the turn arrived after the
   fight. comet-5's debrief asked for it after the Janus (380) beat the Stouts (350).

## 6. Options: dead weight, and missing (`--section options`, pooled over five games)

Offered on nearly every builder ask and never chosen: `converter` (1,092 offers, 0), `wind_generator` (1,092, 0:
the wind is dead here), `construction_turret` (677, 0), `reclaim` (602, 0), `repair` (267, 0), `lab` (1,092, 0:
forbidden by every packet), `walk_to` (1,092, 5), `radar_at` (1,092, 3). Groups: `join_group_<X>` for every other
group, 70 distinct names, most never chosen (a group ask carries up to eight of them); `scout` 1,161 offers, 23
chosen; `split` 1,394, 35; `retreat` 1,806, 46. Labs before the whitelist (comet-1, comet-2): Beaver, Rascal, Janus,
Podger, Pincer, Whistler 264 offers each, 0 chosen; `nothing` 605 offers, 57 chosen, 54 of them in the opening at
"nearly empty". Dead weight costs little in tokens; it costs probability mass (K-jev-split-vote) and, for `nothing`,
`scout` at the guess and `join_group_parent`, it costs orders.

Missing, from the player's debriefs checked against what the menus offered:

- **A rule that holds.** "never X" as a removal, not a sentence (3.1). Everything the player wrote as a prohibition
  in five games was overridden at least once.
- **A detachment that stays out** until its party is dead or gone (4a).
- **A scout that walks to the unknown** when the group stands on the guess, and a base guess that is called a guess
  (4c); radar as a step the bot takes on its own after the second extractor, since Jev will not.
- **Rebuild the lost extractor first** (comet-5's packet): the `extractor` option goes to the nearest free spot, and
  a lost spot is free like any other.
- **Extractor and turret as one job** (comet-3's wish): a list does it now; the menu has no pair.
- **A plant whose output goes to two groups by unit type** (comet-4's wish): `produce` sets the mix, groups form by
  proximity (H-HANDS-GROUPS); nothing routes Blitzes home and Stouts to the ball.
- **Cancel a list**: fixed (5a034f3) after comet-5's 24:11.

## 7. The Jev classifier as a reviewer (`run/jev_sweep.py`)

The sweep put all 3,503 unscripted decisions back to Jev with the packet, the actor's entry, the menu and its
words, the choice, the parameters, what was done and what of ours died in the next 30 s, and asked five Nouls:
`contradicts` (the play goes against the instructions for this actor), `better_option` (an option on the menu fits
more clearly), `missing_option` (the instructions call for something no option expresses), `blind` (the next
30 s show a danger the entry gave no sign of), `pointless_ask` (nothing called for a change). 7.2 M input tokens,
about $0.30, 2.5 minutes at eight workers.

| question | at 0.6 or more | checked by hand | verdict |
|---|---|---|---|
| contradicts | 10-27 % per game | 6 of 7 top flags right: comet-1 13:51 advance to shelling under "never advances to shelling"; comet-2 8:13 detachment under "never splits"; comet-3 15:02 and 7:03 the same; comet-4 16:47 the 25 % commander's attack; comet-2 6:24 one soldier split off a group of two (the packet allowed detachments of four; a fair flag). comet-4 8:35 "hold" on the empty base guess: arguable | keep; it agrees with the mechanical "never" check and finds what that check's regex does not name |
| missing_option | 6-36 % | 2 of 3 wrong: a plant building the one allowed unit flagged at 0.77 (comet-3 12:33) and 0.69 (comet-1 11:54) | drop as written; it flags labs by preference |
| better_option | 57-82 % | saturated | drop |
| blind | 35-63 % | 0.71-0.84 when something of ours died in the next 30 s, 0.07-0.09 when nothing did: a death detector, not a gap detector | drop, or ask it only over deaths with the entry's own warnings listed |
| pointless_ask | 4-25 % | no separation: 0.33-0.47 on a holding group told to hold, 0.27-0.41 on the rest | drop; the mechanical no-op count is exact |

What this says about Jev as a reviewer: a single-hop check of a play against a text it can see works at scale for
cents ("does this go against these words"); anything that needs a model of the game beyond the text (what was
missing, whether the ask was needed, whether the danger was foreseeable) does not, which is the same boundary the
in-game answers show (words not numbers, no counting, no initiative). Against the user's alternative of Opus
subagents: the sweep and the audit together took under five minutes and produced the flag lists that sections 3-4
are built from; the reading of the flagged moments (the packet, the entry, the menu) took the reviewer, not the
model. The working shape is: audit and sweep to triage, a reader over the top twenty moments. `contradicts` should
run after every player game with its top ten in the debrief.

## 8. What to change (for discussion; nothing here is built)

Ranked by the damage measured above.

1. **Prohibitions as removals.** A `forbid` tool, or the bot reading the packet's "never" clauses per actor, that
   takes the named option or place off that actor's menu. Jev will not do it from the words (3.1). This is the one
   change that touches every game.
2. **Detachments that stay out.** Do not ask a detachment until its party is dead, out of sight or 30 s old; no
   `join_group_parent` on its menu before then (4a).
3. **Odds with health.** `force_of` scaled by health for the odds words, and the retreat option's words naming the
   commander's health when it is low (4d, the comet-4 death).
4. **Scouting as harness work.** Call the guess a guess in the picture and the places; when a group stands on the
   guess and the base is unfound, `scout` goes to the nearest enemy-side spot not seen; radar after the second
   extractor as a bot step, not a menu option (4c, 5.2).
5. **Stall words and a wake for builders** (4i; already on the design's open list).
6. **Fewer no-op asks.** A holding group with nothing in sight, under no fire and no new packet is asked every 15 s,
   not 5 (4f).
7. **The `nothing` option** only when the store is empty and usage exceeds income for a while, not on "nearly
   empty" in the opening (3.3).
8. **The player's picture of the enemy**: the truth is not available, but a scouting line ("the base has not been
   seen; nothing of ours has looked east of D since 6:00") and the enemy's factory type once any factory is seen
   would replace the number the player plans on with the fact it lacks (5.1, 5.2).
