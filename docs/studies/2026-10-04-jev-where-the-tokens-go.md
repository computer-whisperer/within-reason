# Where a game's Jev tokens go (2026-10-04)

For the user's review, to find what else to cut. One game is taken apart in full, going-opus-1 (Opus 5.5 medium
as the player under the Opus commander, against BARb hard_aggressive on Comet Catcher, won 20:19, 41.4M input
tokens, $1.74); the other games are beside it where they differ. Jev's price is $0.042 a million input tokens and
the output is free, so the bill is what we send. The target is $1 for a 40-minute game, 23.8M tokens.

Tools: `run/jev_bill.py`, `run/jev_load.py`, `run/jev_ask_causes.py`, `run/jev_place_bar.py`; the figures of
§3 to §5 are from scratch scripts over the same log (`jev-0.jsonl`).

## 1. The bills

| Game | Player | Length | Tokens | Cost | Per 40 minutes |
|---|---|---|---|---|---|
| packet-example-1 | Sonnet | 18.9 min | 32.7M | $1.37 | $2.91 |
| going-1 | Sonnet | 30.6 min | 45.6M | $1.92 | $2.51 |
| going-opus-1 | Opus | 20.3 min | 41.4M | $1.74 | $3.42 |
| going-2 | Sonnet | 39.0 min | 58.8M | $2.47 | $2.53 |
| apex011-carrot-sym50-opus-2 | Opus | 18.5 min | 38.4M | $1.61 | $3.49 |
| 1v2-easy-1 | Sonnet | 27.5 min | 60.3M | $2.53 | $3.68 |

The cost is in the back of the game. going-opus-1 by game minute, in millions of tokens: minutes 0 to 7 hold 2.8
(7%), minutes 8 to 15 hold 17.4, minutes 16 to 19 hold 19.9 (48%, about 5 a minute, which is $8 for 40 minutes at
that rate). The gate goes 50 to 60 times a game minute from minute 8 on; what grows is what a gate carries (15,000
tokens at minute 8, 49,000 at minute 17).

## 2. What a second sends

Three kinds of request:

| Request | When | Share of the game | Calls | Tokens a request |
|---|---|---|---|---|
| The gate: every open actor's questions, the places, the parties | a second in which something is news | 74% | 823 (1,119 requests: a large gate is split in batches, each with the picture) | 27,300 |
| The pick: which of the flagged changes to play | after a gate that flagged something | 13% | 552 | 9,700 |
| The decode: what the packet gives each actor | a new packet, and whatever is new under it | 13% | 345 | 2,000 at the median; 38 large ones hold 4.5M of 5.3M |

Every gate and pick request carries the picture: about 7,000 tokens.

| Picture section | Tokens in the game | Share of the game |
|---|---|---|
| `actors` (one entry for each of ours) | 6.1M | 15% |
| `rules` (the fixed text) | 2.5M | 6% |
| `instructions` (the player's packet) | 1.8M | 4% |
| `enemy` | 0.85M | 2% |
| `economy`, `recent`, `ours`, `player`, `allies` | 0.97M | 2% |
| **The picture, all requests** | **12.2M** | **29%** |

A pick request is about 78% picture: 7,700 tokens of picture for 2,200 of questions.

## 3. The questions, by kind

Yes is the share answered at 0.5 or over. Played is how many times the pick put a move of that kind in force.

| Question | Asked | Tokens | Share | Characters each | Yes | Played |
|---|---|---|---|---|---|---|
| A place: "does any of ours have a reason to go to, fight at or build at spot_N now" | 26,098 | 4.3M | 10.4% | 530 | 6% | (opens moves; the bar is 0.3) |
| Builder: build a type at a spot | 28,526 | 3.4M | 8.2% | 372 | 1% | 17 |
| Group: advance to a place, fighting | 15,575 | 3.3M | 7.9% | 673 | 1% | 33 |
| Group: walk to a place | 12,803 | 2.7M | 6.6% | 688 | 1% | 45 |
| Group: send n soldiers to hunt a party | 12,479 | 2.2M | 5.2% | 548 | 4% | 31 |
| Group: shell a party or his buildings at a place | 5,590 | 1.5M | 3.6% | 843 | 1% | 27 |
| The pick's own questions | 2,106 | 1.4M | 3.3% | 1,730 | | |
| Builder: walk to a place | 13,745 | 1.1M | 2.7% | 260 | 3% | 98 |
| Group: is this detachment forbidden by the instructions | 5,465 | 0.9M | 2.1% | 511 | 99% | |
| Group: attack a party with the whole group | 3,001 | 0.7M | 1.6% | 715 | 3% | 49 |
| Group: follow another group | 2,825 | 0.5M | 1.2% | 558 | 2% | 17 |
| A party: does it need answering this second | 2,665 | 0.4M | 1.0% | 483 | 26% | |
| Builder: help a plant or a builder | 4,496 | 0.4M | 0.9% | 258 | 2% | 38 |
| Group: join another group | 1,182 | 0.25M | 0.6% | 688 | 12% | 54 |
| Builder: should it do something else (`change`) | 1,704 | 0.24M | 0.6% | 449 | 45% | |
| Group: should it do something else (`change`) | 1,240 | 0.23M | 0.5% | 577 | 64% | |
| Group: send its fastest to rove | 888 | 0.19M | 0.5% | 696 | 0% | 0 |
| Group: hold | 960 | 0.19M | 0.5% | 631 | 4% | 10 |
| Group: gather | 873 | 0.17M | 0.4% | 611 | 10% | 46 |
| Builder: repair an extractor | 874 | 0.06M | 0.2% | 230 | 0% | 0 |

What stands out:

- **99 of 100 move questions are answered no.** 28,526 build questions for 17 builds played; 28,378 walk and
  advance questions of groups for 78 played.
- **A group's move question is mostly text that every one of its questions repeats.** Each opens "is this what
  group_B should do now, rather than [what it does now, 200 to 400 characters]" and ends with the same sentence
  about its recent course changes. Of the text of a group's own-move questions in one gate, 60% is that shared
  opening and ending sent once a question (builders: 33%; moves aimed at a party: 4%, their text is the party's).
  That is about 5M tokens of a group's 8.5M, 12% of the game. The earlier studies cut the question's opening and
  found the answers move (`docs/studies/2026-10-02-jev-question-cuts.md`); what was not tried is saying it once,
  in the actor's entry or as one Choice over the actor's moves.
- **The forbidden question is nearly a constant**: 99% yes, 0.9M tokens.
- **Two moves were never played**: the rove (888 asked) and the extractor repair (874).

One group question in full (17:05, 690 characters, one of 120 asked of group_D1 in that gate):

> Is this the move to make against party_114 now, rather than what stands (nobody moves for it)? The move:
> group_D1 attacks party_114 (3 unidentified, under 3 turrets: 1 armguard, 2 armllt, at spot_35, heading
> south-east) with the whole group (far, about 80 s of walking): it outweighs us, and it outranges us (turrets
> covering it reach 1220 to our 710): the approach is paid under its fire, so the group goes in as one body or not
> at all; its speed is unknown (radar contacts only); group_D1 has changed course 13 times in the last 28 s and
> stands 229 from where it stood at the first of them: a body that keeps turning walks and does not arrive or
> fight.

## 4. Who is asked, and why

| Actor kind | Asks | Question tokens an ask | Share of the game |
|---|---|---|---|
| Groups | 1,469 (2,323 counting the gates where only its party moves went) | 5,400 | 30% |
| Builders | 2,203 | 2,500 | 13% |
| Plants | 692 | 200 | 0.3% |

Two groups carry 21% of the game between them: group_B (the army, 79 soldiers at the end) 5.5M, group_D1 3.2M.
group_B was asked 765 times and changed course 140 times. A group ask is 30 questions at the median and up to 120:
about two for each open place and two for each party in sight.

Why a group was asked (the news that named it):

| News | Asks | Questions an ask | Share of the game |
|---|---|---|---|
| A place of its opened at the gate before (the second step of the `places` layer) | 371 | 57 | 10% |
| None of its own: its course or the party beside it changed, or 20 s had passed | 419 | 36 | 6% |
| It was hit | 301 | 35 | 6% |
| It reached a place | 309 | 32 | 5% |
| It met or sighted a party | 69 | 35 | 2% |

Why a gate went: an opening 443, the picture changed or a re-ask was due 234, a place reached 125, a party met 12,
an event after 5 s 9.

- **Openings**: half the gates. Until today the gate after an opening sent the actor's whole menu again; the
  `again` layer (built 2026-10-04) sends what is new alone. The repeats were 6.0% of this game. Four re-asks in
  five change no course.
- **Hits**: of 301 asks on a hit alone, 249 came with no member lost since the group's last ask. Counting a hit
  as news only at 5% of the body lost drops 6.1% of this game and 3 to 4% of the others; the dropped asks played
  51 of 317 course changes here ("attack" the most), which the replay cannot place elsewhere. Not built.
- **Reached**: group_B's entry at 16:34 lists spot_53, spot_55, spot_53, spot_55, spot_53 reached within 16 s,
  each an immediate gate.
- **No news of its own**: not split yet between a real course change and the nearest party flickering in a fight.

## 5. The decode

345 requests, 5.3M tokens (13%). A new packet is read whole: for each builder each spot and each building type,
for each group each place, each join and each follow. In this game the readings were 36,309 of a place for an
actor, 9,012 of a building type for a builder, 5,874 follows, 661 joins, 380 detachments, 154 places to step
back to. The 38 packets after minute 10 cost 120,000 to 190,000 tokens each (1,800 to 2,000 readings), 4.5M in
all. apex011-carrot-sym50-opus-2 has one of 256,000. The Sonnet games have about twice the packets at smaller
sizes (going-2: 73 large ones, 4.6M).

The count is builders times spots: it is the term that grows fastest with the map and the seat count.

## 6. What has been measured as a cut, and what it buys

| Cut | Saves, of the game | Cost | State |
|---|---|---|---|
| `again`: the gate after an opening sends what is new alone | 3 to 6% in repeated group questions, more in repeated place questions | 2 to 4% of the standing answers would have crossed 0.5 | built, in a game now (again-opus-1) |
| A hit is news at a share of the body lost | 4 to 9% | drops asks that played 8 to 16% of group course changes | measured, not built |
| The best 12 places a gate (groups) | 2.5 to 8% | shuts 1 to 6% of the place moves groups played | measured, not built |
| The best 8 places a gate (groups) | 5 to 11% | shuts 5 to 11% | measured, not built |
| A place bar of 0.4 | 8 to 11% (groups) | shuts 10 to 23% | measured; worse than the rank |

## 7. Not measured yet, by size

1. **The shared opening of an actor's questions, said once** (about 12% for groups, 4% for builders): §3.
2. **The decode by difference**: read only the actors and sentences a new packet changed, or read builders by
   kind and not one by one (up to 10%).
3. **The picture's `actors` section** (15%): every actor's entry rides on every request, whichever actors are
   asked. Sending the asked actors' entries alone was rejected on 2026-10-02 (it moved answers); sending the
   others as one line each was not tried.
4. **The pick's picture** (10%): the pick is asked in a request of its own with the whole picture again.
5. **A group's menu by places times two and parties times two**: the walk and the advance to one place are two
   questions with the same facts (15,575 and 12,803 asked); so are the hunt and its forbidden mark.
6. **Build questions** (8%): 28,526 asked, 1% yes, 17 played by the pick. How many builds the player's lists
   placed beside those 17 is not counted here.
7. **Moves never played**: the rove and the extractor repair (0.6% together).

## 8. For 8 against 8

A seat's bill is groups times (places plus parties), plus builders times spots in the decode, plus the picture
times the requests. A team seat sees more parties and more places than a duel seat and there are eight of them.
The cuts that remove a product (2 and 5 above, and the place rank) matter more there than the ones that trim a
constant.

## 9. The `again` layer in a game (again-opus-1)

The same launch as going-opus-1 with `again` added to the layers. Won at 14:53 (going-opus-1: 20:19), 11.6M tokens,
$0.49. The two games are different games, so the bills compare only loosely: through 14:53 going-opus-1 had spent
18.3M ($0.77).

- `again` let 2,959 answers stand, 7.7% of the 38,442 questions the gate would otherwise have sent.
- A group's ask after an opening carried 26 questions (57 in going-opus-1); openings still called 224 of 531 gates.
- Its audit: 48 asked anyway, 3 on the other side of 0.5 from the standing answer. All three are place questions
  (0.56 to 0.49, 0.53 to 0.48, 0.59 to 0.28), where 0.5 is not the bar that matters (a place opens at 0.3 and closes
  under 0.2); none of the 29 audited move, change and forbidden answers crossed. The largest move on those was 0.11.
- The `places` layer's own audit beside it: 14 faults of 238 (5.9%).
- One pass row's `same` count wrapped below zero (an id that came twice in a gate's questions was counted twice as
  standing); fixed after the game.

## 10. The products in the costliest games (`run/jev_products.py`)

The three costliest games under the current layers: human-18 (three seats, $7.40, 32 min), player-54-experience
($3.58, 34 min), 1v2-easy-1 ($2.53, 27.5 min).

| Product | human-18 | player-54 | 1v2-easy-1 | Largest in one gate |
|---|---|---|---|---|
| The picture in the gate: every actor's entry in every request, a large gate split in up to 8 requests | 24.9% | 17.5% | 19.9% | 48 actors x 7 requests |
| Groups x parties (a detachment of 1, 2, 4, 8 with its forbidden mark, the attack, the shelling) | 20.1% | 7.5% | 15.2% | 35 x 16, 597 questions; 1.9 to 3.4 questions a pair |
| Groups x places (the walk and the advance) | 10.9% | 15.3% | 13.1% | 32 x 44 |
| The pick (the picture again) | 15.4% | 15.1% | 16.1% | |
| The decode (all of these once a packet) | 8.0% | 11.5% | 12.1% | |
| A place, once a gate | 7.3% | 12.3% | 7.9% | |
| Builders x types x spots, and builders x places | 4.2% | 10.8% | 3.8% | 23 x 48, 1,089 questions |
| Groups x groups and builders (join, follow) | 2.6% | 3.7% | 5.5% | 27 x 48, 402 questions |

The count of groups is the multiplier, and it is made of single soldiers. At human-18's costliest gate (27:31, 330,000
tokens, 887 questions) the seat had 31 groups, 23 of them one soldier, and every one was asked about every party,
place and other group (27 questions at the median). 1v2-easy-1 at 22:32: 24 groups, 12 of them one or two soldiers,
34 questions each. player-54 at 25:00: 17 groups, 12 of one or two. A detachment is a group of its own from the
moment it is sent, with the whole menu, and it stays one.

The second multiplier is the request split: a gate of 887 questions goes out as 7 requests and each carries the
picture, so the picture's cost follows the question count.

## 11. Two menu cuts for groups, replayed (`run/jev_group_menus.py`)

**A short menu for small groups** (keeps `change`, `hold`, the moves against parties and the joins; drops the walks
and advances to places, the shelling of places, detachments of its own, the rove, the gather, the follows):

| Game | For a detachment of 8 or fewer | Their played moves shut | For a group of 1 or 2 | Their played moves shut |
|---|---|---|---|---|
| human-18 | 1.8% of the game | 84 of 165 | 3.9% | 131 of 406 |
| 1v2-easy-1 | 14.3% | 282 of 505 | 7.7% | 175 of 391 |
| player-54-experience | 6.4% | 106 of 216 | 7.3% | 134 of 424 |
| going-2 | 5.9% | 135 of 269 | 8.9% | 182 of 596 |

What is shut is mostly walks to places (27 to 89 a game), follows, gathers and detachments of detachments. The
army itself often carries a `split_from` from long ago, so origin alone does not mark a detachment; size does.

Asking a group of 1 or 2 only about its nearest party saves 0.6 to 3.2% more and shuts 4 to 16 of 33 to 60 plays;
its two nearest, 0.2 to 2.3% and 0 to 8.

**Joins and follows with the nearest groups only**: 1 to 3% of the game at the three nearest, shutting 69 to 165 of
311 to 436 played (20 to 53%). The joins the pick plays go to far groups about as often as near ones; rank by
distance is not what it chooses by. Not worth building.

**Where the group cost sits, by size** (all of a group's questions, share of the game): human-18: over 20 soldiers
12.8%, 9-20 7.2%, 3-8 7.4%, 1-2 8.6%; 1v2-easy-1: 3-8 14.7%, 1-2 11.2%, over 20 6.6%, 9-20 4.7%. So small groups are
a third to two thirds of the group cost, not all of it (§10 said "the multiplier": at the costliest gates, yes; over
the game, in part).

**Groups against parties by question** (human-18 / 1v2-easy-1, share of the game): the detachment ladder (1, 2, 4, 8)
10.0% / 6.8%; the whole-group attack 4.9% / 3.5%; shelling 3.8% / 0.5%; the forbidden mark 1.4% / 4.4%.

## 12. The 8 against 8 bill, and two cuts that did not hold (2026-10-04 night)

glitters-8v8-hard-1: 155.8M tokens, $6.54 over eight seats. Cut four new ways (the script is not kept; the figures
are scaled from characters as `run/jev_bill.py` does):

| | tokens | of the bill |
|---|---|---|
| clauses over 40 characters said twice or more inside one gate | 31.1M | 20% |
| the rules, on every request | 22.7M | 15% |
| the picture in gates of 20 questions or fewer (3,790 of 10,229 gates) | 14.3M | 9% |
| picks in which nothing was taken (2,358 of 5,197) | 12.4M | 8% |
| gates after which no actor changed course (7,441) | 73.8M | 47% |

Answers by family: a place 72% under 0.2 and 5% at 0.5 or over; group x place 60% and 3%; builder x party 90% and
0%; builder x type x spot 50% and 1%; group x party 36% and 29%. The walk to a party does not separate the party
questions: "far" is answered at 0.5 or over 25% of the time, "right here" 33%.

**Skipping the pick on the gate's rating: not built.** K-jev-the-gates-rating-does-not-foretell-the-pick
(`run/jev_pick_predict.py`): the ratings sit at 0.5 to 0.6 and the pick takes half of those.

**Words said once: does not hold.** K-jev-does-not-read-a-mark-for-words-said-once: the marks save 10 to 16% of a
gate and the forbidden answers collapse. What holds is small: the "Given ...:" opening dropped (5.1%, the openers
move 1.3 points) and the closing "no when ..." sentence of the place and party questions said in the rules (2.2%).
