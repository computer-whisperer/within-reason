# Facing apart from travel, and formations in the ground (2026-10-31)

The user, after the apex-v0409-comet-sym0-opus-1 movement review: superposed companies shuffle badly at trajectory
changes "if the ranks are interspersed with another group", and "there is no reason we have to constrain our ranks to
move only perfectly forward with regard to the placement grid". Agreed and ordered the same day: "proceed with those
changes and tests", then an Apex game on the full stack. Two changes to the micro engine's group (`crates/micro`),
judged on the drill bench, nothing in the hands.

## 1. What the engine does today

`Group::tick` (`group.rs`): the anchor walks the planned way at the pace; `heading` is recomputed every tick as the
direction from the anchor to its next target; the slots are laid in the heading's frame (`form::slots`: six files
across the heading, ranks `RANK_GAP` behind, the front rank centred on the anchor); the deal of members to slots is
sticky by index and re-dealt (`form::assign`, least squared travel) only when the set of members changes or at a
corner of `REDEAL_TURN_DEG` (20) or more, where the anchor is first set down on the body (`settle_on_body`). So
between corners every bend of the way, and the straightened way bends at every cell move the line of sight could not
cut, rotates the whole grid about the anchor: a rear slot of a 300-deep block moves about 100 for a 20-degree
drift, and every member walks a different vector to catch its slot. Two interleaved blocks rotate about two anchors
and cross. That is the shuffle.

The way (`ground.rs`): one Dijkstra field per leg over 16-elmo cells priced by clearance for the block's half-width,
structures of both sides and wrecks overlaid as blocked cells, descended from the anchor and straightened. Other
formations are nowhere in it; the only interactions are `keep_off` at the way's end (§5.3 of the contact cleanup) and
the companies' merge under the cap. The reviewed game: 1,746 company-pair-seconds of two marching companies within
190, every pair over the cap; two companies of 22 marched together 327 s unbroken; a third company at an end two
others held never settled (spot_46 13:45-14:15, 8-21 of 24 formed, its placed end moving about).

## 2. Facing apart from travel

`facing` is a second unit vector of the group, the frame the slots are laid in. `heading` stays the direction of
travel and keeps every use that is about travel: the lead that carries each slot ahead (`carried`), the drag's
"behind" projection, `heading_ref` for the lead's learning, `GroupState::heading` for the record and the picture.
`facing` takes the uses that are about the ranks: `form::slots`, `anchor_for` (the set-down), the footprint (its
rectangle and `keep_off`'s beside and behind).

`facing` changes at four moments and nowhere else:
1. A new order for a body the anchor has lost (`reaim`, D23): facing toward the first point, as the heading is.
2. A corner the anchor passes whose turn against the **facing** (not the previous heading) is `REDEAL_TURN_DEG` or
   more: the turn procedure as now (set down on the body, re-deal), the facing set to the new heading. Between such
   corners the facing stands and the slots translate with the anchor: a course change is one displacement vector for
   every slot, nobody crosses anybody.
3. The close and the guard at contact (§5, §5.2): facing toward the target, as the heading is there.
4. A hold or an unpaced walk (`!march`) whose anchor moves to the end: facing toward the end once, as the heading.

The cumulative-drift check is 2's "against the facing": a way of many small bends re-faces once their sum reaches the
threshold, and the threshold is the drill's arm (`WITHIN_REASON_REFACE_DEG`, default `REDEAL_TURN_DEG`'s 20 at first;
60 and 120 measured on the circuit). Units move at one speed in every direction, so a block translating at an angle to
its facing loses nothing on the march; facing matters at contact, which 3 covers.

Judged on: the circuit drills (plain laps and the fed circuit: lap time, formed share, speed share, unchanged or
better), the pair drill (two blocks sharing a route: overlap seconds, the second's arrival), the assault and the
contact slice (the front still faces the enemy at contact: margin within noise).

## 3. Formations in the ground

Today the structures overlay (`ground::structures`) is AND-ed into a group's ground at build. Formations move, so
they go in at plan time instead, without rebuilding the ground: `Ground::way_found_off(from, to, blockers, dear)`
copies the costs once per plan and stamps every cell a standing formation's footprint covers (grown by
`KEEP_OFF_MARGIN`) `IMPASSABLE`, and every cell a marching formation's covers with `FORMATION_COST` added, then runs
the field as before. (As first built the route's end also refused a cell inside a standing footprint,
`Ground::stand_off`; it chose another free cell than `keep_off`'s beside and the end moved between the two at every
re-plan, foot1 / foot2, so the end is `keep_off`'s alone.) `Group::plan` takes the other formations' footprints (set by the lane each tick before `tick`, the same vector
`keep_off` is given, less this formation's own and its own army's musters), and a plan is marked dirty when the set of
standing footprints that intersect the remaining way changes (checked every `REPLAN_CADENCE` ticks), so a block that
stops on the way is walked round within seconds. The copy is 200k cells of `u32` on Comet, under a millisecond; plans
happen on orders and rebuilds, not every tick.

`keep_off` stays: it is the end's rule within a tick and the planner's answer arrives a plan later; with the
stand refusing occupied cells the two agree. The merge stays.

Judged on: the pair drills (walk, follow, attack: the second block's end beside the first by the planner, overlap
seconds not above pair2's 35 / 0 / 40, the second's arrival not later than 49 / 54 / 59); a new `pair` arm with a
third block (`--behind` twice) for the spot_46 shape; the single-formation arms unchanged.

## 4. Order of work

1. Baseline on the current binary: the circuit arms (pawn20 nofeed, pawn12feed8), the pair arms (walk, follow,
   attack), labels `face0-*`.
2. §2, tests in `group.rs`, the same arms as `face1-*` with the re-face threshold at 20, 60, 120.
3. §3, tests in `ground.rs` and `group.rs`, the same arms plus the three-block pair as `foot1-*`.
4. Ledger rows, H-MICRO-FACING and H-MICRO-GROUND-FORMATIONS in `docs/heuristics.md`, the claim in
   `docs/knowledge/formations.md`, the formation-drills design §2 amended.
5. One Apex game on the flagship line (`apex-v0409-comet-sym0-opus-2`): the full stack including H-HANDS-ARRIVAL.

## 5. Results

**face0 (the binary before either change, 2026-10-31 02:20) / face1 (§2 at 60 degrees):** plain circuit (20 Pawns, 3
reps) 177 / 179 / 181 s at formed 0.95 → 182 / 184 / 186 at 0.93; the same at 20 degrees 179 / 182 / 184 (0.93) and
at 120 degrees 182 / 185 / 186 (0.92), so the cost is the facing held between corners, not the threshold; the fed
circuit 179 / 180 / 181 → 180 / 181 / 183 at formed 0.82 both, the join median 59 → 51 s; the pair drill (8 reps):
walk overlap 34.5 → 32.1 s, the second's arrival 49 → 46 s, 433 → 435 apart; follow 0 → 0.4 s, 56 → 51 s, 432 → 432;
attack 43.0 → 41.9 s, 57 → 57 s, 229 → 158 apart, taken 7 of 8 → 8 of 8. Kept at 60: about 3 % of a lap on a
corridor circuit for the translation the interleaving needs; the game judges the rest. A mitigation if it bites in
play: re-face when the ground's room at the anchor is less than the skewed block's lateral extent.
