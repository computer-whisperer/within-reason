# No live heuristics above the micro engine

**Status.** Stages 1-3 built 2026-09-28 night (f2cf462 and the stage-3 commit): the standing tool, every default, and the game-keyed gates are gone. Stage 4 (the user's rulings, late 2026-09-28): the hands' follow re-send, its 900 leash and the named-target re-attack are deleted with the `follow` lane word; H-ARMY-MARCH is deleted (the lane's march stands); the hunt-end release stays as the end of an order; the nano guard, the resurrection crews and the newcomer walk stay as mechanics; rove holds as-is, a rework expected; the brief stays large as in-context training data. The lane's `release` no longer re-sends an order the host gave this tick (the doubled stops). Stage 5 (2026-09-29, after player-23's 4:00): a hunt is a group of its own, split from its parent; the hunt-end rejoin is gone; the merge back is Jev's join state. Proposed 2026-09-28 evening, after player-22-commit. The user's ruling: "any higher level heuristics making
live game decisions should be removed completely"; the micro engine stays because it runs faster than a model call
returns. Nothing is deleted until the cut line below is ruled on.

## The principle

Three things may decide what a unit does in the live game:

1. **The player** (the LLM): its orders, lists, marks, lanes and standing rules, through the tools.
2. **Jev** (the rating model): the pick among the states the hands compose each second, and the engagement plan.
3. **The micro engine** (`crates/micro`, 10 Hz): footwork between orders (flee, fan, kite, form, march, follow, rove,
   hunts by id), because a model call cannot return in time for it.

The hands (`brain/pianist`) **compose and execute**: they turn the game into the picture and the states, put the
question to Jev, and carry out what was picked or ordered. They do not decide. A code path that changes what a unit
does, hides an option from Jev, or cancels a pick, without one of the three behind it that second, is a live
heuristic and goes.

Why (the user, 2026-09-28): heuristics do not generalize; a wrong move is usually Jev without the right options or
information; they compound until fixing one breaks five; and effects applied without Jev or the player in root
control are against the theme of every iteration since one-decider (the 09-26 ruling: a response is a decision,
not a default). The evening of 09-28 added nine such gates from model reviews of nine games, none run twice, none
run past the user; the scout-body ban took a Rover hunt of a Tick off the table at 2:17 of player-22 and the
commander left its assist; a forgotten-mark check from 09-21 stopped every group walking to the hands' own
`shelling` and `standoff` places, an order storm in every game since the posing set.

## The inventory

`2026-09-28-no-live-heuristics-inventory.md`: **27 sites that give or cancel an order on their own (A), 43 that prune
or hide a state before Jev sees it or refuse a pick (B), 18 defaults that play when Jev does not (C) plus 6 modifiers
that decide when a default fires.** What fired most in player-22: the quiet gate (320 s where the base world stood and
nothing was asked), the FLAG threshold (273 of 976 gates opened nothing), the march's leader stops (124), the lab
default (110 of the 142 `rule` plays), OVER_RULE 0.7 (99 gates where a state rated 0.5-0.7 stayed under a default),
the confidence bar (12 group moves not played), list steps held after a diversion (26), hunt ends (25, each declining
the party 30 s), the forgotten-mark stop (21, every one a pick to `shelling`/`standoff`), the station default (17).
The player's standing rules played 29 of the 142 `rule` plays (station 17, detachment 5, `job expand` 5).

## The cut line: what goes, what becomes information, what stays

**Goes, all of it: every C default and the `standing` tool.** An idle lab, an idle constructor, a station, a
raiders rule, a fall-back, a retreat: each is a state Jev sees and picks or a line the player writes in `instruct`
that the picture quotes beside the state. The tool's pruning keys (`never`, `ignore`, `no_detachments`) become
words in the state ("the instructions forbid spot_50") for Jev to decline. `station` becomes the ordinary walk-and-hold
order it already is; `plan: no` becomes a lane-style switch on the group. With the defaults gone, OVER_RULE goes too.

**Goes: every B gate that keys on the game** (odds strings, reach, never places, scout share, the confidence bar,
the plan hysteresis, GROUPS_PER_PARTY, HUNT_PARTY_MAX, `declined` timers). The fact each keyed on goes into the
state's words or the odds; the option it removed is offered. **Stays, as the cost of asking, not as a decision:** the
quiet gate (nothing changed since the last ask: do not ask) and FLAG/IDLE_BAR, which are thresholds on Jev's own
"should this actor change" answer, on the condition that the base world is `keep` for every actor and never a default.
`diet::shed` stays (request size).

**A sites: three kinds.**
- *Execution of a pick or a list step, keep:* the plan's phases (A8), the queued-task promotion (A24), the timed
  assist's own timer (A14, the player's N), arrival becoming a hold (A23), the head-of-list stop.
- *Tick-rate work that belongs in the micro engine, move down and fix there:* following a moving target (A16, A20;
  replaces the 900 leash A7), the march's leader waiting (A1, already a lane word), hunt ends (A3, A26: keep the
  release, drop the 30 s decline), the named-target re-attack (A15).
- *Decisions, remove:* the forgotten-mark stop (A4), the 90 s give-up (A12: keep the stall words), the assist skip on
  the store (A13), the list holds after a diversion or a retreat (A2, A27: the list resumes; the pick that diverted
  it is the decision), the base-world Leave (A6), the party-gone stop (A9: the target's absence is in the picture),
  the never-chase stop (A11), the newcomer's automatic walk (A18: a produced soldier stands at the plant until its
  group is told; or the `produce ... group` order carries the walk, the user's call), the nano guard (A10), the
  resurrection crews (A25), the task drops on clocks (A21, A22: the stuck and the wait are words; the player or Jev
  drops the task), the engagement's lost-party hold (A17: a fact for Jev).

## Questions for the user

1. The `standing` tool: delete it (recommended), or keep `never` as a tool-side place filter?
2. A produced soldier: walks to its group on its own (today), or stands at the plant until the player's `produce
   ... group` order is read as "and walk to it"?
3. The engagement plan: keep as a Jev decision with its phases executed by code (recommended), or fold it back
   into the pick?
4. The pursuit let-go inside `crates/micro` (600 elmos, 30 s, from one game): keep as a micro rule, or make the
   let-go a lane word the player sets?

## Order of work

Write the target (this doc) → delete the category A/B/C sites in one commit per module, tests updated → run the
E3/F3 harness arms and the 37 bot tests → one arena game on the target (player-23) → review against player-22 on
the same numbers (orders a minute, plays by source, the 2:17 shape).
