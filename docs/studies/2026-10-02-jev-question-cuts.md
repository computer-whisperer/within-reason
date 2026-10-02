# Cutting the gate's questions (2026-10-02, evening)

The user, after player-40 (lost 34:48 from a 22-minute lead, Jev $5.37): "we probably don't need to re-ask from each
constructor every turn unless something drives an event on it. We can also look into a 4th stage that runs as a
pre-gate and blanks off things like 'does anyone have a reason to go to spot_43'. One other idea is to reduce the
per-question boilerplate and lean on the input context for information that is repeated across questions."

Everything is counted on player-40's Jev log (`run/matches/1790967663-player-40-frozen/00/jev-0.jsonl`) or asked
again offline with `run/jev_cut_ab.py` on 24 of its 1,561 gates (16 spread over the game, the 8 largest; 19,362
questions, 4.47M tokens as recorded). Jev spend: $0.59 ($0.33 the first three arms, $0.15 `short2`, $0.11 five
wordings of the place question). Data: `docs/studies/data/question-cuts-2026-10-02/`. Characters to tokens at the
recorded rate, 2.78 a token; $0.042 a million tokens.

## 1. Where the gate's tokens go

Player-40's gates are 100.9M tokens ($4.24 of the $5.37; the picks and the decodes the rest). Of a gate's text:

- **The questions are 82%** (230.6M characters), the picture 18% (49.9M, sent once per request: 4,454 requests for
  1,561 gates, since a gate past 140,000 characters is split). An actor's own moves (go, build, fight_to, help,
  ...) are 89% of the question text: builders' 42%, groups' 41%. Answers to enemy parties, openers and forbidden
  marks are the rest.
- **Why an actor was open.** The `news` layer asks an actor with a course only on its news; an idle actor is asked at
  every gate. Attributed from the pass rows (an actor's own events, the packet changing, its course changing, the
  20-second re-ask, else the store or a party in its entry):

  | kind, why open | questions | share of text | at the bar (0.5) | played within 3 s |
  |---|---|---|---|---|
  | builder, the store flag | 112,237 | 21% | 117 | 73 |
  | group, an event names it | 80,293 | 20% | 1,815 | 528 |
  | group, idle | 63,182 | 11% | 1,400 | 413 |
  | builder, an event names it (mostly "opened") | 46,640 | 9% | 163 | 97 |
  | builder, the packet changed | 31,922 | 6% | 54 | 35 |
  | builder, idle | 24,921 | 4% | 169 | 105 |

  The store flag (`mod.rs` `eco`) had three parts: the store crossing empty or full, energy stalling, and whether an
  idle lab can now afford its cheapest unit. Reconstructed at the gate frames, the third part flipped 149 times of
  231, the other two 82: **every lab cycle flipped it twice, and the news rule applied it to every builder**, so each
  cycle reopened every constructor (8 to 9 at 13:41). 46% of the reopened builders' `change` openers said yes, each
  yes asked about 75 moves the next second, and 0.1% of those reached the bar. The part was written for the lab.
- **Each own-move question is 467 characters**: 141 of fixed wording ("Given `actors.X`, `economy`, `ours` and the
  player's `instructions`: is this what X should do now, rather than ... ? The move: "), 85 the course it would
  replace, 241 the move. Across an actor's moves of one verb the move words share a prefix and suffix worth another
  10% of the text ("our 23rd (4 under way already); stops helping constructor_11407").
- **Place-aimed moves are 79% of the question text.** A gate asks about a median 48 distinct places (68 in gates of
  500 questions or more) through 146 questions (702). Of 71,080 (gate, place) pairs, 1,760 (2.5%) had any move at
  the bar.

## 2. The questions asked again in shorter words

| arm | question text | answers crossing 0.5 against the record | idle actors' moves at 0.3 in the record that came back under 0.3 (of 219) |
|---|---|---|---|
| control (asked again as recorded) | 100% | 42 of 19,362 (0.22%), mean change 0.010 | 16 |
| short (the move's words; the framing once in `rules`) | 58% | 78 (0.40%), 56 of them to no; mean change 0.038 | 95 |
| short2 (the words, then "Rather than: {course}") | 75% | 93 (0.48%), 52 yes / 41 no; mean change 0.037 | 56 |

By kind of actor (crossings, control / short / short2): builders with a course 14 / 21 / 16 of 14,112; idle
builders 3 / 3 / 4 of 197; groups with a course 20 / 34 / 53 of 3,818; idle groups 4 / 18 / 16 of 1,167. **A
builder's answer barely moves with the framing; a group's moves twice to three times as much as asking again, and
an idle group's move loses its rating when the question no longer says it stands idle** (K-jev-the-course-in-each-
question-is-what-the-move-is-weighed-against found the same on player-36 without the split by kind). The follow
verb drifted most (-0.10 mean), join and go next.

So: the fixed wording can go from builders' questions (30% of their text, about 13% of the gate's) and not from
groups'. Not built.

## 3. One question per place

For each place with a move aimed at it, one noul over the whole picture: "does any of ours have a reason to go to,
fight at or build at {place} now ({what stands there, how far}), rather than carry on with what it does? Those
with a move there: {actor (verb, distance); ...}. No when nothing there needs doing now or the instructions send
everyone elsewhere." (`v0`.) 1,324 place questions stood for 16,356 move questions, 10% of the recorded text.
Scored against the recorded per-actor answers:

| bar on the place's noul | places kept | move questions kept | of 25 places with a recorded move at 0.5, kept | of 83 places where an idle actor's move was at 0.3, kept |
|---|---|---|---|---|
| 0.2 | 69% | 79% | 25 | 79 |
| 0.3 | 44% | 53% | 24 (the miss: passage_2 at 19:04, place 0.21, move 0.56) | 69 |
| 0.5 | 11% | 11% | 16 | — |

Asked again, the control itself kept a move at 0.5 at 21 of the 25 places. The place's noul ranks the places well
(a place with a move at the bar is rated above 90% of the places without one) but the distribution is lenient
(median 0.27, 44% at 0.3 or over), so the cut at 0.3 is about half of the place-aimed questions, net of the place
questions' own cost about 27% of the gate's text.

### The wording round

Five other wordings, same gates ($0.11): `v1` strict ("Yes only when the instructions send someone there or
something there needs doing this minute ... no when the place is merely reachable"), `v2` instructions first and
"most places are not", `v3` "would the player send any of ours there this second", `v4` compact (no "Given"
opening, 20% fewer characters), `v5` "worth its cost in walking and in what the actor leaves". Compared at equal
catch rather than at a fixed bar:

| wording | text | bar that keeps all 25 hits: questions kept | bar that keeps 24: questions kept | ranking (share of no-hit places rated below a hit, mean) |
|---|---|---|---|---|
| v0 | 0.93M | 0.21: 77% | 0.37: 34% (0.30: 53%) | 0.896 |
| v1 strict | 1.14M | 0.12: 84% | 0.20: 52% | 0.817 |
| v2 | 0.90M | 0.20: 77% | 0.25: 64% | 0.838 |
| v3 | 0.91M | 0.17: 68% | 0.17: 68% | 0.846 |
| v4 compact | 0.74M | 0.29: 57% | 0.32: 47% | 0.868 |
| v5 | 1.11M | 0.24: 68% | 0.28: 56% | 0.801 |

**A stricter wording lowers every place's noul, the hits with the rest, and ranks worse.** The first wording ranks
best; the compact one is close and a fifth cheaper. A per-gate top-N rule on the ranking is worse than the bar
(top 20 of a median 67 places: 34% of the questions kept, 23 of 25 hits, 55 of 83 idle places). The pre-gate at
this granularity halves the place-aimed questions; it does not cut them by five.

## 4. What was built, and what the three levers are worth on player-40

- **Built: the store flag's lab part reopens labs only** (`layers::News.afford`, `mod.rs` `pass`; H-HANDS-LAYERS
  `news` amended). Law: a builder is reopened by the store only when it crosses empty or full or energy starts or
  stops stalling; whether an idle lab can afford its cheapest unit reopens the lab alone. Counted on player-40,
  about 36M characters of the 56M the store-opened builders cost, about $0.5 of the $4.24; the 73 played moves
  come instead at the builder's own news or its 20-second re-ask. Unmeasured in a game.
- **Built later the same evening (the user: "Let's build these improvements and run a game with it"): the place
  pre-gate as the `places` layer** at 0.3 with the first wording, ridden in the openers' request (H-HANDS-LAYERS):
  net about $0.75 after the first. Law: a move aimed at a place is asked only while Jev says, this gate, that
  someone has a reason to go there; Jev decides per place, code the bar.
- **Built with it: builders' questions without the fixed wording** (the diet's `builder_words`, H-HANDS-DIET), the
  course kept as "Rather than: ..." and the framing said once in the rules: about $0.25 after the first two.

Together about $1.5 of the $4.24, and the game at about $3.9. What remains is the menu's breadth times the number
of actors; the picks and the decodes ($1.13) were not looked at.
