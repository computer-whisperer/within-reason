Written by a subagent of the main session on 2026-10-01 for `docs/design/2026-10-01-hands-rebuild.md`; read from the code, nothing run. The main session checked against the code: the builders' `retreat_home` with no threat condition (plan.rs:513), the `leave` arm of `apply_plan` (execute.rs:82), the missing confidence bar (mod.rs:1034-1039), the paragraph rule (diet.rs:69), `walk_places`, `resolve` and `compose`'s `taken`. The rest is the subagent's reading.

# The hands today: an inventory (main @ e690e5e, read 2026-10-01)

Read-only inventory of `crates/bot/src/brain/pianist/` (plan.rs, threats.rs, execute.rs, schedule.rs, lists.rs,
groups.rs, diet.rs, the pass-driving parts of mod.rs, picture.rs where the menu reads it, rules.md, default.md) and
the code outside it that acts on the same units (brain/micro.rs, nanos.rs, reclaim.rs, economy.rs, routes.rs,
crates/jev). Everything below is from reading code; nothing was run. Paths are relative to
`crates/bot/src/brain/pianist/` unless they start with `brain/` or `crates/`. "Unsure" is said where I am.

Constants used throughout: `FRAMES_PER_SECOND = 30` (brain/mod.rs:63), `BRAIN_FRAMES = 15` (a think every 0.5 s,
brain/mod.rs:66), pass interval `INTERVAL_SECONDS = 1.0` (mod.rs:46, `WITHIN_REASON_JEV_INTERVAL`).

---------------------------------------------------------------------------------------------------------------------

## 1. Actor families and their options

A slot is pushed only when it has more than the standing state (`states.len() > 1`). Index 0 is always `keep`
(`Response::Keep`; `Leave` for a threat). `current` marks a state already in force; a slot's base (world 1) is its first
`current` state, else index 0 (plan.rs:194).

### 1a. Builders (every finished mobile builder, the commander included; plan.rs:437-768)

Slot-level gates, in the order the code applies them:

| # | Condition | Effect | Where |
|---|---|---|---|
| B0 | `listed` = `scripts[name]` non-empty, or (`list_steps` has the builder and it has a task) | see B1, B5 | plan.rs:461 |
| B0 | `threatened` = hit in the last `UNDER_FIRE_FRAMES` (3 s, mod.rs:54) or this tick, or **any** party (armed or not, lone scout or not) within `STARTED_ALARM = 800` | see below | plan.rs:277 |
| B0 | `queue_ahead` = started build's share `>= QUEUE_AT (0.6)` and not threatened and nothing in `queued` | options are worded "then ..." and ordered behind the build | plan.rs:278 |
| B1 | `!threatened && listed` | no slot, except the one below | plan.rs:466-476 |
| B1a | B1 and `energy.storage > 0 && energy.current < 0.25*storage && usage > income` and the builder can build `kit.solar` and no solar of ours is being built | slot with exactly `keep` + `<name>.<solar>` | plan.rs:465-474 |
| B2 | started build, not `queue_ahead`, not threatened | no slot (H-HANDS-STARTED) | plan.rs:478-480 |
| B3 | `listed` (so threatened here) | slot carries only what was pushed so far: keep + the ways out (rows 1-3) | plan.rs:558-563 |
| B4 | task is `Build`/`Reclaim`/`ReclaimUnit`/`Repair` and not `queue_ahead` | slot carries only keep + the ways out (rows 1-3). An `Assist` or `Walk` task does not gate. | plan.rs:568-574 |
| - | `idle` = no task and the engine says idle | idle slot: always open, never closed by `settle` | plan.rs:486 |

Options (id is `<name>.<key>`):

| # | Key | Response | Offered when (exactly) | Where |
|---|---|---|---|---|
| 0 | `keep` | Keep | always | plan.rs:489 |
| 1 | `retreat_home` | RetreatHome | builder farther than `AWAY = 400` from home. **No threat condition**: the threat only adds words. `threat` = nearest armed party within `ALARM = 600` that is not a lone scout; "stands at home" words from `home_party` (armed, not lone scout, `home.dist(party) < ALARM+200` or `home.dist(party)+300 < unit.dist(party)`, nearest home). current when walking to `home`. | plan.rs:513-518 |
| 2 | `step_away_<place>` | WalkTo(place) | `threat.or(home_party)` exists, and a place exists with: name != home, `> 100` from the builder, `> ALARM+200 (800)` from the party, `dist(place,party) > dist(unit,party)+300`, (its picture entry `what` starts with `"our "` or it is not a spot), reachable for the builder's class. The **nearest such place to the builder**, one only. Words carry arrive-before/after-contact timing ("said, never pruned"). | plan.rs:519-545 |
| 3 | `under_flak_<place>` | WalkTo(place) | builder under fire and `first_seen` holds any air-domain type (the enemy has ever shown aircraft; not "aircraft are what hit it"), and a place within `AT_STRUCTURE = 400` of a finished unit of ours whose glossary class contains `anti-air`, `> 100` from the builder. Nearest one. | plan.rs:547-553 |
| 4 | `extractor_spot_<i>` | Extractor(i) | builder can build `kit.extractor`. From `free_spots` (below), sorted by the builder's walking seconds: the nearest `NEAREST_SPOTS = 2`, plus the first `NAMED_SPOTS = 8` that the **whole packet** names (`diet::names_spot`), plus, when the first offered has `enemies_near` in the picture's place entry, the nearest spot without it. | plan.rs:576-619 |
| 5 | `<unit>` | Building(def) | def in the builder's menu (below), not `placed_at_place` (so: energy, factories, nano turrets, storage, converters ...). Skipped when glossary flag `on_water` and no water within `build_distance + 300`; a nano turret skipped while no finished factory of ours stands. Words say "beside itself". | plan.rs:646-666 |
| 6 | `<unit>_<place>` | BuildingAt(def, place) | def `extracts_metal > 0` (tier-2 extractor): the **nearest** finished extractor of ours to the builder with no building of this def within the spot radius, at the place nearest it within `AT_STRUCTURE = 400`. One option. | plan.rs:667-678 |
| 7 | `<unit>_<place>` | BuildingAt(def, place) | def armed (`weapon_count > 0`): beside the **nearest 2** finished extractors of ours within `TURRET_REACH = 1200` of the builder that (a) have no non-lone-scout party within `ALARM = 600`, (b) have no armed static of ours within `TURRET_COVER = 350`, (c) have no armed build ordered or queued within 350; each needs a place within 400. "outer" is said when the extractor is `> OUTER = 500` from home. | plan.rs:679-707 |
| 8 | `<unit>_<mark>` / `<unit>` | BuildingAt(def, mark) / Building(def) | the other `placed_at_place` kinds (radar, jammer, sonar): at the nearest 2 of the player's marks within `WALK_REACH = 3000`; beside itself only when no mark is in reach. `current` is hard-wired false for the mark form. | plan.rs:708-718 |
| 9 | `help_<factory>` | Assist(factory) | the nearest finished factory of ours, any distance. Words price the help (starved: `metal.current < 100 && income < draw`; full store; else). | plan.rs:722-735 |
| 10 | `help_<builder>` | Assist(builder) | every other builder with a **started** build within `HELP_REACH = 900` | plan.rs:736-740 |
| 11 | `help_<builder>` (pair_only) | Assist(builder) | every other builder within 900 without a started build. Never asked on its own; used only by `compose`'s pair worlds. | plan.rs:741-744 |
| 12 | `reclaim` | Reclaim(at) | the wreck field with the **most metal** among fields with `metal >= 100` within `RECLAIM_WITHIN = 1800` | plan.rs:746-749 |
| 13 | `repair` | Repair(unit) | the **nearest** finished unit of ours under 70 % health within `REPAIR_WITHIN = 1200` that is a commander or static | plan.rs:750-758 |
| 14 | `walk_<mark>` | WalkTo(mark) | the nearest 2 marks within `WALK_REACH = 3000` and farther than 150 | plan.rs:759-764 |
| - | (threat slot) `party_N.attack_<name>`, `party_N.dgun_<name>` | Attack / DGun | see 1d; these live in the party's slot, not the builder's | threats.rs:211-255 |

"Marks" for rows 8 and 14 = places that are not spots, not `home`, not `shelling*`, not `passage_*` (plan.rs:434).

`free_spots` (plan.rs:284-313) hides a spot when: another builder's `Build{spot}` in `tasks` or `queued` holds it
(unless that order is unstarted and that builder has a non-empty list); another seat claims it; it is in
`refused_spots` (90 s, `REFUSED_FRAMES`, mod.rs:52); an extractor of ours or an ally's stands on it; a known enemy
extractor stands on it; `spot_open_to_us` is false (an ally's side of the map, 3 min, brain/allies.rs:42); or the
builder's class cannot reach it. Only spots that are in `picture.places` are candidates (picture.rs:793-864: ours and
taken ones, the 10 nearest free, 6 of his, every spot the packet or a list names).

The builder's menu (rows 5-8; plan.rs:621-645): with a `produce` allowance that has units and names at least one thing
this builder can build, only entries that still permit (`entry_permits`, counted against `produced`); otherwise the
builder's build options cut to `roster::usual_menu` (brain/roster.rs:48-73: a fixed per-faction list, no converters).
`kit.extractor` is always excluded here.

### 1b. Factories ("lab_N", "plant_N", "factory_N"; plan.rs:770-834)

| Key | Response | Offered when | Where |
|---|---|---|---|
| (no slot) | - | `lab_queue[factory]` non-empty (a unit ordered and not yet begun) | plan.rs:777 |
| (no slot) | - | a counted `produce` entry is left (`sequence_unit`): made without asking by `play_sequences` | plan.rs:782, lists.rs:160-189 |
| `keep` | Keep | always; words differ with a unit on the pad (a being-built mobile unit within 120) | plan.rs:785-792 |
| `<unit>` | Next(def) | every build option the allowance still permits; with no allowance (or one with no units) **every** build option of the factory (no usual-menu cut here) | plan.rs:796-830 |

`idle` = nothing on the pad and engine-idle. `queue_ahead` = a unit on the pad (set but not read by `execute_lab`).

### 1c. Groups (every non-roving group with a body; plan.rs:836-1125)

`centre` = `body.at` (the member at `BODY_SHARE = 0.6` of the core from the front, groups.rs:167);
`nearest_party` = nearest party with any member within `ALARM = 600` of it (plan.rs:866); `paragraph` = the group's
own paragraph of the packet, **else the whole packet** (plan.rs:884).

| # | Key | Response | Offered when (exactly) | Where |
|---|---|---|---|---|
| 0 | `keep` | Keep | always | plan.rs:861 |
| 1 | `walk_<place>` / `advance_<place>` | Walk{place, fight} | `walk_places` (plan.rs:1139-1153). `named` = places not `shelling*` that are home, a non-spot, or a spot the whole packet names; within `WALK_REACH = 3000` or a packet-named spot or named in its own paragraph; farther than `STATION_SLACK = 300`. `offered` = every `named` place its own paragraph names (home only when an own paragraph exists and names it; **with no own paragraph, every place the packet names anywhere counts as its own**), then the **nearest 3** of the rest. `fight` (and the `advance_` key) = `paragraph.contains("advanc")` and the place is named in `paragraph` and is not home. Words add what stands within `AHEAD = 800` of the place, and for a route stop that is a spot with nothing known, "nothing of his was there when last seen" (seen >= 10 s ago) or "never seen". | plan.rs:884-926 |
| 2 | `raid_<place>` | Walk{place, fight: true} | places (not home, not `shelling*`, reachable) with known enemy buildings within 500, at `STATION_SLACK (300) <= d <= RAID_REACH (4000)` from the body's front; the **nearest `RAID_STATES = 3`**. | plan.rs:935-961 |
| 3 | `escort_<builder>` | Escort(id) | group not air, has an own paragraph, and that paragraph names the builder's actor name | plan.rs:967-985 |
| 4 | `sweep` | Walk{place: next, fight: true} | at least one spot **among the picture's places** that nobody has seen, reachable, `> 300` from centre. `next` = minimum of (`seek_his && !in_his_box`, distance from front); words list the next 3 by distance from `next`. | plan.rs:990-1000 |
| 5 | `gather_<place>` | Gather(place) | `body.strung_out()` (front-to-tail `> STRUNG_OUT = 600`, groups.rs:182) and the task is not a hold; the non-`shelling*` place nearest the front. current = `group.gathering`. | plan.rs:1002-1007 |
| 6 | `close_on_shooter` | Walk{place: `shelling_<g>` or `shelling`, fight: true} | `shelling_for(members)` exists, a member is under fire, the shelling place exists, the estimate is reachable. current hard-wired false. | plan.rs:1012-1021 |
| 7 | `pull_out_<place>` | Walk{place, fight: false} | same shooter condition; the non-`shelling*` reachable place **nearest the centre** that is farther than `s.range + 150` from the estimate | plan.rs:1022-1024 |
| 8 | `shell_<party>` | Shell(party) | a member with `reach >= ARTILLERY_REACH (600)` and `speed > 0`, and a `nearest_party`. current = `group.shelling` and the task place is `standoff from <party>`. | plan.rs:1028-1036 |
| 9 | `fall_back` (current) | FallBack | the task is `Move{fight: false}` whose place starts with `LAST_HOLD` ("where it last held,"): the state in force | plan.rs:1048-1052 |
| 10 | `fall_back` | FallBack | not walking back (to home or last hold), task busy, `last_hold` set, `> 300` from centre, and (no nearest party, or the party is `> 300` farther from the hold than from the centre). Words still end "less far than the base". | plan.rs:1053-1060 |
| 11 | `scout` | Scout | `units >= 2`, no group with `scout && roving` exists, not air | plan.rs:1063-1070 |
| 12 | `split_<place>` | Split(n, place) | `units >= 2`; the nearest 2 of `named` (row 1) other than home; `n = (units/2).max(1)` | plan.rs:1071-1074 |
| 13 | `join_group_<other>` | Join(other) | the **nearest** other non-roving group of the same domain, any distance | plan.rs:1080-1100 |

Slot flags: `idle` = task is `Hold` (plan.rs:860); `stop_cost` (the sentence in world 1's line) when idle and the last
`reached` place is within 300 and the paragraph names later or other unreached places (plan.rs:1103-1118); `on_route`
when the paragraph names 2+ places (home excluded) and the task is a Move to one of them (plan.rs:1120-1121).

### 1d. Threat slots (one per enemy party; threats.rs:36-259)

A party gets a slot when it is nearer home than `enemy_start`, **or** within `RAIDER_REACH = 1200` of a structure of
ours (extractor, kit turret, factory, mobile builder, anything static), **or** heading toward home (summed velocity
`>= 0.3` with a positive component toward home) (threats.rs:56-64).

| Key | Response | Offered when (exactly) | Where |
|---|---|---|---|
| `party_N.leave` | Leave | always (index 0) | threats.rs:100 |
| `party_N.unreachable_group_X` | **Keep** | one of the nearest `GROUPS_PER_PARTY = 3` non-roving groups (by front distance, any distance) whose class cannot reach the party's place. It is an ordinary state: asked as a move, composable, and counts the group as "sent" (see 6). | threats.rs:111-114 |
| `party_N.hunt_group_X` | Hunt(ids) | group not air, has armed mobile members. `fast` = members faster than the party's slowest identified member (`UNIDENTIFIED_SPEED = 87` when none is identified); if none, the members within 1.0 of the group's top speed. Sorted by distance to the party; `k` = fewest whose odds are `>= 1.3` (`hunters_for`), else `min(len, 2)` with "which do not outweigh it". current when the group hunts a member of the party. | threats.rs:134-169, 263-277 |
| `party_N.whole_group_X` | Whole | the group has units. current when it engages the party. | threats.rs:171-184 |
| `party_N.next_extractor_group_X` | Walk{place, fight: true} | the party moves (`>= 0.3`), an extractor of ours lies on its heading (`along > 200`, `across < 0.6*along + 200`, nearest along), that extractor has a named place within 400 and is `> 300` from the group's front | threats.rs:77-91, 186-199 |
| `party_N.dgun_<builder>` | DGun(party) | an armed mobile builder within `RAIDER_REACH = 1200` of the party with a D-gun, and the party's nearest unit within `dgun_reach + 100`. current hard-wired false. | threats.rs:211-233 |
| `party_N.attack_<builder>` | Attack(party) | every armed mobile builder within 1200 of the party, **listed or not**, whatever its task. current when its task is a walk to that party. | threats.rs:211-255 |

---------------------------------------------------------------------------------------------------------------------

## 2. Where code reads the packet's prose

`diet::names` / `first_named` (diet.rs:83-98): substring match whose **right** neighbour is not a digit or `_`; the left
side is not checked. `diet::paragraph` (diet.rs:69-74): split on blank lines; a paragraph is the actor's when the text
before the first `:` and before any `(`, trimmed, equals the actor name exactly.

| Text read | Trigger | Effect | Where |
|---|---|---|---|
| whole packet | token `spot_N` | spot N becomes a place in the picture (if any class reaches it) | picture.rs:852-858 |
| whole packet | token `passage_N` with `N > PASSAGES (3)` | that passage becomes a place | picture.rs:859-862 |
| whole packet | a run after a spot (`spot_10, 5, 6`, `and/then/or` allowed) | those spots become places | picture.rs:839, diet.rs:111-140 |
| the lists' steps | second word `spot_N` | that spot becomes a place | picture.rs:840-846 |
| whole packet | `names_spot(instructions, i)` | builder option 4: up to 8 named free spots offered to **every** builder (not per paragraph: a spot named in a group's route counts) | plan.rs:583-592 |
| whole packet | `names(instructions, spot)` | group walks: the spot is in `named`, however far | plan.rs:1140, 1145-1146 |
| group's paragraph (else whole packet) | names a place | the place is always on the group's walk menu, however far; `home` only from an own paragraph | plan.rs:1141-1151 |
| group's paragraph (else whole packet) | `contains("advanc")` | walks to places that text names become `advance_*` with `fight: true` ("never advances" also matches) | plan.rs:890-893 |
| group's paragraph (else whole packet) | names a spot place with nothing known there | the walk's words add the scouting age | plan.rs:910-918 |
| group's **own** paragraph only | names a builder's actor name | `escort_<builder>` offered | plan.rs:967-970 |
| group's paragraph (else whole packet) | `contains("base") \|\| contains("his ")` | sweep prefers spots in his start box (`"his "` is also inside "this ") | plan.rs:991-994 |
| group's paragraph (else whole packet) | `named_in_order` over the picture's places, home excluded | `stop_cost` sentence in world 1 (next stop after the last reached one, or the unreached ones) | plan.rs:1103-1118 |
| same | 2+ places named and the walk is to one of them | `on_route`: `settle` does not open the group on party news | plan.rs:1120-1121, 1266 |
| `packet_seen`, group's paragraph (else whole) | `named_in_order` | `group.route`; a different list clears `reached` and `met` | groups.rs:455-461 |
| same | names a place within `ARRIVED = 300` of the body | `reached` gets the place, a "group_X reached P" event (urgent ask) and a `recent` note | groups.rs:462-472 |
| whole packet | differs from `packet_seen` and from `logged_instructions` | event `packet`, `packet_frame` (part of the signature), nothing is closed this second | mod.rs:698-703 |
| whole packet | `names(instructions, actor)` | the actor's entry is kept whole in the gate state (else one line) | diet.rs:178 |
| whole packet + lines | `names(text, name)` | pick state: actor entry kept; place kept (moot, see 6) | plan.rs:1748-1761 |
| whole packet | `names(instructions, place)` | `shed`: places kept when the state is over budget (moot, see 6) | diet.rs:213-217 |

Not prose, but the player's other levers that change the menu: `produce` allowance (builder and factory menus, the
factory sequence, the `group` a factory's output joins), `queue` lists (`listed`), `mark` (places), `lane` (roving),
`remove`, `transfer`.

The rules text (rules.md, read from disk each picture, plus a wind sentence, picture.rs:1504-1508) is sent in the
state as `rules`. It still tells Jev, in words, things that are decisions: "An enemy party in sight within 600 of a
group that outweighs it is fought now, whatever the instructions call it"; the fall-back law; the commander's retreat
law; "A factory standing idle builds now whatever the store holds".

---------------------------------------------------------------------------------------------------------------------

## 3. The pass

### 3a. Order within one think (`run_pianist`, mod.rs:647-720; a think is every 15 frames)

| Step | What | Where |
|---|---|---|
| 0 | (before) `tend_nanos`: idle construction turrets guard the nearest factory in reach | brain/mod.rs:409, brain/nanos.rs:34 |
| 1 | `pianist_housekeeping`: tasks against engine events (see 4) | mod.rs:1176 |
| 2 | `keep_groups`: membership, adoption, roving switch, route facts, arrivals, lost parties, escorts (see 4) | groups.rs:289 |
| 3 | `end_timed_assists` | mod.rs:626 |
| 4 | resurrection crew: `to_mend` repairs; every idle resurrector `work_wrecks` | mod.rs:654-660 |
| 5 | growth, field, cards, `take_removals`, `take_transfers` | mod.rs:663-670 |
| 6 | return before `FIRST_ORDER_FRAME = 60`; return while the player's opening turn is pending | mod.rs:671-679 |
| 7 | realtime only: `collect_answer` (plays an answer that has come; also on every non-think tick via `poll_hands`) | mod.rs:680-682, 1087, 1124 |
| 8 | `schedule_asks`: hits, alarms, events | schedule.rs:17 |
| 9 | due = no request pending **and** `frame - last_ask_frame >= interval_frames` (30). Not due: return. In realtime everything below, the lists and the factory sequences included, waits while a request is in flight. | mod.rs:684-691 |
| 10 | `take_lists` (the player's `queue` calls) | mod.rs:723 |
| 11 | `picture`; packet and places events; `pianist.places/parties` | mod.rs:694-711 |
| 12 | `play_lists`, `play_sequences` (no pick) | lists.rs:63, 171 |
| 13 | `pass` | mod.rs:787 |
| 14 | `publish_hands` | mod.rs:1373 |

After the think, every tick: `note_commitments` and the micro lane (brain/mod.rs:389-391).

### 3b. `pass` (mod.rs:787-921)

1. `slots()`; a party with a threat slot now and none in `pianist.slots` gets an event `"party_N appeared"`.
2. `eco` = `"{empty|full|}|{STALLING}|{idle_lab_afford}"`: empty = `metal.current < 100`; full = `current >= storage - 1`;
   STALLING = the picture's energy words contain it; `idle_lab_afford` = a factory slot at base 0 whose cheapest
   offered unit costs `<= metal.current` (mod.rs:820-825).
3. `settle` (plan.rs:1258-1271). A slot is closed (`quiet`: no question, world 1 keeps it) only when all hold:
   not a threat; not idle; no `packet` or `lists` event; it has an entry in `asked`; its `course` (the base state's
   `dim`, or `keep`, plus `:idle`) equals the one recorded; `frame - asked_frame < RE_ASK` (20 s = 600 frames,
   plan.rs:46); no event string starts with its name followed by a space; and no outside news: for a group
   `parties changed && !on_route`, for a builder or a factory `eco` changed (both compared with their values when the
   gate last went).
4. `gate_questions`; the signature `sig` = every slot's (party name@place:count, `!` when harming, current state ids)
   or (`name:course`), plus `eco`, plus `packet_frame` (plan.rs:1214-1228, mod.rs:838).
5. No questions or no Jev client: log and return. Otherwise the gate goes only when `changed` (mod.rs:851):
   no previous sig, or sig differs, or `RE_ASK` since the last ask; or events exist and `EVENT_GAP` (5 s, mod.rs:248)
   has passed; or an event contains `" reached "` or `" met "`; or some open non-threat slot has no `asked` entry or
   one `RE_ASK` old. Else the line is logged "quiet".
6. When it goes: `sig`, `asked` (course and frame of every open non-threat slot; entries older than `RE_ASK` dropped),
   `asked_parties`, `asked_store` are written, **events are cleared**, before the call is made.
7. The state: `trim_state` (diet.rs:145: at the default `normal`, actors neither asked nor named in the packet are one
   line), then **`places` is removed entirely** and `asking` (plan.rs:1156) added (mod.rs:906-909), then `shed` to
   `STATE_CHARS = 160_000` (diet.rs:192-249: `recent` to 6 lines, places, then the largest actor entries to a line).
8. `call` (mod.rs:930): lockstep makes the call in place (the game waits); realtime sends it to the worker with a
   `Follow` and remembers `Pending`.

### 3c. The Jev calls and their texts

Call 1, the gate: nouls (plan.rs:1158-1208, threats.rs:281-304). `jev::Client::ask` splits a body over
`REQUEST_CHARS = 150_000` into concurrent batches (crates/jev/src/lib.rs:28, 204-231).

| Question id | Asked when | Template |
|---|---|---|
| `<actor>.change` | open, not idle | "Given `actors.{a}`, `economy`, `enemy` and the player's `instructions`: should {a} do something other than what it does now ({standing})? Yes when the instructions call for a different job now, when what it does is finished or pointless, or when something near it needs answering. No when its course is what the instructions want and nothing has changed." |
| `<state id>` (full) | every state of an open slot except base, index 0 and pair-only | "Given `actors.{a}`, `economy`, `ours` and the player's `instructions`: is this what {a} should do now, rather than {standing}? The move: {words}." |
| `<state id>` (bulk) | a group's state whose id contains `.walk_` or `.advance_` | "{a}, rather than {standing}: {words}." (read under the state's `asking` line) |
| `<state id>.told` | builders only, only with `WITHIN_REASON_TOLD=1` (off by default) | "Read the player's `instructions` alone, with `actors.{a}` and `ours` for what stands: do the instructions make this {a}'s next step now? The step: {words}. Yes only when ..." |
| `party_N.answer` | the threat slot has any state besides base and leave | "Given `enemy`, `actors`, `player` and the player's `instructions`: does {party} ({composition}, {place}{turrets}{harming}) need answering this second by someone other than what stands ({standing})? Yes when it is killing, taking apart or about to kill something of ours that nothing there can stop; no when a turret handles it, it is leaving, or the instructions say to leave such a party." |
| `party_N.<state>` | each such state | "Is this the move to make against {party} now, rather than {standing}? The move: {words}." |
| `party_N.hunt_group_X.forbidden` | each hunt state | "Read the player's `instructions` alone: do they forbid this move for the group that would make it? The move: {words}." |

Call 2, stage one (skipped when there is exactly one deviation): a Choice over `w2..wN`, instructions = `COMMON`
(plan.rs:1625: "Given `economy`, `ours`, `enemy`, `actors`, `player` and the player's `instructions`, which plan is
best this second? Each option is one world: ... An idle factory or builder with metal in the store is a cost, not a
course, unless the instructions say to wait. The instructions were written before this picture: where they name a
place, a party, a building, a unit or a rule, follow them; where the situation has changed, pick the world they would
call for.") + " Every option is a change from what stands (w1, not offered here): one or more actors' courses changed,
the line saying only those changes and what they alter. Pick the best of the changes; whether to change at all is
asked next." (plan.rs:1628-1631). Its answer is **sampled** from the probabilities renormalised over `w2..wN` with a
uniform draw from the standard library's hasher seed (`sample`, `draw`, plan.rs:1640-1665); with no probability mass
it takes `choice`.

Call 3, stage two: a Choice between `w1` and the candidate, `COMMON` + " w1 changes nothing: every actor keeps its
course and the idle ones stay idle. The other option is the best change on offer this second, its line saying only
what it changes from w1 and what that alters. Pick one." (plan.rs:1634-1637). The pick is the answer's `choice`
(argmax, not sampled; `pick`, plan.rs:1697). **No confidence bar is applied** (mod.rs:1034-1039).

Both stages use `pick_state` (plan.rs:1745): the gate's state with actors no world sends and nothing names cut to a
line.

### 3d. `compose` (plan.rs:1292-1516)

| Piece | Rule | Constant |
|---|---|---|
| World 1 | every slot's base, then `resolve` | - |
| `taken` | actors world 1 already sends at a threat | - |
| Threat opens | `party.answer >= FLAG`; then **every** state against it is a candidate whatever its own noul, rank = `answer * max(noul, 0.01)`. With no answer, a state opens alone at `noul >= FLAG`. | `FLAG = 0.5` (plan.rs:31) |
| Threat prune | a state whose actor is in `taken` (and is not the base state's actor) is dropped: a group engaged with one party is not offered at another | plan.rs:1333 |
| Threat drop | base is a `current` state and `answer < FLAG`: a Leave deviation with rank `2.0` (always first) | plan.rs:1316-1323 |
| Actor opens | idle: change = 1.0; else `<actor>.change >= FLAG`. An actor in `taken` gets no deviation. | `FLAG` |
| State bar | state noul `>= FLAG`; or, for an idle actor at base 0 none of whose states reaches `FLAG`, `>= IDLE_BAR` | `IDLE_BAR = 0.3` (plan.rs:93) |
| Told | (off by default) a told noul `>= TOLD_BAR` opens the slot and the state | `TOLD_BAR = 0.25` (plan.rs:70) |
| Rank | `change * noul` | - |
| Pair world | a builder's `Building`/`BuildingAt` deviation whose `dim` is `factory` or whose words are `dear` (any number `>= PAIR_COST = 300` in the words and the word "metal": a distance or a store figure counts) is doubled with each other builder (idle or `.change >= FLAG`, not taken) that has an `Assist` state on it; rank `* 0.99` | `PAIR_COST = 300` (plan.rs:74) |
| Depth | each slot keeps its best `DEPTH` deviations (pair worlds compete for these two places) | `DEPTH = 2` (plan.rs:44) |
| Singles | round-robin over slots: every slot's best, then every slot's second | until `cap` |
| Joint worlds | slots ranked by best deviation; greedily as many as keep `out + product - 1 - singles <= cap`; every combination of 2+ of their deviations, rank = product, best first; dropped when two deviations set one slot differently or one actor is at two threats | `CAP = 255` (plan.rs:23; `WITHIN_REASON_WORLDS`) |
| None | nothing opened: no pick is asked | - |

`resolve` (plan.rs:1273-1280): any non-threat slot whose name is the actor of a non-zero threat state in the world is
set to index 0 (keep). The threat slot wins over the actor's own slot, also over the actor's own `current` state.

Lines (`consequence`, `parts_of`, plan.rs:1531-1618): world 1 = "Nothing changes: every actor keeps the course its
entry under `actors` describes" + Met / Left to nobody / "{names} idle, doing nothing, while the metal store reads
{store}" / the stop-cost sentences. A deviation = "As w1, and: {moves}" + only what it adds to those lists, + "The
player's instructions forbid this hunt for {actor}" for a started hunt whose `.forbidden` noul `>= FORBIDDEN = 0.7`
(plan.rs:42, 1709-1718). "Met" words use `odds_by_metal` thresholds 2.5 / 1.3 / 0.8 (plan.rs:210-221).

`fit` (plan.rs:1730-1739): worlds are cut to the longest prefix whose deviation lines total
`<= LINE_CHARS = 40_000` characters (plan.rs:29); world 1 and the first deviation always stay.

### 3e. Putting the pick in force (`apply_plan`, execute.rs:62-104)

For each slot of the picked world: a `current` state is skipped; a roving group (or a join into one) is skipped; a
closed slot at its base is skipped; `Keep` does nothing; a threat's `Leave` calls `leave_party`; anything else is
executed. Note that `Leave` at index 0 is not `current`, so for every **open** threat slot the picked world leaves at
0, `leave_party` runs and ends any group's hunt or engagement of that party that is not that slot's `current` state
(a group beyond the nearest 3, or one now unreachable) (execute.rs:82-86, 107-125). Read from the code, not observed.

### 3f. Failure and lateness

| Case | What happens | Where |
|---|---|---|
| Gate call fails | error logged, "every actor keeps its course". `sig`, `asked` and the events were already consumed, so nothing re-asks until the sig changes, a new event comes (`EVENT_GAP`), or `RE_ASK` (20 s). | mod.rs:964-969, 867-875 |
| Stage one fails or yields no candidate | no stage two, nothing played | plan.rs:1685-1692, mod.rs:1022 |
| Stage two fails | error logged, nothing played | mod.rs:1028, 964 |
| Jev client | 20 s timeout; a 429 or 5xx retried twice, waiting at most 5 s each | crates/jev/src/lib.rs:19-22 |
| Lockstep | all three calls are made in place; the game waits | mod.rs:942, 989-993 |
| Realtime | one request in flight; the worker answers only the newest queued request and runs the pick's two calls itself; an answer not back after `STALE_FRAMES` (3 s) is dropped unplayed; an answer that comes is played against the slots and picture of its request frame | mod.rs:246, 283-314, 1087-1120 |

---------------------------------------------------------------------------------------------------------------------

## 4. Code-side decisions still present

### 4a. In the menu (what is hidden, capped, or chosen before Jev sees it)

| What code decides | Constant / rule | Reason in a comment | Where |
|---|---|---|---|
| A listed, unthreatened builder has no slot | - | "the list is the player's order and the pick keeps off it (onepass-player-1: 117 picks on listed builders)" | plan.rs:453-476, 554-557 |
| ... except a solar when energy drains | store `< 0.25`, `usage > income`, no solar under way | "builders on lists were never offered one through 120 s of STALLING (game 7 8:00-10:00). Offered, not a default" | plan.rs:462-474 |
| A builder on a started build under 60 % has no slot | `QUEUE_AT = 0.6` | H-HANDS-STARTED / H-HANDS-QUEUE, no game cited at the site | plan.rs:478-480 |
| A builder with an unstarted build, a reclaim or a repair is offered only the ways out | - | "never over a build it has not started, unless queued behind it" | plan.rs:567-574 |
| A threatened listed builder is offered only the ways out | - | "onepass-player-2: a constructor sent home from a Pawn never got its list back" | plan.rs:554-563 |
| The way out's destination is home | `AWAY = 400` | none for the destination | plan.rs:513-518 |
| The step-away place: the nearest of ours beyond the party | `> 800` from the party, `+300` farther than the builder | "onepass-player-2, 12:03-12:40 ... the packet said 'walks away west or south' and no state could" | plan.rs:520-545 |
| Unarmed parties and lone scouts are no threat for the ways out (but any party within 800 still makes the builder `threatened`) | glossary class "scout"; `weapon_count` | "constructors fled unarmed air constructors for a minute (bluegecko-3v1-comet-catcher-2, 19:50)" | plan.rs:432, 497, 508; 277 |
| Extractor spots: nearest 2, up to 8 named, one safe | `NEAREST_SPOTS = 2`, `NAMED_SPOTS = 8` | "a slot that held only spot_2 and spot_19 ... sent the first constructor across the map (game 10)"; "onepass-smoke-1 ..." | plan.rs:579-600 |
| Spots hidden from a builder (`free_spots`) | refused 90 s; ally's ground 3 min; another builder's claim | "bluegecko-3v1-comet-catcher-8, 6:20" for the list exemption | plan.rs:284-313 |
| Builder menu cut to `usual_menu` without an allowance; converters never | fixed lists | "the hands built them unasked whenever energy banked (bluegecko-3v1-comet-catcher-8, 5:52)" | brain/roster.rs:48-66 |
| An exhausted allowance permits nothing; an allowance naming nothing this builder builds is no restriction | - | "models-medium-gpt6-astra"; "player-14" | plan.rs:622-645 |
| Nano turret hidden without a factory; sea buildings hidden without water in `build_distance + 300` | - | none | plan.rs:649-655 |
| Tier-2 extractor only over the nearest upgradable extractor | 1 | none | plan.rs:668-677 |
| Turret only beside the nearest 2 uncovered extractors in reach, none with a party at it | `TURRET_REACH = 1200`, `TURRET_COVER = 350`, `ALARM = 600`, 2 | "onepass-smoke-3: 54 turrets for 16 extractors" | plan.rs:680-705 |
| Radar/jammer/sonar at the nearest 2 marks in reach, beside itself only without one | `WALK_REACH = 3000`, 2 | none | plan.rs:708-718 |
| Help: the nearest factory; builders within 900 | `HELP_REACH = 900` | "a pick at 0.32 over an extractor was the default answer of an idle constructor" (for the words) | plan.rs:720-744 |
| Reclaim: the richest field in reach; repair: the nearest hurt static or commander | `>= 100` metal, 1800; `< 70 %`, 1200 | none | plan.rs:746-758 |
| Builder walks: nearest 2 marks | 3000, `> 150`, 2 | none | plan.rs:759-764 |
| A factory with a unit ordered ahead has no slot | - | "one order ahead (H-HANDS-MENU)" | plan.rs:770-779 |
| Group walks: its paragraph's places plus the nearest 3 | 3, `WALK_REACH`, `STATION_SLACK = 300` | "raids his north corner (spot_9, then ...) put nothing on the menu ... game 10"; player-33 for home | plan.rs:877-890, 1134-1153 |
| Walk vs advance from the word "advanc" | substring | "the E3 advance's whole text was its station's name" (for the words) | plan.rs:887-893 |
| Raids: nearest 3 places with his buildings | `RAID_REACH = 4000`, `RAID_STATES = 3`, 500 | "the user, game 10 at 5:58: two groups in position with 22-27 unguarded buildings within 3,000 held their stations" | plan.rs:927-961 |
| Sweep's first spot; his box first from the words "base"/"his " | - | "routes-in-prose 4.5: at 6:01 the sweep named four spots of our own half" | plan.rs:986-1000 |
| Gather's place: nearest the front | `STRUNG_OUT = 600` | none at the site | plan.rs:1001-1007 |
| Pull-out's place: nearest beyond the shooter's range | `range + 150` | "game 3 19:30: a group under Bull fire had neither" | plan.rs:1008-1025 |
| Shell: who is artillery, the standoff and the screen | `ARTILLERY_REACH = 600`; `0.85` and `0.55` of the reach | "game 9: four Shellshockers never used in the nest fight" | plan.rs:1026-1036, execute.rs:482-504 |
| Fall-back destination: where it last held (15 s); hidden when the hold is toward the party | `STATION_FRAMES = 15 s`, 300 | "wake-2 ... group_L fell back to where it stood ... eleven times in 30 s"; player-33 for removing home | plan.rs:1037-1061, groups.rs:41-44, 495-499 |
| One scout out at a time; the scout is the fastest soldier | - | "player-9-posing: three scouts walked at his base by the hands' Moves" | plan.rs:1063-1070, execute.rs:450-469 |
| Split: half the group, to the nearest 2 named places; the members nearest the destination go | `units/2` | none | plan.rs:1071-1074, execute.rs:435-448 |
| Merge target: the nearest same-domain group | - | "player-24, 7:26" (for the words) | plan.rs:1076-1100 |
| Which parties are threats | `RAIDER_REACH = 1200`; nearer home than his start; toward home | none at the site | threats.rs:55-64 |
| Groups offered against a party: nearest 3 | `GROUPS_PER_PARTY = 3` | "the question count stays bounded" | threats.rs:21, 103-105 |
| Hunters: who and how many | faster than the quarry, else the fastest; fewest with odds `>= 1.3`, else 2 | "a Blitz at 101 never outruns a Tick at 132 ... onepass-player-4 5:59" | threats.rs:129-147, 276 |
| No hunt for air groups; unreachable parties get a no-op state | - | "Cape Violet 11:17" | threats.rs:108-114, 134 |
| The stand at the next extractor on the heading | `along > 200`, `across < 0.6*along + 200` | "game 5: Incisor swarms bled the extractors 33 to 9 while the ball chased" | threats.rs:72-91 |
| Builder attack and D-gun offered within 1200 | `RAIDER_REACH`, `dgun + 100` | "onepass-norules-hard-1, 2:44"; "onepass-player-3, 2:36-3:04" | threats.rs:204-233 |

### 4b. In asking and composing

| What code decides | Constant | Reason in a comment | Where |
|---|---|---|---|
| Which actors are asked at all (`settle`) | `RE_ASK = 20 s`; own event; parties/store news | "player-29-hard ... 258,000 option questions (57 % of the gate's characters) bought 194 moves" | plan.rs:1244-1271 |
| A group on a route leg is not opened by parties | - | "flipped group_G 13 times in 53 s at E3, player-9 18:32-19:25" | plan.rs:1254-1257, 1266 |
| When the gate goes at all | `EVENT_GAP = 5 s`, `RE_ASK` | "onepass-medium-3: a group under fire asked every second, 118 of 663 asks" | mod.rs:840-852 |
| Store news only at the extremes | `< 100`, `>= storage - 1` | "the stock words' five buckets flapped ... onepass-medium-3: 58 of 663 asks" | mod.rs:815-825 |
| The flag bars | `FLAG 0.5`, `IDLE_BAR 0.3`, `FORBIDDEN 0.7`, `TOLD_BAR 0.25` | "onepass-medium-2: the commander idled six minutes on an extractor rated 0.48"; player-29-hard for FORBIDDEN | plan.rs:30-42, 64-70, 91-93 |
| Two deviations a slot | `DEPTH = 2` | "A slot's deviations in the second call at most" | plan.rs:43-44, 1413-1414 |
| An actor at a threat keeps no option of its own; a group at one party is not offered at another | - | compose's doc | plan.rs:1273-1280, 1300, 1333, 1359 |
| Pair worlds | `PAIR_COST = 300` | "A build this dear, or any factory, is worth a second builder's hands" | plan.rs:73-74, 1391-1409, 1519 |
| Worlds cut by count and by characters | `CAP = 255`, `LINE_CHARS = 40_000` | player-27, 10:20-13:00 (172 refusals) | plan.rs:19-29, 1730 |
| What Jev sees of the state | diet level; `STATE_CHARS = 160_000`; places removed | wake-1; player-10-routes; player-28 offline | diet.rs:35-42, 188-249; mod.rs:903-910 |

### 4c. Executed without any pick

| What | Rule / constant | Reason in a comment | Where |
|---|---|---|---|
| The player's lists | next step when the builder is free, when its build is 60 % done (queued), at once over an assist/walk/reclaim/repair, or over an unstarted build ordered before the list; steps that cannot be done are skipped and said; `assist` waits for a factory; `assist N` ends after N s | "plan-1: the commander helped a plant from 3:46 to 14:24 while four lists waited" | lists.rs:63-157, 219-311; mod.rs:626-644 |
| List held after a pick's retreat | while still threatened | "player-31: 83 of 92 such retreats were undone by the list within three seconds" | execute.rs:353-357, mod.rs:592-597, lists.rs:100-106 |
| A diverted list step goes back to the front of its list | - | "the list resumes where it was interrupted" | execute.rs:343-352 |
| List ids: one building, others help | - | the user, 2026-10-01 | mod.rs:395-419, lists.rs:37-59, 282-301 |
| Bare `extractor` step: the spot this builder reaches soonest | - | "the opening list is written before the start is known" | lists.rs:226-251 |
| Factory sequences | the first counted `produce` entry not yet made, ordered one ahead | "player-30's plant, told `armcv:1, armfav:3, ...` made ... three constructors by 2:23" | mod.rs:343-355, lists.rs:171-189 |
| Newcomer adoption | joins the group `produce` names for its factory, else the factory's own group; walks to the nearest standing member when the body is `> ADOPT_RADIUS = 400` away; else a new group, which is given stop orders | "human-8, eight Grunts lost by 3:14"; "Cape Violet"; "player-10 18:20" | groups.rs:336-406 |
| Soldiers given by another seat | one new group per domain, holding | - | groups.rs:343-363 |
| A joiner that reaches the body (or is stuck) gets the task's orders | 400 | "onepass-norules-hard-3, 4:47: a hunter released 850 from its holding group ... stood there for the rest of the game" | groups.rs:434-447, 209-223 |
| Arrival turns a walk into a hold | `ARRIVED = 300` at the 0.6-th member; `committed` when it advanced or shells | "Cape Violet 9:40-11:17" | groups.rs:500-518 |
| `last_hold` set after 15 s holding | `STATION_FRAMES` | wake-2 | groups.rs:495-499 |
| An engagement ends in a hold when the party is unseen | `LOST_FRAMES = 6 s`; air: search once, hold after `AIR_LOST_FRAMES = 60 s` | "evidence-1-bombers" | groups.rs:526-560 |
| A hunt ends (the micro engine): quarry dead, unseen 6 s, or `HUNT_LEASH = 900` from where it began; the group holds; the failure is remembered 3 min | `HUNT_LOST_FRAMES` (crates/micro/src/lib.rs:122) | "threats-smoke-1 ... 12 of 16 hunts one order long" | groups.rs:62-64, 667-684; brain/micro.rs:129-150 |
| Escort follows its ward; ends in a hold when the ward dies | `ESCORT_STEP = 150` | player-31, 3:12-3:22 | groups.rs:574-587 |
| A gather ends when the group is no longer strung out (flag only) | 600 | - | groups.rs:487-493 |
| Roving groups (a scout, or the player's `lane`): the lane drives them, no slot, no orders from the hands | - | H-MICRO-ROVE; "the user, 2026-09-28" | groups.rs:412-432, brain/micro.rs:104-109, crates/micro/src/rove.rs |
| Route facts recorded (`reached`, `met`) | `ARRIVED = 300`, `MET_REACH = 1200`, `REACHED_KEPT = 12` | "Code records; Jev picks the next leg" | groups.rs:451-486 |
| Wakes to the player: losses, stalls | `LOSS_WAKE_COUNT = 2`, `LOSS_WAKE_SHARE = 0.25`; `STALL_FRAMES = 45 s`, `PROGRESS_STEP = 60` | upgrade-2; pianist-player-2 | groups.rs:295-335, 519-523 |
| Each task's footwork commitment in the micro lane | advance or committed hold: all; hold/escort: priced, no turrets; engage: priced, his commander only at `>= 450` metal; walk: flees everything; hunt: raw | pianist-player-2, pianist-player-4 | brain/micro.rs:46-95 |
| A builder's tasks ended by timers | unstarted build idle after `ORDER_GRACE_FRAMES = 45` frames: refused, the spot off the menus 90 s; a walk ends within 150, after 40 s, or (an attack) after 10 s, only once the unit is idle; stuck 20 s: stopped | smoke-4; onepass-medium-1 | mod.rs:1283-1307, 1344-1368 |
| A builder's attack follows its party and stops when the party is gone | re-issued when the party moved `> 150` | "onepass-norules-hard-3, 2:55" | mod.rs:1308-1338 |
| Queued task promoted when the build finishes; dropped when the list was replaced | - | "player-32-hard, 0:17" | mod.rs:440-476, 1213-1260 |
| Construction turrets guard the nearest factory in reach | `RETELL_FRAMES = 10 s`, reach `+ 100` | "escalate-7: five nano turrets beside the plant ... idle" | brain/nanos.rs:14-50 |
| Resurrection bots: the field, raise or reclaim, **walk home** when nothing is to do | worth = metal / (dist + 500); raise when energy `> 0.5` of storage; home when `> 600` away | "2026-09-24: a produced Lazarus would have idled all game" | mod.rs:651-660, brain/reclaim.rs:105-162 |
| `remove` and `transfer` tools carried out | - | - | remove.rs, transfer.rs |

### 4d. What `execute_builder` / `execute_group` do beyond the picked order

| What | Detail | Reason in a comment | Where |
|---|---|---|---|
| The same build is not ordered twice | same type started (pick), or same type at the same place (list, `SAME_SITE = 250`); same lab already helped | "a list's `corsolar` step re-issued a pick's solar 15 frames after it"; fable-2-medium, 2:33 | execute.rs:154-180 |
| A factory order becomes help | a frame of that type within `FRAME_HELP = 900` is guarded instead | "game 10: a second advanced vehicle plant at 18:35, 224 from the first" | execute.rs:183-199 |
| **Where a `Building` goes** (`place_planned`): a factory in the yard toward the front of home; a nano turret beside the nearest factory, else the back field; a generator **at home's back field when the builder is `> GENERATOR_HOME = 1000` from home**, else beside the builder. The option's words say "beside itself" in every case. | `LAB_YARD 350`, `BACK_FIELD 150`, `TURRET_LINE 650` | "the user, 2026-09-28: 'we tend to build solars etc on the front lines rather than back at base'" | brain/economy.rs:60-85; plan.rs:663 |
| A defence stands 120 toward the enemy base from its place | `DEFENCE_FORWARD = 120` | "the user, 2026-09-28: 'our turrets are not built in reasonable locations'" | execute.rs:238-250 |
| A tier-2 extractor: helps one already under way; with none at the place, **upgrades the nearest other extractor** | - | none | execute.rs:219-237 |
| Extractor site offset toward the builder; sites snapped to the walker's ground (not for sea buildings) | - | Cape Violet, 2026-09-27 | brain/economy.rs:113-144, execute.rs:251-256 |
| Reclaim: the nearest 5 wrecks, not those worth raising while a crew exists | `QUEUE = 5` | "the commander games, 2026-09-20" | brain/reclaim.rs:127-134 |
| Commander's way home by one safe waypoint | 30 degrees, the route's midpoint | - | execute.rs:307-317, brain/routes.rs:233-254 |
| An attack is a fight order to where the party stood, remembered as a walk to `party_N` | - | - | execute.rs:300-306 |
| A D-gun order at the party's nearest unit (no step in is ordered, though the words say "it steps in") | - | - | execute.rs:284-291, threats.rs:229 |
| `queue`: the order goes behind the build and into `queued` | - | H-HANDS-QUEUE | execute.rs:200-205, 320-328 |
| Whole: an air group attacks a named unit (his commander, else the dearest) | - | "a fight order makes a bomber bomb the nearest thing on its line" | execute.rs:387-394, 523-532 |
| Hunt: the hunters become a group of their own; the quarry is the party's unit nearest them | - | the user, 2026-09-29 | groups.rs:686-723 |
| Walk / gather / fall back: every member is ordered, joiners included | - | - | execute.rs:401-434, 470-481 |
| Gather is a walk without fighting, then a hold; nothing sends the group "on where told" afterwards, as its words promise | - | - | execute.rs:470-481, plan.rs:1005 |
| Join: the members are moved to the other group's centre (home if neither has one) and are of its body at once | - | - | execute.rs:506-515 |
| Leave: hunts and engagements of the party are stopped (see 3e) | - | - | execute.rs:107-125 |

---------------------------------------------------------------------------------------------------------------------

## 5. State carried between seconds

### 5a. On `Group` (groups.rs:99-146)

| Field | Written by | Read by |
|---|---|---|
| `task` (`Hold{since, committed}`, `Move{to, place, fight, since}`, `Engage{..}`, `Hunt`, `Escort{ward, name, at, since}`) | `execute_group`, `leave_party`, `keep_groups` (arrival, lost party, dead ward, roving switch), `hunt_event`, `start_hunt` | `slots` (goal, idle, current flags, leave words), `threat_slots`, `signature`, picture, micro commitments, `rejoin_orders` |
| `members` | `keep_groups`, split/scout/join/hunt, transfer | everywhere |
| `last_hold` | `keep_groups` after 15 s of holding (never cleared) | `slots` (fall_back offer), `execute_group` FallBack, `rejoin_orders` |
| `reached` | `keep_groups` (cleared on a new route) | `slots` (`stop_cost`), picture (`route_seen`) |
| `met` | `keep_groups` | picture only |
| `route` | `keep_groups` | `keep_groups` only (to detect a new route) |
| `gathering` | `execute_group` Gather; cleared by `set_task`, by the roving switch, and when no longer strung out | `slots` (the gather state's `current`), picture (`doing`), arrival (kept across it) |
| `shelling` | `execute_group` Shell; cleared by `set_task` | `slots` (`current`), arrival (`committed` hold) |
| `joining` | `keep_groups` adoption; cleared on arrival within 400 or when stuck | `Group::body` (front, tail, arrived), picture (`reinforcements`), arrival measure |
| `scout` | `execute_group` Scout | `roves`, `slots` (`scout_out`), picture |
| `roving` | `keep_groups` (from `roves`), `execute_group` Scout | `slots`, `threat_slots`, `schedule_asks`, `apply_plan`, micro commitments, picture |
| `rove_log` | `rove_events` | picture |
| `best_to_go`, `progressed`, `stall_warned` | `keep_groups`, `set_task` | `keep_groups` (stall wake), picture (`progress`) |
| `losses`, `losses_since`, `loss_warned` | `keep_groups` | `keep_groups` (wake), picture |
| `parent`, `born` | split/scout/hunt | picture |
| `domain` | at creation | hunts, escorts, scouts, merges, orders |
| `last_order` | six sites in execute.rs and groups.rs | **nothing** |

### 5b. On `Pianist` (mod.rs:120-241)

| Field | Written by | Read by |
|---|---|---|
| `tasks` | `execute_builder`, housekeeping, `take_lists` (stop), `end_timed_assists`, `promote`/`take_queued`, `play_lists` | `slots` (task, `current`, `over_a_build`, started builds), `builder_status`, `free_spots`, turret cover, `count_of`, picture, `tag_state`, the team's spot claims |
| `queued`, `queued_steps`, `stale_queue` | `execute_builder` (queue), `promote`/`take_queued`, `list_replaced`, `take_lists`, `play_lists` | `builder_status`, `free_spots`, turret cover, housekeeping, picture |
| `scripts` | `take_lists`, `next_list_step`, `execute_builder` (step back to the front) | `slots` (`listed`), `free_spots`, `play_lists`, picture, `execute_builder` (`list_held`) |
| `list_steps` | `execute_builder`, `promote`, `end_timed_assists`, `list_replaced`, `play_lists` | `slots` (`listed`), `play_lists`, `end_timed_assists` |
| `list_held` | `execute_builder`, `list_is_held`, `list_replaced` | `play_lists` |
| `script_frame` | `take_lists` | `play_lists` |
| `tagged`, `helping` | `note_tag`, housekeeping, `tag_state`, `play_lists`, `list_replaced` | `next_list_step`, `play_lists` |
| `lab_queue` | `execute_lab`, housekeeping | `slots` (factory skipped), `play_sequences`, `production_draws` |
| `produced`, `allowed_seen` | housekeeping | menus (`permits`), `sequence_unit` |
| `groups`, `next_group`, `produced_by`, `rally` | groups.rs, execute.rs, housekeeping | everywhere |
| `asked` | `pass` when the gate goes | `settle`, `pass` (`due`) |
| `asked_parties`, `asked_store` | `pass` when the gate goes | `pass` (news) |
| `sig` | `pass` when the gate goes | `pass` (`changed`, the event gap) |
| `events` | `schedule_asks` (hit, sighted, alarmed), `run_pianist` (packet, places), `take_lists` (lists), `pass` (appeared), `keep_groups` (reached, met, escort end); cleared when the gate goes | `pass`, `settle` |
| `hits` | `schedule_asks` | `under_fire` -> `slots`, `play_lists` |
| `alarmed` | `schedule_asks` | `schedule_asks` |
| `hunts_failed` | brain/micro.rs on a hunt's end without a kill (pruned to 3 min only when a new one is pushed) | `threat_slots` (the hunt's words, looked up by party name alone) |
| `refused_spots`, `refused_sites` | housekeeping | `free_spots`, lists, site search |
| `packet_frame`, `packet_seen` | `run_pianist` | `pass` (sig), `keep_groups` (route) |
| `places`, `parties` | `run_pianist`, `collect_answer` | `keep_groups`, `schedule_asks`, housekeeping, picture (party names) |
| `places_seen` | `run_pianist` | `run_pianist` (the `places` event) |
| `slots` | `after_gate`, only when worlds were composed | `pass` (the "appeared" events) |
| `candidate` | `after_gate` | `after_pick` (log) |
| `worlds` | `pass` (cleared), `after_gate` | **nothing** |
| `pending`, `worker`, `next_request`, `last_ask_frame` | `call`, `collect_answer`, `run_pianist` | `run_pianist` (due), `collect_answer` |
| `logged_instructions` | `log_call` (only when the log is on) | `log_call`, **and the packet-change test** (mod.rs:699) |
| `recent`, `done`, `played`, `hunt_events`, `rove_events`, `stats`, `party_memory`, `next_party` | various | picture, the player's report, the log |

---------------------------------------------------------------------------------------------------------------------

## 6. Rot

### 6a. Comments and docs describing what the code no longer does

| Where | Says | Code |
|---|---|---|
| mod.rs:2-6 (module doc) | "puts the base world in force"; "asks Jev twice: the pre-pass's nouls, then one Choice" | the base world is never applied (the only `apply_plan` call is the pick, mod.rs:1039); three calls |
| mod.rs:121, 478-479, 569-574 | "without it only the lists and the rules' defaults play"; `model()` returns "standing orders and lists, no Jev" | no rule defaults or standing orders exist; without Jev `pass` returns at mod.rs:855 |
| mod.rs:1036-1038 | "A pick below the bar moves no group that is fighting (player-17 21:34 ...)" | no confidence bar anywhere |
| mod.rs:588 | doc "A new group's name: A, B, C, ..." | sits on `list_is_held`; `new_group_name` (mod.rs:608) has none |
| mod.rs:185 | "`groups.rs` `tick_hunt`" | no such function (`hunt_event`) |
| mod.rs:1372 | "the standing orders in force" | not published |
| execute.rs:1-4 | "`rule` for the base world" | no `rule` source |
| execute.rs:56-61 | "declines it for a while"; "`held` marks the slots this world does not touch" | no decline timer, no `held` parameter |
| execute.rs:75-77 | "had its base put in force by the rule when the gate was asked" | no rule |
| plan.rs:1-5 (module doc) | "which kinds of action matter now"; "The plan runs in code until the picture's signature changes; nothing else decides" | the per-kind nouls are gone (said at plan.rs:1132); lists, sequences and 4c decide |
| plan.rs:154 | `dim`: "for the pre-pass's dimension nouls" | used only for `course`/signature, the pair test and the log; `Keep` has dim "threat" |
| plan.rs:507 | "the rule's default" | none |
| plan.rs:564-566 | numbered step "2." about the builder attack | a note only; no code at that step |
| plan.rs:582 | "The nearest spot, and a safe one, only when the instructions name none" | the nearest 2 are always offered |
| plan.rs:659-662 | "A solar the moment energy stalls is the rule's default (`only_when_stalling`) ... a solar from a free builder whenever the store is under a quarter" | no rule, no such logic at that site |
| plan.rs:680-682, 720, 1062, 1212 | `beside_each_outer_extractor`, `job help_factory`, `no_detachments`, `job expand` | standing-rule names with no code |
| plan.rs:868-869 and threats.rs:51-54 | H-HANDS-ENGAGEMENT-PLAN: "A party of it gets no whole-group attack from this slot" | no code follows either comment; the register says the plan was deleted (code at 3fbb5a0) |
| plan.rs:1130-1133 | the doc of `gate_questions` | sits on `walk_places` |
| plan.rs:1363-1366 | "a clear call against a rule's default" | none |
| plan.rs:1059 (words sent to Jev) | "less far than the base" | the base alternative was removed in e690e5e |
| threats.rs:148 | "A hunt of this party by this group" | looked up by party name alone: another group's failed hunt is said as "its last hunt" |
| groups.rs:1-4 | "a march arrives together, an engagement follows its party" | `keep_groups` issues no march orders and, without a named target, only updates `at` (unsure whether crates/micro's formation code is what is meant) |
| groups.rs:339 | "the standing rule `join`" | none |
| groups.rs:733 | a comment cut off mid-sentence, citing `tick_groups` | no such function |
| lists.rs:7-8 | "nothing in the pass reads a list" | `slots` and `free_spots` read `scripts` and `list_steps` |
| picture.rs:140, 420 | stray doc lines ("How many constructors we have ...", "The fight simulator's odds ...") | sit on `soldier_words` and `production_draws` |
| picture.rs:776 (`task_course`) | "helping lab_{id} build" | the target may be a plant, a factory or another builder |
| brain/roster.rs:48-50 | "always by the policy" | the policy was deleted |
| rules.md | `send_against`, `fight_to`, `walk_to`, "\"continue\" keeps an actor's task", "A group already walking back, to where it last held or home", "`sweep` looks at the spots", "the place the instructions name for that case" | no state has those names (ids are `hunt_`, `advance_`, `walk_`, `keep`); the hands' walk home for groups is gone |
| default.md | `fight_to`, `scout`, `passage_1`, "Retreat home at once when under fire" | same vocabulary; read only when neither a player nor `--packet` gives a packet |

### 6b. Dead or ineffective code, and behaviour the comments do not describe

| Where | What |
|---|---|
| groups.rs:106 `Group::last_order` | written at six sites, never read |
| mod.rs:175 `Pianist::worlds` | written, never read |
| mod.rs:833-834 | `base` world computed and resolved in `pass`, never used |
| execute.rs:97-98 | `if source == "plan" { }`, empty |
| execute.rs:75-80 | the closed-slot skip does nothing for actors (a base state is `current`, skipped at :69, or index 0 `Keep`, a no-op); its one effect is that a threat slot holding only `leave` does not run `leave_party` |
| execute.rs:544-546 | `frame_now(tick)` wraps `tick.frame`; at :193 `frame` shadows the frame number with a unit id |
| plan.rs:610 | reads `places[spot].ground`; the picture never writes that key, so every extractor option says "ground " and nothing |
| plan.rs:832 | a factory slot's `queue_ahead` is set and never read |
| threats.rs:111-114 | `party_N.unreachable_group_X` is a `Keep` state inside a threat slot: it is asked as "the move to make", can be composed and picked, `resolve` then treats the group as sent (its own slot forced to keep), and the line says the party is "met with 0 metal: it outweighs us" (plan.rs:1274, 1594-1600). Read, not observed. |
| execute.rs:82-86 | `leave` at index 0 runs `leave_party` on every applied pick for every open threat slot (see 3e) |
| mod.rs:684-691 | realtime: while a request is in flight the lists and factory sequences are not played either (they sit behind the same `due` test) |
| mod.rs:867-875 | a failed or dropped gate is not asked again: `sig`, `asked` and the events are consumed before the call |
| mod.rs:906 with diet.rs:162-173, plan.rs:1759-1761, diet.rs:213-220 | `places` is removed from the gate state, and the pick's state is cut from the gate's, so Jev never sees `places`: the diet's `places_reach`/`places_held_too`, the pick's place filter and the shed's place step have nothing to act on |
| diet.rs:192 vs crates/jev/src/lib.rs:28 | `STATE_CHARS = 160_000` is above `REQUEST_CHARS = 150_000`: a state between the two would put every question in a batch of its own (read, not observed) |
| plan.rs:64-70, 1199-1201, 1348-1354 | the told noul: off unless `WITHIN_REASON_TOLD=1`, and its own comment says it failed |
| threats.rs:57 and 75 | the party's summed velocity computed twice |
| threats.rs:17-19 | a constant between two `use` lines |
| execute.rs:486, 497 | `600.0` typed out instead of `ARTILLERY_REACH` |
| mod.rs:796 | "appeared" is measured against the slots of the last ask that composed worlds, not the last second's |
