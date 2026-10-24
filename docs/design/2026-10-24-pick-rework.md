# The pick is one draw over the worlds, the course among them (2026-10-24)

**Status.** The user's rulings of 2026-10-24, in order: "any kind of move-based hysteresis is the wrong lever, and we
should remove that final gate altogether"; "the correct policy at any time is purely a function of the state of the game
at that point, and I think the policy we were taking at t-1 should not come in directly"; "if jev is telling our bot
engine that ten different courses of action are all similarly good at a given frame, then sampled alternation between
them should be acceptable"; "let's start with removing that final stage and allowing the current control state to exist
in the world set. We should make sure the wordings don't privilege that option"; "Once we have that version, we can run
a basic BARb game with it to reveal the full scope of the current issues"; and "we should also review our gate and world
proposals logic overall now, since built up clauses and conditions could easily contribute to alternation problems."
§3 is built; §4 is the review, for him to rule on; §6 the next cuts his principle names.

## 1. What the games showed (nine games of 2026-10-23/24)

The symptom he sees most: a group cycling between advancing and retreating under fire. Measured on rework-series-2
(three games), rework-ladder +10/+35/+50 and three human games on Comet (`run/hands_thrash.py`; the decomposition
scripts were scratch, their numbers here):

| Game | Group picks | Within 10 s of the group's last | Worst minute, one group |
|---|---|---|---|
| rework-series-2 (three games) | 148 / 434 / 231 | 42% / 52% / 48% | 10 / 16 / 20 |
| rework-ladder +10 / +35 / +50 | 119 / 328 / 551 | 48% / 58% / 54% | 10 / 14 / 17 |
| three human games (Comet) | 154 / 176 / 76 | 64% / 61% / 58% | 16 / 14 / 9 |

- Of the 1,195 quick picks: 96 re-picked the same move, 109 reversed to the move of two picks back (A-B-A), 990
  switched to a different move. The advance/retreat cycle is the visible subset; the engine is a group re-decided
  every second it is asked.
- Evenness at the gate: the played move's rating has a median of 0.58 (57% under 0.6); the gap to the actor's next
  candidate has a median of 0.01, for quick and spaced picks alike.
- Stage two took the change at any confidence (code: `taken = choice == wk`): confidence median 0.31 to 0.49 a game,
  60 to 76% of changes taken under 0.5. A 0.5 confidence bar would have held 53 to 72% of quick picks and 44 to 70%
  of spaced ones: it separates nothing. Stage two declined 3,216 of 5,313 group candidates (61%); of a group hit this
  second, 309 of 892 (35%).
- Stage one's sampling, ruled 2026-09-29 for 96-world joint picks, ran on `split` components of one or two changes
  81% of the time; in two-change components the minority option was drawn 34% of the time (846 of 2,491), 277 played.
- 22 to 57% of group picks were of an idle group (arrived, or a hold nobody picked): `change` is 1.0 by construction,
  the bar 0.3, asked at every gate, world 1's line saying "with its next move X, not ordered" as a cost. The rebuild's
  §15 item 1 flagged it as the user's to rule; it was not ruled.
- A bug: a picked `gather` sets a move to the body's front, and the arrival test (the 60th-percentile member within
  300 of the goal, `groups.rs`) fires at the next upkeep for exactly a dense front with a straggling tail; the group
  reads idle a second later and gather is picked again (rework-series-2 game 3, group_B 15:28-15:39: eight picks in
  twelve seconds, the tail 625 to 691 behind throughout).

The gate's trace of the human game's group_B (14:00-14:45, twenty soldiers under a party of thirty): fifteen courses in
45 seconds, four detachments shed, two of them under a forbidden mark; `change` 0.5 to 0.86 every second because
"something near it needs answering" is always true under fire; the moves 0.4 to 0.6; stage one sampled among them and
stage two accepted (14:26: a detachment drawn at 0.15 of the mass, taken at 0.08).

## 2. The principle, and what it says about the words

The right action is a function of the state. The register line ("picked X 6 s ago"), "this undoes its last pick" and
"it took this course N s ago" are t-1 entering directly; they go (§6, not in this step). What stays is state: the
body's position in the fire geometry, its health, its heading, the orders its units hold. Those are consequences of
t-1 and facts about now, and the lines do not carry them.

**What the advance/retreat case is missing** (the user: "the part jev is not being informed of correctly is how much
damage a given army will take from shelling if it takes path A or path B"): the fire the body takes now, as a rate;
on each move the fire on the way (seconds inside known reach times the shooters' rate, the metal lost getting there,
the body's health on arrival); the odds as they will stand when the body is in, not the slice in reach this second
(that slice swings as the front enters and leaves reach, the engine of the D4 thrash of human-10); for a retreat the
fire crossed on the way out, and for a re-advance the whole band crossed again at the health the body has left. That
last fact is what makes re-engaging expensive with no reference to the earlier pick. The pieces exist: the shelling
tracker (`brain/shelling.rs`), the turret reach map (`picture.rs` `turrets_covering`), the route fields
(`brain/routes.rs`), the sim's damage rates (`combatsim`), per-unit health in the snapshot.

**A switch has a cost of its own that is also state.** The user, 2026-10-24: units have acceleration, deceleration and
a turn radius; a column moving one way churns for a while and then anneals into a functioning column at full speed; a
column told to turn around loses that structure and spends a fair time churning, each unit running through its turn
radius and bumping into the others before accelerating back. So two courses equally good to commit to are not equally
good to switch to from each other, and today's gather re-pick is the pure form (eight orders, the tail never moved).
That cost belongs on the line as a fact about this second's body (its heading, how far it is into its stride, how
strung out), never as a count of past turns (H-HANDS-TURNS (1), retired 2026-10-07).

Why stage two was the wrong lever: it stood between a sampled candidate and the order with a symmetric question
("Pick one") and held 61% of candidates by what amounts to a second coin; the course's privilege lived in the rules
text ("choose a move only when it is clearly better than the course"), a hysteresis in words.

## 3. Built (this step)

- **One Choice over every world, world 1 among them, the world drawn by its probability** (`compose.rs`
  `pick_question`, `run_pick`, `sample`, `split_questions`, `split_pick`). Under `split`, one `worlds.c<j>.pick` per
  component over its world 1 and its changes (a component with one change is a two-option Choice); the world drawn is
  put in force, world 1 drawn is nothing. Without `split`, one `worlds.pick` over the joint worlds. Stage two,
  `stage_one`, `stage_two`, the candidate field and `compose::pick` (the top of a Choice) are deleted.
- **Why a draw and not the top.** The 2026-09-29 fault that brought the two stages (K-hands-a-plurality-over-near-
  duplicates-picks-nothing, `docs/studies/2026-09-29-pick-framing.md` §1-§3) was world 1 taking the plurality while a
  change's mass split over near-duplicate joint worlds; it was not a wording preference (framed without its sentence,
  as a plan, or last, w1's share stayed 0.10-0.12). Under a draw, w1's share is its chance and no more: a w1 at 0.33
  against ten changes at 0.067 each is a change two times in three.
- **The wording privileges nothing.** The question names w1 as one world among the rest ("w1: every actor keeps the
  course its entry under `actors` describes ... Pick the best world this second"). The one sentence of the rules
  text that privileged the course ("choose a move only when it is clearly better than the course"; the short form in
  `rules_gate.md`, "An actor keeps its course unless a move is clearly better") is cut; "An actor's
  course goes on unless a move is picked for it" stays as the mechanic it describes.
- **The log.** A `plan` line carries `pick` (the world drawn, 1 = world 1), `sampled_p` and `confidence`; under `split`
  each part's `candidate` is the world drawn (1 = w1, `taken` false). The split audit's joint pick is one call, its
  row gains `joint_p`. `docs/harness/record-format.md` amended; `run/hands_window.py` and the viewer say "drew w1".
- **Not changed in this step, so the game isolates the gate's removal:** the register line and "this undoes its last
  pick" (t-1 in words, §6), the young-order words, what counts as news, the idle actor's bar and the "not ordered"
  sentence, the gather arrival, every clause of §4.

## 3b. Built (step 2): the t-1 words out

The user, after pick-one-1 was launched: "let's pick a set (or all) of the t-1 leakage terms to strip and run again."
The set stripped is every word that says what we chose or how long we have been doing it; what stays is state, and
the sequence position the instructions are read against. Deleted:

- the `last_pick` line in every actor's entry, "this undoes its last pick" on a move's line, and with them the
  `Register`, `register_words` and the pianist's registers;
- the young order's age ("it took this course N s ago and is on its way ...: it has not had the time to arrive"):
  the standing words now say "nothing of it has the party in reach (the nearest ... away, on its way)" when its
  units have orders and "..., standing with no order" when they do not, a fact of the orders this second;
- every clock on a course in the entries: "holding here on your pick for N s" (now "holding here"), "standing with no
  order for N s ... since it arrived by advancing" (now "standing with no order, fighting everything here, turrets
  included"), "advancing to X: ..., for N s", "told to attack X at Y, N s ago", "following W for N s", "hunting P
  since N s ago", "roving since N s ago", "gathering on its front for N s"; the builders' "since N s ago" and
  "ordered N s ago" on every task; the chase's "for N s"; on other groups' lines "has attacked it for N s" (now "is
  attacking it"); a detachment's "the group it split from N s ago" and "split from it in the last N s";
- the rules' "A group already walking away from such a party carries on with its walk";
- the dead fields behind them: `GroupTask`'s `since` on Hold, Move, Engage and Rove, `Group::born`, `Menu`'s
  `course_key` and `course_words`.

Kept, as state or as the instructions' sequence position: `reached` and `met`; the `losses` window of 30 s with the
last loss's age (a measurement of the fire, and the player's instruction is dated against it); the stall and stuck
clocks ("has not got nearer its goal for N s", "its moves have failed since N s ago", "standing at the site ... for
N s with nothing started": the state of a failing order); the build frame's percentage; the `player` line's clocks
(the player's turn, not our policy). Left for the gate review: the `change` question's "nothing has changed" (§4
row 2), a reference to the course's start.

## 4. The gate and the worlds, clause by clause (the review; the user's to rule)

Read from `compose.rs`, `menu.rs`, `layers.rs`, `mod.rs` on 2026-10-24. "Suspect" marks a clause that can by itself
make alternation likelier, by the evidence of §1 or by construction.

| # | Clause | Where | Suspect |
|---|---|---|---|
| 1 | Every party is asked `answer` at every gate: "does it need answering by someone other than what stands" | `gate_questions` | the framing asks for a change of hands by construction; a party answered by a group not yet in reach reads as "told to attack it, but ..." |
| 2 | An actor with a course is asked `change` ("should it do something other than what it does now ... yes when something near it needs answering") | `gate_questions` | under fire "something near it needs answering" is true every second: `change` 0.5-0.86 throughout the trace |
| 3 | An idle actor has no `change`; it counts as 1.0, its bar is 0.3 when nothing reaches 0.5, it is asked at every gate, and world 1 names its best move "not ordered" | `candidates`, `news`, `parts_of` | yes: 22-57% of group picks; an arrival ends in this state by design (`groups.rs`) |
| 4 | One noul per move, asked against the course ("rather than {standing}"); a party-aimed move as an answer to the party | `gate_questions` | the differential frame: whichever course stands, its opposite is the change on offer (the thrash-words study) |
| 5 | A move goes forward at 0.5 with its opener open (the actor's `change`, or the party's `answer`); the party path needs the move at 0.5 as well (2026-10-03); rank = opener x rating | `candidates` | a cliff at 0.5 and no margin: the played move's median is 0.58, the gap to the next 0.01 |
| 6 | Two moves an actor go on (`DEPTH`) | `candidates` | truncation by rank, not by margin |
| 7 | World 1 all `stay`; singles round-robin; joint worlds as the product over the best-ranked actors under 255; clashes dropped; 40,000 characters of lines | `worlds_over`, `clash`, `fit` | no |
| 8 | `split`: components by touching names, parties and spots; nine worlds a component | `components`, `split` | 81% of components offer one or two changes: the pick is mostly a two- or three-way draw |
| 9 | World 1's line: "Nothing changes" plus the parties left to nobody (answer at 0.5) plus each idle actor with its best move "not ordered" (plus the store for makers) | `consequence` | the idle sentence prices doing nothing as a cost every second (see 3) |
| 10 | A change's line: "As w1, and: ..." plus what it adds (Met, Sent at, Not met, Left to nobody, idle) plus the forbidden sentence | `consequence` | the forbidden sentence informs and does not hold: two marked detachments were played in the human trace |
| 11 | A move's words: the fight's facts on `stay` and on every move; "leaves ..."; "stepping back from / closing on"; "a walk back toward our side"; "this undoes its last pick"; the young-order words on an attack course | `menu.rs` `group_menu`, `Standing::words` | the odds are the slice in reach now (see §2); `undoes` and the young order are t-1; nothing says the fire on the way or the turn's cost |
| 12 | The forbidden noul per detachment (0.7, a sentence on the line) and the `fuse` marks (on at 0.8, off under 0.2) | `gate_questions`, `decode.rs` | a hysteresis of its own (two bars) |
| 13 | `places`: a place question opens its moves at 0.3 and closes under 0.2 | `layers.rs` | a hysteresis of its own (two bars); built against flapping (player-41) |
| 14 | `news`: a busy actor is asked when its course or its entry's party changed, an event names it (hit, a death, reached, met, opened, chase meets), the packet or a list changed, the store crossed (builders, plants), or 20 s passed; an idle actor always | `layers.rs` `news`, `schedule.rs` | an event alone re-asks no oftener than every 5 s (`EVENT_GAP`), but a changed signature (each actor's course and idleness, the parties, the store, the packet) re-asks at once, and under fire it changes most seconds (the trace: asked every second); harmless once the state is valued right, a token cost until then |
| 15 | `openers`: a move is held while its opener said no within 20 s; `same`/`again`: identical words' answers stand 20 s, 3 s after an opening; `tick`: no-event gates every 2 s | `layers.rs` | no (savings; audited) |
| 16 | The rules text: the step-back sentence (a party that outweighs the group, or a `losses` line that says it is losing, steps the group back unless the instructions name this fight); "a party in sight within 600 of a group that outweighs it is fought now, whatever the instructions call it"; "an instruction that reached the hands after the group's last loss stands over this rule" | `rules.md` | three precedence rules on one decision; the first two pull opposite ways as the odds slice swings (human-10, the D4 outpost) |

Deleted in this step from this list: the rules' "choose a move only when it is clearly better than the course" and
stage two.

## 5. The game

`pick-one-1`: the arena target line on seed 20 (Opus 5.5 medium, BARb hard_aggressive, Comet Catcher Remake 1.8,
mirror, lockstep at speed 50, the rework brief, layers `news,same,fuse,openers,split,tick,places,again,near`, Jev budget
10, claude2), the seed rework-series-2 won at 14:32. It is expected to play worse: the user, "I agree that removing
the final stage will likely cause our performance to drop, but I think that is acceptable for how it reveals the actual
underlying issues." What to read in it: `run/hands_thrash.py` (the quick-pick share and the worst minute, against the
table of §1), the share of picks that drew w1, the group traces at the fights (`run/hands_window.py`), and which of
§4's clauses the alternation runs through when the gate no longer hides it. `pick-one-2`: the same line with the t-1
words out (§3b), launched when pick-one-1 ends; read the same way, against pick-one-1.

**pick-one-1 (f51a2a0): WON 24:24** (`docs/experiments.md`). A change drawn in 64% of the components with a candidate
(groups 69%, against stage two's 39% taken before), w1 drawn in 36% at a median mass of 0.55. Groups: 380 picks in 24
minutes, 40% within 10 s of the group's last (inside the nine games' band, at its low end), worst minute 14; of 153
quick picks 19 re-picks, 12 A-B-A, 122 switches. What the gate's removal reveals: **68% of group picks were of an idle
group** (22-57% before), §4 row 3: an arrived body is asked every gate at the 0.3 bar and a change is drawn two times in
three; the gather arrival (§1) makes it a loop (group_Z at spot_27, 20:02-20:23: gather drawn at 0.06 of the mass and
re-drawn five times, w1 three times between, a walk of 337 to the spot it stood at, gather again, an advance at 0.53).
A body of six shredded into three detachments of two by `send` draws at 0.41-0.70 of the mass in fifteen seconds
(group_B, 5:14-5:27). Quick picks of a group hit this second: 47 of 380; the advance/retreat alternation is the
minority. Group verbs played: join 119, go 99, fight to 61, attack 49, send 36, gather 28, hold 13, follow 13, shell 3.
Evenness: the quick picks' played rating median 0.55, the spaced picks' 0.66 (gap 0.03): the draw plays the decisive
gates well and the idle ones by lot. The lead (4,012 against 2,135 at 12:00, 8,262 against 2,654 at 16:00) went
unconverted until the push of 20:50; 21 extractors lost to raids. Jev $0.72, the rate of the series.

## 6. Next, under the principle

1. The state words of §2 (the fire on each path, the odds as the body will meet them, the turn's cost as this
   second's body), replayed offline on the recorded thrash windows before a game: does the walk back fall under 0.5
   while the body is deep in the band and the re-advance fall under 0.5 while it walks out, at the control's drift.
2. The t-1 words out: built the same evening (§3b); the game pick-one-2.
3. An arrival is a hold by the pick's own end, not idleness; the "not ordered" sentence for a newborn or where the
   packet names a next stop.
4. The gather arrival (§1).
5. §4's rows as he rules them.
