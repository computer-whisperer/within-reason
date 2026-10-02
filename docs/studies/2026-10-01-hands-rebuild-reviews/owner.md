# Review of `docs/design/2026-10-01-hands-rebuild.md` (draft 1), from the owner's side

Reviewer: a read-only subagent (Fable 5.1), 2026-10-01 about 22:40. Nothing in the repo was edited; no game, build or
model call was run. What I ran: the `--read` halves of `run/jev_read_ab.py` and `run/jev_fallback_ab.py`, `run/menu_size.py`
on player-33, and three short counting scripts over files already on disk (`docs/studies/data/*2026-10-01*`,
player-33's `jev-0.jsonl`). Where I say "on disk" I mean those files as they stood at 22:38; the main session was still
writing route replays (`route-ab-2026-10-01-two-stage.jsonl` appeared while I read), so those numbers can move.

Sources read: the draft and the inventory in full; the owner's messages in the brief; the memory directory (MEMORY.md
and 30 files, every one on the brief's list); the five design notes; `jev_experiments/REPORT.md` and `results/j.md`;
the ten named claims in `docs/knowledge/jev.md` plus about twenty neighbours; `plan.rs` `walk_places` and
`picture.rs` `enemy_parties` for two fact checks.

Verdict in one paragraph: the verb table (§3) is the right kind of answer to "why is the only action space available
for a group not cover basic movement actions?", and deleting the party-side menus answers the letter of the second
message. But the draft does not yet answer either message in substance. Movement exists only toward places Opus
happened to name for that actor, as read once by a Jev call the owner has already deleted once; the two directions
survive in the openers, where it matters (a fight opens by the strong question, leaving a fight by the weak one);
two mechanisms the owner agreed on 2026-09-28 are reversed without being called reversals, and one of those reversals
is already failing its own offline check in the data on disk; and the thing that has lost four games running (the
group flipping course every few seconds at a nest) is not mentioned. I would not expect approval of draft 1.

---------------------------------------------------------------------------------------------------------------------

## A. Objections he would raise

Severity: **blocks** = he would not say "agreed" with this in; **answer** = needs a stated answer before approval;
**minor**.

### A1. §5 "the reading" is the decompression stage again, used as a prune. **Blocks.**

- *The part.* §5, law 5, and "Follow: the builders the packet gives the group (§5)".
- *The objection.* One Jev call per packet, one question per (actor, place) over a fixed four-word vocabulary ("a stop
  of its route, its post, its target, or where it falls back to"), a 0.5 bar in code, and the result removes moves
  from menus until the next packet. That is the shape deleted on 2026-09-27, and its named harms apply word for word:
  the reading is frozen at packet time while the condition it read is live; a conditional sentence loses its
  condition; a sentence about a class of actors has to land on someone.
- *What it rests on.*
  - `docs/knowledge/jev.md`, K-jev-a-packet-decompresses-to-standing-orders, status: it "hurt through ... pruning read
    from another actor's sentence or with its condition dropped ... one-seat packets read into every seat ... The
    literalness was the problem as much as the strength: routes, conditions and 'every group' had no rule to land in,
    and a question that could not take them was answered with what it could."
  - `docs/design/2026-09-28-routes-in-prose.md` §5, on his register idea: "What this avoids: a transition table the
    player must write and the hands must parse, which is the decompression stage again (deleted 2026-09-27 for the
    harm it did)."
  - `docs/design/2026-09-27-posing-changes.md` 7.5: the sentence classification "is ~~STRUCK~~ (2026-09-27, the user:
    empower Jev and the player; no rigid rule where another change resolves the cause): it needs a reader of prose
    that 13b deletes."
  - `memory/no-live-heuristics-ruling.md`: "Opus doesn't have unlimited context, and doesn't have the thinking time to
    reverse engineer complex effects we baked into our harness mid-game. The harness should be complete, but
    straightforward, and leave the real gameplay to jev and the player." A per-actor place list Opus must audit every
    turn, and re-word its prose against until a place "registers", is exactly a baked-in effect to reverse-engineer.
- *What the draft's own data says (my counts on `read-places-business-2026-10-01.jsonl`).* The headline "160 of 218"
  and "5 of 845" cover only actors that have a paragraph headed by their own name, which is the case today's code
  already gets right. They are 1,063 of the 2,939 answers. The other 1,876 (64%) are not reported at all. Of the 53
  group-packets in the sample, 20 have an own-headed paragraph, 30 are named somewhere in a heading, and **20 are
  named nowhere in the packet** (fragments, detachments, groups born since the last turn). Under the reading 13 of the
  53 groups get no place at all, so no `go` and no `fight to`. The own-paragraph nouls are soft, not sharp: median
  0.64, with 45 of 218 at 0.3-0.5 and 64 at 0.5-0.7, so the 0.5 bar cuts through the middle of the distribution. "The
  58 not read are mostly right not to be" is asserted, not counted. And player-33's packets "named no place for a
  group to go to when it declines a fight" (K-hands-the-only-way-to-decline-a-fight-was-to-walk-home), so the reading
  of a fall-back sentence, the one case this rebuild exists for, has not been measured once.
- *Both readings of what he would say.* (i) Against: the above. (ii) For: nothing plays from it, the bar is on Jev's
  own answer (the 09-28 cut line kept FLAG "as the cost of asking, not as a decision"), and the player sees it. I
  think (i), because his own precedent for carrying a Jev reading of the packet into the pick is the forbidden mark.
  His law of 09-30, as quoted in `docs/heuristics.md` (H-HANDS-ONE-PASS): "For each hunt on offer, Jev is asked
  whether the player's instructions forbid it for that group, and a hunt it rates forbidden says so on its line in
  the pick"; the row adds, in the author's words, "nothing is pruned, both stages of the pick read the line". A
  reading that removes is the other thing.
- *The change that meets it.* Drop the reading as a filter. Places the packet, a list or a mark names are every
  actor's (the draft's own alternative), and if a per-actor reading is kept at all it is a sentence on the move's
  line, as the forbidden mark is. Then pay for the questions honestly (A13).

### A2. Movement still exists only where Opus named a place: message (a) is not answered. **Blocks** (on one reading).

- *The part.* Law 4 ("A place comes from the packet"), §3 Objects.
- *The objection.* The old harness had one way out with the base wired in. The draft has as many ways out as Opus
  named for that actor, as read at packet time, and for a group the packet does not name, none. With no place on its
  menu the only moves that take a group out of a fight are `hold` (it stops where it stands) and `join` (it walks to
  another group): `join` becomes the new walk home. A person with units selected can move them anywhere; the draft's
  own law 3 says the verbs are "the orders a person has with that unit selected" and then gives `go` an object set a
  person does not have.
- *What it rests on.* "why is the only action space available for a group not cover basic movement actions?"; "Jev
  clearly is the one calling the retreat now, and if its only option is to retreat all the way home then that is a
  mistake in the hands harness."; "The harness should be complete, but straightforward".
- *Both readings.* (i) He means the places are Opus's and the timing Jev's ("we should remove the home-wired answers
  and instruct opus to routinely communicate a fallback point"; `memory/no-code-chosen-retreat-destinations.md`:
  "Offer destinations only from what the player named", which is the assistant's note, not his words). Then the
  draft's object set is right in kind, but every place named anywhere must be on every mover's menu, or the 20-of-53
  case has nothing. (ii) He means movement is basic and always there. Then the object set must not depend on the
  packet at all. I cannot tell which; the draft should put the question to him in those words instead of settling it
  inside law 4.
- *The change.* At least: no per-actor filter (A1). Better: our own groups, builders and held places are `go`
  objects too ("go to group_B" is falling back onto reinforcements; the object is Jev's choice among things the
  picture already names, not a destination the code computed). See D.

### A3. The two directions survive in the openers, and the survivor is biased toward fighting. **Blocks.**

- *The part.* §4 Openers: "A move aimed at a party is also a candidate when the party's `answer` is 0.5 or over,
  whatever the actor's `change`."
- *The objection.* Message (b) asked why one case goes from our units out and the other from the enemy back. The
  draft removes the party's menu and keeps the party's opener for attacks only. So `attack`, `send N at` and `shell`
  open by the question measured at 0.71-0.80, and `go`, `hold` and `join` (every way to decline) open only by the
  actor's own `change`, the side measured at 0.31-0.43 and "never opened". That is the old asymmetry with the sign
  flipped: before, the only strong-side way out was the walk home; now there is no strong-side way out.
- *What it rests on.* K-jev-a-response-opens-by-the-party-noul-not-its-own. K-hands-the-only-way-to-decline: the
  group's own ways back were "asked and left": a walk to home 799 times at a median 0.11, never played; "falls back
  to where it last held" 263 at 0.20, never. In player-33 a group's walks and advances were asked about 75,000 times
  and played 65 (my count from the log).
- *It gets worse in composition.* "The two best per actor go on." With every party on every armed actor's menu and a
  ladder of four sizes, a group's two places will usually both be party moves ranked by the party's answer (often two
  sizes at one party). The step back does not reach the pick. Same symptom as `taken` (0 of 14), new mechanism.
- *The offline check does not cover it.* `run/jev_fallback_ab.py` (data on disk, written after the draft) adds the
  walk to the named place straight into the pick as a world, with step-back words written for the purpose. It never
  asks the gate. Its result is good as far as it goes (the named place was stage one's top world in 32 of 38
  fall-backs and beat "nothing changes" in 37; in 38 winning fights it was top in 2), but it is an upper bound on a
  pipeline whose weak link it skips.
- *The change.* One opener rule: a party whose answer opens puts the whole menu of the actors it concerns on the
  table, the ways out included, and declining is worded as an answer to the party ("steps back from party_7 to ...").
  Replay the 38 picks through gate, depth and composition before any code.

### A4. World 1 without "next stop" reverses the 09-28 route design, unnamed, and the data on disk says it fails. **Blocks.**

- *The part.* §4: "the places read for it that it has not been to since (no 'next stop': the order is in the prose)";
  check 3 listed as still to do.
- *The objection.* The stop's cost naming the next stop is §4.2 of `2026-09-28-routes-in-prose.md`, agreed with him,
  and the measured reason routes work: the next leg picked 24 of 24 with the sentence, 2 of 24 without. The draft
  writes its removal into §4 as settled.
- *The data* (`docs/studies/data/route-ab-2026-10-01*.jsonl`, 24 arrivals each, my tally of `picked`): with the
  next-stop sentence (E) 24 of 24; with the draft's unordered list (F) **10 of 24**; with a fall-back place also in
  the prose and in that list (G) **8 of 24**; naming Jev's own best-rated move instead (H) 15-16 of 24; no cost
  sentence at all (B) 7 of 24. And at the 3:58 fork, G sent the scout to `fall_back` 4 of 4 where E and F kept it on
  its route 4 of 4. The draft's line recovers 3 of the 17 legs the sentence was worth.
- *What it rests on.* "Jev needs to be able to react quickly when a group reaches the destination so it can keep
  momentum moving to the next location." (`routes-in-prose.md` §1). And `memory/one-pass-arc.md`: "a note the user
  approved in outline does not license reversing a design decision of theirs inside it; name reversals as reversals."
- *A second fault in the same line.* The reading lumps the fall-back place with the route's stops (one noul for four
  roles), so world 1 tells Jev every second that the group "has not been to" its fall-back place. The claim file
  predicted this ("world 1 would say 'the route's next stop <that place> not ordered'") and the draft walks into it.
- *The change.* Keep the next stop said in world 1 (code recognising names in order, as agreed), or have Jev name the
  next stop at each arrival and print it back as a register (D). Either way, report F/G/H in the note.

### A5. Nothing about state or thrash, and the one anti-thrash rule is deleted. **Blocks.**

- *The part.* §4 "The exception for a group on a route leg goes"; the absence of any register.
- *The objection.* Every lost lead in the last week has the same picture: standing-2 25 flips on one group; E3 "13
  task changes in 53 s"; player-31 "the body went back and forth at D6"; player-32 the ball's course changed 18 times
  in two minutes at the nest; player-33's own note "hands flip-flopping attack/fall back" and six orders in 30 s. The
  draft adds movement options to that loop and takes away its only damper. A group in a fight has party news every
  second, so it is asked every second; a holding group is "idle", so it is asked every second; stage one of the pick
  is sampled. No hysteresis is allowed in code (rightly), so the only remedy is words: what this group last chose,
  when, and why. The draft has none.
- *What it rests on.* "Thirdly, jev needs to be able to track some kind of state on its own allowing it to follow
  sequences of orders with a possible fork in the middle."; "Some kind of register set for jev. The stateless hands
  are definitely smart enough to drive a state machine, we just need some way to have opus's prose translate into a
  state machine that jev will instantly understand how to drive." The route-leg exception is §4.4 of the agreed note:
  "This is the one departure from one-pass's 'every actor every second', and it is what stops the E3 flip-flop."
- *The change.* Registers in the actor's entry and on world 1's line: the last pick with its clock and its object
  ("14:02 stepped back from party_7 to spot_30 (Jev), 6 s ago"), reached and met as today, and on a move's line
  "reverses the pick of 6 s ago". A picked `hold` is a course, not idleness. A flips-per-minute figure in the checks.

### A6. Parties put under the 20-second rule: a new rule presented as "the rule of 2026-09-30". **Answer.**

- *The part.* §4 "When things are asked ... made the same for everything".
- *The objection.* His law of 09-30 asks "an idle actor and every threat ... whenever the gate goes"
  (H-HANDS-ONE-PASS; `plan.rs` `settle`). The draft closes a party until it "appeared, moved to another place, began
  or stopped killing something of ours, its count changed". With stage one sampled, a raid whose answer did not win
  the draw is not put again for up to 20 s. The saving is "a share not yet measured".
- *What it rests on.* "we need the hands system to be able to coherently react to threats quickly"; "see it through
  at high speed" (`threat-response.md`).
- *The change.* A party stays open while its answer is at the bar and nothing answers it. Measure the share first.

### A7. "One move per actor" cannot answer two things at once. **Answer.**

- *The part.* Law 2.
- *The objection.* A body between two raids can send a detachment at one of them per pick. The joint worlds were
  restored on 09-29 for this case in so many words: "a raid on two flanks needs two changes in one second"
  (`one-pass.md` §4). The party-side menus were how one group gave soldiers to two parties while its body kept its
  course; the draft deletes that and says nothing replaces it.
- *What it rests on.* Jev must "decide what set of units to send after it (splitting off of groups as needed) in
  combination with all the other necessary activites".
- *The change.* The body's move and its detachments are separate slots of one actor (a world may hold one body move
  and several `send`s whose sizes fit), or say plainly that it takes two picks and guarantee the second (A6).

### A8. "Code never chooses the object" holds by moving the choices out of sight. **Answer.**

- *The part.* Law 4, §3, §6.
- *The objection.* Still chosen by code and not said: which N soldiers go on `send N at`; the scout ("its fastest
  soldier"); whether one scout at a time survives; who counts as artillery (600) and where the standoff and the
  screen stand (0.85 and 0.55 of the reach); where `hold` gathers ("the front stands", which at a nest is under his
  turrets); "each party near the actor" (a reach); the ladder itself (a body of 40 cannot send 16); `DEPTH` 2,
  `IDLE_BAR`, the hunt leash of 900; the builder timers (refused spots 90 s, walks ended at 40 s, stuck at 20 s),
  which the 09-28 cut line listed under "Decisions, remove"; `free_spots`; `usual_menu`. And "a damaged unit in its
  own entry", "the builds under way that the builder's entry lists": the nearest-N moves from the menu into the
  picture, which the draft declares out of scope.
- *What it rests on.* "any higher level heuristics making live game decisions should be removed completely." And his
  question on the escort: "Is escort something jev still coordinates, or did you add another regex-based heuristic
  to start key engine behavior again?" The 09-28 method (`memory/no-live-heuristics-ruling.md`): "Before removing,
  write the inventory and the cut line ... and get the user's ruling on the boundary cases".
- *The change.* A cut line over inventory §4a-4d, row by row: goes / becomes words / stays by ruling of (date). The
  inventory exists; §6 is a summary of it, not a disposition.

### A9. Builders are recommended in (Q5) and not designed. **Blocks** if Q5 is yes.

- *The part.* §3 builder rows; §9 Q5.
- *The objection.* Unsaid: whether a listed builder has a menu (today it has none unless "threatened", a code test at
  800); the started-build and queue-ahead gates; which spots an extractor may go to (today nearest 2 + 8 named + a
  safe one; change 7.5 of `2026-09-27-posing-changes.md`, which he passed while striking its rigid half, reads "the
  offer always carries the nearest free spots beside the named ones ... and Jev chooses": the note's words, not
  his); where turrets, radars and tier-2 extractors may go. And `go | a place` replaces "go home, step away,
  under flak": constructors live on lists and have no paragraph, so they get no place and no way out. That removes
  the builders' retreat without saying so, and the step-away exists because a game was lost without it
  (K-hands-home-is-no-way-out-with-the-party-at-it; onepass-player-2, 12:40).
- *The change.* Either leave builders out of this arc, or write their rows to the same standard and add a builder
  survival check.

### A10. Three fixes that came from his own replay reviews are undone. **Answer.**

- *The part.* Law 4; "Not carried over".
- *The objection.* (1) Raids: places with his known buildings came from the picture, after "the user, game 10 at
  5:58: two groups in position with 22-27 unguarded buildings within 3,000 held their stations"
  (K-hands-no-state-named-a-building). Under law 4 such a place is on a menu only if the packet names it for that
  group. (2) Pulling out of an unseen shooter's reach has no object unless Opus named one. (3) The split is dropped
  and the player has no tool to divide a standing group ("the player forms groups with `produce ... group`" is at
  birth only), though "Groups are the player's" (`one-decider.md` §3).
- *The change.* Places the picture knows something about (his buildings, a shooter's estimate) are objects for
  everyone. Keep a split, or give the player the way to divide a group, and say which.

### A11. Is it a rebuild? **Answer.**

- *The part.* Scope; §4 "Unchanged in form"; §8 "the pass's composition without its special cases".
- *The objection.* The gate, FLAG, IDLE_BAR, DEPTH, CAP, the two stages, ask-on-change, the forbidden mark,
  `rules.md` and `compose` all stay; `compose` is to be edited, not rewritten. He said "the whole lot of it obviously
  needs to be ripped out", and his standing instruction is delete first, because "Morphing old code preserves its
  hidden assumptions."
- *Both readings.* The joint worlds, the noul gate, the sampled first stage, the forbidden mark and ask-on-change are
  his own, so keeping them is right. But the draft does not separate "kept: his ruling of (date)" from "kept: it is
  there" (DEPTH 2, IDLE_BAR 0.3, EVENT_GAP, LINE_CHARS, the question forms, the `answer` question's wording). He
  would want that list, and `compose` rebuilt against the new slot model rather than trimmed.
- *Q4 (`rules.md`) should not be a question.* It holds "is fought now, whatever the instructions call it" and names
  states that no longer exist (inventory §6a). His rule is that the harness states observations and the packet
  judges (K-hands-menu-verdicts-beat-the-packet; `memory/tempo-not-rules.md`). Cut it to the vocabulary, move the
  judgments to the brief, replay the two sentences that measured as helping before they go.

### A12. Near-duplicates by construction. **Answer.**

- *The part.* The ladder 1, 2, 4, 8; `join` to every other group.
- *The objection.* "send 2 at party_7" and "send 4 at party_7" are the near-synonyms `REPORT.md` says to merge or not
  offer ("a near-synonym of the leading option takes about half its mass"), and ten fragments each offered nine joins
  give symmetric and cyclic worlds (A joins B, B joins A). K-hands-a-plurality-over-near-duplicates-picks-nothing is
  the measured cost. The ladder is also the bill: every group x every party x (whole + ladder) is 104,988 questions
  in player-33, the size of today's whole group budget, against about 15,700 recorded for the moves it replaces
  (hunt 8,674, whole group 4,363, the stand at the next extractor 2,610, the builders' attack 80; my count).
- *The change.* Check 4 has to measure the split of mass across sizes, not only whether a raid gets answered.

### A13. The cost table compares unlike things, and Q1 is asked on it. **Answer.**

- *The part.* §7, §9 Q1.
- *The objection.* Row 1 is what was asked, with ask-on-change closing busy actors. Rows 2 and 3 count every group at
  every gate with nothing closed (`menu_size.py`). "About four times" and "roughly double the bill" are therefore
  upper bounds, and he is asked to choose between the reading and a price that has not been measured. No latency
  figure appears anywhere, though the gate past 150,000 characters splits into batches and player-29 "ran slower than
  realtime on Jev calls".
- *What it rests on.* `memory/pianist-arc.md`: "The user's targets: Jev sub-second, player turns ~5 s or faster."
  "The point of jev is to do the two calls in a few hundred milliseconds and respond coherently".
- *The change.* Run check 5 first; give questions, tokens, dollars and seconds of calls per game minute for each
  option under the same asking rule.

### A14. "One question form for every move of every actor". **Minor / answer.**

The short form sank a builder's extractor from 0.77 to 0.12 and he ruled "restore the long form". Say which form; if
long, the 40% gate saving of 09-30 goes.

### A15. Validation is thin and has no pass line. **Answer.**

No number says what success is. The earlier notes each had one ("noop% under 10", "minute 13 is the bar"). Missing:
the fall-back through the gate (A3); flips per minute; builder survival; a realtime run; a two-seat run. "A
pianist-alone game on a fixed packet; one player game" is one game a setup. The order of work leaves out
`docs/heuristics.md`, the claims, `player.md`, `default.md`, `rules.md` and the 110 KB brief, whose lessons are
written in the old states' names; his position (`memory/scouting-glance.md`) is that lessons tainted by a harness
fault carry a tag so they can be cleaned out with the fix.

### A16. Smaller points

- The move's words promise "what it leaves (its post, its route)". The reading gives one noul for four roles, so
  code cannot know which place is the post. Either more reading questions (the vocabulary growing back) or the words
  go. Minor.
- `hold` merges hold and gather. player-33: "the gather stalled at 19 of 59 (stream from home never 'arrives')". A
  hold that waits for a tail fed from the plant never ends. Answer.
- Home-wired or mislabelled things kept under "by ruling": resurrection crews "walk home when nothing is to do"; a
  generator ordered "beside itself" is placed at home's back field when the builder is over 1,000 away. He said
  "remove the home-wired answers". Minor, but say them.
- Five questions are put to him in §9 before checks 2-5 have run; two of the checks were run after the draft and are
  not in it. He builds his model from the note; the note should be current before he rules.

---------------------------------------------------------------------------------------------------------------------

## B. What the draft leaves out

1. **Jev's state across seconds.** Only "reached". No record of the last pick and its reason, no "since", no
   register. His third change of 09-28 and his register idea are unanswered (A5).
2. **Thrash.** No design, no measure, and the route-leg rule removed.
3. **What the player is told.** Only the reading is shown. Not: the verbs Opus should write in (he prefers Opus
   adapting to Jev's vocabulary: "tell opus to use the internal unit names (armrectr) in orders to jev"); what each
   group did since the last turn and why; that an unnamed group cannot move. Players have misread the hands in every
   recent game ("they split detachments and feed them in" was wrong).
4. **Latency.** No budget per pass, no figure for the reading call, nothing on the 3 s stale rule.
5. **Cost per game.** A relative factor, no ceiling. Recent games are $1.3-2.4; player-29 at $5.07 started the load
   work.
6. **Realtime against people.** One request is in flight at a time and lists wait behind it (inventory §6b). When a
   packet lands, which places are on the menus until the reading returns? Packets land every few seconds.
7. **Several seats.** Whose packet is read for whose actors; allied groups as `join` or `follow` objects; "one-seat
   packets read into every seat" was a decompression harm.
8. **Builders** (A9) and the **commander** (D-gun, "the commander only when nothing sooner is at hand").
9. **Air and sea.** No row. Today: no hunt, no scout, no escort for air; an air attack names a unit; an air hold is
   a move state. Bombing a building has no verb.
10. **Opportunities.** The party question is worded for threats ("killing ... or about to kill something of ours").
    Nothing opens a group onto a target of opportunity.
11. **Verb to footwork.** Which lane commitment each verb carries (`go` flees everything today; a picked hold is
    priced; an advance commits all). That mapping decides fights and is code's.
12. **Failure paths.** A failed gate is not asked again for up to 20 s (inventory §3f). A rebuild should fix it.
13. **The rot list** (inventory §6) and the docs loop.
14. **A success line.**

---------------------------------------------------------------------------------------------------------------------

## C. Is it general enough: twelve situations against the menu as specified

"Words" = what Jev would need on the line or in the entry.

| # | Situation | Is the right move on the menu? | Words needed, and what goes wrong |
|---|---|---|---|
| 1 | 40 meet a stronger force; step back 800, wait; packet names a fall-back place | **Partly.** `go <that place>` if the reading registered it (unmeasured for fall-back sentences) and if it survives depth 2 against the party moves (A3). It goes where Opus named, which may be 3,000 back, not 800. | "steps back from party_7; 760 from it, outside its reach; 12 of ours 40 s behind". After arriving it is idle, asked every second, and world 1 lists its unreached stops: it is pushed back in. Nothing says "wait for the rest". |
| 2 | The same, no place named | **No.** `hold` (stops in the fight) or `join` (walks to another group). | The missing object is "back onto group_B" or "back toward our turrets at spot_54". |
| 3 | Lone raider kills an extractor far from every group | **Yes**, `send 1/2/4 at` from every group, the builders' attack. | Drive-off seconds, what it is killing, which soldiers go (unsaid: code picks). The detachment then holds where the hunt ends and is a new idle group with a full menu every second. |
| 4 | Two raids, opposite flanks, one army between | **No, in one pick.** One move per actor (A7). The second flank waits a second if the party is asked again, up to 20 s if not (A6). | - |
| 5 | Group on a six-stop route sees his commander alone | **Move yes** (`attack`), **opener doubtful.** A lone commander killing nothing rates low on the threat question; the group's own `change` is the weak side. | After the fight ends in a hold away from any reached stop, world 1 has no line for it (the cost sentence applies only "at a place it has reached"), so nothing but idleness argues for resuming the route. |
| 6 | Artillery in a mixed group, turret line outranges the rest | **No.** `shell` takes a party, and a party is "Enemy mobile units in sight" (`picture.rs:270-275`): a turret line with nobody under it is not an object. `fight to <place>` sends the whole body in. One move per actor means the guns and the screen cannot differ; the split that would separate them is dropped. | "shells the armhlt and 2 armllt at spot_18 from 820, outside their 620; the rest stand 300 behind". |
| 7 | Ten fragments should be one body | **Yes**, `join` to every group (better than nearest-only). About five slots fit the product under 255, so two or three picks. Cycles and symmetric pairs are possible (A12); fragments are idle, so 90 join questions a second. | "makes one body of 14 at spot_30, 20 s"; which group the packet names as the body. |
| 8 | Constructor far out, raider coming, no place named | **No.** stay, build, help, take apart, repair; `go` has no object; it is unarmed. `help <factory>` is a flight in disguise. Today's go-home and step-away are deleted (A9). | - |
| 9 | Commander at home, a party it cannot beat walking in | **Partly.** `D-gun` and `attack` yes. Leaving: only to a place the packet gives the commander; "go to group_L5" is not an object. | "walks to group_L5 (38 soldiers, 22 s)". |
| 10 | An air group | **Not specified.** Verbs written for ground. No strike on a building, no patrol; whether `send N at`, `scout`, `follow` exist for air is unsaid; `fight to` makes a bomber bomb the first thing on its line (H-HANDS-AIR-TARGET). | - |
| 11 | Group guards a builder moving spot to spot | **Yes**, `follow`, if the reading registers the builder (unmeasured for builders). Only builders can be followed: not a gun group, not a plant. | "stays within 150 of constructor_N; ends in a hold if it dies". |
| 12 | "Hold at spot_30 until group_B arrives, then go together to spot_41" | **Moves yes, behaviour doubtful.** Holding is idle: asked every second while world 1 says spot_41 is unreached, which measured as the lever that makes Jev walk on. Another group's arrival is not on the news list and no fact says where group_B is. "Together" is either `join` (two picks) or two `go`s in one world arriving apart. | "group_B is 1,400 away, 40 s"; on world 1: "waits at spot_30 for group_B by its instructions". Needs an offline test: prose conditions stated one way only have failed before (K-jev-a-job-sentence-is-not-a-per-second-signal). |

Count: clearly yes in 3 of 12 (3, 7, 11); partly in 4 (1, 5, 9, 12); no or unspecified in 5 (2, 4, 6, 8, 10). The
failures share two causes: objects are too few (places only, from the packet only, parties mobile only), and one move
per actor. This is the "not quite general enough" shape he named.

---------------------------------------------------------------------------------------------------------------------

## D. A design that meets his rulings better (the draft's verbs, four changes)

**1. Objects are the picture's nouns; nobody filters them per actor.** A move's object is any party, any place the
packet, a list or a mark names, any place the picture knows something of his at (buildings, a shooter's estimate),
and any group or builder of ours. `go to group_B` and `go to spot_54 (2 of our turrets)` are the step back onto
reinforcements; they need no fall-back sentence, no tool and no computed destination: Jev chooses among things
already named. No reading call, no bar, nothing for Opus to audit. Which place the instructions mean for which group
is read by Jev each second with the live picture, as every other instruction is.

**2. One opener rule, the same from both sides.** `<actor>.change` and `<party>.answer` stay (both measured). Either
one opens the whole menu of the actor concerned, ways out included. A move that leaves a party is worded as an answer
to it. The body's move and its detachments are separate slots, so one pick can send soldiers two ways.

**3. State is said, in registers Jev fills.** Per actor, printed in its entry and on world 1's line: reached and met
(as today); the last pick, its clock and its reason; and at each arrival one question to Jev, "by its instructions,
which place is next for group_A, or none", whose answer is printed as `next: spot_46 (Jev, 6:24)` and is what the
stop's cost names. Code keeps facts and Jev's own answers; it parses no order out of prose. This is his register set
taken literally. Until it is measured, keep the agreed next-stop sentence (24 of 24).

**4. A cut line, not a summary.** Every row of inventory §4a-4d marked goes / words / stays by ruling.

What it gives up: money and time. Without a per-actor filter the group questions are near the draft's 406,000 row
less whatever ask-on-change closes (unmeasured), so the bill may double and late passes may leave realtime. Levers
that do not put code in the decision: two sizes on the ladder until check 4 says more are used; the gate in two
steps (the openers first, then moves for open actors only: 47% of gate characters in player-29 sat under a closed
opener), at the price of one more call. It also gives up the reading's one real service, keeping another group's
places off a menu; the forbidden-mark form (a sentence, not a removal) can carry that if it turns out to matter.
Unmeasured in this proposal: whether the arrival question reads a route as well as the code's order does; whether
`go to <our group>` is taken in the 38 fall-backs. Both replay for cents on recorded seconds.

A second, smaller idea I measured while reading: a paragraph belongs to every actor its heading names anywhere (not
only "exactly"). On the 14 sampled packets that takes groups with a paragraph from 20 to 30 of 53. It does nothing
for the 20 named nowhere, which is why I would not build on paragraphs at all.

---------------------------------------------------------------------------------------------------------------------

## E. Fact check

Checked and right: 37 code choices and 19 prose readers (inventory §4a and §2 row counts); the paragraph rule and
"advanc"; 0 of 14 attacking groups with a move of their own; the 0.31-0.43 / 0.71 opener figures; the forbidden mark
as his, 0.7, hunts only; the sampled first stage as his ruling; §5's 160 of 218, 5 of 845, median 2 and p90 11 of 27
(`--read` on the business file); §7's 105,127 / 104,988 / 406,064 for player-33 (`menu_size.py`); walks and advances
75,815 asked and 65 played in player-33; sweep and split played 0 there; Jev $1.57 for player-33.

Wrong, loose or incomplete:

1. **§1 "A group has no plain 'go there' unless the packet happens to make one."** `walk_places` (`plan.rs:1139-1153`)
   always offers the nearest three of home, marks, passages and packet-named spots, and the same section says walks
   are 72% of the questions. The accurate finding is the claim file's: the walks are asked, rate 0.08-0.2, and do not
   reach the pick for a group held at a party. The rebuild as drafted offers fewer walks than today, not more.
2. **§4 "the rule of 2026-09-30, made the same for everything."** That rule asks every threat whenever the gate goes.
   Closing parties is a new rule (A6).
3. **§4 "no 'next stop': the order is in the prose."** Contradicted by the route replays on disk: 10 of 24 against
   24 of 24 (A4).
4. **§8 checks 2 and 3 "next".** Data for both exists on disk, timestamped 22:34-22:38 against the draft's 22:31;
   neither result is in the note.
5. **§5's measurement.** "14 packets" is every fourth packet (`--every 4`). The headline leaves out 64% of the
   answers and the 20 of 53 group-packets named nowhere; 13 of 53 groups get no place. The other question form on
   disk ("about") reads 208 of 218 and 22 of 845 with a median of 0 places a group; the note reports only the form
   it chose.
6. **§7 "from 26 to about 4".** 3.9 is the mean; the median is 2 and a quarter of groups get none.
7. **§7 the table.** Row 1 has ask-on-change, rows 2-3 do not (A13). The note does say "asked every gate"; the
   "about four times" in §5 does not carry that caveat.
8. **§1 "Fifteen special group options".** The sentence lists sixteen names (fifteen if walk and advance are one).
9. **§3 `shell | a party`.** A party is mobile units only; a turret line is not a party (case 6). True of today's
   shell as well, but the table reads as if artillery had a general target.
10. **§3 `go` "replaces ... pull out, fall back".** Those took places from the picture or the group's own history;
    `go` takes only packet places, so it replaces their verb and not their reach.
11. **§6 "either the player's order or a mechanic the user ruled to keep".** True of the items listed. Not listed and
    not ruled kept: the builder task timers, the lost-party hold at 6 s, the hunt leash, `free_spots`, the list hold
    under threat, the tier-2 extractor's "upgrades the nearest other" (inventory §4c-4d).

Not checked: "over five games" figures other than player-33; "a route reads at 0.95-0.98"; "coverage lists sit at
0.42-0.68".
