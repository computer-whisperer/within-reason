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
