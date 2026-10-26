# Companies: the cap, the muster and the army (2026-10-25 night)

The micro engine's group grows a size cap, a muster for its newcomers and a second level, the army of companies. From
the Glitters lobby game (`run/matches/1792971120-human`, the ledger row "human (Glitters 2v6)") and the user's question
afterwards: "What if we just add a cap on how many units can exist in a single group? What else would we have to add
for the rest to work cleanly?" The answer agreed: the cap as the forcing function (a group that cannot grow has to be
founded again at the plant, and a group founded at a plant is born as a body), plus the muster, the army as one actor,
a floor, and arrival and pacing by the body's share. The user: "Agreed, let's begin implementing and testing this."

## 0. What the game showed (the fault the design answers)

group_I was declared at 13:30 from one unit and fed by every plant's `produce ... group: group_I` with no `join_in`.
551 members passed through it in 30 minutes; the median one first appeared 6,931 from the anchor, 391 of 540 over
4,000; 174 never reached a slot. The lane's pacing counts only members within `exception_at` of a slot, so the anchor
marched at full pace with 1-20 formed members while 60-130 joiners trailed 2,000-5,500 behind it (formed share 0.12,
46 joining at any moment). `arrived` fired eight times with every joiner in `away` (one formed member at the way's end
was enough: `in_body > 0`); the hands held and asked, the next Go left the mass behind again; the anchor jumped over
800 in a second 18 times on re-aims and set-downs on the few at the front. Not the cap's size: five groups of 28 built
the same way would each have been a point unit with 27 stragglers. (Half the fault. The user's half, after companies-1: the mass
shared one movement domain, so the head's choke set the column for every unit behind it, and the members walked
thousands of elmos to make the shape. The cap is for that: small groups that move independently, §2.)

## 1. Where it lives: the engine, not the hands

The user, 2026-10-24: group controls built into the micro engine, not on top, drilled by the bench with no model. So
the lane's group gains the cap, the muster and the companies; the hands pass the cap (the player's setting) and the
plants' muster points; the duel bench drills it with `--cap`. The hands' own muster (`Group.waiting`,
`release_waiting`, the `join_in` walk, written 2026-10-16 for undeclared groups) dies for declared groups: one muster,
the engine's. (Undeclared groups, air and the hunts, keep the hands' `joining` as today.)

## 2. The engine's model

- `GroupOrder` gains `cap: usize` (0: no cap, one company, today's behaviour exactly), `musters: Vec<Vec3>` (where
  newcomers gather: the plants' beside-lane points; the drill's lab point) and `release: Option<usize>` (how many
  mustered newcomers walk together; none: the cap).
**Reworked 2026-10-25 night, after companies-1 (the user's correction).** The cap is there so that no group has to
inherit one movement domain: "The rules that govern movement of a small army don't scale to a large army -- so forcing
the groups to stay small and allowing them to move independently solves the problem. Different mechanics may be needed
to have the groups travel together, but that sounds like a later problem." The first build (04b70ee) had one **lead**
company walking the army's way and the others **slaved** to its anchor at offsets, in column when the lead's room read
narrow, their pace dragging the lead's: the Glitters coupling one level up. companies-1 showed it: the slaved anchors
swung up to 4,229 elmos at the lead's turns, the slaved companies formed 0 of 3 for four minutes. Rejected because the
head's geometry governs every company; the companies are independent instead. Deleted with it: the lead, slaved and
joining roles, the offsets, the abreast count, the slots' assignment, the drag between companies (`band_lag`,
`external_lag`), the carried pace of a slaved company, `WAIT_FOR`, `SLAVE_WITHIN`, `FOLLOW_STEP`.

- A declaration holds `companies: Vec<Company>`, each today's formation (`group::Group`) with its members and a role:
  - **Marching**: a full group with the army's order: its own way planned from where it stands, its own ground
    reading (room, files), its own pacing, contact and arrival. Nothing of another company's moves it.
  - **Mustering { at, since }**: a Hold about the muster point; it marches (the army's order, its way from the muster)
    when it holds `release` members, or `MUSTER_PATIENCE` (90 s) after its first member came with `MUSTER_MIN` (4) or
    more standing, or twice that with fewer.
- **The partition** (a membership change): a member no company has (1) goes to the nearest marching company under the
  cap whose anchor is within `JOIN_NEAR` (800) of it; (2) else to the muster company of the nearest muster point within
  `JOIN_NEAR` (founded on the first, with every pending newcomer near the same point); (3) else founds a marching
  company where it stands with the pending newcomers near it. A member gone leaves its company. With `cap` 0 everyone
  is one marching company, the musters unused: the group as before the cap.
- **A change of the cap** (the order's cap differs from the one the companies were made under) dissolves the companies
  and partitions every member afresh (companies-1: the player's `company: 0` left the standing companies split).
- **The merge**: two marching companies whose anchors stand within `JOIN_NEAR` of each other and whose members together
  fit under the cap merge into the larger, on the march or at a halt, one merge a tick (its slots re-dealt by the
  membership change). Companies stay apart only where the cap makes them.
- **The state**: `GroupState` at the top is the army: anchor, heading, pace, files and room the largest marching
  company's (the main body; with no cap, the group's); `formed` and `exceptions` the union; `engaged` any company's;
  `arrived` when every marching company has (a muster is not of the march yet). It carries `companies:
  Vec<CompanyState { role (marching, mustering), anchor, members, formed, exceptions, arrived, at (the muster point) }>`.
  Readers that know nothing of companies read the army as they read a group today.
- **Travelling together** is not built: companies sent to one point meet at the same chokes and the same goal on their
  own ways. The drills say how much that costs before anything is built for it.

## 3. Two rules changed whatever the cap (D24)

- A member that has never formed and stands within `exception_at` plus the formation's depth of its slot is Behind:
  it drags the anchor like any body member. Beyond that it is Joining and holds nothing (D22's rule, which stands:
  a joiner a map away froze the body for minutes before it). The depth is the layout's: ranks times `RANK_GAP`.
- Arrived needs the body (the members that are no exception) to be at least half the members, not one. (The vacuous
  case `in_body > 0` closed in migration-2-1 reopened one unit wide: one formed member at the way's end with a hundred
  in `away` read arrived eight times in the Glitters game.)

## 4. The hands

- Every declared group's order carries `cap` (the player's `company` setting for the group from the `lane` tool, else
  `COMPANY_CAP` 24; 0 lifts it), `musters` (the beside-lane points of the plants whose allowance feeds this group) and
  `release` (the plant's `join_in` when set, else none, so the cap). The `lane` tool's value for a group may be an
  object `{"rules": [...] | "raw" | "on", "company": N}`; a string or a list is the footwork alone, as today.
- Dies: `Group.waiting`, `release_waiting`, the wait branch of `admit`, the release loop in `keep_groups`, the picture's
  waiting words, the `join_in` range check's walk meaning (it is the release count now, 1 to the cap).
- Arrival: the army's `arrived`, read as today.
- The picture: the group's entry gains `companies`: "3 companies of up to 24: 24 (20 formed) at spot_148; 24 (24
  formed) at spot_150, 600 behind; 7 mustering beside plant_18450 (they leave at 24 or after 90 s)". The group is one
  actor to Jev and one name to the player, unchanged.
- A merge the player orders (`join_group_X`) appends the members; the partition places them.

## 5. The bench

- `duel --cap N` (0 default) on every drill; a circuit's or gather's lab point is the muster when the force is fed.
- The drill report adds `companies` (how many at the end), `company_formed` (the mean of the companies' formed shares
  while marching) and `muster_waits` (the median seconds a fed unit waited at the muster).
- Arms: walk-48 at cap 0 / 24 / 16 (formed share, arrival spread, width at contact); circuit pawn12feed24 at cap 0 /
  12 (joined per company, left behind, lap time); assault mix36 at cap 12 / 0 (ramp, width at contact, deaths).
- The choke arm (the rework): 60 or more through a choke on a winding way, cap 0 against 24; how far the members
  walk off the route to make the shape, the arrival spread, the formed share.
- Bars: a company's formed share at or above the twelve-member walk's (0.9); the circuit's feed arrives in companies
  with none left behind; the army's width at contact at cap 24 at or above one block's; nothing of the standing
  drills' results moves at cap 0.

## 6. Chunks, in order

- **A** the engine's two rules (§3), tests, the standing drills unchanged.
- **B** the companies (§2 without the muster): cap, partition, roles, offsets, pacing, arrival, floor, state; tests;
  the walk and assault arms.
- **C** the muster: `musters`, `release`, the Muster role, the circuit arm with `--feed`.
- **D** the hands (§4): the settings passed, the deletions, the picture, the lane tool; tests.
- **E** a live game against BARb, then the lobby.

## 7. Open for the user

- The cap's default: 24 (six files by four ranks, under 300 deep) against 32.
- Contact per company (built) against contact as one army (every company takes the arc together).
- From companies-2: a muster at a slow plant releases ones and twos at the patience, each walking the map alone;
  and the picture's company words read to the player as stragglers holding the march (it lifted the cap twice).
- (Settled by the rework: abreast counts, gaps and the slaved anchor's pathing died with the slaving.)

## 8. Built (2026-10-25 night)

Chunks A to D are built in one sitting; E (the live game) follows.

- **A (D24).** The two rules as §3, in `Group::tick`; two unit tests. Regression (d24-plain12, d24-a7-behind8-1800-churn5,
  d24-circuit-pawn12feed8-on, d24-assault-mix12-hlt2llt2-around-on): plain 13 s at formed 0.93-0.95; the rear-party
  arm 7 / 8 / 9 s, anchor back 0; the fed circuit 180-183 s at 0.78, 8 of 8 joined; the detour assault 83-84 s at
  0.93: all as D23's.
- **B and C (the engine).** `crates/micro/src/army.rs`: `Army` (the partition, `ensure_lead`, `assign_slots` by nearness,
  `merge_floor`, `offset`, `abreast_for`, `compose`, `muster_due`), the roles, `CompanyState`; `GroupOrder` gains
  `cap`, `musters`, `release`; `GroupState` gains `companies`. The lane's `group_orders` is a per-declaration loop over
  companies round one `formation_tick` (the old per-group body, unchanged); the ground is built once per declaration
  and shared. `Group` gains `slave`/`unslave`, `set_members`, `set_kind`, `lag`, `band_lag`, `median_off`,
  `external_lag`, `layout_depth`, `files`. Three things the first builds taught, all in the synthetic march test
  (`a_capped_group_marches_as_companies`) before the drills: (1) a company just slaved has every member in `away`, so
  the lead's drag must read a slaved company by a wider band (`band_lag`: the worst lag of any member within the
  joiner's reach of its slot) and release at `rejoin_within`, not `drag_from`, or the company trails at 100-128 for
  ever at the same speed; (2) the carried lead (D20) at a held pace put the lead company's members 186 ahead of their
  slots (they catch the carried point when the slot is slower than they are), so the carry is scaled by the pace
  share's fourth power; (3) a company is founded on every newcomer that would pick it, not on the first, or its anchor
  stands on one unit. A joining company within `WAIT_FOR` (1,600) of its point drags the lead too: a company of the
  same speed never catches a marching army otherwise. The muster's patience is 90 s with four or more standing, 180 s
  with fewer (the first fed circuit at a cap of 12 had the lab make 11 in two minutes).
- **The bench.** `duel --cap N`; a gather's or circuit's rally is the muster; the report's `companies` and
  `company_formed` columns; the group debug (`WITHIN_REASON_GROUP_DEBUG`) says FOUNDED / RELEASE / SLAVED / MERGE.
  Measured (cmp2, cmp3; Quicksilver, lane on, ranks of six):
  - walk of 48 Pawns, 1,200: cap 0 arrives 12 / 13 / 13 s at formed 0.87; cap 24 (two companies) 13-16 / 15-17 / 17-18
    at 0.75-0.78, the companies' own 0.71; cap 16 (three) 14-16 / 17 / 18-19 at 0.73-0.76. The strip is 640 wide, so the
    companies fall into column (abreast 1) and the rear company's lag holds the lead: on a narrow strip one block
    does better by four seconds and a tenth of formed share. Open for the drills on open ground (abreast).
  - the fed circuit, twelve Pawns with twenty-four fed: cap 0 lap 177 s at formed 0.74, 13 joined (median 50 s), 1 left
    behind; cap 12 lap 206-208 s at 0.84, 18 joined (median 70 s), 0-1 left behind, the companies' own 0.80. The
    musters released 8-9 at 90 s and the army waited about 30 s for each company to come in (the lap's price), which
    is the design's intent: the feed arrives as bodies, not one at a time.
- **D (the hands).** The order carries the player's cap (`lane` tool, object form `{"rules": ..., "company": N}`, the
  default `COMPANY_CAP` 24, 0 lifting it), the plants' beside-lane points as `musters` (kept up in `keep_groups`,
  `Pianist::musters`) and the plants' smallest `join_in` as `release`. Died: `Group.waiting`, `release_waiting`,
  `admit`'s wait, the release loop, the picture's waiting words, the `join_in` test. `join_in` is 1 to 64 now, the
  count that walks together. The picture's group entry gains `companies` (`company_words`). Tests: bot 144, micro 86.

- **E (live, companies-1, WON 24:33).** No group reached the cap; every company came from a muster, and the musters'
  companies of 1-4 slaved to the lead and stayed separate on the march (the floor merges at a halt only): group_B, the
  main body, stood in 2-5 companies nearly all its life at a mean of 11 members. The lead's room read 100-300, so the
  companies stood in column, each straight behind the lead's heading by its depth plus the gap: the rear companies'
  slots swung hundreds to thousands of elmos at every turn of the heading (span up to 4,229; formed 0 of 3 from 20:30),
  the Glitters shape again at a small scale. The player's `company: 0` at 19:12 left the existing companies split. Ledger
  row companies-1.

- **The rework (independent companies, after companies-1).** Built as §2 says: `Role` is marching or mustering, each
  company ticks as a group of its own with the army's order (`Company::order`), `Army::merge` any time, a new cap
  partitions afresh, the top state the largest marching company's; and the muster is for newcomers to a marching army:
  the declaration's members (the first partition, or the first after a new cap) found marching companies wherever they
  stand (the 60-unit lap at cap 24 mustered its whole spawned army by the lab before that). Deleted from `group.rs`:
  `lag`, `median_off`, `band_lag`, `external_lag`, the carried pace, `slave`/`unslave`/`slaved`, `set_kind`,
  `set_members`, `files()`; `layout_depth` private again (and out of `body_depth`'s doc comment, where the first build
  had put it). The carry's fourth power stays: a held pace is any drag's, not the slaving's. The picture's companies
  words: each marching company's size, formed, place and distance from the main body; a muster's release is the plant's
  `join_in`. Measured (ledger ind-*; Quicksilver, lane on, ranks of six):
  - 60 Pawns on the circuit's first leg (3,657 as the crow flies, 4,442 on foot): cap 0 arrives in 58-59 s at formed
    0.80, member backtrack 0.013; cap 24 (three companies) 56.6-58.5 s at 0.94-0.96, backtrack 0.002.
  - 36 Pawns and 24 Stumpies on the same leg: cap 0's group arrived about 74 s at 0.76, one Stumpy left behind as a
    straggler in both reps (the drill waits for it to the time limit); cap 24 68 s at 0.94, nobody left.
  - the lap (15,309 on foot) with no feed: 60 Pawns cap 0 timed out with two left behind (the rest in at 188-191 s,
    0.945), cap 24 195 s at 0.92, nobody left; the mix cap 0 timed out with four left (226-234 s, 0.94), cap 24 247 s
    (238-245) at 0.91, one left.
  - 48 Pawns on the 640 strip: cap 24 16.5 s at 0.93 (the slaved build 13-18 s at 0.75-0.78; cap 0 12-13 s at 0.87).
  - the fed circuit (12 Pawns, 24 fed) at cap 12: all 24 joined (the slaved build 18), median 43 s at the muster, but
    each released company walks the army's whole route from its first point, a lap behind the army: the drill ends at
    519 s against the army's 183. The travelling-together problem, first shape (§9).
  - regression at cap 0: plain 12 in 13-13.8 s at 0.93-1.0, the fed circuit (8) 178 s at 0.81: unchanged.

- **Live, companies-2 (WON 27:01).** While capped (to 20:02) the main body's largest company stood formed 0.73, the
  group over the game 0.44; the player lifted the cap twice, reading the picture's far companies as stragglers holding
  the march, and the one formation that followed read 0 of 31 formed at times. The companies beside the main one were
  ones and twos released from far musters at the patience (slow plants never fill a release of 24), walking alone.

## 9. Open after the build

- Travelling together (the user: a later problem). Its first shape is the feed: a released muster takes the army's
  whole route from its first point and walks a lap behind it on the circuit, where a joiner should meet the army (the
  user, 2026-10-24: the trailing units "all inherit the full order set -- running a lap around the map behind the army
  -- rather than directly meeting the army halfway around the circuit"). On a one-point order, the hands' usual, the company walks straight to the goal.
- The cap's default (24) against the drills: on the 640 strip one block of 48 is still 4 s faster.
- An assault with companies against a defended position: companies reach contact one after another.
- The ground is the declaration's, built from every member's class: a company of bots is priced by the tanks' room.
- From companies-2: a muster at a slow plant releases ones and twos at the patience, each walking the map alone;
  and the picture's company words read to the player as stragglers holding the march (it lifted the cap twice).
- (Settled by the rework: the column's price, the slaved company's way, `WAIT_FOR`, the fold into the lead and the cap
  change's re-partition.)
