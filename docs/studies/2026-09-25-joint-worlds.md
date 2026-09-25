# Joint worlds offline: one Choice over the groups' joined courses (2026-09-25)

The user, on ../jev_experiments battery J ("255 different worlds scored at once looks like it works perfectly well"):
"Let's try it offline now. Go pick some critical hand moments from previous matches and try the world enumeration."
`run/worlds_ab.py`: at the detectors' moments of family-1, bank-1 and 2v1b-hard (a raider ignored beside a group, a hold
beside a base under attack, a forbidden detachment), the collision calls (two groups on one lone raider in the same
call) and confident ordinary picks, the groups asked in one call are put to Jev as ONE question over the worlds their
candidate actions make (keep the course; fight the nearest raider party whole or as a detachment sized to it; the way
back when the odds are against), each world with its computed line (who meets which party with what metal and odds,
how far, what stays unanswered and what it keeps killing, what each mover gives up), worlds with two groups on one
small party pruned, the count capped at 64 and the question at 40k characters (a 104k-character question was refused).
Data `docs/studies/data/worlds-ab-2026-09-25{,-cost,-score}.jsonl` and the dumps beside them.

## Three forms on the same 205 calls (worlds a call: median 3, max 48; groups: median 2)

| form | raider_ignored: flagged group does a wanted action / the raider answered by some group | hold_beside_attack | never_split | collisions repeated (of 37) | confident controls kept | p(top) median | cost |
|---|---|---|---|---|---|---|---|
| Choice, gains only on the lines | 71% / 83% | 64% | 61% | 0 | 59% | 0.60 | $0.066 |
| Choice, each move says what it gives up; `never split` prunes detachments | 60% / 68% | 53% | 100% | 0 | 59% | 0.57 | $0.066 |
| one Score per world (five levels), argmax in code | 29% / 40% | 43% | 100% | 0 | 77% | 0.24 (margin) | $0.074 |

For scale: the recorded flat picks were 0% right at these moments, the two-level layout 51-59% offline
(`jev-ab-two-level`), and the per-actor form collided on 37 of these calls.

## Reading

- **Coordination is solved by construction**, as battery J said: enumerating the worlds in code and pruning the
  double answers leaves nothing for Jev to get wrong there. The picked world never sent two groups after one lone
  raider; the recorded per-actor picks did so in every one of the 37 collision calls.
- **The raiders are answered at the two-level layout's rate or above** (71% against 59%), and the holds beside a base
  under attack better (64% against 51%), from one question instead of a `do`, a `where`, a `whom` and a `how_many`
  per group.
- **The pick diffuses with the world count**: p(top) 0.63 under 8 worlds, 0.36 at 8-15, 0.23 at 24 and more. Battery
  J's 255-world result had crisp value differences between worlds; ours are near-equivalent (which of two groups
  answers, which of two keeps), so the argmax over many worlds is closer to a draw. The remedy is fewer, sharper
  worlds: only the concerned groups vary, the nearer group is the only candidate for a party two could answer.
- **The gain-only lines are fight-happy**, as battery I warned: seven confident holds and walks of the 22 controls
  became engagements, and the two-level layout had the same tilt (18% of controls). Saying what each move gives up
  cost raider answers (71% to 60%) and did not save those seven: the cost words are read, but the recorded holds may
  simply have been wrong, since the same games ignored raiders 16-29% of the time. A cleaner control set is needed
  before the tilt is judged.
- **The Score form is the conservative one** (controls 77%, raiders 29%): the level words put "the courses kept" in
  the best level, and Jev scored keeping highest nearly everywhere. Battery J's 100% with Scores was on worlds with a
  known value model; ours needs one first.

## What follows

The world form replaces the group `do` question for the arm the outline calls `worlds`
(`docs/design/2026-09-25-menus-from-scratch.md`): the executor's joint result as world 0 and the single-actor
deviations as the rest, which keeps the count small and sharp and makes a deviation winning the record of a rule
breaking down. Before the live arm: the candidate pruning above, the cost words kept, and a control set drawn from
moments the detectors and a reader agree were right.
