# One decider: delete the heuristic bot, keep the player, order the army by worlds (target design, 2026-09-25)

Written before the first deletion, as the ground truth that survives compaction. The reading it rests on is the
standing-2 row of `docs/experiments.md` and the layer inventory of the same evening: six layers touched a group each
second (footwork, group bookkeeping, the standing executor, the Lua policy, Jev through the two-level menu, five
player channels), and in standing-2 two deciders took a different choice on one actor within 20 s 25 times on group_I
and 25 times on one constructor. The user (2026-09-25): "this is building up to too many rules layered on each other
when we haven't properly pulled out the bad ones ... let's work on deleting old obsoleted systems. I think most of the
old heuristic system can go now that we are using jev, and we should definitely stop using the known-bad jev menu."
On the two questions asked: the Sonnet commander modes go with the deciders, and the pianist alone (Jev, no player) is
the arena instrument for the micro agent's A/Bs at cents a game. "Keep the build order search as one of the player's
tools." "Don't remove the player system. That's the most effective part of this."

**Status.** §1 done 2026-09-25 evening in four commits (60b81a0 deciders, 17ae210 modes, 9669036 policy and family,
3fe5a1f the registry and docs); §2 done (e49b9a7 `--packet`; smoke-pianist-1: the pianist alone played 17 minutes against
easy for $0.07 in 87 s of wall time); §3 built the same night (the factory's own group, `produce` ... `group`, the `join`
rule); §4 built (H-HANDS-WORLDS, `pianist/worlds.rs`, the default mode; worlds-smoke-1: 80 questions of 2-4 worlds, no
two deciders on one actor) with the builders' retreat hold; then (the user, watching worlds-smoke-1: the old `where`
questions were still asked for groups without rules) every group through the worlds question with walks to the named
places as candidates and the groups' own questions deleted, and the builders' `where` split into `where_build` and
`where_walk`; `run/worlds_replay.py` replays recorded worlds questions (§5). worlds-1 (the player, hard_aggressive)
played on the build before the last two changes; worlds-2 is their live test. worlds-2 (2026-09-26 morning, the same brief): LOST 31:01, 532 questions, every group play by the picked world; lost at the E3/F3 nest to a lone-unit fight whose line had no turrets, the player's ignore and never rules unread by the candidates, and 15 tool sets refused a turn late; the four fixes (turrets with a party in every odds line, the rules gating the fights and a chase ending at a never place, the tool's check in the turn with any walkable spot a place) are in before the noul-gate arm (nouls-1). nouls-1 (the gate as a filter over the same candidates) LOST 22:01 and changed nothing but the ask count. §4's mechanism is superseded 2026-09-26 by `docs/design/2026-09-26-threat-response.md` (H-HANDS-THREATS): enumerated by threat, the plan standing in code, the rules as defaults and pruning; §1-3 and the builders' side stand.

## 1. What is deleted

Delete first, then rebuild; morphing keeps hidden assumptions. Each line names what the code assumed, so the reader can
check the assumption is gone.

**The heuristic bot's deciders** (`crates/bot/src/brain/`): `army.rs`, `raid.rs`, `contact.rs`, `squads.rs`,
`march.rs`, `tier2.rs`, `bases.rs` (the enemy-base guesses; the pianist shows evidence only, H-HANDS-ENEMY-EVIDENCE),
the deciding half of `scout.rs` (`survey_spots` and `spot_seen` stay), `run_economy` and its rules in `economy.rs`
(the site and spot helpers stay: `claim_spot`, `build_site_for`, `place_planned`, `spot_occupied_radius`, the yard
lanes), `protect_commander`, and in `planner.rs` the plan's execution (`run_planner`, `plan_step`,
`plan_factory_batch`, `step_begun`, `assist_over`, `unqueue_step`, `install`): the search (`buildorder_game`,
`scenario`, `search`, `publish_plan_context`) stays as the player's `plan` tool. The `think` branch that ran them, the
`rules:` line of `bot.log` and the arena's firing table go with them. `journal.rs` keeps the pianist's notes and loses
the waves and squads. `micro.rs` loses `note_commitments` (commitments came from the raid, the contact answers, the
waves and the squads; under the pianist the lane prices nothing that way).

**The commander and strategist modes**: `Mode::Commander`, `Mode::Strategist`, their prompts and briefs, `--commander`,
`--commander-each`, `--strategist` in the bot and the arena, the commander's tools in `mcp.rs` and their fields in
`shared.rs` (raid, scout, wave directives), `briefing.rs`'s commander briefing (`read_directives`, `track_losses`,
`track_enemy_buildings`, `trigger` stay: the player's wakes and the record read them), `seats.rs` only where it served
those modes. `Mode::Player` is the one mode; `--player` implies `--pianist`.

**The Lua policy**: `pianist/policy.rs`, `policy_pass`, `apply_policy_changes`, `Menu.policy`, `PolicyStats` and its
report line, the `policy` tool and `PolicyChange`, `--policy`, `POLICY_PROMPT`, `docs/design/2026-09-22-policy-replay.md`
marked superseded. It assumed Opus would write Lua cheaper than prose; measured the other way (the standing arc), and
unused in the last three games.

**The two-level menu** (`pianist/family.rs`, `family_on`, `--family`, the `raw` log field): registered as overshooting
after family-1 and never measured better since. Standing games ran it by default.

**The registry**: every row whose code is deleted is marked `retired (2026-09-25, the heuristic bot deleted)` in one
pass, with the measured ones kept in `docs/knowledge/` as claims (H-ARMY-MARCH, H-ARMY-REGROUP, H-ARMY-GATE-ALL-SEEN,
H-MAP-TERRITORY, H-ECO-BASE-LAYOUT, H-ARMY-STAGE, H-T2-GATE: their numbers stay as knowledge about the game, not as
rules). The registry then says what runs in a pianist game and nothing else.

What is not deleted: the pianist and the player, the standing layer, the micro lane and the duel harness, the world
model (`territory`, `reclaim`, `shelling`, `routes`, `combat`, `yards`, `allies`, `nanos`, `roster`, the spot survey),
the planner's search, the arena, the recorder and the viewer. `run/` scripts that read the heuristic bot's `rules:`
line or firing table are deleted with it (`git grep 'rules:'` and `fired` in `run/` before the commit).

## 2. The arena instrument

`--pianist` without `--player` runs today: Jev plays every actor from the picture with an empty packet. It becomes the
bot of every arena batch that has no player, with one addition: `--packet <file>` gives the pianist a fixed packet
(the text the `instruct` tool would have sent), decompressed once at the first order frame, so the micro agent's A/Bs
play a stated brief rather than nothing. Cost: standing-2's rate is $0.34 for 29 minutes with the player re-asking;
a fixed packet and the worlds question below should run under $0.20 a game. The arena's `--bot <binary>` arm stays
(the pre-fix bot as a binary is how lane-ab and lead-ab were run). The heuristic bot's arena arms (`--disable`,
`--ab-disable`, the firing table) go.

## 3. Groups are the player's

The tool and the packet name groups, and worlds are enumerated per group, so a group must outlive its orders.
H-HANDS-GROUPS today merges every newcomer into the largest group within reach; standing-2 had 22 groups of a median
1.3 minutes and the whole army merged into group_I by the bookkeeping. New rule (H-HANDS-GROUPS amended):

- A newcomer out of a factory joins the group the player named for that factory's output (`produce` gains
  `group: group_X` or `new`), else forms its own group. Nothing merges by proximity.
- `join_group_X` stays an option on a group's menu and a `standing` rule (`join: group_X`), so merging is an order.
- A group keeps its name until it is empty. A split names the child `group_X1` as now.

## 4. The worlds question: one decider for the army

**Superseded twice:** by `2026-09-26-threat-response.md` (threat-centred passes in place of per-group candidates) and then by `2026-09-26-one-pass.md` (one pre-pass and one pick over every actor, builders and labs included; the executor deleted). Kept as the record of why.

The standing executor stops deciding for groups. It becomes the candidate generator, and one Jev question over the
groups' joined worlds is the only decider. No group is ever played by two deciders in one second, so the flip pairs of
standing-2 (Jev hold then the station walk, 26 times on group_I) cannot occur.

**Candidates.** Each second each group has a short list, built from its rules and the picture:

- `keep` (its course: hold, the walk it is on, the party it engages), always;
- the rule's literal result when a rule fires (`station` walk, `engage_party`, `raiders_*`'s engage or
  `send_against:N`, `fall_back_to`, `no_chase`'s hold);
- the raider answers the rules allow for a party within reach: `engage` whole, `send_against:N` with N from
  `detachment_for` (the smallest detachment that outweighs the party), `ignore` (which is `keep`);
- `back` (fall back to the last hold or `fall_back_to`) when a party within 600 outweighs the group or it lost a
  quarter of its metal in 30 s;
- the player's `never` and `no_detachments` and `hold_line` remove candidates, as now.

A group with nothing within reach and no rule firing has one candidate, `keep`, and does not vary.

**Worlds.** The product of the varying groups' candidates, pruned: two groups do not both take the same party unless
together they are needed to outweigh it; a group beyond 1,200 of every event has only `keep`; the count is capped at
8 by dropping, in order, the `back` of groups not outweighed, then the whole-group engage where a detachment
outweighs, then the farthest group's variation. Offline (`docs/studies/2026-09-25-joint-worlds.md`) the pick diffused
past 8 worlds and held 60-83% of the raiders under it.

**The question.** One Choice per second over the worlds, each option one line: the groups' moves and the computed
consequences as differences from `keep` (metal at risk at the extractor, the party's odds against the movers, the
walk in seconds, what is left uncovered). The state is the picture as now, trimmed to the groups and parties that
vary. Jev's answer plays every varying group at once through `apply_orders`; the rest continue.

**When it is asked.** Only on an event: a party newly within reach of a group, a party that moved to a structure of
ours, a loss in a group, a group arriving at its walk's end, a packet or tool change. Between events the rules'
literal results play with no call (the `station` walk, `engage_party`, the courses kept). The quiet review of the diet
is kept for groups without rules only. The scorecard's `noop%` (asks answering hold on a holding group, 38% in
standing-2) is the number this is judged by, with `jev$`.

**The filter arm** (`--standing filter`) survives as one extra option in the worlds question: `panic` (every varying
group falls back and the player is woken). The `rule`/`near`/`other` verdicts go: the worlds are the near and other.

**The noul gate** (mode `nouls`, the user's proposal after worlds-1: "a jev pass with a bunch of nouls asking which
dimensions or partial action states are interesting, then pose the composed world combination in the follow-up").
The first call carries, per varying group, one noul per live dimension over its first candidate of that kind ("should
group_C do this now, rather than the course it is on? The move: group_C attacks party_3 ..."): `fight`, `walk`,
`back`. A dimension at 0.5 or above stays; the group's candidates are cut to its course, its rule and the kinds kept;
a group with one candidate left plays it now (its rule, or nothing). The second call, made at once in lockstep and
through the worker in realtime, is the worlds question over the groups still varying. Two calls of 130-250 ms; the
gate is the "keep everything" answer the fight-happy tilt lacked (worlds-1: a deviation picked 117 of 191 times, the
first fight 110). Log: the `standing` line's `gate` (the question ids), a `worlds_gate` line (`flags` per group and
dimension, `asked` the groups of the second call), and the second call's own `call` line.

**Builders** are not in the worlds. A builder is played by its list (`queue`), else its standing rules, else one
question of its own. The rules' precedence with hysteresis, the standing-2 fault: `retreat_when_enemy_near` holds
for 30 s after it fires, and `turrets`/`job`/`solar` do not fire while the party that sent it home is within 600 of
the extractor. The builder question is the from-scratch outline's (`docs/design/2026-09-25-menus-from-scratch.md`):
`do` over the roster, and one `where` per purpose (`where_build`, `where_walk`) asked only for the purposes the
answer can take, never one `where` shared by walks and turrets.

**The `standing` tool** keeps its shape and loses its use as a command channel: `engage_party` and `station` set a
rule that the worlds question sees as the group's `keep`, not a play the executor forces. The tool's report line says
which candidates the rules generated and what the worlds question picked, per group, since the last turn.

## 5. Records and the scorecard

The `standing` log line becomes `{"t": "worlds", "f", "event", "groups": {name: [candidates]}, "worlds": n,
"answer": {"choice", "confidence"}, "played": [...]}`; `decompress` stays. `docs/harness/record-format.md` amended;
`played.source` is `worlds`, `rule` (a literal result played between events), `list` or `jev` (a builder's own
question). `run/floor.py`: `standing%` becomes `rule%` (plays with no call over all plays); `noop%` and `jev$` stay.
`run/worlds_ab.py` is rewritten to build the question from the bot's own candidate lines (a `worlds` line carries
them) so recorded events replay offline under wording changes, as `run/jev_ab.py` does for menus.

## 6. Validation

- Unit tests: candidates for a group under each rule; the pruning's cap and order; the consequence lines; the
  builders' hysteresis; a newcomer joins the named group and nothing merges by proximity.
- Offline: `run/worlds_ab.py` over the recorded events of standing-1 and standing-2 before a game is spent.
- Games on the bank-1 brief, Comet Catcher, hard_aggressive, Opus 5.5 medium, the `claude2` account: `worlds-1`
  (standing on, worlds), then `worlds-2` with the panic option, read by `run/floor.py`, `run/jev_ab.py --list`,
  `run/fire.py`, the player's notes, and the layers count (two deciders on one actor within 20 s: the target is zero
  by construction, so the count is a test of the build).
- The micro agent's next arena A/B runs on the pianist-only bot with a packet file, and its numbers are the
  instrument's baseline (lost per killed, friend on the line, muzzled).
- Success line: `noop%` under 10, `jev$` under $0.20 for 30 minutes, raider_ignored under 5 moments, the trade not
  worse than standing-2's, the player's notes naming no case of the hands undoing an order.

## 7. Order of work

1. This note committed. 2. The deletion (§1) in one commit per block: deciders, modes, policy, family, registry;
`cargo test` green after each; the bot plays a pianist-only game against BARb easy as the smoke test. 3. `--packet`
and the pianist-only arena arm; tell the micro agent. 4. Groups the player's (§3). 5. The worlds question (§4) with
its tests and the offline replay. 6. The builders' question and hysteresis. 7. worlds-1.
