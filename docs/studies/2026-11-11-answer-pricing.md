# The answer priced whole: tonight's picks asked again

**Status.** 2026-11-11 night, on the user's word ("Build the joint pricing and replay tonight's moments offline").
Offline: the recorded picks of two games on the slots asked again as recorded and in two new wordings
(`run/jev_answer_ab.py`; rows in `docs/studies/data/answer-pricing-2026-11-11.jsonl` and `-b.jsonl`; 8.8M tokens,
about $0.37). Built from it the same night: the second wording (`docs/design/2026-11-11-groups-as-slots.md` §5.1,
H-HANDS-ANSWER). **Not judged in a game with a player:** two smokes of the pianist alone ran on the build.

## What was asked

The first flagship game on the slots (slots-apex-v0507-comet-opus-1) lent 91 times at 31 parties, 55 of them within
ten seconds of a lend at the same party, and withdrew 16 answers, five of them lent at again within ten seconds.
World 1 read "Not met: party_5 (...): group_answer_party_5 is told to attack it, but nothing of it has the party in
reach (the nearest of its 2 soldiers is some way off, about 10 s of walking away, on its way)", with no odds; a
world lending four more read "with the 2 already answering it, we outweigh it heavily ... Sent at: party_5 by
group_A".

The picks: every split pick of that game and of slots-smoke-2 (the pianist alone against BARb medium on a packet
that says one soldier and never two) in which a party had an answer or was lent at: 269 of 578, with 281
components of three kinds. Sixteen picks could not be read and are left out.

| kind | components | what it is |
|---|---|---|
| top-up | 135 | world 1 has an answer at the party and a world lends more to it |
| withdraw | 36 | a world withdraws an answer |
| first | 120 | a world lends at a party nobody answers: the control |

The arms: `again`, the request as recorded; `joint`, every world saying the answer once and whole, world 1 among
them, its verdict set beside world 1's ("as in w1", or "(in w1, with its 2: an even fight)"), the lend in short
words, no world that both withdraws an answer and lends to it; `joint2`, the same with the party kept in the
rung's own words on the lend (its heading and the extractor it is making for), the answer's clause naming the
party alone in a world that lends, and the lent ones' arrival set beside the arrival of those already on their way
("no sooner than those on their way").

The lines are rebuilt from the log, not by the bot (the menus are not in it): the standing answer's own verdict is
the last recorded lend line that priced as many soldiers at that party, and a world of two lenders at one party
(52 components) is priced by the recorded verdict of as many soldiers, else by the better of the two. The bot's own
lines in slots-smoke-5 and slots-smoke-6 read as the rebuilt ones do.

## What it showed

Mean share of the pick's mass, by where it went. Asked again twice, a component's mass on world 1 moves 0.02 at
the median and 0.07 at the ninth decile; the means below move 0.01.

| kind | arm | on world 1 | on a lend | on a withdrawal | the top world is a lend | the top world withdraws |
|---|---|---|---|---|---|---|
| top-up | recorded | 0.51 | 0.47 | 0.01 | 0.42 | |
| | again | 0.52 | 0.46 | 0.01 | 0.41 | |
| | joint | 0.64 | 0.32 | 0.01 | 0.13 | |
| | joint2 | 0.65 | **0.31** | 0.01 | 0.12 | |
| withdraw | recorded | 0.40 | 0.06 | 0.54 | | 0.61 |
| | again | 0.39 | 0.06 | 0.55 | | 0.56 |
| | joint | 0.45 | 0.04 | 0.48 | | 0.50 |
| | joint2 | 0.47 | 0.04 | **0.47** | | 0.39 |
| first | recorded | 0.49 | 0.47 | | 0.41 | |
| | again | 0.50 | 0.46 | | 0.40 | |
| | joint | 0.53 | 0.44 | | 0.30 | |
| | joint2 | 0.48 | **0.49** | | 0.38 | |

The top-ups by what the answer already was (mass on a lend at its party; `joint2`):

| the answer in world 1 | components | again | joint2 | of it on a lend that changes the verdict, again and joint2 |
|---|---|---|---|---|
| already outweighs the party heavily | 70 | 0.45 | 0.30 | 0.02, 0.02 |
| outweighs it | 29 | 0.54 | 0.32 | 0.45, 0.25 |
| an even fight, or outweighed | 25 | 0.47 | 0.40 | 0.37, 0.32 |
| every lend on offer changes no verdict and is there no sooner | 63 | 0.39 | 0.22 (joint 0.28) | |

- **A top-up loses a third of its mass** (0.46 to 0.31) and is the top world in 12 % of the picks where it was in
  41 %. The lends that change nothing lose most; the lends at an answer that is even or outweighed keep theirs.
- **It is not ended.** Where no lend on offer changes the verdict or arrives sooner, 0.22 of the mass still goes on
  one, at every second the answer walks. The pick is a draw by probability, so some of these are still played.
- **Withdrawals lose a little** (0.55 to 0.47): world 1 now says the answer's own odds, where only the withdrawal's
  words did.
- **First lends hold only with the party's heading on the line.** `joint` cut the rung's party words to the
  party's name, and a first lend fell 0.46 to 0.44 (the top world in 30 % for 40 %); with the rung's own words
  back (`joint2`) it reads 0.49 and 38 %.
- Both games agree: the flagship game's top-ups 0.45 to 0.31, the smoke's 0.48 to 0.32.

## Limits

- The lines are rebuilt by a script from the log; the build's own lines were checked against them in two smokes,
  by eye.
- Jev alone: no game with a player has been played on the build. The lends and withdrawals of the two smokes on it
  (ledger slots-smoke-5, slots-smoke-6) are one game each against BARb medium.
- The measure is the pick's mass on a kind of world, not the fight's result.
- What is left for a ruling: the lender's rung is offered at every pick while the answer walks, whatever it
  changes. Not offering a rung that changes neither the verdict nor the arrival would be a rule of code on the
  menu.
