# Jev as a decompression agent: the player's packet into standing orders (2026-09-25)

The user's idea (2026-09-25): Opus writes prose cheaply and Jev reads it literally, so Jev could turn each packet
into standing orders from a fixed vocabulary once per player turn, and the hands would run those in code instead of
re-reading the packet forty times a minute ($0.59 of Jev a game, family-1 and bank-1). `run/decompress.py` is the
offline test: every `instruct` packet of four recorded games (family-1, bank-1, 2v1b-hard, 2v1b-hard_aggressive; 293
packets) put to Jev with one extraction question per (actor, rule) over the vocabulary in the script's docstring (the
group rules station, raiders, detachment_size, no_chase, no_detachments, fall_back, move, never_<place>; the builder
rules job, attack_raiders, no_chase, solar, turrets, rebuild_lost, retreat_when_enemy_near, never_<place>). Data
`docs/studies/data/decompress-2026-09-25.jsonl`, the readable dump beside it (`.txt`).

## Numbers

- **Cost.** 293 packets, 23,032 questions, 2.03M input tokens, $0.085: three hundredths of a cent a packet, against
  about $0.0005 a Jev ask today. A game of 90 player turns decompressed costs 2.6 cents.
- **Consistency.** Where an actor's paragraphs were identical in two consecutive packets, the extracted rule was the
  same 2,076 of 2,199 times (94%).
- **Hedging** (p_top under 0.6): move 28%, raiders 25%, no_chase 23%, job 19%, no_detachments 17%, turrets 16%;
  station 6%, detachment_size 2%, solar 5%, rebuild_lost 3%, never_<place> 8%.
- **Coverage at the detectors' moments** (the rule extracted from the packet in force for that actor):
  raider_ignored 649 moments: a raider order stood in 447 (69%: whole_group 419, detachment 28), not_said 86, the
  actor unnamed in the packet 88, forbidden 28. hold_beside_attack 80: an order stood in 62. never_split 74: a
  no_detachments order stood in 46 (62%). So at two thirds of the moments where the hands held while a raider stood
  at our extractor, the packet in force had already said what to do, and a standing order would have done it.

## Accuracy, read by hand

A regex truth over the packets disagreed with Jev often (station 78%, no_chase 44%, raiders=detachment 33%,
never_<place> 19%), and reading the disputes Jev was right in most of them and the regex wrong: "no soldier stands
at spot_50" is not a station; "sends one scout" is not a detachment against raiders; group_E's "never sends
detachments" is not group_N's rule; "it does not hold at home" is a negation. Two real faults remain:

1. **The question's wording.** "Does it tell the commander never to chase scout cars, Ticks or raiders?" came back
   no at 0.8 on every family-1 packet whose commander paragraph says "it never chases scout cars or Ticks", because
   the same paragraph tells it to attack raider parties: the disjunction was read literally and answered for its
   false part (battery D's literal reading). Reworded "Does it say the commander never chases scout cars or Ticks?"
   the twelve packets gave 0.97-0.99 on six and 0.33-0.44 on six with the same words present; "Does the paragraph
   contain the words 'never chases'?" gave 0.99 on all twelve. Extraction is reliable when the question and the
   packet share an idiom, and hedges when they do not.
2. **The vocabulary's gaps.** "It never goes east of spot_38" is a boundary, not a place ban, and was rightly not
   read as never_spot_38; a group told to "engage any raider party at those extractors, and send two soldiers after
   a lone raider" has both a whole-group and a detachment rule keyed on the party's size, and the vocabulary offered
   one; "never past spot_39 without soldiers near" is a conditional. Each is a rule to add (boundary with a
   direction; raiders split into lone and party; a condition on friendly soldiers near), not a failure of the
   reading.

## Reading

The decompression is cheap, stable and literal. The design that follows from it: the vocabulary's canonical phrases
are published to the player in its role text, so a packet written in them is parsed at 0.99 and a packet written
near them is parsed by Jev's tolerance (the `quote` form for the canonical phrase, the semantic form as the fallback,
both in the same call). The player keeps prose for what the vocabulary cannot say. What this does not measure: the
effect in play, which needs the rule executor in the bot (`docs/design/2026-09-25-menus-from-scratch.md` §9d) and a
game read against the detectors and the Jev bill.
