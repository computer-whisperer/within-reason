# The menus from scratch (outline for discussion, 2026-09-25)

The user, after family-1 and the `where` reading: "I think we should try a completely from-scratch menu set with what
we know now." This is the outline, written before any code. Nothing here is built. The receipts are the Jev batteries
(`../jev_experiments/REPORT.md` and `results/a-i.md`, about 4,500 calls), the K-jev and K-hands claims in
`docs/knowledge/jev.md`, the deep dive `docs/studies/jev-comet-series.md`, the rules A/B
(`docs/studies/2026-09-22-jev-rules-ab.md`), the two-level A/B and family-1 (ledger rows jev-ab-two-level, family-1).

## 1. What we know, and what each fact costs the present menus

1. **Questions in one request do not see each other** (battery G: thirty siblings asserting a false premise moved a
   target not at all). Every speculative parameter question (`where`, `whom`, `how_many`) is answered from the state
   alone under a hedged "suppose". The present `where` premise covers a turret, a radar, a tier-2 extractor and a walk
   in one breath; a third of the turret picks in family-1 and bank-1 landed on a walk-flavoured place (a bare spot a
   constructor was going to take, yard3, refuge, a spot never seen). K-jev-where-needs-its-premise said this in
   September; the per-purpose fix was applied to extractors only, where it scores 0.15 to 0.30 higher p_top.
2. **Chaining requests through code is the only multi-step reasoning Jev has** (battery H: an 8-move walk 0.10 in one
   shot, 1.00 chained; battery D: 130-250 ms a call whatever the size). A decision that has two parts, what and with
   what, can be two calls, the second written with the first's answer as its premise.
3. **The option's text steers the pick; the state does not** (batteries A, I, F: computed consequences in the option
   text cut chess loss from 313 to 115 cp, the same facts in the state 266; a reasonable extra option wins 21 of 48).
   Our options carry words about the action; the consequence (what this group leaves uncovered, how far the party is,
   how long the walk takes, what the packet ordered and how long ago) is mostly on the state's lines.
4. **Jev cannot compute a quantity it is not given** (batteries A, B, D: nearest of 20 from coordinates 60%,
   Lanchester odds under 1.4 at chance, counting off by one). It reads a number that is printed. `how_many` asks Jev to
   size a detachment; the odds table can size it.
5. **A many-way Choice over places is weak; a Score per candidate or a short list is not** (battery B: Choice 65-85%
   top-1 at N=10..40, per-candidate Score 85-95%; the deep dive: 128 to 149 place answers a game under p_top 0.35).
   Our group `where` lists every place in play, names only.
6. **Vote-splitting is real but modest** (battery F: a synonym takes half the leader's mass, flips only under a 0.75
   lead; our logs: family-level flips 2-5%). Merge synonyms in code, one option per kind, and keep option counts small.
7. **A structure question frees the course, and it overshoots** (jev-ab-two-level, family-1: raider_ignored 19-29% to
   3%, and a busy group kept its packet course 7% of asks against 81-88%; the player wrote that every group was pulled
   after every raid). The packet's standing course needs its own weight and its own consequence words, not a seat in
   the `stay` family.
8. **"Never" is not heard** (K-jev-never-is-not-heard: forbidden detachments and shelling advances chosen at 0.4-0.8;
   two rule phrasings did nothing). A prohibition must remove the option or the place from the menu.
9. **Confidence is (p_top - 1/n)/(1 - 1/n)** (battery F, 9,852 answers): padding inflates it. Threshold on p_top or
   top-minus-second, never on `confidence`. p_top >= 0.9 on a semantic judgment was never wrong (battery C).
10. **Fluent text before the problem hurts; fields last are read best** (battery H). Our state opens with the packet
    and a 2,000-character rules paragraph; the actor's entry, the thing the question is about, comes after them.
11. **Dead weight costs mass and orders** (deep dive §6: converter 1,092 offers 0 picks, lab 1,092/0, walk_to
    1,092/5, `join_group_X` eight per ask, `nothing` chosen 54 times at "nearly empty" in an opening).
12. **A `none` option is honoured** (batteries C, F): offered `cannot_say`, Jev takes it rather than guessing; battery
    B's "between" named some spot when the truth was none because none was not offered.

## 2. The shape: decide, then furnish

Each ask is up to two calls, and the second is written after the first is read.

- **Call 1, the decision.** One question per actor on the call, over a few families of action, each family's words
  carrying its computed consequence for this actor now. The actor's packet course, when it has one, is its own family
  with the order and its age in the words. Every actor's call-1 question is batched in one request as now (batching is
  free).
- **Call 2, the furnishing.** Only for actors whose chosen family needs more: which option of the family, and its
  arguments (a place, a party, a building), asked with the decision as the premise ("group_A fights: with the whole
  group or a detachment, and which party"). Candidates are few (five or fewer, ranked by code), every candidate carries
  its numbers, and `none: nothing here fits; keep the course` is always offered. All furnishing questions of the tick
  go in one second request.
- **Nothing speculative.** No `where` for a group that holds, no `whom` for a builder, no `how_many` anywhere.

Cost: today each group carries `do`, `where` (every place in play), `how_many` and `whom` on every ask; family-1 sent
710 two-level calls in 31 minutes. Under this shape call 1 is smaller than today's request and call 2 is sent for the
minority of asks that change course, so tokens fall and wall time rises by one Jev latency (130-250 ms) on those asks.
The worker path (realtime) already handles a pending request; it grows a second stage.

## 3. Call 1 per actor

**Group.** Families, each with computed words:
- `course` (only when the group has a task under the current packet): "advancing to spot_13, 900 to go, 40 s in; the
  packet's order, written 70 s ago". Its consequence: what leaving it gives up.
- `stand`: hold here; what holding leaves to die (H-HANDS-PARTY-KILLING's clause, kept) and what it protects.
- `fight`: the nearest party's name, composition, distance and odds as words and a number, what it is killing, time to
  reach it at the group's speed; whether a detachment would do (the odds table's answer, see §4).
- `go`: the places the packet names for this group, with the distance to the first; otherwise the nearest place worth
  standing at.
- `back`: the last hold or home, with the distance, and the group's own losses line.
- singletons as themselves: `scout` (only while something is unknown: the base unseen three minutes, or spots never
  looked at in reach), `join` (one option; the nearest same-domain group and its distance in the words, the second
  question only when two are within reach).
The switch rule (§6) reads the `course` family against the rest.

**Builder.** Families, each with computed words:
- `course`: finish the frame ("42% built, 90 metal sunk; the frame decays if left"), or the standing task with its age.
- `extractor`: the nearest free spot with its walking time and ground; how many free spots remain in reach.
- `energy`: the store's fate over the next minute (income, usage, what the factories draw), and what one generator adds.
- `production`: factories standing, the bank and income, what is already under way of that kind (the fixes-1 clause).
- `defence`: the nearest building of ours without a turret in reach, whether it was raided lately and by what; radar
  when nothing of ours sees the approach.
- `help`: the nearest factory and whether metal is flowing (the store) so help adds anything.
- `field`: wrecks (metal lying, safe or not) or a hurt structure (health), whichever is nearer.
- `walk` and `home`: the packet's named place for this builder, the distance home, the ground.
- `attack`: as today (H-HANDS-COMMANDER-FIGHTS), a singleton with the odds.
Only families with a member on the builder's allowance (the player's `produce` list or the usual list) are offered.

**Factory.** As today, the allowance is the whole menu, the have-counts in words on each option. `nothing` only when the
store is empty and spending has exceeded income for 30 s.

**Global.** The two Nouls (`base_in_danger`, `attack_coming`) go to the journal only. Kept if something reads them,
deleted if not (to verify before building).

## 4. Call 2 per family

- `fight` (group): the refinement in one question (whole group / a detachment of N / every soldier on the party's
  commander or dearest unit), with N computed from the odds table as the smallest detachment that outweighs the party,
  said in the option ("a detachment of 2 Blitzes: enough for 1 armflea"); `whom` over the parties within reach (at
  most five, nearest first) with distance, odds number, what it is killing; `none` keeps the course.
- `go` (group): walk / advance / a detachment of N advances (the old `split`, N computed for a picket: two for a spot);
  `where_go` over the packet's places for this group, the group's present destination, and the nearest spot toward the
  enemy, at most five, each with distance, walking time, what stands there and enemies near; `none`.
- `back` (group): home or the last hold, one question, distances in the words.
- `defence` (builder): which building (turret kinds, radar, jammer on the allowance) and `where_defence` over our
  buildings in reach without cover, ranked by the threat grid and recent raids, each with what it covers and who raided
  it; `none`.
- `production` (builder): which factory or nano; the yard place is code's (as now), a nano beside the nearest factory.
- `energy` (builder): which generator on the allowance (solar, wind, advanced), placed beside the builder as now.
- `walk` (builder): `where_walk` over the packet's places and home, distances and ground.
- `extractor` (builder): `where_extractor` as today (it already scores well), `none` added.
- `join` (group): which group, only when two are within reach.

Where a candidate list would exceed five, code ranks and cuts it; where a judgment over many is unavoidable, one Score
per candidate and argmax in code (battery B), never a twenty-way Choice.

## 5. Removals and prohibitions

- The player's prohibitions become removals: a `forbid` tool (per actor or `all`: an option family, an option, a place)
  taking the entry off the menu before the question is written; the picture says what is forbidden on the actor's line.
  "never" sentences in the packet still reach Jev as text but no longer carry the burden.
- Dead weight off by rule: wind at dead wind, converter without an energy surplus, factory beyond the packet's cap, nano
  without a factory, `scout` with nothing unknown, `join` with no group in reach, `nothing` outside a real shortage,
  `reclaim`/`repair` only with something within reach (as now).
- Synonyms merged in code: one `join`, one `attack`, one extractor; the families themselves are the merge.

## 6. The switch rule and the answer's use

- A busy actor keeps its course unless another family's mass beats the `course` family's by the margin (0.15 today; to
  be re-measured on the new layout). The pick within the `course` family needs no margin (there is none).
- A hedged call 1 (top-minus-second under 0.1) on a free actor is played; on a busy one it keeps the course.
- Call 2's `none` returns the actor to its course; an argument answered at p_top under 0.35 over five candidates is
  taken (the list is short by construction), but logged as hedged for the readers.
- Confidence is never read.

## 7. The state

- Order by relevance last: economy, enemy, places, then the actors asked this call (the group's own entry last of
  all), then `instructions`, then a rules paragraph cut to what the menu words mean (the game-model sentences move into
  the option words they belong to, as consequences).
- The rules paragraph today is 2,000 characters of fluent prose first in the state; battery H says that is the worst
  place for it. The A/B tool can measure the order change before the build.
- The actor's entry keeps its computed lines (`enemies_near`, `losses`, `stuck`, `detachments_out`); the option words
  point at them by field name where they weigh (battery D: an unnamed derived field is recomputed and wrong).

## 8. What is deleted

`menu.rs` question builders (the speculative `where`, `where_extractor` stays, `how_many`, `whom` as a standing
question), `family.rs` one-request layout and its composition, `split` as a standing option (it becomes `go`'s
detachment), `join_group_X` per group, the `nothing` gate on "nearly empty", the global Nouls if unread. `hands.rs`
plays picks as now; the picks gain the computed N. The log keeps `questions` and `answers` for both calls under one
`call` line with `stage: 1|2`, so the readers (`jev_ab.py`, `jev_audit.py`, the viewer's Pianist tab) keep their layout.

## 9. Validation before and after the build

Before: `run/jev_ab.py` gains the new call-1 layout as a variant on recorded moments (the families with consequence
words approximated from the recorded state's lines; exact numbers where the record has them) and measures the detectors
and, new, `course_kept` (busy asks that keep the packet course) and `turret_off_cover` (a defence placed where nothing
of ours stands). Targets from what we have: raider_ignored well under the flat 16-29% and course_kept well above
family-1's 7% at once; controls at 0.6 or more kept above 90%.

After: two check games against hard_aggressive on Comet Catcher read by the same detectors on live logs, `run/floor.py`,
`run/fire.py`, and the player's own notes on whether its packets were followed. The success line is the pair holding
live: raider answers up, course kept up, turrets on cover.

## 9b. Bundling, by cost (the user, 2026-09-25)

Money is not the constraint: a call is 3-6k tokens of state at $0.042 a million, a fiftieth of a cent; latency is,
130-250 ms a call whatever its size. So a question bundled in call 1 is free in time and nearly free in tokens, and a
follow-up pays a latency and the state again. The rule: bundle a furnishing question when its premise is known from
the family alone and the family is common (group `whom` under "suppose it fights", `where_go` under "suppose it
goes", builder `where_extractor`); follow up when the argument depends on another answer's content (a detachment's
size on `whom`, a defence place on which building) or the family is rare (builder defence 2% and walk 3% of picks in
family-1; fight 28%, go 23%, back 7% for groups). The schedule is re-read from each new game's log.

## 9c. Actions in combination

Jev answers each actor alone (battery G). Family-1 had 71 same-call collisions in 415 calls with a fight pick (two or
more groups after one party; 13 with a detachment), bank-1 8, 2v1b-hard 11. Three layers, cheapest first:
1. Claims across calls: a target or place one actor has taken is said on every other actor's option words (the
   `engaged` filter and the under-way factory words, extended to every target).
2. Sequencing within a call: of two actors that would be offered the same contested target, only the best placed is
   asked this call; the other next second with the claim in its words. No follow-up, one second of delay.
3. A scenario question for ties and equally placed actors: code composes the few joint scenarios with each one's
   computed consequences in the option text, both the gain and the cost, and Jev picks; the other actors' call-1
   picks are stated as facts in the premise (battery H: supplied results are authoritative, so only facts). The
   user's ../jev_experiments battery on scenario questions decides how far this layer reaches.

## 9d. Standing orders before menus (the user, 2026-09-25)

A game costs about $0.59 of Jev (family-1, bank-1: 1,250 calls of 11k tokens, questions half of it, a quarter to a
third of calls changing nothing), which bounds how many test games can run. Most of what the hands decide forty
times a minute is threshold-shaped and the player already writes it as prose that Jev re-reads every ten seconds.
The layer: **standing orders**, a fixed vocabulary of per-actor rules with parameters, executed in code every second
without a Jev call; Jev is asked only where no order fires or an order's condition sits in its grey band.

**Jev as the decompressor.** The user's lever: Opus writes prose cheaply (editing Lua cost it more tokens than
shouting another order), and Jev reads prose literally and reliably (batteries C, D: intent 100%, rule application
9/10, literal quantifiers). So the packet is put to Jev once per player turn with one extraction question per
(actor, rule) over the vocabulary, the answers become the standing orders, and the per-ask questions those orders
cover are suspended until the next turn. `run/decompress.py` is the offline test on 293 recorded packets of four
games: consistency (identical paragraphs give identical rules), coverage at the detectors' moments (would the
extracted order have answered the moment), and a dump for hand reading. Results in the study when it lands.

**The vocabulary, v0 (from the packets of family-1, bank-1, 2v1b-hard, 2v1b-hard_aggressive):**
- group: `station` place; `raiders` whole_group / detachment / forbidden; `detachment_size` 1 / 2 / 4 / half;
  `no_chase`; `no_detachments`; `fall_back` outweighed / never_while_even_or_better, `fall_back_to` place; `move`
  advance_and_fight / walk / engage_party / hold / scout_route with `move_to`; `never_<place>`.
- builder (commander, constructors): `job` help_factory / follow_list / expand; `attack_raiders`; `no_chase`;
  `solar` only_when_stalling / never / freely; `turrets` beside_each_outer_extractor / beside_each_extractor / none;
  `rebuild_lost`; `retreat_when_enemy_near`; `never_<place>`.
- factories: `produce` as today.
What the vocabulary cannot say stays prose for the menus: a conditional ("never past spot_39 without soldiers
near"), a named party to kill, an argument about timing.

**Execution.** Each rule is a small code path over the picture (the raider rule: a party of at most N at one of our
structures within R of the group; the nearest group whose order says detachment sends the odds table's size). The
picture shows each actor's orders in force and which one fired, so the player sees its prose land. A `standing` tool
lets the player set or clear an order directly, bypassing the decompression, and `forbid` (§5) is the `never_*`
family of it.

## 10. Open for the user

1. Two calls per ask is the core of this. It adds 130-250 ms of wall time on the asks that change course, and one
   stage to the worker. Agreed, or is a single request with per-purpose speculative questions (§4's questions asked
   for every family every time) preferred despite the token cost?
2. `forbid` as a player tool against parsing "never" out of the packet: the tool is explicit and cheap; parsing is
   fragile. Tool?
3. Detachment size from the odds table rather than from Jev: the table is the sim's, and its odds ignore health
   (K-hands-odds-ignore-health). Good enough for sizing a picket against a Tick, or should health scale it first?
4. The global Nouls: delete if nothing reads them?
