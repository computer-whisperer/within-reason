# No live heuristics above the micro engine

**Status.** Proposed 2026-09-28 evening, after player-22-commit. The user's ruling: "any higher level heuristics making
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

(To be filled from `scratchpad/inventory-live-decisions.md` once verified: every site by category A (acts without a
pick), B (prunes or cancels), C (defaults), with file:line, trigger, heuristic id, and how often it fired in
player-22.)

## The cut line: what goes, what becomes information, what stays

(Per site: **remove**, or **convert** into words/options Jev sees or a tool the player has, or **keep** as execution
mechanics. The conversion rule: the fact the gate keyed on goes into the state's words or the odds; the option the gate
removed is offered; Jev declines it or not.)

## Questions for the user

1. Standing rules the player writes (`standing`: `raiders_party`, `no_chase`, `never`, `fall_back_to`,
   `retreat_when_enemy_near`) are applied by code without a pick (source `rule`, 142 plays in player-22). Is a
   player-authored rule "the player in root control", or must its effect also go through a pick?
2. The lists (`queue`, `produce`): code advances a list step by step (source `list`, 129 plays). Same question; and
   the timed `assist N` skipped on a metal store under 20 is a code decision inside a list.
3. Hunts by id, the follow leash and the forgotten-mark stop live in the hands but act at tick rate: do they belong
   to the micro engine (keep, fix the bug) or to the hands (remove, make them Jev's)?
4. Engagement plan hysteresis and the confidence bar: remove outright, or replace with the information (the last
   answer, the confidence) in the question so Jev holds its own line?

## Order of work

Write the target (this doc) → delete the category A/B/C sites in one commit per module, tests updated → run the
E3/F3 harness arms and the 37 bot tests → one arena game on the target (player-23) → review against player-22 on
the same numbers (orders a minute, plays by source, the 2:17 shape).
