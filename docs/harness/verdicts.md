# Verdicts: `verdict.json` and its tags

One `verdict.json` per reviewed match, written into the match directory by the reviewer that follows
`.claude/skills/bar-review/SKILL.md` (the review skill: a subagent, or you). `run/tally_verdicts.py <batch dir>` adds
them up by `primary_cause`, so that we fix the commonest cause rather than the latest one seen. The match must have been
played with `WITHIN_REASON_OBSERVE=1` so the opponent's side is ground truth (`arena.md`, "Post-game analysis").

The procedure (2026-09-23 to 2026-09-27) was: `run/analyze_match.py` first, find the last minute the game was still
level and the first it clearly was not, read the engagements in that window, decide on the earliest sufficient cause.
That is now step 1 of the review skill, which adds the pro baseline (`run/pro_baseline.py`), the raid ledger, the
player's turns and the hands' plays, and writes `review.md` beside this file. The two are one pass; this page keeps the
file's shape and the tags.

## `verdict.json`
```json
{
  "match": "run/matches/<batch>/<NN>",
  "last_level_minute": 10,
  "decided_by_minute": 13,
  "decisive_moment": {"time": "11:54", "grid": "G6", "what": "one sentence"},
  "primary_cause": "<one tag>",
  "contributing": ["<tag>", "..."],
  "evidence": ["short statements with numbers from the report or scenes"],
  "preventing_rule": "the rule or lever that would have prevented it, concretely",
  "confidence": "high | medium | low",
  "notes": "anything that does not fit the tags, or looks like a bug in the bot or in the analysis tool",
  "review": {"skill": "bar-review <date>", "findings": [], "clean": [], "stronger_player": [], "player": {}, "hands": {}, "harness_faults": []}
}
```
The `review` object is the skill's (its section 4 gives the fields): findings ranked by cost with the checklist item
each falls under, the items checked and found clean, the stronger player's orders, the player's and the hands'
numbers, and the harness faults. Files written before 2026-09-27 have no `review` object; the tally reads both.

Reading the curves for `last_level_minute`: against BARb medium the two measures part early (extractors in minute 3-5,
army in minute 6-8): take "level" on extractors when raids are the story and on army value when fights are, and say
in the evidence where both stood. `run/analyze_match.py`'s "on the spot before" is a five-second snapshot: an attacker
arriving during a long engagement shows as 0 there, so read the scenes. The record ends when the game does, so our
commander's death is rarely in it; the report gives its state in the last sample instead. Prefer the earliest
sufficient cause: do not blame the final base fight for a game that was lost ten minutes earlier.

## Tags (use `other` with a note rather than stretching one)
- `lead_not_converted`: we held twice the opponent's army value (and the extractors) for six minutes or more without
  its factories under attack; the pros close 3.4 minutes after that moment (`.claude/skills/bar-review/baselines.md`).
- `blind_wave_into_defence`: an attack wave walked into a force or turret line worth clearly more than itself.
- `expansion_raided_undefended`: extractors and constructors lost to raiders with no defenders or turrets in place.
- `defenders_out_of_position`: defenders existed but were elsewhere, arrived late or arrived strung out.
- `lost_fight_at_equal_or_better_value`: we had the value on the spot and still lost: unit mix or target handling.
- `out_massed_at_level_economy`: economies level, but their army grew faster with no big engagement to explain it.
- `opening_starved`: the plant made half the pool's units by 4:00 (the store at 0, the commander away from it), and the
  game never caught up.
- `economy_never_grew`: behind on extractors and builders from the opening, without raids to explain it.
- `idle_resources`: metal banked or energy stalled for minutes while behind.
- `commander_exposed`: the commander died while the game was otherwise still contestable.
- `tech_gap`: tier-2 or heavier units we had no answer to.
- `won`, `stopped`: a win, or a game ended by the batch's clock or by hand; the review's findings stand without a cause.
- `other`
