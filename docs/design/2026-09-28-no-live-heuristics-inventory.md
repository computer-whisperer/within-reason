# Inventory: every live decision above the micro engine (2026-09-28, at a6d31c6)

Built by an Opus survey agent from the code and player-22-commit's logs; nine of the most-fired sites' file:lines were
re-checked by hand (mod.rs:725/737/1290, plan.rs:841/959/1350/1362, march.rs:20, lists.rs:45). The cut line is in
`2026-09-28-no-live-heuristics.md`.

# Inventory: live decisions made without Jev or the player in root control

Scope: `crates/bot/src/brain/pianist/*.rs` and `crates/bot/src/brain/*.rs` except `micro.rs`, at a6d31c6 (main). No file
was edited. Line numbers are those of the working tree today.

"Fired" = counted in `run/matches/1790646080-player-22-commit/00/` (player-22). The method for each count is at the end.
"Off" = whether the player can switch it off (a `standing` key, a `lane` word, a list, nothing).
"Jev sees" = whether the trigger is in the picture or the state words Jev reads before the decision.

## A. Code issues or cancels an engine order on its own

Ordered by player-22 firings. "Continuation" marks sites that carry out a pick or list step after the fact (follow a
moving party, the next phase of a picked plan). Their trigger is still code's alone.

| # | file:line | function | what it decides | trigger | cited id | Off | Jev sees | fired (player-22) |
|---|---|---|---|---|---|---|---|---|
| A1 | march.rs:26-47 (called groups.rs:571-573, 662-670) | `march` | stops leaders of an advancing group and lets them go when the body is up | group >= 4, advancing (`Move{fight}`), leader 350 ahead of the body, no enemy within 900 | H-ARMY-MARCH | lane word `march` | yes, the entry's `ranks` line | 124 firings |
| A2 | lists.rs:49 (set at execute.rs:337) | `play_lists` | the player's list does not resume for 8 s after the pass diverted the builder | a pick moved a listed builder | none (player-11 3:12) | no | no | 26 diversions held |
| A3 | groups.rs:744-772 | `hunt_event` | when the micro engine ends a hunt: releases the hunters with `rejoin_orders`, declines the party 30 s, records a failed hunt | micro `HuntEvent` (leash 900, lost 6 s, dead, no hunters) | H-MICRO-HUNT | no (`no_detachments` stops new hunts) | no; the failed hunt is said on the next hunt state | 25 ends (12 leash, 4 lost, 2 dead, 7 no hunters) |
| A4 | groups.rs:549, 558-565 (`is_mark_name` at 429) | `keep_groups` | "forgotten mark": a walk to a place that is not a mark stops and holds | the task's place is not home/spot/passage/last hold and is not in `shared.marks` | none | no | no | 21, every one a pick to `shelling`, `shelling_F` or `standoff from party_N`, stopped the same second (see note 1) |
| A5 | mod.rs:1294-1317 | `pianist_housekeeping` | a build the engine never started: task dropped; its spot (or site) hidden 90 s; the spot goes to `centre_only` | idle after `ORDER_GRACE_FRAMES` with no nanoframe | H-HANDS-REFUSED | no | after the fact, in `recent` | 16 refusals |
| A6 | execute.rs:94-115 via apply_plan with source `rule` | `leave_party` | the base world's `Leave` calls off a hunt or ends an engagement and declines the party | a threat slot whose base is Leave while a group hunts or engages a unit of it (the quarry's party renamed or split, so the state is not `current`) | none | no | no | 2 by rule (12 more by pick, not counted as A) |
| A7 | groups.rs:580-585 | `keep_groups` | follow leash: an engaging ground group drawn 900 from where it engaged holds and wakes the player | `centre.dist2d(from) > FOLLOW_LEASH` | none (K-hands-follow-had-no-leash) | lane word `follow` | no | 7 |
| A8 | engagement.rs:778-861 | `tick_plan` | runs a picked plan: phase ends (targets dead, out of sight 10 s, 60 s), steps out of cover, re-issues Fight/Attack/Move every 2 s, holds at the end | the plan's own clocks | H-HANDS-ENGAGEMENT-PLAN | `plan: no` stops new plans | the plan line is in the picture | 9 plans taken, 2 phase lines, 4 finished (continuation) |
| A9 | mod.rs:1278-1288 | `pianist_housekeeping` | a builder's attack on a party: stop when the party is gone | party name no longer in the picture | none (onepass-norules-hard-3) | no | no | 5 |
| A10 | nanos.rs:34-50 | `tend_nanos` | idle construction turret guards the nearest factory, re-told every 10 s | idle nano with a finished factory in reach | H-ECO-NANO-GUARD | no | picture says N nanos on a factory | 4 guard orders |
| A11 | groups.rs:590-595 | `keep_groups` | a chase into a `never` place stops and holds | quarry within 400 of a `never` place | none (worlds-2) | the rule is the player's | the `standing` line, yes | 2 |
| A12 | groups.rs:558-565 (`given_up`), 47 | `keep_groups` | an advance or walk that has not got 60 nearer in 90 s is given up; the group holds | `GIVE_UP_FRAMES` | H-HANDS-STALL | no | the stall words, from 20 s | 0 |
| A13 | lists.rs:162-164 | `next_list_step` | a timed `assist N` step is skipped while the store is under 20 metal | `ASSIST_STORE_FLOOR` | none (K-open-comet-our-plant-starves) | no | yes, the store | 0 |
| A14 | mod.rs:493-511 | `end_timed_assists` | stop after a timed `assist N` step's N seconds; the list goes on | the step's own clock | H-HANDS-SCRIPT | the player writes the N | no | not logged |
| A15 | groups.rs:598-619 | `keep_groups` | named-target engagement: re-attack every 2 s; ground holds when the target is unseen 6 s; air searches once, holds at 60 s | `LOST_FRAMES`, `AIR_LOST_FRAMES` | H-HANDS-AIR-TARGET | no | no | not logged |
| A16 | groups.rs:620-626 | `keep_groups` | follow: an engaging group re-sent to fight at the party's new centre | party moved 150, 2 s since the last order | none | lane word `follow` | no | not logged (continuation) |
| A17 | groups.rs:627-630 | `keep_groups` | an engagement whose party is unseen 6 s ends in a hold | `LOST_FRAMES` | none | no | no | not logged |
| A18 | groups.rs:400-406, 476-481 | `keep_groups` | a newcomer walks to its group's nearest member; once within 400 of the body it takes the group's task | produced soldier, body beyond `ADOPT_RADIUS` | H-HANDS-GROUPS, H-HANDS-GROUP-BODY | `produce ... group` picks the group, not the walk | the `reinforcements` line | every soldier made; not logged |
| A19 | groups.rs:420-421, 376-377 | `keep_groups` | a new group or a group of given soldiers is stopped where it stands | first soldier of a factory, a transfer | H-HANDS-GROUPS | no | no | not logged (transfers: 0) |
| A20 | mod.rs:1260-1273 | `pianist_housekeeping` | a builder attacking a party is re-sent to fight at the party's new place | party moved 150 | none | no | no | not logged (continuation) |
| A21 | mod.rs:1253-1257 | `pianist_housekeeping` | a walker stuck 20 s is stopped and its task dropped | `yards.rs` `stuck` 20 s | none (onepass-medium-1) | no | stuck words, yes | not logged |
| A22 | mod.rs:1236-1247 | `pianist_housekeeping` | task dropped (no engine order): a walk after 40 s or at 150, a walk at a party after 10 s, reclaim/repair idle | clocks | none | no | no | not logged |
| A23 | groups.rs:551-557 | `keep_groups` | arrival: a move becomes a hold (no order; `committed` if it was an advance) | 60% of the core within 300 | H-HANDS-GROUPS | no | yes | not logged |
| A24 | mod.rs:1165-1166, 1205, promote at 339-362 | `pianist_housekeeping` / `promote` | a queued next task becomes the task; an assist is ordered (Guard) as the build finishes | build finished or frame started | H-HANDS-QUEUE | no | no | not logged (continuation of a pick or list step) |
| A25 | mod.rs:521-527 and reclaim.rs:136-165 | `run_pianist`, `work_wrecks` | resurrection bots raise or reclaim wrecks on their own, or walk home | idle resurrector | H-REC-CREW, H-REC-RESURRECT | the player need not produce one | no | none produced (0) |
| A26 | groups.rs:679-693 | `tick_hunt` | hunt ended in the book when every hunter is dead (no order) | no hunter left | H-MICRO-HUNT | no | no | counted in A3's "no hunters" |
| A27 | lists.rs:46 | `play_lists` | the player's list is held 30 s after the builder went home from an enemy | `RETREAT_HOLD` | none (standing-2) | no | no | not logged |

Not A (a player order stands behind the second): mod.rs:604-620 (`stop` at the head of a list), remove.rs:46-116,
transfer.rs:12-83, groups.rs:449-459 (the lane switched to or from `rove`: the hold orders at 457 follow the player's
`lane`).

## B. Code prunes or hides a state before Jev sees it, or refuses a pick

Ordered by what player-22 measured; most pruning leaves no trace in the log, so most rows say "every pass".

| # | file:line | function | what it removes or refuses | trigger | cited id | Off | Jev sees | fired (player-22) |
|---|---|---|---|---|---|---|---|---|
| B1 | mod.rs:725-735, 754-759 | `pass` | no question goes out; the base world stands ("quiet") | signature unchanged, `RE_ASK` 20 s, `EVENT_GAP` 5 s | H-HANDS-ONE-PASS | no | no | 320 quiet seconds |
| B2 | plan.rs:1352 (actors), 1308, 1327 (threats) | `compose` | an actor or threat whose gate noul is under `FLAG` 0.5 opens nothing | Jev's own noul < 0.5 | H-HANDS-ONE-PASS | no | Jev set the noul | 273 of 976 gates opened nothing |
| B3 | plan.rs:1364 (`OVER_RULE` 82) | `compose` | a deviation from a rule's default needs 0.7, not 0.5 | base is a default not in force | H-HANDS-ONE-PASS (onepass-hard-1) | clear the rule | no | 99 gates with a state at 0.5-0.7 held under a default (under the lab default 58, `job expand` 24, station 13, fall_back_to 2, solar 1, turret 1) |
| B4 | threats.rs:134-136, 184-185, 187, 227 | `threat_slots` | `scouts_vs_armed`: a scout body is offered no hunt and no whole-group attack on an armed party | scouts > half the armed metal and the party armed | H-HANDS-SCOUTS-FIGHT-NOTHING-ARMED | no | yes, a Keep state says why | 120 slot-states |
| B5 | mod.rs:884-908 (`ENGAGED_PICK_BAR` 290) | `after_pick` | a pick under 0.25 confidence does not move a group that is engaging or on a plan | pick confidence | H-HANDS-ENGAGED-PICK-BAR | no | no | 180 picks under 0.25; 12 picked group moves not played |
| B6 | groups.rs:131-133, 756-757; execute.rs:101, 110; read at threats.rs:170, 187, 199, 228 | `hunt_event`, `leave_party`, `threat_slots` | `declined`: for 30 s no hunt state and no rule default against a party this group left or finished hunting | any hunt end or Leave | none | no | no | about 39 declines set (25 hunt ends + 14 Leaves) |
| B7 | mod.rs:1307, plan.rs:305 | `pianist_housekeeping`, `free_spots` | a refused spot is off every extractor state for 90 s | engine refusal | H-HANDS-REFUSED | no | no | 16 refusals (see A5) |
| B8 | mod.rs:786-790; diet.rs:145-190, 197-249 | `trim_state`, `diet::shed` | places beyond 1,500 (lean) and unasked actors cut to a line; over 160,000 characters the state is shed | `WITHIN_REASON_HANDS_EFFORT`, size | H-HANDS-DIET | env setting | n/a | every gate (lean in this game); shed 0 |
| B9 | plan.rs:1548-1566 | `pick_state` | the pick's state drops places and actors the worlds do not name | every pick | none | no | n/a | 703 picks |
| B10 | mod.rs:691-705 | `pass` | a group walking a leg of its route is closed (quiet) unless an event names it | `on_route` and no order event | routes-in-prose §4.4 | no | no | not logged |
| B11 | threats.rs:78 | `threat_slots` | a party in his half, away from our structures and not coming home, gets no threat slot | position and heading | none | no | the party is in `enemy` | not logged |
| B12 | threats.rs:196, 370 | `hunters_for` | a hunt is offered only when some fast members outweigh the party at 1.3 | combat odds | H-MICRO-HUNT | no | odds words, yes | not logged |
| B13 | threats.rs:227 | `threat_slots` | no whole-group state when the odds read "it outweighs us" or "we cannot hit", when the position is planned (`planned` 63-70), or beyond the station under `no_chase` | odds string, plan, `no_chase` | H-HANDS-ENGAGEMENT-PLAN | `no_chase` is the player's | the odds, yes | not logged |
| B14 | threats.rs:263 | `threat_slots` | the way back only when the odds read "it outweighs us" and within 600 | odds string | none | no | yes | not logged |
| B15 | threats.rs:309 | `threat_slots` | a builder's attack only when its odds start "we outweigh" | odds string | H-HANDS-COMMANDER-FIGHTS | no | yes | 87 attack states offered |
| B16 | plan.rs:512, 516-518 | `slots` | retreat home offered only from a party it does not outweigh ("we outweigh"), not a lone scout, and not when home is unsafe or banned | odds string, `never home` | H-HANDS-STEP-AWAY | `never` | yes | not logged |
| B17 | plan.rs:501-502 | `slots` | an unarmed party is no threat to a builder | no weapon | none | no | no | not logged |
| B18 | threats.rs:21, 187 | `threat_slots` | `HUNT_PARTY_MAX` 6: a larger party is never hunted; no hunt for air groups or under `no_detachments` | size, domain, rule | none | `no_detachments` | no | not logged |
| B19 | threats.rs:23, 121 | `threat_slots` | `GROUPS_PER_PARTY` 3: only the three nearest groups get states | distance | none | no | no | not logged |
| B20 | threats.rs:119, plan.rs:908, execute.rs:52-59 | several | a roving group gets no state, and a pick for it is not played | `roving` | H-MICRO-ROVE | lane `rove` | the `lane` line | not logged |
| B21 | engagement.rs:654-701 | `engagement_of` | no engagement plan for roving, air, scout bodies, `plan: no`, a holding body, a position beyond `PLAN_REACH` 1,200 or at a `never` place | gates | H-HANDS-ENGAGEMENT-PLAN | `plan: no`, `never` | no | not logged |
| B22 | engagement.rs:1004-1029 | `ask_reason` | hysteresis: no re-ask for 20 s (30 s after a decline), none while the asked phase runs; else only on a death, a new static, a new party > 20% or 45 s | memory | H-HANDS-ENGAGEMENT-PLAN | no | no | 9 asks |
| B23 | engagement.rs:1080-1087 | `choose` | the top plan is refused for `decline` unless it is above 0.4 and above decline | `PLAN_BAR` | H-HANDS-ENGAGEMENT-PLAN | no | no | 0 |
| B24 | plan.rs:480-482, 573-579 | `slots` | a started build under 60% has no states; an unstarted build, reclaim or repair only the escapes | task | H-HANDS-STARTED | no | no | not logged |
| B25 | plan.rs:462-477, 563-568 | `slots` | a listed builder is out of the pass (a solar offered while energy drains; the escapes when threatened) | the player's list | H-HANDS-SCRIPT | the player's list | the actor says "on its list" | not logged |
| B26 | plan.rs:813-815 | `slots` | a lab with an order waiting is not asked | `lab_queue` | H-HANDS-MENU | no | no | not logged |
| B27 | plan.rs:284-313 | `free_spots` | spots taken by another builder, a team mate's claim, near an ally's start for 3 min (allies.rs:42-45, H-TEAM-ALLY-GROUND), an enemy extractor, unreachable | several | H-HANDS-LISTS, H-TEAM-ALLY-GROUND | no | no | not logged |
| B28 | plan.rs:599-612 | `slots` | extractor states: the nearest 2, up to 8 the instructions name, one safe | counts | H-HANDS-ONE-PASS | name spots | no | not logged |
| B29 | plan.rs:644-660; roster.rs:68 | `slots` | kinds pruned to the `produce` allowance, else the faction's usual menu | allowance | H-HANDS-PRODUCE, H-HANDS-ROSTER | `produce` | no | not logged |
| B30 | plan.rs:667-681 | `slots` | no sea building off water, no nano without a factory; `solar never` / `only_when_stalling`, `turrets none` prune | map, rules | H-HANDS-FORBID | the rules | no | not logged |
| B31 | plan.rs:715-740 | `slots` | turrets only beside our uncovered extractors within 1,200, nearest two, none where a party stands | cover, parties | H-HANDS-ONE-PASS | no | no | not logged |
| B32 | plan.rs:744-753 | `slots` | radar-type buildings only at the nearest two marks, else beside the builder | marks | none | `mark` | no | not logged |
| B33 | plan.rs:939-942 | `slots` | `odds_against` read off the odds string ("we outweigh", "it cannot hit us"): gates the station default, the fall-back default, `hold_line` and the fall_back state | odds string | H-HANDS-FALL-BACK | no | the odds, yes | not logged |
| B34 | plan.rs:1113-1147 | `slots` | `hold_line` prunes the ways back; fall_back to the last hold offered only when odds are against or losing 10% in 30 s | rule, odds | H-HANDS-FORBID, H-HANDS-FALL-BACK | `hold_line` | partly | not logged |
| B35 | plan.rs:1091 | `slots` | close on the unseen shooter only when the estimate's words contain "we outweigh" | odds string | H-HANDS-SHELLED | no | yes | not logged |
| B36 | plan.rs:992-995, 1039, 1055 | `slots` | walks: `WALK_REACH` 3,000 unless named, three off-route places; raids within 4,000, nearest 3 | distance | H-HANDS-GROUP-STATES | name places | no | not logged |
| B37 | plan.rs:1149-1161 | `slots` | `no_detachments` prunes scout and split; one scout out at a time | rule | H-HANDS-FORBID | the rule | no | not logged |
| B38 | plan.rs:1327-1331, 1356 | `compose` | a state whose actor world 1 already sends at a threat is dropped | base world | H-HANDS-ONE-PASS | no | no | not logged |
| B39 | plan.rs:1366 | `compose` | an idle actor with nothing at the flag opens its best at `IDLE_BAR` 0.3 (loosens rather than prunes) | idle | none | no | no | not logged |
| B40 | plan.rs:23, 27, 1411, 1420 | `compose` | `DEPTH` 2 deviations a slot, `CAP` 16 worlds | counts | H-HANDS-ONE-PASS | env `WITHIN_REASON_WORLDS` | no | not logged |
| B41 | execute.rs:154-157, 163-177 | `execute_builder` | a picked build of a kind already started is absorbed; a picked factory beside a frame of that type becomes an assist | tasks, frames within 900 | H-HANDS-STARTED | no | no | not logged |
| B42 | threats.rs:145, 148; plan.rs:484, 517, 589, 728, 744, 795, 961, 992 | several | `raiders_* ignore` and `never` prune states | the player's rules | H-HANDS-FORBID | the rules | the `standing` line | every pass with the rules set |
| B43 | threats.rs:154-157 | `threat_slots` | a party the group cannot reach gets a Keep state saying so | walker | none | no | yes | not logged |

## C. A default that plays when Jev does not answer (base world, source `rule`)

142 base-world plays in player-22 (`pass` rows' `played[].source == "rule"`).

| # | file:line | what it defaults | trigger | cited id | author | Off | Jev sees | fired (player-22) |
|---|---|---|---|---|---|---|---|---|
| C1 | plan.rs:841-876, 888 | the idle lab's next unit: the allowance's first permitted entry, else its most made, else its cheapest armed unit | lab idle | H-HANDS-LAB-DEFAULT | code (reads the `produce` list) | no switch; `produce` shapes it | yes, "what it makes unless told otherwise" | 110 (armstump 67, armflash 26, armfav 10, armcv 4, armart 2, armjanus 1) |
| C2 | plan.rs:961-972 | the station walk or advance | group away from `station` by 300, not engaging, planning, hunting, walking back, and odds not against | H-HANDS-STANDING | player (`station`, `station_mode`) | clear the rule | the `standing` line | 17 |
| C3 | threats.rs:195-199 | hunt by N fastest | `raiders_lone`/`raiders_party: detachment[:N]`, not declined | H-HANDS-STANDING | player | the rule | yes | 5 |
| C4 | plan.rs:614, 618 | the nearest safe free spot's extractor | `job: expand`, builder free | H-HANDS-STANDING | player | the rule | yes | 5 |
| C5 | threats.rs:228 | whole-group attack | `whole_group` or `engage_party` names the party, not declined | H-HANDS-STANDING | player | the rule | yes | 2 |
| C6 | execute.rs:94-115 via base world | Leave (see A6) | quarry's party changed | none | code | no | no | 2 |
| C7 | plan.rs:689-693 | a solar beside the builder | `only_when_stalling` and STALLING, or (no rule needed) the store under 25% and draining with no generator under way, unless `solar never` | none (comet-catcher-7..9) | code, with the rule as an addition | `solar never` | yes, the draining words | 1 |
| C8 | plan.rs:518-522 | go home | `retreat_when_enemy_near` and a party it does not outweigh within 600 | H-HANDS-STANDING | player | the rule | yes | 0 |
| C9 | plan.rs:535-548 | step away to the nearest place of ours out of reach | the same rule, home unsafe | H-HANDS-STEP-AWAY | player | the rule | yes | 0 |
| C10 | plan.rs:731-735 | a turret beside the nearest uncovered (outer) extractor | `turrets` rule, builder free | H-HANDS-STANDING | player | the rule | yes | 0 plays (33 offers each of five turret kinds) |
| C11 | plan.rs:757-770 | help the nearest factory | `job: help_factory`, builder free | H-HANDS-STANDING | player | the rule | yes | 0 |
| C12 | plan.rs:973-979 | stop the chase and hold | `no_chase` and the quarry beyond the station's reach | H-HANDS-STANDING | player | the rule | yes, the words say `no_chase` | 0 plays (17 offers) |
| C13 | plan.rs:1121-1125 | fall back to `fall_back_to` | odds against or losing, not walking back, not planning | H-HANDS-FALL-BACK | player | the rule | yes | 0 plays (51 offers) |
| C14 | plan.rs:1163-1166 | merge into the named group | `join` | H-HANDS-STANDING | player | the rule | yes | 0 |
| C15 | threats.rs:315 | a builder attacks a party it outweighs | `attack_raiders` | H-HANDS-COMMANDER-FIGHTS | player | the rule | yes | 0 |
| C16 | groups.rs:385-388, 833-841 | a newcomer joins its factory's own group | a soldier made, no `produce ... group` | H-HANDS-GROUPS | code | `produce` group | yes | every soldier; not logged |
| C17 | micro.rs:104-107 (outside the survey) | a scout made by the hands' `scout` state roves | no `lane` set for it | H-MICRO-ROVE | code | `lane` | the `lane` line | group_H: 45 give-ups, 19 attacks, 25 retreats in `rove` lines |
| C18 | remove.rs:96, 120-126 | the builder that takes a unit apart when the player names none | `remove` without `by` | H-PLAYER-REMOVE | code | name `by` | n/a | not logged |

Modifiers of the defaults (they decide when a default fires, not what):
- mod.rs:737-744: the base is put in force at once on slots with nothing to decide, and on every slot on a quiet
  second or when Jev is absent or failed (source `rule`).
- plan.rs:584-585: no builder default for `PICK_HOLD` 60 s after a pick, or `RETREAT_HOLD` 30 s after a retreat.
- plan.rs:1263-1271 `hold_current`: a default is replaced by keep when the actor is doing a current state of another slot.
- plan.rs:1273-1280 `resolve`: an actor sent at a threat keeps its course slot at keep.
- threats.rs:57, 158, 199, 222, 228, 243, 315, 336 `defaulted`: one rule default per actor per second, first party wins.
- plan.rs:202-203 `Slot::open`: a threat whose default is due counts as a decision (it is held for the pick).

## The player's standing rules: applied by code without a pick

Yes. A rule's default is played from the base world with source `rule`: on quiet seconds, on slots with nothing else
to decide, and after a failed or dropped answer (mod.rs:737-744). Its pruning applies on every pass. Nothing asks Jev
before a default fires on a quiet second. In player-22, 29 of the 142 `rule` plays came from player keys (station 17,
detachment 5, job expand 5, whole_group or engage_party 2); 110 were the code's lab default.

Vocabulary: standing.rs:22-36 (`GROUP_RULES`), 39-46 (`BUILDER_RULES`). Where each key acts:

| key | default (C) | pruning (B) or order (A) |
|---|---|---|
| `station`, `station_mode` | plan.rs:961-972 | threats.rs:163 (the `no_chase` anchor) |
| `raiders_lone`, `raiders_party` (`whole_group`, `detachment[:N]`, `ignore`) | threats.rs:195-199, 228 | threats.rs:125, 145 (`ignore`) |
| `engage_party` | threats.rs:126, 228 | threats.rs:145 |
| `no_chase` | plan.rs:973-979 | threats.rs:164, 227 |
| `no_detachments` | none | threats.rs:187; plan.rs:1149 |
| `hold_line` | none | plan.rs:1113-1114 |
| `fall_back_to` | plan.rs:1121-1124 | threats.rs:264 (the back state's place) |
| `join` | plan.rs:1164 | none |
| `never` | none | plan.rs:484, 517, 531, 554, 589, 702, 728, 744, 795, 929, 961, 992, 1033, 1063, 1076, 1096; threats.rs:124, 148; engagement.rs:685-689; A at groups.rs:433-440, 590-595 |
| `plan: no` | none | engagement.rs:667 |
| `job` (`help_factory`, `expand`) | plan.rs:618, 759 | none |
| `attack_raiders` | threats.rs:315 | none |
| `retreat_when_enemy_near` | plan.rs:520, 537 | none |
| `solar` (`only_when_stalling`, `never`, `freely`) | plan.rs:691 | plan.rs:471, 672 |
| `turrets` (`beside_each_outer_extractor`, `beside_each_extractor`, `none`) | plan.rs:732 | plan.rs:675, 717 |

The `lane` words gate code orders in A: `march` (groups.rs:571), `follow` (groups.rs:580 leash, 622 re-send), `rove`
(groups.rs:443-464; micro.rs:104). The player used `standing` 50 times in player-22 (key mentions: station 30, never 16,
raiders_lone 15, raiders_party 11, station_mode 9, engage_party 9, fall_back_to 7, no_chase 6, no_detachments 5, plan 4,
solar 2, job 1, turrets 1, hold_line 1) and `lane` 7 times.

## D. Information and options only (KEEP)

- picture.rs: the picture's words (economy, ours, enemy, places, actors), party naming and memory, `odds_words`,
  `group_odds`, the unseen shooter, reach words, production draws, lane and standing lines.
- plan.rs: the state enumeration's words (`build_words`, `leaves_words`, walks in seconds), `gate_questions`,
  `signature`, `consequence`, `question`, `pick`, `log_slots`. The enumeration itself is D; its cuts are in B.
- threats.rs: the threat states' words (drive-off seconds, catch or not, heading and next extractor), the D-gun state,
  `gate_questions`.
- engagement.rs: `armed_elements`, `positions`, `battlefield`, `battlefield_words`, `candidates`, `request`.
- schedule.rs: the events that feed the signature (hits, sightings, alarms).
- diet.rs: `paragraph`, `named_in_order`, `names`, `names_spot`, `brief` (text helpers).
- glossary.rs, replay.rs (offline replay tool), standing.rs (vocabulary, checking, `in_force`).
- groups.rs: `Body`, the route facts (`reached`, `met`, 487-522), the loss wake (331-354, H-HANDS-LOSS-WAKE), the stall
  warning (567-570, H-HANDS-STALL), the rove log (698-739).
- execute.rs: execution of a chosen state (site placement, `DEFENCE_FORWARD` 221-226, the commander's waypoint home).
- brain/: combat.rs (the odds table), routes.rs (walking, reachability, waypoints), shelling.rs (the estimate),
  scout.rs, bases.rs, briefing.rs, journal.rs, planner.rs (the player's plan context), wake.rs (the player's wakes),
  yards.rs (stuck and lane tracking, the yard wake), economy.rs (build sites), reclaim.rs `track_wrecks` (fields),
  allies.rs (allied units, the team board), roster.rs (kits).

## Counts

- A: 27 sites (A1-A27), 11 of them with a measured firing in player-22.
- B: 43 sites (B1-B43).
- C: 18 defaults (C1-C18) plus 6 modifiers.
- D: 13 pianist modules and 13 brain modules, KEEP.

## Notes

1. **A4 stops every shelling pick (likely a bug).** `is_mark_name` (groups.rs:429) treats any place that is not home,
   a spot, a passage or the last hold as a mark. `shelling`, `shelling_<group>` and `standoff from party_N`
   (execute.rs:499) are never in `shared.marks`. So a pick of `close_on_shooter` or `shell_party_N` is stopped and held
   in the next `keep_groups`. In player-22 this happened 21 times, all to group_F between 14:38 and 22:57, each within
   the second of the pick (compare the `plan` rows with the `recent` lines).
2. Sites whose classification is uncertain:
   - B2 and B39 (`FLAG`, `IDLE_BAR`) are thresholds on Jev's own nouls, so they are closer to Jev deciding than to
     pruning.
   - A2, A13, A14, A27 act on the player's lists (holding, skipping, ending steps). A list is a player order, but the
     timing is code's.
   - A8, A16, A20, A24 carry out a pick after the fact.
   - C1 reads the player's `produce` list, but it plays with no rule set.
   - C7 fires without any rule.
   - A6/C6 is a Leave that the base world plays only because the party changed its name or split.
3. Counting method (all over `run/matches/1790646080-player-22-commit/00/`):
   - Sources: `plan` and `pass` rows' `played[].source` in `jev-0.jsonl`: plan 372, rule 142, list 129, engagement 9.
   - Messages: unique `M:SS <text>` matches over every string in `jev-0.jsonl` and `strategist-0.jsonl`, deduplicated by
     clock and text, for these patterns: `no longer a marked place` 21; `was drawn \d+ from where it engaged` 7;
     `where it never goes: it holds` 2; `hunt of party_\d+ ended after \d+ s: (the leash|party_\d+ out of sight|the
     quarry is dead|no hunters)` 12/4/2/7 (the same totals as the `pass` rows' `hunts` arrays); `its hunt of party_\d+
     called off` 9 (7 plan, 2 rule); `leaves party_\d+ and holds` 5; `its attack ends, the party is gone` 5; `the
     engine refused a` 16; `its list step '[^']*' waits for it` 26; `gave up its (advance|walk)` 0; `adds nothing at an
     empty store` 0; `takes the engagement plan` 9.
   - Other rows: `pass` rows with `quiet` 320; `worlds_gate` rows with `worlds: null` 273 of 976; `plan` rows with
     confidence < 0.25: 180; no `shed` rows; `engagement` rows (9), where `taken` always equals the top option.
   - OVER_RULE: each asking `pass` paired with the next `worlds_gate`; slots whose base is a default not current and
     whose best other state rated 0.5 to 0.7: 99.
   - ENGAGED_PICK_BAR: deviations of the picked world, by a group, missing from `played`, in picks under 0.25: 12.
   - H-ARMY-MARCH: the sum of the `rules:` lines in `bot.log`: 124.
   - Nano guards: `guard` commands in `record-0.jsonl` to units that received nothing but guards: 4.
