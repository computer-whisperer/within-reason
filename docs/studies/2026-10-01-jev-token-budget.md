# A Jev token budget: $1 a game (2026-10-01)

The user, 2026-10-01: "We are already on the expensive side for jev games -- we really should be aiming for $1 per game
or less. I suspect we don't need anywhere near the number of tokens we are putting into it each turn"; "Consider also
ways to reduce what we ask each time, and how often we ask it. We possibly don't need the complete world combination
each time if we can find clever ways to split it into smaller parallel combinations that don't interact too badly. We
should also figure out what it will take for a game as large as a 40 minute 8v8 to run on $1 per jev hands instance."

Nothing in the bot is changed. Everything here is counted on recorded calls (`run/jev_budget.py`) or asked again
offline (`run/jev_trim_ab.py`, `run/jev_split_ab.py`, `run/jev_read_ab.py`; about $0.50 of Jev). Games: player-33-hard
(24.6 min, won, $1.57) and player-32-hard (39.5 min, lost, $3.11). $1 is 23.8M input tokens; $1 for a 40-minute game
is 0.60M tokens a game minute, or 9,900 a game second.

## The short answer

- **A 1v1 game as we play it today reaches $1 without changing what Jev decides.** Seven changes to what is sent and
  when take player-33 from $1.57 to $0.73 and player-32 from $3.09 to $1.43. Player-32 is a 40-minute game with a
  late game of 3 to 4M tokens a minute; one more step from §5 (one request a second) brings it to about $1.
- **The rebuilt menu costs more and reaches about the same place**: modelled at $0.94 for player-33 and $1.96 for
  player-32 under the same seven changes. Under $1 for 40 minutes needs the three further steps of §5.
- **Half of what we send is the picture, sent again.** 44% of a game's tokens are the state, repeated in all three
  calls of every second. Another 16% are walks that are played 65 times a game.
- **Splitting the pick works.** Asked per component in one request, it costs 58% of the two calls, saves a round
  trip, and lands more changes a second (1.58 against 1.23).
- **An 8v8 seat at $1 cannot be this architecture scaled up.** The shared picture alone would pass the budget at one
  request a second. It needs requests that carry only the local picture, a first cheap question that says which
  pairs are worth asking about, and a token rate set as a dial. $1 then buys about 5 asks a second whatever the
  size of the game; a 1v1 today averages 4.6 to 6.6 over the game and 7 to 11 late.

## 1. Where the tokens go

A second with something to decide is three calls: the gate (nouls), the pick's first stage, the pick's second stage.
Each carries the whole picture. Measured against the service: the state costs 0.34 tokens a character, gate questions
0.28, an empty request 280.

| player-33, 37.4M tokens | Tokens | Share |
|---|---|---|
| The picture in the gates (1,132) | 6.6M | 18% |
| The picture in the picks (1,763 calls) | 9.9M | 26% |
| Gate questions: a group's walks and advances | 6.1M | 16% |
| Gate questions: builders' options | 4.7M | 13% |
| Gate questions: other group moves, attacks and hunts, openers, forbidden | 5.6M | 15% |
| The worlds' lines | 3.8M | 10% |
| Request overhead | 0.8M | 2% |

The picture is 16,000 characters a call: the actors 5,500, the rules text 4,500, his side 2,300, the packet 1,800. Of
142,560 gate questions 74% were answered under 0.2 and 5.5% at 0.5 or over; 1,015 moves were played. A third of the
picks (307 of 955) played nothing. Of the worlds' lines, 7.2M characters are joint worlds and 1.5M the single changes
they are built from. The cost of asking one actor or one party about anything, everything counted, is 4,200 to 6,000
tokens and does not move through the game; what grows is how many are asked (2 a gate at minute 3, 8 to 12 late).

## 2. Seven changes that leave Jev's decisions alone

Applied one after another to the recorded calls. Millions of tokens:

| | player-33 | $ | a minute | player-32 | $ | a minute |
|---|---|---|---|---|---|---|
| as played | 37.5 | 1.57 | 1.53 | 73.6 | 3.09 | 1.86 |
| 1. the forbidden question once per packet, not per hunt | 37.0 | 1.55 | | 72.2 | 3.03 | |
| 2. walks only to the places Jev read as the group's | 34.1 | 1.43 | | 66.8 | 2.80 | |
| 3. an actor's moves asked only while its opener says yes | 28.2 | 1.18 | | 56.3 | 2.37 | |
| 4. a question whose words did not change is not sent again | 26.4 | 1.11 | | 52.6 | 2.21 | |
| 5. the pick asked per component, in one request | 20.5 | 0.86 | | 40.8 | 1.71 | |
| 6. lines written in word buckets, so more of them repeat | 18.9 | 0.79 | | 37.2 | 1.56 | |
| 7. a gate with no event waits up to 2 s | 17.4 | 0.73 | 0.71 | 34.0 | 1.43 | 0.86 |

What each is, and what it costs in behaviour:

1. **Forbidden once per packet.** Today every hunt on offer carries a second question that repeats its whole line.
   Whether the packet forbids a group to send detachments is a fact about the packet: asked once per packet per
   group and said on the line. Not replayed in this form.
2. **The reading** (the packet decoded once by Jev, as the user allowed 2026-10-01). On a second game it reads as on
   the first: player-32, 117 packets, a group keeps a median 2 of 23 named places; places in a paragraph headed by
   its name 956 of 1,316, places elsewhere 118 of 5,745. Its own cost is not in the table: as run it was 1.3M and
   2.2M tokens a game, because the test sent every actor's entry with it; sent lean and only for sentences that
   changed, about 0.5M.
3. **Openers first.** Nearly half the gate's characters are options of an actor whose own `change` came back under
   0.5 in the same call. The moves are asked from the second after the opener says yes. Price: the first reaction
   to a new situation is one second later. (Asking them in a second call the same second was measured on 2026-09-30:
   no saving, because of the extra picture.)
4. **Unchanged questions.** With the clocks taken out, 38 to 43% of a group's walk questions repeat their last words
   within 20 s, and the answer then moves a median 0.01 (crosses 0.5 in 0.1%). Re-asked as recorded, a whole gate's
   answers cross 0.5 in 0.7%: that is the noise floor.
5. **The split pick** (§4).
6. **Word buckets.** Another 30 to 40% of builder, group and party questions differ from their last ask only in
   numbers ("1970 away", "20 s"). Jev reads words, not numbers (K-jev-words-not-numbers); answers to such pairs
   crossed 0.5 in 0.7 to 2.8%. This row is the ceiling of what bucketed words would save.
7. **A two-second tick.** 855 of 1,132 gates had an event (a hit in 734), so this saves little in a fight and most
   in quiet minutes.

## 3. Trims that do move the answers

Asked again on 50 gates and 120 picks of player-33 (`docs/studies/data/trim-2026-10-01.jsonl`). "Strong" is an
actor's best-rated move at 0.5 or over as recorded (72 of them).

| Arm | Tokens | Strong move still the best | ... and still 0.5 or over | Stage one, same choice | Stage two, same choice |
|---|---|---|---|---|---|
| the call as recorded (noise) | 100% | 69 of 72 | 66 | 53 of 60 | 58 of 60 |
| the rules text out | 94% of a gate | 62 | 53 | 49 | 56 |
| the rules cut to what the words mean | 95% | 63 | 48 | - | - |
| questions cut to the actor and the move | 75% | 59 | 53 | - | - |
| the pick's picture cut to the actors its worlds name | 77% and 56% | - | - | 36 | 54 |

The rules text is worth 12% of the bill (1,019 tokens in every call) and it does steer: without it one strong move
in five falls under the bar. Whether that is worse play or only the hands' voice going quiet is the design
question of the rebuild note (its judgments move to the brief), not a free saving. The short question form saves a
quarter of a gate and moves about as much. A thinner picture for the pick changes which change is preferred in 4 of
10 picks; whether to change at all holds (54 of 60).

## 4. The pick in parts

Today's pick offers every combination of the opened actors' changes: a median 7 worlds, 54 at the 90th percentile,
98 at most, though a median 2 actors have a change on offer. `run/jev_split_ab.py` groups the slots into components
(two are of one component when a change of one names the other's actor, group, builder or party) and asks each
component its own pick: the best of its changes, and that change against "nothing changes", for every candidate at
once, all in one request. 120 recorded picks of 3 worlds or more:

| | The game's two calls | Split, one request |
|---|---|---|
| Tokens a pick | 16,700 | 9,700 |
| Picks of 20 worlds or more | 25,100 | 14,200 |
| Characters of world lines | 11,100 | 2,100 |
| Actors changed a pick | 1.23 | 1.58 |

A median 2 components a pick, a median 1 candidate each. Where a component had a choice (133), the split's best
change was the one the joint pick's probabilities weighted most in 96. Both changed a component in 119 cases, to
the same change in 92; the split alone changed 71, the game alone 29. What the split cannot see: two components
spending the same metal; two groups leaving one area. The lines still say, per world, which parties are left to
nobody.

## 5. The rebuilt menu, and the last third

`run/jev_budget.py --rebuilt` replaces the groups' and parties' questions with a model of the rebuilt menu (every
group asked: both ways to each place read as its own, a join to every other group, hold, gather, scout; an opener for
every party in the picture; for each party that needs answering, every group's attack and two detachment sizes),
with the recorded question lengths. Modelled, not measured.

| Millions of tokens | player-33 | $ | player-32 | $ | a minute |
|---|---|---|---|---|---|
| the rebuilt menu, nothing saved | 66.9 | 2.81 | 125.4 | 5.27 | 3.17 |
| with the seven changes | 22.3 | 0.94 | 46.7 | 1.96 | 1.18 |

What is left in player-32's 46.7M: the picture 21.1M (gates 11.5, picks 9.6), attacks at every party by every group
8.3M, joins and the other group moves 5.3M, openers 3.0M, lines 4.0M, the rest 4.9M. Three more steps, none
measured for its effect on play:

- **One request a second.** The pick for last second's gate goes out with this second's gate, so the picture is
  sent once. Saves the picks' picture: 9.6M. Price: a move lands a second later.
- **"Who answers this party" as one question.** A Choice over the groups and sizes for a party that needs
  answering, in place of a noul for every group and size. About a quarter of the characters: 6M.
- **Joins from the reading.** A join offered to the groups the packet tells this one to join, and to its parent,
  not to every group: 3M.

Together about 28M, $1.18 for the 40-minute game; with the rules text cut, 25M, $1.07. The same first step on the
game as played (§2's 34.0M) gives 24M, $1.02.

## 6. An 8v8 seat for 40 minutes at $1

No recorded game is near this size (the 3v1 games ended by minute 20 with 12 actors a seat), so this is arithmetic on
stated assumptions: one seat late in the game holds about 70 actors (12 groups, 45 builders, 10 factories and labs)
and sees about 40 enemy parties; a late 1v1 has 25 and 9.

- **The shared picture breaks first.** The state is about 9,000 + 300 a actor + 400 a party characters (fitted on
  the two games' five-minute bins). At 70 and 40 that is 46,000 characters, 15,600 tokens a request: more than the
  whole budget at one request a second.
- **Pair questions break second.** Ten parties that need answering, twelve groups, three sizes: 55,000 tokens in one
  gate (an attack and two sizes each). Twelve groups each offered eleven joins: 21,000.
- **What does not grow** is the cost of asking one thing once its request is local: about 280 for the request, 1,000
  to 1,300 for its own picture (its entry, the parties in it, its sentences of the packet), 700 for its questions
  after the changes of §2, and its share of a split pick: about 2,000 to 2,500 tokens. $1 for 40 minutes buys 4 to 5
  such asks a second, whatever the size of the game.

So it takes four things, the first three of them architecture:

1. **Local requests.** One request per actor or per component, carrying only what its questions are about, sent side
   by side (the service takes 40 a second). §3's thin-picture arm says this changes answers: it needs its own
   replay before it is trusted.
2. **Nothing asked as actors times things.** Static pairs come from the reading once per packet (places, joins,
   whom to follow, what is forbidden). Moving pairs (which group answers which party) come from one cheap question
   per party first.
3. **A token rate as the dial.** Asks are served in the order of their news until the second's budget is spent; a
   busy second stretches the wait of the quiet actors, not the bill. $1, $2 and $4 an instance are 5, 10 and 20 asks
   a second.
4. **Fewer things for Jev to attend.** The 55 builders and factories run on the player's lists and sequences and
   are asked only in danger; the packet keeps the army in ten or twelve groups.

At 5 asks a second a seat's 12 groups and 40 parties get about a third of the attention a late 1v1 gives its 5
groups and 9 parties. Whether that plays well cannot be known from here. It can be rehearsed in 1v1 once the dial
exists: the same game at about 3 asks a second gives each actor the attention an 8v8 seat would have.

## 7. What is not known

- The seven changes are counted, not played. Changes 3 and 7 delay some reactions by a second or two.
- The rebuilt menu's numbers are a model with recorded question lengths; the reading's lean cost is an estimate.
- The split pick was compared with the joint pick's own probabilities, not with a better answer.
- Two games, one map, one opponent. Player-32's late game is twice player-33's rate.
- The 8v8 unit counts are assumed.

## 8. After the rebuild: what was measured on 2026-10-02

The rebuilt hands with `news`, `same`, `fuse`, `openers` and `tick` played player-36 (17 minutes, won) for 27.8M
tokens, $1.17. `run/jev_bill.py` on its log:

| | Tokens | Share |
|---|---|---|
| the gate | 16.7M | 60% (the picture 4.5M, the questions 12M) |
| the pick's two calls | 9.1M | 33% (the picture 6.3M, the worlds' lines 2.9M) |
| the decode | 2.1M | 7% |

By the picture's sections over all calls: `actors` 4.2M, the rules text 2.8M, `enemy` 1.5M, the packet 1.2M. The
gate's question text: a move's own words 46%, the standing course repeated in every question of the actor 21%,
the preamble 21%, the fight's facts repeated on each move 7%. By whom it is asked of: groups with a course 64%,
builders with a course 16%, idle groups 14%.

**Built on it.** `split` (§4; the design note §17). `openers` asks the opener first: 16% of the gate's question
text was the own moves of actors whose `change` came back under 0.5 in the same request (an opener never asked, or
asked over 20 s ago, had brought the whole menu with it), another 3% moves at a party whose `answer` said no.

**Tried and not built** (each replayed on recorded requests):

- *The standing course cut from each question* (`run/jev_words_ab.py`, 40 gates of player-36, 4,063 own-move
  answers; `docs/studies/data/words-ab-2026-10-02.jsonl`). "rather than what it does now" in place of the course's
  words saves 11% of a gate and the actors' strong moves stop being strong: of 56 actors whose best move was at 0.5
  or over as recorded, 48 kept one there when the gate was asked again unchanged, 29 with the course cut and 33
  with the course's words moved into the actor's entry. The course in the question is what the move is weighed
  against; it stays.
- *The rules text out of the split pick* (`run/jev_pick_rules_ab.py`, 120 picks of player-38): 14% of a pick's
  tokens, and the changes taken fall from 285 to 248 of 530 (287 asked again unchanged).
- *The decode for a detachment the packet does not name* (`run/jev_detach_decode.py`, player-36: 81 detachments,
  4,884 readings): it reads "join its parent" under 0.2 in 63 of 88 readings where the gate rated that join 0.5 or
  over in 62 of 149 asks, and a join to another group under 0.2 where the gate said yes in 329 of 1,357. Opus does
  not write about detachments; the decode cannot fuse for them. Fusing a detachment's walks by its parent's
  readings does not separate either (2.3% of the gate's answers at 0.5 or over where the parent read under 0.2,
  5.6% where it read over).
- *More of the `same` layer.* Of the questions asked again within 20 s, the move's own words differ in 82% (the
  way's seconds, the fight's facts, what a party is killing), so the layer skips 1% whatever is done to the
  course's words; where only the course's words differ the answer crosses 0.5 in 0.8 to 1.7% of pairs.

**Where the unnamed detachments cost.** 52% of the walk questions to groups in player-36 went to groups the packet
does not name, nearly all detachments under 30 s old: asked their whole unfused menu at birth (the hunt in force)
and at every gate once the hunt ends and they stand idle. Asking the opener first takes the first; the second is
the cost of the churn (95 groups in 17 minutes).

**Left, in the order of what they would save** (player-36's tokens): `one` (the pick rides with the next gate:
about 3M, and a move lands a second later); an opener for an idle actor ("should it do anything now?" before its
menu: up to 1.9M, a rule the design does not have); the decode kept across packets for an actor whose paragraphs
did not change (about 1.5M).


## 9. player-38: the split in a game, and the gate by what opens an actor

**The split pick, measured** (player-38-split, 30:30, lost; the audit asked the joint pick as well on 158 picks).
The split's one request cost 58% of the joint pick's two calls on the same picks (10,206 tokens against 17,451),
the study's figure. It takes more: 1.12 moves a pick against 0.89, the extra ones groups walking, joining and
sending. Over the first 17 minutes the picks cost 5.63M against player-36's 9.10M and the game 23.1M against
27.8M.

**The bill by game minute.** From minute 6 on it is flat: about 53 gates a minute of 120 questions over six actors,
$0.09 a game minute. A 17-minute game is $1, a 30-minute one $2.10; player-37 (no split) was at $4.73 by minute 38.
The price of a game is its length more than anything left to trim.

**The gate's question text (85.6M characters) by what opened the actor**, own moves only (`news` reasons read from
the `pass` rows; the party-aimed moves and the parties' answers are the rest, about 16M):

| what opened it | groups | builders |
|---|---|---|
| it stands idle | 10.1M (629 asks) | 3.2M (153) |
| it was hit | 10.2M (572) | 1.2M (169) |
| never asked before (newborn) | 4.8M (104) | 1.9M (20) |
| its course was changed by the pick a second before | 5.0M (272) | 1.3M (144) |
| it reached a place or met a party | 3.4M (293) | |
| its `change` said yes the second before | 3.1M (151) | 1.7M (113) |
| a party came into or left its entry; the store crossed empty | 2.7M (196) | 3.1M (439) |
| a new packet or list | 2.1M (214) | 2.2M (392) |

No one cause is over an eighth of the gate. Two-thirds of the text (57.6M) is a question that was asked under 5 s
before, in the same words in 12% of those (answers 0.03 apart, 2.7% crossing 0.5) and with another course or
another move's words in the rest.

**The walks.** `go_<place>` and `fight_to_<place>` are 40% of the gate's text (34.6M, 60,000 questions): 2% come
back at 0.5 or over and 277 were played. 14.3M is bodies of eight or more that the packet names, asked every
place it names for them in both manners each time they are opened; 13.9M is detachments of one to seven that the
packet does not name (504 answers at 0.5 or over of 29,729, 76 walks played).

**Tried and not built:**

- *One question a place in place of the two manners* (`run/jev_walk_ab.py`, 40 gates, 907 walks, twice): saves
  10 to 12% of those gates. Of 85 walks at 0.5 or over as recorded, 71 and 75 stayed there asked again unchanged,
  59 and 62 with both manners said in the one question, 64 with neither said; of 51 actors with such a walk, 47
  kept one asked again and 39 merged.
- *The walks as one Choice an actor* among its places and staying (`run/jev_walk_choice_ab.py`, a trial of 12
  gates, 19 actors): the options still carry each walk's words, so the request is 84 to 88% of the recorded one;
  of 6 actors with a walk at 0.5 or over, the Choice walked 2 (6 asked again as recorded).
- *An idle actor not asked again for 5 s after a menu with nothing at 0.5* (counted, not replayed): 5.1M of the
  text, and for groups something was at 0.5 or over in 185 of the 735 asks it would skip.

With §8's three, every trim of the questions' words tried so far has moved the answers; what is left is asking
fewer actors or fewer moves.

**Found, not changed.** The design's news for an actor (§4, "When things are asked") is its course *ended*; the
code's `news` also opens an actor whose course *changed*, and so asks an actor again the second after the pick
moved it (the fifth row above: 6.2M, 7% of the gate's text; its `change` comes back yes in 558 of 695 asks within
2 s of its own pick). Of the 702 group picks that followed an earlier pick on the same group, 340 came within 2 s.

**Left, each the user's to rule:** `one`; an opener for an idle actor (the first row: up to 13M of the gate's
85.6M, and an idle actor's move a second later); the pick's own change taken out of the news (7%, and a pick
stands until something happens to the actor or 20 s pass); no walks to named places for a detachment the packet
does not name (16%, 76 walks a game).
