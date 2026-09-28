# Target design: Jev plans the engagement (stage 2 of the micro answer)

**The ruling (the user, 2026-09-28).** "Let's see if we can get jev to plan out an engagement intelligently. This
might be a separate call from the world choice system, but given a picture of the battlefield and some basic
examples it may choose a decent order of operations." Stage 2 of the micro answer; stage 1 (the body's shape) is
`docs/design/2026-09-28-body-shape.md` on branch `tas-micro`.

**Status (2026-09-28).** Design written; nothing built. Branch `engagement-plan`, worktree `../bar_bots-plan`.
Step 1 built: the position finder and the battlefield with the cover-priced odds (`crates/bot/src/brain/pianist/engagement.rs`;
the synthetic E3 scene's test: the Centurions' stand-off exists, the turrets' do not).
Step 2 built: the candidates and their words (`engagement.rs` `candidates`), `bot --plan-replay` (`pianist/replay.rs`, the
brain rebuilt from the record's header and terrain, `recorder.rs` `hello_from_record`) and its wrapper
`run/plan_replay.py --dump`. Found on the way: at the named clocks the positions were mostly not yet in sight (E3's
Centurions first seen at 9:15, the turrets at 9:16/9:19); player-14's group_C was holding at its station when E3 came
in sight, so the gate takes the hold an advance arrived in as an attack; player-17's body was fourteen Rovers and one
Blitz, which the scout check by every armed member missed (now: scouts more than half the armed metal).
Step 3, the validation (`docs/studies/2026-09-28-engagement-plan.md`, 28 Jev calls): no named clock had a position in
reach, so each fight was replayed at its first planned second. E3 (9:16, 9:17) ranks screen_first 0.76-0.81 over
statics_first and nearest_first (0.01-0.03): the TAS's verdict met. The F3 test fails (no shell candidate, the
decline last at 0.01-0.03, every phase "we outweigh it"), as does the decline of an Overwatch that out-ranges the body
(player-16 16:10: 0.21 with the examples, 0.11 without). The examples block moved no top pick. A candidate added:
`one_at_a_time` (the statics our reach reaches from outside every other reach, one at a time from there), offered in
place of screen_first when no mobile element has a stand-off; a turret line had only walk-ins before.
Step 4 built: the live question once a second after the one pass (`engagement_pass`; in realtime on its own worker
thread, an answer older than 3 s dropped), logged as `{"t":"engagement", ...}` (not `"plan"`: that line is the worlds
pick's), the plan as the task `GroupTask::Plan` (`tick_plan`: a Fight to the phase's point, Attack by id on the
nearest target in sight, a building out of sight fought where it stands; a phase ends with its targets dead, 10 s out
of sight, or 60 s; after a covered phase a Move out of the reach first; the gather at 80% within 300), the lane
priced against the current phase's statics only (`micro.rs`), the player's line in the group's entry, the standing
`plan: no`. Deleted from the pass: the whole-group attack on a party of the position a body's plan answers (running
or opening this second), and the group's rule defaults (station walk, fall-back) while it runs a plan. Not built:
stage 1's lane shape taking the phase's target (branch `tas-micro`, not merged here).

## Why

The TAS study (`docs/studies/2026-09-28-tas-e3.md` on `tas-micro`; the claim
K-micro-the-front-stops-on-the-nearest-and-blocks-the-rest in `docs/knowledge/army.md`) found the E3 position
falls 24 of 24 to an order of operations and 0 of 24 to the recorded orders: the mobile screen first, at the rock's
corner outside turret cover; out of the reach after the kills; gather outside the turrets' 430; the turrets last.
Turrets-first fell 5 of 24, attack-move 21 of 24. The hands today offer the pass's `whole_group` attack on the
nearest party and the body meets the screen and the turrets at once (player-14 9:17 E3, player-15 9:27 F3,
player-16's pushes into F2, player-17's Rovers at 4:53). The pass also priced "2 armpw" while 324 of soldiers and a
turret stood on the spot (K-hands-a-scout-body-offered-an-armed-fight, open item).

## The shape

1. **When.** A body (a group with a course or an attack, H-HANDS-GROUP-BODY) whose front comes within `PLAN_REACH`
   (1,200) of a **position**: two or more armed enemy elements (parties, turrets, remembered armed buildings) within
   600 of each other, at least one static or the elements' metal above a third of the body's. One plan per body and
   position; re-planned when an element appears or dies or after `PLAN_STALE` (45 s). Not for a scout body
   (H-HANDS-SCOUTS-FIGHT-NOTHING-ARMED), not for a lone raider (the threat pass's hunts stay).
2. **The battlefield picture** (words, the picture's conventions: `crates/bot/src/brain/pianist/picture.rs`,
   `docs/design/2026-09-26-one-pass.md` §8 on what Jev reads). Our body: count by type, reach (shortest and longest),
   health, front and centre, speed; whether it is gathered (the tail within 300 of the front) or strung out. His
   elements, each: name, type and count, reach, metal, health if known, its place and bearing from our front, its
   distance, **what covers it** (every static reach it stands inside, with the turret's name and reach), **whether
   our reach reaches it from outside every other element's reach** (the stand-off test: a point at our shortest
   reach less 20 from the element that lies outside every other reach; say the point's place), the odds of our body
   against it alone and against it with everything that covers it (`combat.rs` `odds`, the cover priced in: this is
   the odds fault's fix), and whether it is a screen (mobile, within 400 of a static) or a static.
3. **The candidate plans.** Enumerated by the hands, each an ordered list of phases `{targets, stand_off, until}`:
   - the screen first from its stand-off point (outside the statics' reach), then the statics, then the rest;
   - the statics first (an attack order walks each unit in; the screen shoots meanwhile);
   - the nearest element first, then by distance (what the pass does today);
   - gather at the stand-off point of the first phase until most of the body is there, then any of the above
     (a `gather` phase);
   - hold at the stand-off point and shell (when the body has artillery out-reaching the statics and a spotter with
     the statics in sight);
   - decline: fall back to the last hold (the position outweighs the body with its cover).
   Each is worded with its geometry (the phase's stand-off place, what is in reach there and what is not, the odds
   of each phase with the cover priced in, the walk) and priced (the body's metal at risk in the phase, the target's
   metal). At most six candidates.
4. **The call.** A separate Jev question (`jev::Question::choice`), not a world in the one pass: instructions = the
   packet's words on fights (the player's `instruct` and standing orders for the group), the examples (below), the
   battlefield picture; options = the candidate plans. The answer's probability per option ranks them; the top
   plan is taken when its probability is above `PLAN_BAR` (0.4) and above the decline's, else the decline. Logged
   as `{"t":"plan", ...}` in `jev-<ai>.jsonl` with the picture, the options, the probabilities and the choice, so
   `run/plan_replay.py` can replay recorded moments.
5. **The examples** (a fixed block in the instructions, from the studies; kept short, one line each): the E3 scene
   and its numbers (screen first 24/24 at +0.318; turrets first 5/24; attack-move 21/24; the recorded orders 0/24);
   the F3 scene (four Rocketeers under a turret out-range Blitzes and Stouts: the stand-off is beyond 475 or the
   fight is declined); the bench (a Stout ball against a few Fatboys loses worse the tighter and bigger it is: spread
   and few against area); thebluegecko's Rovers (unguarded things only, never a fight with Pawns).
6. **Execution.** The plan is the body's task (`GroupTask::Plan { phases, phase, since }`): each phase is a
   whole-group attack on its targets from the stand-off (a Fight to the stand-off place, then Attack on the targets
   by id as they come in sight; an attack on an unseen unit is dropped by the engine: `docs/harness/duels.md`), and
   the phase ends when its targets are dead or out of sight for 10 s, or after its `until` (60 s), then the next.
   After a phase whose targets stood under a static's reach, the body steps to the next phase's stand-off first
   (out of the reach). The gather phase ends when 80% of the body is within 300 of the point. The lane's shape
   (stage 1) takes the phase's target as the body's target.
7. **What the player sees.** The group's entry says the plan in a line ("plan: the screen (party_12) from spot_44,
   then the turrets, then party_9; phase 1, 20 s in"); the player's `standing` can decline planning for a group
   (`plan: no`) and the packet's words are in the call.

## Validation (before any game)

`run/plan_replay.py <match dir> <clock> [--body group_X]`: builds the battlefield picture from a recorded moment
(the record's own units and enemies in sight at the clock, the remembered buildings, the group's task) exactly as
the bot would, enumerates the candidates with the same code (a `--dump` of the words), asks Jev, and prints the
ranking. Run it on: player-14 9:05 (group_C before E3), player-15 9:20 (before F3), player-16 11:05 and 16:00 (the
F2 pushes), player-17 4:50 (the Rovers, expected: not planned). The test: the E3 moment ranks the screen-first plan
above turrets-first and nearest-first (the TAS's verdict); F3 ranks the hold-and-shell or decline above the walk in;
the same picture with the examples block removed, for the examples' worth. Then one arena game (the main session
runs it) read with `run/analyze_match.py --engagement`, the plan lines in the Jev log, and `run/fire.py`.

## Order of work (each ends with a commit here and a line in the status above)

1. The position finder and the battlefield picture with the cover-priced odds; a unit test on a synthetic scene
   (three Centurions and two turrets: the Centurions' stand-off points exist, the turrets' do not).
2. The candidate enumeration and the words; `--dump` in `plan_replay.py` from a recorded moment.
3. The Jev call, the log line, the replay tool's ranking; the validation runs above and their table in
   `docs/studies/2026-09-28-engagement-plan.md`.
4. Execution as the group's task, the player's line, the standing `plan: no`.
5. The heuristic row (H-HANDS-ENGAGEMENT-PLAN) and the claims (`docs/knowledge/jev.md`), the design's status.

## Constraints

This worktree only (`../bar_bots-plan`, branch `engagement-plan`); the main tree is read-only (its `run/matches`
records may be read for the replay tool). No arena, no engines, no `run/install_ai.sh`, no `run/install_to_bar.sh`;
never kill a process; never `pkill -f`/`pgrep -f`; never touch `~/.local/share/BeyondAllReason`. Tests and builds
with `CARGO_TARGET_DIR=<scratchpad>/target-plan` if the tool policy refuses the worktree's own target. Model calls:
only Jev through `crates/jev` (`Client::from_env`, the key file `~/.config/within-reason/jev.env` read by the
client; never print or copy the key, never log it); the replay tool's calls are a few dozen. Never `git add -A`;
commit at each step's end with the two attribution lines. Every brain change registers a heuristic in
`docs/heuristics.md` and a claim in `docs/knowledge/` (`docs/README.md`). Delete what the plan replaces in the pass
(the whole-group attack on a covered party by a body that has a plan) rather than keeping both.
