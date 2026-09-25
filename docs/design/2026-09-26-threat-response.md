# Threat response: the hands answer raids in seconds, the standing orders only bend it

**Status.** Written 2026-09-26 morning after worlds-2 (LOST 31:01) and nouls-smoke-1; agreed by the user ("Agreed,
proceed") on the order: the hunt primitive and the raid scenario first (the micro agent), then the threat-centred
pre-pass and post-pass replacing the per-group candidate generator (the main thread), then ask-on-change, then the
validation ladder. nouls-1 (the player under the gate mode as built) was running when this was written and tests
none of it (LOST 22:01). Built 2026-09-26 morning in the brain: §2 (`Hunt` on the group, `declined`), §4-6 (`threats.rs`, `threat_pass`, `apply_plan`, the two calls, ask on change) with the per-group generator deleted; §1's engine primitive (H-MICRO-HUNT) and §3 (`duel --scenario raid`, `run/raid_read.py`) landed by the micro agent the same day (merge 919fceb), ladder step 1 measured: first raider killed median 11 s with the hunt against 45-100 s without, extractors lost 0.12-0.25 a raid against 1.1-1.4, both raiders dead in 15 of 16 raids against 6 of 16; the brain hands its hunts to the engine's commitment from threats-smoke-5. Next: ladder step 2, the raid scenario under Jev's passes.

## The ruling

The user, after reading worlds-2's first five minutes: "for this to ever take off we need the hands system to be able
to coherently react to threats quickly with the standing orders only twisting the behavior. We can't wait for opus to
react on things like this, so it has to be jev being smart and tuned well enough to respond. That's why I suggested
the pre+post pass: giving jev back the broad option space necessary while hopefully being able to trust it. Jev needs
to be able to observe the threat, decide what set of units to send after it (splitting off of groups as needed) in
combination with all the other necessary activites, and see it through at high speed. In addition, the micro engine
definitely needs to give jev the option to chase a given unit."

## The evidence (worlds-2, `run/matches/1790309898-worlds-2`, minutes 2 to 5)

One Pawn (2:33-3:08) and one Tick (3:37-4:38) killed six extractors; the count went from 3 at 2:00 to 2 at 3:00 while
the player was at its third turn. The fast layer failed in three separable places:

1. **The chase could not end.** Four Rovers (speed 168) never closed on the Pawn (87) from 350-600 away in 28 s: the
   engage order is "fight to where the party was", re-issued every 1-2 s, so the chasers run to a point the raider
   left; the formation transform (H-MICRO-FORM-FLANK, an enemy within 700) gave two of the four fight points 400-500
   short of the Pawn every tick, the line reforming at the frontmost unit's depth; the flee rule fired ten times on
   the Rovers in that minute (move orders away from the Pawn at 2:44-2:52). Against the Tick (132) the Rovers ran at
   full speed and still trailed it by 175-380 the whole way for the last-position reason.
2. **The hands vetoed the player.** With station spot_54 set through the tool, the worlds question offered "keeps
   holding at spot_43" against "walks to spot_54 (its standing rule)" seven times in 22 s (3:24-3:46) and Jev kept the
   hold every time at 0.4-0.75 while the Tick killed two extractors.
3. **The raider trigger waited for the raider to reach a building**, and a detachment became a group: the standing
   raider rules fire only for a party at a structure of ours (the Pawn was offered to Jev at 2:35 while crossing
   spot_38, the rule fired at 2:45 with the extractor at spot_36 already dying); the one Rover sent by `send_against`
   was named group_B, took the Blitz paragraph's "gather at spot_38" rule and walked off at 4:06.

The gate mode as built (nouls-smoke-1) only filters the same per-group list, so it fixes none of these.

## The shape

The loop that must be right runs in seconds: Jev plus code. Opus bends it through the standing orders, which prune
options and set defaults, and are never put to a vote against the course. Jev is asked when the threat picture
changes, not every second; between changes the chosen plan stands and code executes it to its end.

### 1. The hunt primitive (the micro engine; the micro agent)

A commitment kind `Hunt` for a set of hunters on one quarry (a unit id):

- **Attack by id**, re-issued every tick from the latest sighting (sight or radar), never a fight-to-point.
- **Hunters run raw**: no formation, no flee, no march, no kite; the engine's attack order does the closing.
- **Only units faster than the quarry hunt** (the code knows every speed: Rover 168, Tick 132, Blitz 101, Pawn 87);
  a hunt with no faster unit in reach is not offered.
- **Ends** when the quarry dies, is out of sight and radar for 6 s, the leash (900 from where the hunt began, or the
  group's station) is reached, or a hunter falls under a third of its health (that hunter drops out). The engine
  reports the end and its reason in its output so the brain can release the hunters.
- Measured in the duel harness before anything is built on it (§3).

### 2. Detachments are temporary bodies (the brain, `groups.rs`; the main thread)

A detachment on a hunt is `Detachment { from: group, members, hunt }`: not a named group, not adoptable, no rules of
its own, invisible to the player except as "3 of group_A hunting party_9"; when the hunt ends its members rejoin
their group and take its current task. The `send_against` order and the path that makes a new group from it are
deleted. A whole group on a hunt keeps its name and returns to its station when the hunt ends.

### 3. The raid scenario (the duel harness, `--scenario raid`; the micro agent)

Ours: a picket body (4 Rovers and 2 Blitzes by default, any list) at a station, four extractors along a strip 600
apart with a constructor working on the far one. Theirs: a Tick and a Pawn on a scripted route that hits the
extractors in turn and runs from anything that engages (a director script, not BARb). Three minutes. Scored by
extractors lost, seconds to the first kill, hunters lost. Two instruments: the code's defaults alone (the standing
raider rules as world 1, no Jev) and the pianist's hands (Jev's passes); each with the lane on and off. This is the
cheap tuning loop for the hunt and for the lines' wording, at cents a run and no Opus (floor skill first).

### 4. The threat-centred pre-pass (the brain, `worlds.rs` rewritten; the main thread)

Enumerate by threat, not by group. A threat is a party in our half, within 1,200 of a structure of ours, or moving
toward home. For each threat the code lists the partial action states it can execute, and the pre-pass asks one noul
per state:

- `party_N.answer`: does party_N need answering this second (given what it is, where it is, what it is killing)?
- `party_N.by_group_X`: group_X whole, for each group within reach that outweighs it (turrets counted, H-HANDS-TURRETS-WITH-A-PARTY).
- `party_N.hunt_from_X`: the K fastest of group_X that outrun it (K from `detachment_for`), the rest keeping their course.
- `party_N.leave`: to the turret or the commander when a rule allows, or to nobody.

The standing orders enter as constraints and defaults, not candidates: `never` places and `ignore` rules remove
states; `raiders_lone: whole_group` or `detachment:N` make that state the default (world 1) without a noul; a
station is the default course and is not offered against. A tool-set rule outranks the packet's as now. Groups with
no threat in reach are not in the pass at all.

### 5. The post-pass

Worlds composed from the flagged states: world 1 is the plan standing now (the current hunts and stations plus the
rules' defaults); each deviation adds one flagged state or drops one current hunt; a state whose noul is under 0.5 is
not composed. The consequence line says who meets what at what metal and odds (turrets counted), which structures
are left within reach of an unanswered threat, what each move gives up (a station left, a hunt dropped), and the
hunters' speed against the quarry's. Capped at 8 (K-jev-one-question-over-joined-worlds-coordinates: the pick
diffuses past 8). The pick becomes the plan; the plan runs in code until a change.

### 6. Ask on change

The pre-pass runs when the threat picture changes: a new party, a party at a new place (400), a loss of ours, a
packet or rule change, a hunt ending. Otherwise nothing is asked and the plan stands. worlds-2 asked group_C 227 times
about one chase; the smoke's two passes cost about 150 ms each, so latency is not the constraint and repetition is
the waste.

## What is deleted

The per-group `candidates` in `worlds.rs` (course, rule, the nearest party's fight, walks, the way back) and the
gate over them (`gate_questions`, `follow_up`, mode `nouls` as built); `send_against` and `engage` as forced group
orders from the worlds pick; the executor's raider arm in `standing_order` (its rules become the pre-pass's defaults
and pruning). The builders' standing rules and questions are untouched. The walks to named places and the way back
are re-homed: a walk is a player's order (station) or the march, and the way back is a threat state
(`party_N.back_X`: group_X falls back to its named place) when the odds line reads against it.

## Validation ladder

1. The raid scenario, the code's defaults alone, lane on against off: the hunt catches a Tick and a Pawn (seconds
   to the first kill, extractors lost).
2. The raid scenario with Jev's passes: extractors lost no worse than the defaults, the picks' confidence, the asks
   per minute.
3. The pianist alone on standing-2's packet against medium (the micro agent's instrument): minute 13 is the bar.
4. A player game on standing-2's brief against hard_aggressive, read as worlds-2 was.

## Order of work

1. This note. 2. The hunt primitive and the raid scenario (the micro agent, in its worktree), landed with its ladder
step 1. 3. The pre-pass and post-pass in the brain with the generator deleted first (the main thread), against the
recorded worlds-2 moments offline and then ladder step 2. 4. Ask on change. 5. Ladder steps 3 and 4.
