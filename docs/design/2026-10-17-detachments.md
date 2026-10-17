# Detachments: the prompt matches the hands, the chase can be recalled (2026-10-17)

Decisions, not findings; the findings are `docs/studies/2026-10-17-detachments.md` and the claim
K-hands-the-chase-does-not-withdraw-and-the-player-forbids-every-detachment.

## The user's rulings (2026-10-17)

1. "We usually need the freedom for jev to detach units and repel an enemy party, and if jev is not using that power
   coherently then that is something to fix." The player's "it sends no detachments" is wrong; the fix is in the
   hands and the prompt, not in more prompt doctrine.
2. "Sending a group after a group, then forking that group between two threats sounds completely reasonable." The
   forks stand. The one mistake: "the hands not withdrawing a chase when it encounters danger."
3. The change set: "ensuring the player's prompt matches what jev is prompted for", and "ensuring that a hunting
   party is able to be recalled by jev when it encounters something unexpected". A detachment made of the units the
   player names is "a different problem that I don't think we can solve directly: the naive solution will be far too
   fragile"; an exact-detachment tool for the player comes later.
4. The four additions proposed and agreed: the send's words say who is already on the party; the send's words say
   what the quarry runs toward; the name stays with the larger part on a split; an instrument for the ledger.
   "A fifth of the army loses to what all of it would walk over" goes.

## What is built

- **One source for the hands' rules.** The player's role text holds `<!--hands-rules-->`, replaced at session start
  with `crates/bot/src/brain/pianist/rules.md` whole, every marked part kept (`strategist/mod.rs` `with_hands_rules`,
  `brain::pianist::hands_rules_whole`, test `the_player_prompt_carries_the_hands_rules_whole`). An edit to the rules
  reaches both on their next read. The defence paragraph around it says the mechanics the rules do not: the ladder
  of one, two, four or eight of the fastest against a party the group outweighs four times, no whole-group attack
  until every size is forbidden; a detachment's life (a group of its own, the hunt's three endings, the hold, the
  way back as a pick); the name rule; the packet's levers. The brief's "Defence is yours to write" bullet and the old
  brief's are reworded.
- **The recall is an ask, not a rule.** A hunting or attacking group's entry carries a `chase` line (H-HANDS-CHASE):
  his armed buildings whose reach the line from the front to the quarry enters within 600 of the front and where
  that reach begins, the parties within 800 of its soldiers that are not the quarry with the odds by metal, the
  chase's length and the leash run, and "any other pick ends the chase". Each new thing met is an event once
  (`schedule.rs`, "group_H chase meets turret 123"), which is news under the `news` layer, so the group is asked
  that second. The join to the group it split from says so on the move. Nothing in code ends a hunt or an attack
  on danger: the standing rule that an outweighed group steps back is Jev's to apply, which keeps
  docs/code-decisions.md unchanged. Air-only turrets are left out of a ground chase's words and ground-only ones
  out of an air group's (`can_hit`).
- **The send's words.** "already on it: group_F (8 soldiers) has hunted it for 5 s" on the attack and every
  detachment; "toward his Guard (corllt) at spot_27 (reach 430), whose reach begins 600 ahead of it" on every move
  aimed at a moving party (`CHASE_TURRET_AHEAD` 1500 on its line).
- **The name with the larger part.** A send of at least half the group leaves the hunters under the group's name
  and makes the rest the new group with the old task, the joiners on their way going on to the rest
  (`groups.rs` `start_hunt`).
- **`run/hunts.py`** for the ledger row of every game from here: sends by size and by share of the group, repeats
  within a minute, sends by a detachment, hunts by ending and length, soldiers lost while hunting and while attacking
  a party, chase asks and whether the group withdrew or kept on.

## Open

- The ladder by speed, so "two" can be two scout cars: deferred with the composition question (ruling 3).
- The whole-group attack's follow has no leash; the chase line covers it, a leash does not (unmeasured whether the
  ask is enough).
- Measured once in chase-smoke (five minutes on easy: the new words appear, no chase met); the first real game is
  chase-player-1.
