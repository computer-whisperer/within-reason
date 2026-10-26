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
the same way would each have been a point unit with 27 stragglers.

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
- A declaration holds `companies: Vec<Company>`, each today's formation (`group::Group`) with its members and a role:
  - **Lead**: the army's way is its way (the order's route, the ground, the pacing). One per army: the largest company
    among the lead and the slaved ones; when it dies or merges away, the next largest.
  - **Slaved { slot }**: its anchor is the lead's anchor plus the slot's offset, snapped to the ground, every tick;
    no way of its own, no pacing of its own. Its members' slots hang from its anchor; its exceptions, its been-formed
    set and its arrival are its own.
  - **Joining**: a company on its way to the army (a released muster, a company founded far from the lead): its own
    route to its slaved point (the lead's anchor plus its offset), re-planned when that point has moved more than
    `FOLLOW_STEP` (150) since the route was set, a FightTo. Slaved once its anchor is within `SLAVE_WITHIN` (600) of
    the point.
  - **Muster { at, since }**: a Hold about the muster point; releases to Joining when it holds `release` members, or
    `MUSTER_PATIENCE` (90 s) after its first member came with `MUSTER_MIN` (4) or more standing, or twice that with
    fewer (a trickle goes in fours rather than waiting forever; the first fed circuit at a cap of 12 had the lab make
    11 in two minutes, so a muster full at the cap would have waited the whole lap).
- **The partition** (a membership change): a member no company has (1) goes to the company whose anchor is within
  `JOIN_NEAR` (800) of it and which is under the cap; (2) else to the muster company of the nearest muster point within
  `JOIN_NEAR` of it (founded on the first); (3) else, with no muster near, to the smallest company under the cap, or
  founds a company where it stands (a Joining one; a lone far newcomer still walks, as a company of one, to the army).
  A member gone leaves its company. With `cap` 0 everyone is one company (1 and 3 collapse to it).
- **The offsets** (the army's shape): companies abreast first, the lead in the middle, slot k at a lateral offset of
  `±ceil(k/2) × (company width + COMPANY_GAP)`, left then right; the fourth and on in a second line behind by the
  company's depth plus the gap. Company width is `files × spacing`; the lead's `room` says how many stand abreast:
  with room for fewer, the rest fall into the second line (a narrow way puts the army in column). The user's measure:
  how much of the army can engage at once while keeping its speed; abreast where the ground allows.
- **The army's pacing**: the lead's drag law one level up: a slaved company's body members behind their slots drag
  the lead's pace as the lead's own do (the same `drag_from` to `exception_at` ramp over the worst lag of every
  lead/slaved company's body); a Joining company holds nothing until it is slaved; a Muster holds nothing.
- **Arrival**: the lead at its way's end and every lead/slaved company arrived by its own rule (§3); a Joining company
  on its way blocks it (the army has not gathered); a Muster does not (it is not of the march yet).
- **Contact**: per company, as today per group (its own centre, its own arc, its own `stopped`). A lead in contact
  stops its anchor and the slaved anchors freeze with it; a slaved company in contact drags the lead through its lag.
- **The floor**: at a halt (the order a Hold, or the army arrived) two lead/slaved companies both under half the cap
  merge into the larger (its slots re-dealt by the membership change); never on the march, never above the cap.
- **The state**: `GroupState` at the top is the army (anchor and heading the lead's, `formed` and `exceptions` the
  union, `arrived` the army's, `pace` the lead's, `files` the lead's) and gains `companies: Vec<CompanyState { role
  (a word: lead, slaved, joining, mustering), anchor, members, formed, exceptions, arrived, at (the muster point or
  the slot offset's target) }>`. Readers that know nothing of companies read the army as they read a group today.

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
- The picture: the group's entry gains `companies`: "3 companies of up to 24: the lead, 24 formed at spot_148; the
  second, 24, 310 to its left, 2 behind; the third, 11, joining from plant_14293, 1,900 to go; 7 mustering beside
  plant_18450 (they leave at 24 or after 120 s)". The group is one actor to Jev and one name to the player, unchanged.
- A merge the player orders (`join_group_X`) appends the members; the partition places them.

## 5. The bench

- `duel --cap N` (0 default) on every drill; a circuit's or gather's lab point is the muster when the force is fed.
- The drill report adds `companies` (how many at the end), `company_formed` (the mean of the companies' formed shares
  while marching) and `muster_waits` (the median seconds a fed unit waited at the muster).
- Arms: walk-48 at cap 0 / 24 / 16 (formed share, arrival spread, width at contact); circuit pawn12feed24 at cap 0 /
  12 (joined per company, left behind, lap time); assault mix36 at cap 12 / 0 (ramp, width at contact, deaths).
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
- How many abreast at most (three, then a second line) and the gap between companies (one company width's half?).
- Contact per company (built) against contact as one army (every company takes the arc together).
- The second level's own pathing: a slaved anchor is snapped, never pathed; a slaved company behind a wall the lead
  skirted is left to its members' own paths. The drills will say whether that is enough.

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

## 9. Open after the build

- The column's price on a narrow strip (above). Whether a slaved company should keep its own way along the lead's
  (pathed, not snapped) when the ground narrows.
- The drills on open ground, abreast; an assault with companies against a defended position.
- The lap's price of waiting for each company: `WAIT_FOR` and the muster's patience are the levers.
- From companies-1: a column must trail along the lead's way, not hang straight behind its heading; a company that
  fits in the lead under the cap folds into it when it reaches it (march or halt); a change of the cap re-partitions
  the companies standing, not only the newcomers.
