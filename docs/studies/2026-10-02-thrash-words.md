# The thrash at a fortified place, and the words that stop it (2026-10-02, night)

The user, after human-10: "we will need to pursue why attacking fortified positions caused jev to thrash our column
back and forth within turret range", then "Let's test those improvements offline." Nothing is built. Jev spend:
$0.98 (`run/jev_thrash_ab.py`, two runs). Data: `docs/studies/data/thrash-words-2026-10-02/`.

## 1. What happened (human-10, group_O_t1 at the D4 outpost)

The outpost at spot_42: a constructor and a Leveler under four Gatekeepers (corhllt) and a Pounder (corexp). The
player's orders: "spot_42 NOW, fighting ... From a party that outweighs it, it walks to spot_53" (9:36), "attack his
buildings at outpost NOW" (10:16), "you do NOT walk back to spot_53 or spot_47 because of turret fire" (10:41),
"COMMIT, NO STEP-BACK ... The odds words about the few soldiers nearest the turrets do not matter" (14:02).

From the pass, flags and plan rows, 10:14-10:41: **fourteen course changes in 27 s** between the advance
(`fight_to_outpost` / `fight_to_spot_42`) and the walk back (`go_spot_53` / `go_spot_47`): walk 10:15, advance
10:16, 10:18, walk 10:23, 10:24, 10:26, 10:27, advance 10:28, 10:29, walk 10:33, advance 10:34, 10:35, walk 10:38,
advance 10:40. Every question is asked against the course in force, and:

- while the course was the advance, `change` was 0.76-0.86 and the top move was the walk back (`go_spot_53` 0.62-0.74);
- while the course was the walk, `change` was 0.85-0.91 and the top move was the advance (0.53-0.67);
- stage two accepted whichever was offered (0.3-0.88); candidates sampled at 0.20-0.21 were played.

Two texts pull opposite ways and each wins when it is the change on offer. The walk back's authority is the
rules' step-back sentence ("A group whose own `enemies_near` line says the party outweighs it ... steps back ...
whatever they said about keeping on: they could not see this") over an odds line that counts the soldiers within
800 alone: "for the 6 of its 51 soldiers near it, it outweighs us ... the other 45 are 800-1372 behind and not near
it yet". The whole group was 4,930 metal against 340 of party and 825 of turrets (2,815 with the turrets counted
three times as the rules say): 1.8 to 1. The advance's authority is the player, who had seen exactly what the rule
says it could not. The player also named the step-back place beside the target, so the walk was always on the menu.

Cost: 10,109 metal of ours within 800 of the outpost against 1,380 of his, the army there 10:09-14:48. The review
(`run/matches/1790987321-human-10/00/review.md`) found the same shape at 19:23 at D5-D6 under an unseen Shiva and
Tzar (24 course changes in 70 s, 87 units lost), the decisive moment.

## 2. The words asked again

The 32 gate calls of the two windows (10:14-10:41, 14:00-14:25) that asked group_O_t1, each 87,000 tokens as
recorded, under:

- `odds`: the `enemies_near` line and every move's fight facts gain, after the near-soldiers clause, "; the whole
  group, 56 soldiers worth 4930 metal, against the party and the turrets covering it (340 and 825, the turrets
  counted three times: 2815) outweighs it 1.8 to 1" (computed from the entry and the party's `in_sight` line).
- `override`: the step-back sentence ends "whatever they said about keeping on, unless the instructions speak to
  this very fight (this party, its place or the turrets covering it, by name): those they could see, and a group
  they told to go in against it goes in as one body."
- `standing`: a sentence added to the rules: "A course the group chose in the last ten seconds (its `last_pick`
  line) stands unless something has happened since that the instructions or these rules answer: a group that
  reverses its course every few seconds walks back and forth under fire and loses more than either course would."
- `all`: the three; `pair`: `odds` with `override`.

Scored per call by the course in force: while advancing, is a walk back at the bar (0.5, with `change` at 0.5):
the thrash's left foot; while walking, is the advance at the bar: the right foot, which the player wanted.

| arm | advancing (15): walk back at the bar | best walk back, median | best advance, median | walking (13): advance at the bar | best advance | best walk back | `change`, median |
|---|---|---|---|---|---|---|---|
| recorded | 7 | 0.53 | 0.30 | 8 | 0.51 | 0.54 | 0.77 |
| control (asked again) | 7 | 0.54 | 0.35 | 7 | 0.50 | 0.53 | 0.77 |
| odds | 8 | 0.53 | 0.35 | 10 | 0.61 | 0.51 | 0.73 |
| override | 4 | 0.42 | 0.32 | 11 | 0.57 | 0.46 | 0.72 |
| standing | 10 | 0.54 | 0.33 | 9 | 0.52 | 0.53 | 0.78 |
| **pair** | **1** | **0.40** | 0.40 | **11** | **0.64** | 0.45 | 0.68 |
| all | 1 | 0.43 | 0.40 | 12 | 0.65 | 0.44 | 0.67 |

Drift on 12 other group gates of the same game (4,387 nouls), answers crossing 0.5 against the record: control
0.50%, odds 0.46%, override 0.48%, **pair 0.55%**, standing 0.64%, all 0.68%; mean change 0.012 to 0.017.

## 3. Reading

- **The pair stops the left foot** (7 of 15 to 1) and keeps the right (11 of 13): while the group advances the walk
  back falls under the bar (0.40), and while it walks the advance is the move (0.64). Neither alone does it: the
  whole-group odds give the rule's exception its fact, the exception gives the odds their standing.
- **`standing` does nothing good**: asked alone it offered the walk back more often (10), and it drifts the controls
  the most. Not recommended.
- The drift of the pair is at the control's level: the words bear on a group under a fortified party and on little
  else.
- Not tested: a group that is genuinely outweighed by an army (the words should still step it back: the exception
  needs the instructions to name this fight), and the 19:20 episode under unseen fire, which is a sighting problem
  more than a words one.

## 4. What building the pair would be (not built)

Two word changes, no decision moved: `picture.rs` (the `enemies_near` line, line 698's words, gains the whole
group's weight against the party and its turrets), and `rules.md` (the step-back sentence's exception). Law: *a
group steps back from a party its near soldiers cannot match unless the instructions speak to that very fight;
the odds line says both the near soldiers' and the whole group's weight.* The player's habit of naming the
step-back place beside the target is the brief's to address.
