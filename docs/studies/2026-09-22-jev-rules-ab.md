# The hands' standing rules, A/B-tested on recorded moments (2026-09-22 night)

The user's ask: take the recent games, find the questionable Jev moments, and test variations of the hands'
instructions to see what helps. The instrument is `run/jev_ab.py`: detectors pick the questionable answers out of
`jev-<ai>.jsonl`, the exact request each was answered from is rebuilt (the picture's state with the packet and the
rules in force, this actor's questions) and put to Jev again under each variant of the standing rules
(`crates/bot/src/brain/pianist/rules.md`, sent as `state.rules` with every ask). A control set of ordinary decisions
(chosen at 0.6 or more) measures what a variant disturbs. Fifteen games of 2026-09-22 (evidence-1/2/3, opus5-1/3,
opus55-1/3, ab5/ab55, ab55-medium-effort, air-1, sea-1, plan-1/2, hands-1); data in
`docs/studies/data/jev-ab-2026-09-22-rules.jsonl` and `-rules-2.jsonl`.

**The replay is faithful.** Under the unchanged rules the rebuilt requests give the recorded probability of the
recorded choice within 0.07 on average (one moment 0.5 off: only this actor's questions are sent, where the game sent
every actor's), the pick is the same on 24 of 24 moments over three repeats, and the spread over repeats is 0.04.
One call per variant is enough. 3,124 calls, about two minutes each run.

## Detectors and what they found

| detector | found | replayed | judged right |
|---|---|---|---|
| hold_beside_attack: a group answered hold/continue with a party in sight within 800, and one of our buildings within 800 of it died in the next 30 s | 104 | 40 | engage, attack_unit, send_against |
| raider_ignored: a party at one of our extractors within 1,200 of the group, in the group's own entry, answered hold/continue | 479 | 40 | send_against, engage, attack_unit |
| shelling_over_named: fight_to/move_to with `where` = shelling while the packet forbids it, or names places for this actor and never mentions shelling | 73 | 40 | any place but shelling |
| never_split: send_against/split under "never splits / no detachments" | 19 | 19 | anything else |
| whom_not_commander: the group attacked, their commander's party (one we outweigh) was on the whom list, another party was chosen | 2 | 2 | the commander's party |

## Variants (each one sentence added to or changed in the rules)

- `never`: "A 'never' in the instructions removes the choice: when they say an actor never does something, or never
  goes to a place, that option or place is not chosen whatever its own words on the menu say." `never2`: "The
  instructions' never-clauses are law: an option or a place they forbid for an actor is wrong even when its own words
  fit the moment, and the next best choice is taken instead."
- `home`: "An enemy party in sight at one of our buildings, or within 600 of a group that outweighs it, is fought now
  (engage or attack_unit), whatever the instructions call the raiders: a group holding beside a base under attack
  loses the buildings." `home2`: "An enemy party in sight within 600 of a group that outweighs it is fought now,
  whatever the instructions call it."
- `commander`: "Their commander in sight is the target above every other when we outweigh its party: attack_unit on
  it wins the game, and a raid party far from the group is another group's business."
- `shelling`: the old "`shelling` is where a weapon hitting us from out of sight likeliest stands, and advancing onto
  it kills it" replaced by "`shelling` is only where a weapon hitting us from out of sight likeliest stands; a group
  advances onto it only when the instructions say so, and otherwise goes to the place the instructions name."
  `shelling2`: the old sentence with only ", and advancing onto it kills it" cut.
- `all`: shelling + never + home + commander.
- `words`: not a rule but the menu words H-HANDS-PARTY-NAMES gave every party in `whom` (distance from the group,
  THEIR COMMANDER in plain words), put on the older games' recorded menus.

## Results (second run; the first agrees within a few points)

right%: how often Jev's pick is one judged right; for control, how often it is the recorded pick.

| detector | n | base | never | never2 | home | home2 | commander | shelling | shelling2 | all | words |
|---|---|---|---|---|---|---|---|---|---|---|---|
| control | 40 | 98 | 98 | 98 | 95 | 95 | 98 | 98 | 98 | 98 | 98 |
| hold_beside_attack | 40 | 2 | 0 | 2 | 28 | 32 | 10 | 2 | 2 | 22 | 0 |
| raider_ignored | 40 | 2 | 2 | 0 | 22 | 18 | 5 | 0 | 2 | 20 | 2 |
| shelling_over_named | 40 | 38 | 45 | 48 | 38 | 32 | 40 | 80 | 55 | 82 | 42 |
| never_split | 19 | 26 | 21 | 21 | 37 | 47 | 26 | 16 | 21 | 47 | 26 |
| whom_not_commander | 2 | 0 | 50 | 0 | 0 | 0 | 0 | 0 | 0 | 50 | 50 |

Mean probability on the options judged right moved the same way: hold_beside_attack 0.24 to 0.41 under home,
shelling_over_named 0.47 to 0.71 under shelling; never and commander left every number where it was.

## Reading

1. **The shelling sentence was the harness telling Jev to go there.** "Advancing onto it kills it" reads as a
   standing order; replaced by "only where the weapon likeliest stands; go there only when told", the named place
   wins 80 % of the time instead of 38 %, and cutting the clause alone gives 55 %. No control changed.
2. **A rule to fight a party in sight when we outweigh it works, within limits.** Holds beside a base under attack
   drop from 98 % to about 70 %; where the odds words say we outweigh every party in sight, right rises from 1/25 to
   8-9/25 (hold_beside_attack) and 1/27 to 6-7/27 (raider_ignored); where some party outweighs us it hardly moves,
   which is as it should be. The rule also turns half the forbidden detachments into `engage` (never_split 26 to
   47 %), a whole-group answer to the same raider. It disturbed two of forty controls: a commander at 23:51 went from
   continue to retreat_home, and a group at 4:10 from hold to engage with a party in sight, both defensible.
3. **"Never" cannot be taught by a rule either.** Two phrasings of "a never-clause removes the choice" moved
   never_split from 26 % to 21 %, and the shelling prohibitions from 38 to 45-48 %. K-jev-never-is-not-heard holds
   against rules as it held against packets: the menu must drop the option (the study of the Comet series, §8.1).
4. **A commander-first rule does nothing; the commander's words on the menu do something.** On plan-1's 22:01 moment
   the recorded menu gave the commander's party 0.38 and the raiders 0.54; the rule left it at 0.40; the words
   H-HANDS-PARTY-NAMES now uses (THEIR COMMANDER, the unit whose death wins the game, right here, 431 away) put it at
   0.56 and the pick on the commander. Two moments only: the words are the lever, and it needs games to measure.
5. **What remains after the two rules.** Seven of ten holds beside a base under attack still stand when we outweigh
   the party. The party words say where it is and how far, not what it is doing; "shooting our Advanced Vehicle
   Plant now" on the party line is the next words variant to try, and it can be tested here before it is built.

## Second round: the party line saying what it is shooting (the user: "try that 'killing our Advanced Vehicle Plant' variation")

Evidence of the moment, not hindsight: the record's `dmg` sample at the ask (damage per unit over the last second)
names our units within 450 of a party that took damage, buildings first; the bot has the same from the attacker ids
of its damage events. 29 of the 40 hold_beside_attack moments and 27 of the 40 raider_ignored ones had such a party.
Three phrasings, each alone and on top of the fight-in-sight rule now in force (`home2`):

- `shooting`: the party line (engage and whom) ends ", shooting our Advanced Vehicle Plant now" before the odds.
- `killing`: ", killing our Vehicle Plant, our Advanced Vehicle Plant (3190 metal) now".
- `hold_cost`: `killing` on the party line, and the hold option's own words end "Holding now leaves what party_85 is
  killing to die."

right% on the moments with shooting evidence (`docs/studies/data/jev-ab-2026-09-22-shooting-2.jsonl`):

| detector | n | base | home2 | home2+shooting | killing | home2+killing | hold_cost | home2+hold_cost |
|---|---|---|---|---|---|---|---|---|
| control (with evidence) | 7 | 86 | 86 | 86 | 100 | 86 | 86 | 86 |
| hold_beside_attack | 29 | 7 | 34 | 34 | 17 | 38 | 31 | 52 |
| raider_ignored | 27 | 7 | 19 | 37 | 7 | 26 | 30 | 52 |
| never_split | 13 | 8 | 46 | 54 | 8 | 62 | 15 | 54 |

Over all forty controls no variant disturbed more than the two the rule alone disturbs. Reading: the words on the
party line alone do little (7 to 12-17 %); the same fact on the hold option's own words does more (hold_cost alone
25 %), and on top of the rule it is the best of anything tried: holds beside a base under attack 34 to 52 % right,
raiders at our extractors ignored 19 to 52 %, mean probability on fighting 0.42 to 0.53. Jev reads the option it is
about to pick more than the party it is not looking at. Built the same night as H-HANDS-PARTY-KILLING: the bot keeps the
attacker ids of `UnitDamaged` for three seconds, and the party lines and the hold option carry the words.

## Done from this

The two winning sentences (`shelling`, `home2`) are in `rules.md` from this commit (242 words, from 206);
H-HANDS-RULES registers them. The instrument stays: any change to the rules or the menu words can be replayed over
the recorded moments before a game is spent on it, at about a cent a hundred calls.
