# Target design: the body's shape (stage 1 of the micro answer)

**The ruling (the user, 2026-09-28).** "The answer in human play to most of this is various pitches and concaves of
a line formation, and the fact that we don't do that in practice is part of the problem." Agreed as stage 1 of the
micro answer after the TAS study (`docs/studies/2026-09-28-tas-e3.md`) and the body-size and splash study
(`docs/studies/2026-09-28-body-sizes.md` in the main tree, commit 164a718 and after): a Stout ball loses nothing to
itself at any size but loses to a few area units, worse the bigger and tighter it is (25 Stouts against 4 Fatboys
-0.55 at 56 spacing, -0.22 at 160; 21 against 5 Bulls -0.34 and +0.01); the pros' typical largest body of a type is
a dozen, eight Stouts; the TAS found the front's target choice and the ranks' blocking small on E3 and the body-level
decisions large. The user's first-order heuristic for a body's size: unit range against ball diameter.

**Status (2026-09-28).** Branch `tas-micro`, worktree `../bar_bots-tas`, after phases 1-2 of
`docs/design/2026-09-28-tas-micro.md` (b3562ad, 20834a6). Step 1 built: `UnitDefInfo::blast_radius` (the shim's
`WeaponDef_getAreaOfEffect`, the largest over the ordinary weapons), the record's `unit_defs`, `Stats::area` in the
micro's `View` (the duel's and the brain's), the picture's unit line ("its shells hit everything within 150 of where
they land"). **Correction to the design:** the engine's area of effect is a radius, half the unit file's
`areaofeffect` (`WeaponDef.cpp`: `scaleValue(0.5f)`); the Fatboy's is 150, not 300 (the duel prints it at every start:
armfboy 150, armbull 65, armstump 24, armflash 4); the test is a wire round trip in `bot-protocol` and that line. Steps 2 and 3 built together (the
generator replaced `form.rs`'s old API in place, so the lane moved with it): `form::contact` (the target: the host's,
else the one the body had, else the nearest soldier no other building covers, else the nearest soldier, else the
nearest building; its party chained within 150, fitted with a segment), `form::contact_slots` (slots on the near side
of the points within `R - 20 - spread` of that segment: an arc on a point, a rank with curled ends on a line; the
pitch of nine that leaves fewest slots in another building's reach; a count past the near half continues round the
sides, past the full ring closes the spacing to fit, down to 64), `form::spacing_for` (64, or twice the largest blast
radius about within 1,500, at most 160), `form::arc_capacity`; the cut at 12, the rank at the standing front's depth
(H-MICRO-FORM-FLANK's gate) and the `Close` step are deleted. The lane: `Slot` (walk to the slot: a Move against
area, a Fight otherwise), `Wait` (at the slot, nothing in reach), `Stand`; a unit keeps the slot it was sent to until
it gets there or the target changes. H-MICRO-STEP-OUT: a unit inside the reach of a building outside its target's
party with nothing in its own reach, not walking out already, steps out to the reach plus 40. Two findings while
building it (probe batches `pr-*`, `pr2-*`, `pr3-*`, `docs/studies/2026-09-28-body-shape.md`): a slot pushed out of a
turret's reach leaves its unit out-ranged in the target's reach (dropped); slots re-dealt every tick re-ordered the
Blitzes twice a second (the unit keeps its slot now).

## What exists (read first)

`crates/micro/src/form.rs` (H-MICRO-FORM, H-MICRO-FORM-FLANK; `docs/design/2026-09-25-formation-micro.md`,
`docs/design/2026-09-25-formation-as-a-transform.md`): who makes a body (one group order, chained within 400, raiders
apart from line units, cut across the heading into pieces of at most 12), the march slots (ranks behind the heading),
the engaged slots (one rank across the enemy's direction at the depth of the standing front; a unit at its slot with
nothing in reach closes on the nearest enemy to 0.9 of its reach), who takes which slot. `lib.rs`: the lane, FLEE,
KITE, FAN, HUNT as per-unit overrides. The duel harness: `--formation`, `--spacing`, `--lane`, `--scenario` with
`--script`, `--seeds`, the per-second log; `run/duel_ab.py`. The retired H-MICRO-FOCUS (a body target) and
H-MICRO-SPREAD (an order-level spread) in `docs/heuristics.md`: what was tried and why it went.

## The target

1. **A body has a shape and a target.** `form.rs` becomes the whole of body control: a formation is slots around a
   reference, from four parameters: spacing `s`, width and ranks, curvature, pitch. Pure geometry, as now.
   - **March**: a rank (or ranks) across the heading at spacing `s` (today's march, parameterised).
   - **Contact against a point** (a turret, a tier-2 unit, a small party): a **concave** whose slots lie on an arc
     centred on the target at radius `R - m` (`R` the body's shortest reach, `m` a margin of 20), so every slot has
     the target in reach at once. The arc's half-angle is what the body's count needs at spacing `s`; a body whose
     count exceeds the arc (about `pi * (R - m) / s` slots on a half-circle: nine Blitzes at 65, seventeen Stouts)
     is two bodies on two arcs or two targets, never a queue behind one arc. This replaces the cut at 12 with the
     user's range-against-diameter number, and the "closes on the nearest enemy" clause with "your slot".
   - **Contact against a line** (a party wider than it is deep): a rank across its direction at the standing
     front's depth (today's FORM-FLANK), with the same spacing rule.
   - **Pitch**: the rank or arc turned by an angle about the target for a flank or a kite; the lane's KITE sets the
     pitch and the stand-off radius (`R - m` grows to `R_theirs + m` when the body out-reaches the target).
2. **Spacing from what the enemy shows.** `s` = the hull clearance (two hulls, about 60-70, the pros' 67-70) when no
   area unit is in sight or remembered within 1,500; wider against area, starting at the bench's 160 against a
   Fatboy or Bulls and searched in the harness (the spacing arms of the body-size study). The unit's area comes from
   the definitions: add `blast_radius` (the largest weapon's area of effect, the shim's `WeaponDef_getAreaOfEffect`)
   to `UnitDefInfo` in `crates/bot-protocol` and the shim (`crates/ai-shim/src/engine.rs` reads it for `blast`
   already), default 0 for old records; the micro's `View` exposes it. The glossary line for every unit gains the
   area (the main tree's brief now carries the table; the picture should say it too: a one-line change in
   `picture.rs` `short_words` if the worktree has it, else noted for the main tree).
3. **Who fires.** The per-unit exceptions stay: FLEE (out of a reach the group was not priced against), KITE, FAN;
   one new one from the TAS script: a unit inside a static reach with nothing of its own in reach steps out
   (never idle under a turret). The body's target is the host's (the pass's whole-group attack names the party; the
   stage-2 planner will name the phase's target), and the slot generator takes it; FOCUS is not rebuilt.
4. **The judge.** Each shape is judged before a game in the duel harness:
   - the E3 cuts (`run/scenarios/2026-09-28-player-14-e3-917*.json`) and the F3 cut, our side under the lane with
     the new shapes against the recorded track and the reacting director, `--seeds` and 8 reps, beside the phase-1
     attack-move margins and the TAS's;
   - the tier-2 arms of the body-size study (Stouts against Fatboys and Bulls at 5,400 and 1,800) with the shape on
     and off (`run/duel_ab.py`), the muzzled share and in-reach share beside the margin;
   - a mirror at 24 a side to show the shape costs nothing against a line of the same reach.
   The numbers that must move: in reach while engaged against the Fatboys (0.63 at 56 spacing today), the margin of
   the E3 r1250 cut under the lane (-0.320 today), the friend-on-the-line share.

## Order of work (each ends with a commit here and a line in the status above)

1. `blast_radius` through the protocol and the shim; the `View` field; a test that the Fatboy's is 300.
2. The shape generator: concave, rank, pitch, spacing by area, the arc-count split. Unit tests on the geometry (a
   body of 17 Stouts on a target: every slot within reach; 25 split into two arcs).
3. The lane uses it: contact against a point takes the concave; the never-idle-under-a-turret step.
4. The harness runs above, the table in `docs/studies/2026-09-28-body-shape.md`, and the heuristic rows
   (H-MICRO-FORM amended, a new row for the spacing rule and one for the arc count) with claims in
   `docs/knowledge/` (see `docs/README.md`), written when the numbers are in.
5. `run/tas_diff.py` (the TAS design's step 4) if the above is done: the lane's per-tick orders on a scene beside a
   script's, by second and unit.

## Constraints

Those of `docs/design/2026-09-28-tas-micro.md` (this worktree only; ports 9500-9600; `--parallel` at most 2; never
kill the engine or the arena; never `pkill -f`/`pgrep -f`; never touch `~/.local/share/BeyondAllReason`; never run
`run/install_to_bar.sh` or the arena; the main tree read-only; never `git add -A`, never add `run/data` or
`run/engines`; commit at each step's end with the two attribution lines). The main tree's brain (`crates/bot`) is
not the target here beyond the protocol field and the picture's area words; the stage-2 planner is a separate
design in the main tree (`docs/design/2026-09-28-engagement-plan.md`). Delete what a shape replaces (the cut at 12,
the closes-on-the-nearest clause) rather than keeping both.
