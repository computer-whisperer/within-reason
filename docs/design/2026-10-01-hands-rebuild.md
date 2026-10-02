# The hands, rebuilt: one menu per actor (target design, 2026-10-01)

**Status.** Approved to build by the user on 2026-10-01 ("This looks good, let's begin deletion and implementation"); §14 holds the build's decisions. Draft 5 as approved: Drafts 1 to 4 went
through three reviews and a set of offline replays of recorded seconds; draft 5 folds in the user's rulings of
2026-10-01 on cost and on how to build it (§8); §13 says what each step changed. The row by row disposition of what
the code does today is in `2026-10-01-hands-rebuild-cut-line.md`; the token budget is in
`docs/studies/2026-10-01-jev-token-budget.md`.

The user, 2026-10-01, after K-hands-the-only-way-to-decline-a-fight-was-to-walk-home: "why is the only action space
available for a group not cover basic movement actions?"; "Why do we go from our units out to the enemy in one case,
and from the enemy back to our units in the other case?"; "This all needs to be rebuilt -- we should not be making
fine-tuning iterations when the whole lot of it obviously needs to be ripped out."

**Scope.** Rebuilt from nothing: what an actor can be told to do and how that is listed, the composition of worlds,
the executor's arms for those moves, and every place where code reads the packet's prose. Kept, because they are the
user's designs or rulings (§11 lists them with dates): the gate of nouls, the joined worlds, the pick in two
stages, the lists, the micro engine, the player's tools.

## 1. What is wrong, in numbers

From an inventory of the code at e690e5e (`docs/studies/2026-10-01-hands-inventory.md`) and five recent games:

- **Two directions.** A group stands in its own menu (walks, holds, merges) and in the menu of every enemy party
  near it (attack, hunt). A world cannot give one group two orders, so code settles the overlap: the party's answer
  wins and the group's own menu is closed. In player-33's 38 walks home, 0 of the 14 groups that were attacking had
  a move of their own in the pick.
- **Sixteen special options.** walk, advance, raid, sweep, gather, close on the shooter, pull out, shell, fall back,
  scout, split, join, escort, and in the party menus hunt, whole and the stand at the next extractor. Each has its
  own conditions. The walks exist, but their words say nothing of the fight the group is in, and they rate a median
  0.1.
- **Code chooses what Jev should choose: 37 places in the menu.** The nearest 3 places, the nearest 3 raids, the
  nearest 3 groups for a party, the pull-out place, the merge target, the hunters, half the group for a split,
  where it last held, which parties count as threats.
- **Code reads the prose: 19 places.** A paragraph is a group's only when its heading is exactly the group's name;
  otherwise the group is given the whole packet. In player-33 a median 1 of 4 groups had such a paragraph. The
  letters "advanc" anywhere make every place an advance ("never advances" too). The order of names is a route.
- **What the questions bought.** A group's walks and advances were 60 to 70% of all group questions (75,000 of
  105,000 in player-33) and were played 65 times that game. Sweep: asked 9,000 times over five games, played 0.

## 2. The laws

1. **Three deciders** (2026-09-28, unchanged): the player, Jev, the micro engine. The hands list the moves, word
   them, and carry out what was picked.
2. **One menu per actor, one move per actor a second.** A world gives every actor exactly one move. Enemy parties,
   places and our own units are what a move is aimed at; none of them has a menu.
3. **A move is a verb and what it is aimed at.** The verbs are a short fixed list per kind of actor: the orders a
   person has with that unit selected. Every verb is always on the menu.
4. **Code never chooses what a move is aimed at, and never chooses for whom.** Everything the picture or the packet
   names is on the menu of every actor that can use it. No nearest-N, no reach, no "where it last held", no "this
   place is that group's". A move is left out only when the unit cannot do it.
5. **Code never reads the prose for meaning.** It recognises names (`spot_12`, a mark, `home`, `group_A`). What the
   instructions mean for which actor is Jev's reading: every second with the picture beside it, or once per packet
   where the question is about the instructions alone (§8).
6. **Facts go in words; no fact adds or removes a move** (unchanged).
7. **Code carries a picked move to its end and says the end.** It never starts, changes or stops one by itself.
8. **What Jev needs to remember is printed, never acted on**: what each actor last chose, when, and what it left.

## 3. The verbs

| Actor | Verb | Aimed at | Takes the place of |
|---|---|---|---|
| group | stay | - | keep |
| | hold | - (every soldier stops where it is) | hold |
| | gather | - (the group closes up on its front) | gather |
| | go | a place | walk, pull out, fall back, the walk home |
| | fight to | a place | advance, sweep, close on the shooter, the stand at the next extractor |
| | attack | a party, or a place where his buildings are known | whole group, raid |
| | shell | the same, from its long-reach members' reach (only a group that has some) | shell |
| | send N | N of 1, 2, 4, 8 below its size leave as a group of their own to attack a party | hunt |
| | join | another group of its kind | join |
| | follow | a group or a builder of ours | escort; and the step back onto another body |
| | scout | - (one soldier leaves to rove) | scout |
| builder | stay | - | keep |
| | build | a type, at a place or beside itself | the building options, the extractor at a spot |
| | help | a factory, or a build under way | help |
| | take apart, repair | a wreck field, a damaged unit of ours | reclaim, repair |
| | go, follow | a place; a group | walk, go home, step away, under flak |
| | attack, D-gun | a party | the party menus' builder attack and D-gun |
| factory | stay, make | a type | keep, next unit |

An air group has stay, hold, go, attack (on one named unit, or a place where his buildings are known) and join; the
rest are things aircraft cannot do, as today. A builder on a list has only the moves that take it out of danger
(go, follow, attack, D-gun); the list is the player's order and resumes after.

**What a move can be aimed at.**
- *His:* every party in the picture, and every place where the picture knows buildings of his.
- *Places:* every place the packet, a list or a mark names, for every actor that can walk there, each both ways
  (go, fight to); and a shooter's estimated place for the groups it is hitting.
- *Ours:* every other group of its kind (join), every group and builder (follow), every factory and build under
  way (help).
- *Types:* the `produce` allowance, else the roster's standing menu, as today.

**The words.** One function per verb builds a move's line from the same parts in the same order: the move; the way
(distance, seconds, which soldiers for a detachment); what is known at its end (his buildings, a party, when last
seen); how it stands to each party in the actor's entry ("stepping back from party_7"); what it leaves. A fight's
facts are said once and repeated on every move that keeps it or leaves it, so `stay` and `go` carry the same facts:
the odds for the part of the group in reach, what each side lost in the last seconds, whether the party can follow.
Clocks ("for 3 s") live in the actor's entry, not in the move, and distances and times are said in word buckets
("far", "about 20 s") with the number beside them only where it decides something, so that a move's words stay the
same from second to second (§8's `same` layer lives on that; K-jev-words-not-numbers).

## 4. The pass

The form stays: the gate asks nouls; the best-rated moves are joined into worlds, each with its consequence line; the
pick is two questions. The composition is written anew for one slot an actor. What it no longer has: `resolve`,
`taken`, the dropped party states, the pair worlds, the no-op "unreachable" state, the told noul.

- **Openers** as today: `<actor>.change` and `<party>.answer`. An actor's moves are candidates when it is idle or
  its `change` is 0.5 or over. A move aimed at a party is also a candidate when that party's `answer` is 0.5 or
  over. A party is a question, never a menu.
  Measured (§9): in the 36 seconds a group was sent home from a fight, its own `change` was 0.5 or over each of the
  27 times it was asked (median 0.86). The weak side of K-jev-a-response-opens-by-the-party-noul-not-its-own was a
  builder at work asked about a raider; a group in a losing fight opens from its own side.
- **Moves.** One noul per move. A move aimed at a party is asked as an answer to that party, as today; every other
  move is asked of the actor ("is this what group_A should do now, rather than ..."), in the long form. The two
  best per actor by their own noul go on.
- **World 1** is every actor staying. For an idle actor its line names the move Jev's own gate rated best for it:
  "group_A holds at spot_49, a stop it has reached, with its next move, the advance to spot_46, not ordered". Code
  no longer finds a "next stop" in the packet's order.
- **The registers** (the user's idea of 2026-09-28). In each actor's entry, kept by code as facts: the places it
  reached and what it met there (as today), and its last pick: "14:02 stepped back from party_7 to spot_30, 6 s
  ago". A move that undoes the last pick says so on its line.
- **The forbidden mark** (the user's: 0.7, hunts only) moves unchanged onto `send N`.
- **When things are asked.** As ruled 2026-09-30: an actor with a course when it has news or after 20 s; an actor
  with no course, and every party, at every gate. A hold that Jev picked is a course. News for an actor: its
  course ended, it was hit or lost a soldier, an event names it, the packet or a list changed, a party came or
  went in its entry, the store crossed empty or full. The exception for a group on a route leg goes.
- **Failure.** A gate that failed is asked again at the next second (today: up to 20 s later).

**Two things at once.** One move per actor a second means a body between two raids answers one this second and the
other the next (parties are asked at every gate). It is so today as well: a world that sends one group at two
parties is dropped.

**Thrash.** Nothing in code holds a move against the next pick. What stands against flipping every second: a change
must beat "nothing changes" in the pick's second question; the facts of a fight are the same on both sides of the
choice; the register says what the group chose six seconds ago and a reversal is called one. If it still thrashes,
the fix is in the words, not a hold in code.

## 5. What the player writes and sees

The packet stays prose and no tool is added. The role text gets the verb table as its vocabulary and one new
instruction: every group's paragraph says where it goes when it should not fight. The report gains, per group, its
register line (what it last chose and when), the two moves the gate last rated best for it, and, once the decode of
§8 is on, what the decode read for it ("group_L5 reads: spot_27, spot_18, spot_29; NO place to step back to"), so a
paragraph that is not being read as meant shows within a turn.

`rules.md` is rewritten in the new verbs, and in the base version it keeps its judgments ("a party within 600 of a
group that outweighs it is fought now, whatever the instructions call it", the fall-back law, the commander's
retreat law). They belong in the brief, where the player owns them (K-hands-menu-verdicts-beat-the-packet), but the
text steers (K-jev-the-rules-text-steers: without it one strong move in five falls under the bar), so cutting it in
the same game as the new menu would hide which of the two changed the play. It is cut as its own step afterwards.

## 6. What acts without a pick

The cut-line file disposes of all 88 rows. In short: the lists, the factory sequences, the newcomer's walk, the
construction turrets' guard, the resurrection crews, the rove and the micro lane stay by the user's rulings. The
ends of orders stay as ends (an arrival, a target gone). Some rows are marked "to rule". The builders' walk home
and the code-chosen step-away go, as the groups' did: a builder's ways out are `go` to any named place and `follow`
any group. Each course's footwork in the micro lane is unchanged and is code's:

| Course | Footwork |
|---|---|
| go | flees everything |
| fight to | commits; marches in ranks |
| attack | priced; his commander only above 450 metal |
| hold, follow | priced, keeps out of turret reach |
| send N (a hunt) | raw |

## 7. The menu against twelve situations

| # | Situation | The move | Open point |
|---|---|---|---|
| 1 | An army meets a stronger force; the packet names a place to step back to | `go` there: through the gate in 35 of 36 recorded seconds, picked in 32 of 38 | then holding there until the rest arrive is prose and facts; not replayed |
| 2 | The same, no place named | `follow` or `join` another group: through the gate in 17 of 36. A walk to a place nobody gave it: 4 of 36 | weak by design: the place is the player's to name |
| 3 | A lone raider kills an extractor far from every group | `send 1, 2, 4` from every group; the commander's `attack` | the split of the pick's mass across sizes is not replayed |
| 4 | Two raids on opposite flanks, one army between | one this second, one the next | - |
| 5 | A group on a route sees his commander alone | `attack`; the group's own `change` opens it on the sighting | after the fight, world 1 names its best-rated move, at any place |
| 6 | Artillery in a mixed group, a turret line that outranges the rest | `shell` aimed at the place where his turrets are known | guns and screen standing apart is footwork: the micro engine's |
| 7 | Ten fragments that should be one body | `join` to every group, several a second in one joint world | A joins B while B joins A is possible |
| 8 | A constructor far out, a raider coming | `go` to any named place, `follow` any group | not replayed for builders |
| 9 | The commander at home, a party it cannot beat walking in | `follow` a group, `go` to a named place, D-gun | not replayed |
| 10 | An air group | attack on a named unit or on his buildings at a place | no patrol |
| 11 | A group guards a builder moving from spot to spot | `follow` | - |
| 12 | "Hold at spot_30 until group_B arrives, then go together" | hold, then go: prose and the picture's facts | a condition stated one way has failed before; needs a replay |

Not covered by any verb: fire on one unit inside a party from the hands, choosing soldiers by type, a patrol, stances.

## 8. The base version and its layers

The user, 2026-10-01: "I would lean towards building this as a full version we can run once or twice, then with
layered optimizations we can disable if we suspect them to be at fault. That's where I see decompression working
well: using a decode of the opus's existing orders (not a second tool call) to fuse off jev questions that we can
answer once for the entire span." And on the price: "we really should be aiming for $1 per game or less."

**The base version** is §3's menu asked in full: every move of every actor and every party at every gate, the joint
pick in its two calls, nothing skipped. It is the reference every saving is checked against. Its first games run
with one layer on, `news`: that rule has four games behind it, and without it the bill is about two and a half
times higher again (player-29 $5.07 against player-30 $1.98). So run, the base is counted at about $3 for a game
like player-33 and $5 to $7 for 40 minutes, with late gates of 0.7 s or more. It is run once or twice in lockstep,
to see whether the menu itself plays right.

**A layer** is a saving with a switch. Three laws:

- A layer changes which questions are sent or when, never the menu, the words or what a move does.
- A layer audits itself: for a small share of what it skips (2% to begin with) it asks anyway, in the same
  request, and logs what it assumed beside what came back.
- **The fuse rule.** A fuse answers only a question about the instructions alone, never one about the picture,
  and only when the decode's answer is clear.

| Layer | What it does not send | What makes it safe | Its audit | Counted (budget study) |
|---|---|---|---|---|
| `news` | questions of an actor with a course and no news (the rule of 2026-09-30, in every game since player-30) | the actor is asked again on any event that names it, and after 20 s | a share of quiet actors asked anyway | in the base's figure already |
| `same` | a question whose words are those of its last ask, under 20 s old | re-asked as recorded, an unchanged question's answer crosses 0.5 in 0.1 to 0.7% | a share re-sent | 5% of the bill; about 9% with word buckets |
| `fuse` | a move the decode clearly rules out for that actor, and the forbidden question beside each detachment | the fuse rule | a share of fused-off moves asked anyway: a fault is one rated 0.5 or over | walks fall by about 70%; forbidden goes |
| `openers` | an actor's moves while its own `change` says no; a party's while its `answer` says no | the opener is asked every time; the moves follow one second later | a share of closed actors' moves asked anyway | 16% |
| `split` | the joint worlds: each component of the pick is asked its own two stages, all in one request | components share no actor, group, builder or party | on a share of seconds the joint pick is asked too and logged beside it | the picks at 58% of their tokens: 16% of the bill; one call less a second |
| `tick` | a gate with no event, until 2 s after the last one | events still fire at once | none: it only delays | 4% |
| `one` | the pick's own request: it rides with the next second's gate | the picture is at most one second newer than the worlds' lines | on a share of seconds the pick is asked at once as well | the picks' picture: about a seventh |
| `local` | the parts of the picture a request's questions do not touch | to be shown: a thinner picture moved the pick's first stage in 4 of 10 (budget study §3) | a share asked with the whole picture | needed for team games, not for 1v1 |

**The decode.** When a packet lands, and when an actor appears, one lean request asks Jev about the packet alone:
for each actor and each place the packet names, whether the instructions send it there; for each group, whether
they forbid it to send detachments, which groups they tell it to join, whom to follow, and whether they say where
it goes when it should not fight; for each builder off a list, which buildings and spots are its. Answers under 0.2
fuse the matching move off until the next packet; answers of 0.8 or over on the forbidden question put the mark on
the line without asking each second; everything between is asked each second as in the base. Measured on two games:
72% of place answers are under 0.2; a place in another actor's paragraph reads as this actor's in 1 to 2%; a
reading crosses 0.5 between two packets that did not change its sentence in 3.4%. It is no tool call and no new
syntax: it reads the packet the player already wrote. The player's report shows it (§5).

**Switches and the log.** `--hands-layers news,same,fuse,...` on the arena (an environment variable for the bot),
written into `batch.json` and the Jev log's header. Each pass row counts what each layer skipped; each audit is a
row of its own. `run/layers_read.py` prints, per layer, the skipped, the audited and the faults.

**Built into the base, so that the layers can exist:** the words in buckets (§3); one function that says which
picture goes with a set of questions (the whole picture in the base; `local` swaps it); one function that says
which questions are due (everything in the base; `news`, `same`, `fuse`, `openers` and `tick` each take some away).

**Order.** The base with `news`, one or two games. Then `same` (exact). Then `fuse`. Then `openers` and `split`. Then `tick` and `one`. Each step is one game with its audit read before the next is switched on. With all
but `local` the budget study models a game like player-33 at $0.75 to $0.95 and a 40-minute game at $1.4 to $2.0.
Under $1.2 for 40 minutes also needs "who answers this party" asked as one Choice in place of a noul for every
group and size (budget study §5): a change of question form, to be replayed before it is proposed as a layer. All
of these are models until the base has run.

## 9. Evidence so far

Offline, on recorded seconds; under $0.50 of Jev in all. Scripts in `run/`, data in `docs/studies/data/`.

| Bet | Test | Result |
|---|---|---|
| A step back to a named place gets through the gate | 36 of player-33's walks home; a sentence naming the place added to the packet; the walk asked as a noul and set beside the group's recorded moves (`jev_fallback_gate_ab.py`) | 0.5 or over and among the group's two best in 35 of 36 (median 0.73); asked as an answer to the party, 32 |
| ... and wins the pick | the 38 picks with the walk home taken out and that walk added (`jev_fallback_ab.py`) | the top change in 32 of 38; beats "nothing changes" in 37 |
| ... and a winning fight is kept | 38 seconds where a group attacked a party it outweighed, same sentence, same walk | through the gate in 10 of 38; picked in 2 |
| With no place named, a group can still step back | the same 36 seconds, the packet as it was | onto our largest other group: through the gate in 17. To a place nobody gave it: 4 |
| A route keeps its momentum without code reading its order | player-9's 24 arrivals, two-stage pick (`route_ab.py`) | code's "next stop" in world 1: 24 of 24. No cost line: 7. An unordered list of unreached places: 10. Jev's own best-rated move named there: 24; with a fall-back place named beside the route: 24 |
| A question whose words did not change gets the same answer | player-32 and player-33, every repeated question | median change 0.01 to 0.04 by kind |
| The pick can be asked in parts | 120 recorded picks split into components, one request (`jev_split_ab.py`) | 9,700 tokens a pick against 16,700; the same best change as the joint pick's most-weighted in 96 of 133; 1.58 actors changed a pick against 1.23 |
| A thinner state is not free | 50 gates and 120 picks asked again (`jev_trim_ab.py`) | the rules text out: a strong move still 0.5 or over in 53 of 72 (66 re-asked unchanged); the pick's picture cut to the actors named: stage one the same choice in 36 of 60 (53) |
| Jev can read which places are whose (the decode) | player-33's 58 packets, 11,935 questions (`jev_read_ab.py`); player-32's 117 packets read alike | in a paragraph headed by the actor's name: 579 of 804 read as its places, the rest nearly all "never past" and lists of ground it answers raids on. Elsewhere in the packet: 34 of 3,375. Between two packets that did not change the sentence: 3.4% cross the bar. Not counted: the groups no paragraph names (20 of 53 in a sample) |

Limits. One game for most rows, two for the cost. The fall-back replays use a place I chose by a rule and a line I
wrote; the pick replay adds one world to a recorded pick and does not rebuild the menu around it. The first wording
of the best-move line scored 16 of 24, so the wording carries much of that result.

## 10. Before code, the order of work, and what success is

Still to replay, cents each: a raid answered from the ladder against today's hunters, with the mass across sizes;
a builder's way out; situation 12; the decode's other questions (forbidden, join, follow, the step-back place).

Order: this note and the cut line ruled on; delete (`threats.rs`, the menus and the composition in `plan.rs`, the
group and party arms of `execute.rs`, the prose readers in `diet.rs` and `groups.rs`); rebuild the base (`menu.rs`:
the verbs, what they are aimed at, the words; `compose.rs`; the registers; the two functions of §8 with nothing
taken away); unit tests; `rules.md` and `default.md` in the new verbs, the role text's vocabulary; the registry and
the claims (rows retired, the new heuristic and what it rests on); the brief's lessons that name deleted options
tagged as from the old harness; a pianist-alone game on a fixed packet; one player game on the target with the
base; then the layers in §8's order, each with its audit; a realtime rehearsal once `split` and `one` are on.

Success, read from that game's log against player-33: no walk home that the packet did not order; a named
fall-back place taken in the seconds the odds are against the group; a group's course changes at most 4 a minute
in its worst minute (player-9 at E3: 13 in 53 s); no builder lost with no way out on its menu. For each layer: its
audit's faults under 2% of what it skipped, and the game's bill moving as the budget study counts. At the end: $1 a
game or less.

## 11. Kept, and why

| Kept | Because |
|---|---|
| The gate of nouls before the worlds | the user's idea, 2026-09-25 |
| Joined worlds to 255, with consequence lines | the user's design, restored 2026-09-29 |
| The pick in two stages, the first sampled | the user's ruling, 2026-09-29 |
| The bars 0.5 and 0.3 on Jev's own answers | the cut line of 2026-09-28 |
| The forbidden mark, 0.7, hunts only | the user's word, 2026-09-30 |
| Ask on change | approved 2026-09-30 |
| A hunt is a group of its own | 2026-09-29 |
| Lists, their ids, factory sequences | the user's designs |
| Two moves an actor go on; events at most every 5 s; 40,000 characters of lines; the question forms | they exist and are measured, never ruled: the user's to strike |

**Reversed, by name:** the party menus (threat response, agreed 2026-09-26); a group on a route leg not asked on
party news (routes in prose §4.4, 2026-09-28); code finding a route's next stop from the packet's order (same note,
§4.2), replaced by Jev's own rating; the builders' walk home. **Brought back, by the user's word of 2026-10-01:**
the packet decoded once by Jev (deleted 2026-09-27), as fuses under the fuse rule. **Later, as its own step:**
`rules.md` as the hands' voice on how to play.

## 12. Questions for the user

1. Detachment sizes from a fixed ladder for Jev to pick, or the code's "fewest that outweigh it" as today?
2. `send N` only at a party, or also to a place (pickets from soldiers that already exist)? It multiplies a
   group's place questions.
3. Movement to places nobody named. As designed there is none: a group whose packet names nothing can hold, join
   or follow. Is that what you mean by basic movement, or should every spot we hold be a place to go?
4. The "to rule" rows of the cut line.

Answered 2026-10-01: the price ($1 a game or less); the decode (in, as fuses); how to build (the base first, then
layers that can be switched off).

## 13. How the draft changed under review

Reviewers: a subagent given every recorded ruling and told to object as the user would; an Opus agent as the player
who would write packets for it; my own pass against the inventory. Replays as in §9. The two reviews are kept in
`docs/studies/2026-10-01-hands-rebuild-reviews/`; both read draft 1.

| Objection | From | Change |
|---|---|---|
| Matching names in "the group's paragraph" is code reading prose, and wrong for 3 groups in 4 | own pass | draft 2: Jev reads which places are whose |
| That reading is the deleted decompression used as a prune; 20 of 53 groups are named nowhere | owner review | draft 4: no per-actor filter at all (law 4); the reading demoted to a priced option (brought back in draft 5 as the `fuse` layer) |
| Without code's "next stop" a route loses its momentum | replay: 10 of 24 | world 1 names Jev's own best-rated move: 24 of 24 |
| The ways out open by the weak question and are crowded out by two attack moves | owner review | replayed at the gate: a group in a losing fight opens from its own side 27 of 27; the step back is among its two best 35 of 36 |
| Movement only where the player named a place: with none, `join` is the new walk home | owner review | `follow` any group; measured 17 of 36; put to the user as question 3 |
| Nothing on state or thrash, and the one damper removed | owner review, player review | the registers and a reversal said on the line (law 8); a pass line of 4 changes a minute |
| Parties under a 20 s rule was a new rule called an old one | owner review | parties asked at every gate, as ruled |
| The cost table mixed asking rules and gave no dollars or time | owner review | §8 recounted under one rule, in dollars and requests |
| "Ends stay as ends" hid code decisions nobody ruled on | own pass, owner review | the cut-line file, 88 rows |
| Builders recommended in and not designed; their way out vanished silently | owner review | builder rows written; their way out stated and listed for replay |
| Raids on his known buildings and shelling a turret line had no target | owner review, player review | `attack` and `shell` aimed at a place where his buildings are known |
| A group being ground down has no news | player review | hits and losses are news |
| `stay` and `go` must carry the same facts | player review | §3, the words |
| A group born after the packet had no places | player review | moot: no per-actor filter |
| No success line; realtime and the docs missing from the order | owner review | §10 |
| "A group has no plain go there" was wrong | owner review's fact check | §1 corrected: the walks exist and rate a median 0.1 |
| The menu as the laws have it doubles the bill; the target is $1 a game | the user, 2026-10-01; the budget study | draft 5: §8 |
| Savings baked into the design cannot be told apart when a game goes wrong | the user, 2026-10-01 | draft 5: a base version that asks everything, and layers with a switch and an audit each |
| The decode was demoted to an option in draft 4 | the user, 2026-10-01: the principle was not bad; a decode of the existing orders, not a second tool call | draft 5: the decode is the `fuse` layer, held to questions about the instructions alone and to clear answers |
| Cutting the rules text in the same game as the new menu confounds the two | the budget study: the text steers | draft 5: the base keeps its judgments; the cut is a later step |

## 14. The build: what was taken as decided, and what the code needed that the note did not say

The user approved the note without answering §12, so each question is built as the note's body has it, and each is
one function to change:

1. Detachments come from the ladder (1, 2, 4, 8 below the group's size), as §3 has it.
2. `send N` is aimed at a party only.
3. No movement to a place nobody named.
4. The cut line's "to rule" rows are built as proposed there, with one exception: an extractor "beside itself" is the
   free spot its builder reaches soonest (what a bare `extractor` list step already means), so a builder whose
   packet names no spot can still take metal. The nearest-two and the safe spot go.

Not done before code, against §10: the four small replays (the ladder's mass across sizes, a builder's way out,
situation 12, the decode's other questions). They need the new menu's own lines, so they are run on the first
game's log instead of on lines written by hand.

What the code needed:

- **The course is `stay`.** A move that is the actor's course in force is not listed a second time; world 1 is
  every actor at `stay`. Nothing is "current" and nothing is settled between menus.
- **Which moves go to the pick** (one sentence, as a law): a move goes to the pick when Jev rated it 0.5 or over
  and its opener is open (the actor's `change`, or the `answer` of the party it is aimed at); an idle actor's best
  move goes at 0.3 when none reaches 0.5; and a party Jev says needs answering puts forward the two best-rated
  moves aimed at it whatever their rating. The last clause is today's rule for a party's answers
  (K-jev-a-response-opens-by-the-party-noul-not-its-own: an answer's own noul sits under the bar while the party's
  is high), kept. Of an actor's moves the two best go on.
- **"Left to nobody"** in a world's line lists only the parties whose `answer` Jev rated 0.5 or over: with every
  party a question, his army at home is not our neglect.
- **`fight to` a place where his buildings are known is worded as the attack on them** ("attacks his buildings at
  spot_9: ..."). It is one order with one footwork; two moves for it would be near-synonyms.
- **No-ops are left out as things the unit cannot do**: `go` to where it stands, `hold` for a group already holding
  by a pick, `join` with itself. `gather` is always listed for a group of two or more; its words say how far the tail is.
- **Which buildings go at which places** (corrected after rebuild-smoke-1, 2026-10-02). §3's "build a type, at a
  place or beside itself" was first built as every type at every named place: a constructor's menu was 507 moves,
  93% of the gate's questions in the game's first five minutes were a build at a place, the bill ran at $0.28 a game
  minute, and a builder's two best moves were often one building at two places. As built now: every type where the
  base layout has it (the move's words say where); an extractor at every free named spot and at the free spot the
  builder reaches soonest; a tier-2 extractor over ours at a named spot; a type that stands at a place (a defence,
  a radar, a jammer, a sonar) at every named place; and any type at every mark the player made. A solar or a
  factory at a metal spot is ordered by a mark or by a list step, as today. This is a bound on the menu and so the
  user's to strike.
- **A party's answers are asked at every gate, whatever the `news` layer closed** (corrected after
  rebuild-smoke-1). §4 says every party is asked at every gate; under the rule of 2026-09-30 that meant the party
  and the states against it, since a threat slot was never closed. As first built here the `news` layer closed a
  holding group whole, its moves aimed at a party with it: at 4:39-4:47 a Pawn shot a constructor 300 from a
  holding group_A for eight seconds, the party's `answer` at 0.7 to 0.8 every second, and no answer of the group
  was a question. Now the layer skips an actor's `change` and its own moves only; its moves aimed at a party are
  asked at every gate and go to the pick by the party's `answer`. Skipping them while the party's answer says no is
  the `openers` layer's saving, with its audit.
- **The D-gun is a move only at a party with a unit inside its reach.** Beyond it the commander cannot fire it,
  and walking in is `attack`. (rebuild-smoke-1: the shot was picked five times at parties 900 away under words
  that could not say what the engine would then do.) The shot ends the builder's task, since it takes the place
  of its order in the engine.
- **Named places** are: every mark; every spot, passage or `home` the packet's text names; every spot a list step
  names. The picture gains a place for every spot with buildings of his within 500, so that `attack` and `shell`
  can be aimed at all of them.
- **A listed builder** has no menu unless it is threatened (hit in the last 3 s, or a party within 800), as today;
  then only its ways out. A pick's way out holds its list while the threat lasts (today: walks only; now `go`,
  `follow` and `attack`).
- **A builder's order the engine dropped** (the unit idle, its task not done, the order's grace over) ends at once
  and is said; the 40 s and 10 s clocks go.
- **The picture sent** is today's (the diet level, `--hands-effort`, and the pick's cut state): they have a switch
  already and are not counted among §8's layers.
- **Built first:** the base and the `news` layer with its switch and audit. The other layers follow §8's order, each
  after the game before it is read. Naming a layer that is not built yet stops the bot at start.
