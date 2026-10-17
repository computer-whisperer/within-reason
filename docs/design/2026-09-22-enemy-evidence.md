# Enemy evidence, not an enemy base: the player finds the opponent

Written 2026-09-22 after the objective games, from the user's rulings: "finding the enemies bases is one of the
critical judgement dimensions of an RTS like this, and really under opus's responsibility. Opus should naturally send
scouts at possible locations, and otherwise sweep the army coherently when there is uncertainty"; "the enemy base is
not even a constant thing within their start box. Messy games often result in pop-up bases in obscure corners of the
map... we should be careful with lingering heuristics."

## What stood, and what it did

- The bot guessed one enemy base per seat (H-MAP-ENEMY-BASE, -START): the start box centre or the mirror of our
  start, snapped to the nearest walkable metal spot. Under the player the guess never moved (refinement from
  sightings suspended 2026-09-22 on the belief that the mirror was right each time).
- The picture handed the guess to the player every turn as `enemy.base: "not found; presumed at G4"`, kept an
  `enemy_base` place the packet could send groups to, and the overview's first line called it "the game's guess,
  often wrong by 500 or more". The `fight_to` option's words judged the odds "against the enemy base as we know it"
  at the guess. The report added "THE GUESS IS WRONG" when our soldiers stood on it.
- The scorecard's `look_min` measured how soon a unit of ours stood within 1,500 of the guess; `base_min` the minute
  the picture said "found" (a factory in sight).
- The truth files of the 86 surviving Comet games: with the standard start boxes (every game since rush-7,
  2026-09-20) BARb's commander is placed by the game's own start-point guesser at the metal-spot cluster that scores
  best inside its box, the north end of the east strip, 2,000 elmos from the mirror point. The guess was wrong by
  2,000 in all 44 of them and the base column read "never found" in 11 of 14 scorecard rows; the objective bombers
  waited eight minutes at home for a sighting that scouts sent to the guess could not make.

## Decisions

1. **Under the player the harness holds no model of the enemy base.** No presumed point, no `enemy_base` place, no
   "found" state, no "the guess is wrong" line, no odds "against the base". Where the opponent is, whether it has
   moved, and where to look next are the player's judgment. The bot's own guess (`bases.rs`) stays for the
   heuristic modes and for the routing field it seeds; nothing of it reaches the player's picture, report or menus.
2. **The picture carries evidence.** The `enemy` section: what is in sight; its factories seen (each with its cell,
   coordinates, when last seen, standing or destroyed); its commander's last sighting; its buildings remembered by
   cell; its team's lobby start box as cells (a fact about 0:00, not about now); the metal spots never in our sight,
   nearest first, with the ones inside its start box marked; the spots last in sight more than five minutes ago,
   with the age. A spot's own place words say "never in our sight" rather than "free" when nobody has looked.
3. **The report's opening says the same.** The "to win" line names the factories and commander as last seen and,
   when nothing of the kind has been seen, the start box and how many of its spots have never been in our sight.
   The `map` tool gives every team's start box instead of a presumed start.
4. **Menus judge places, not the base.** `fight_to` carries no odds sentence; each place entry in the picture says
   what enemy is known within 500 of it, and the player's packet decides.
5. **The role texts say whose job it is.** Finding the opponent, and finding it again after it rebuilds elsewhere,
   is the player's: scouts to named spots, chosen from the never-looked list; the army swept coherently when the
   evidence is thin. The default hands text (no player) loses its `enemy_base` lines and no longer finds bases on
   its own; that mode is not a goal.
6. **The scorecard measures against the truth.** `look_min` is the minute a unit of ours first stood within 1,500 of
   the enemy commander's true start (the truth file's first frame); `fac_min` replaces `base_min`: the minute the
   picture first listed an enemy factory seen.
7. **Internal geometry stops leaning on the guess** in the player's code paths: "our half" and far adoption use the
   map's far side (the mirror of home), not a base.

## Steps

1. Delete: the `enemy_base` place and its words, `enemy.base`, `presumed_enemy_start` (briefing, map tool, report),
   `guess_disproved`, `enemy_base_found`/`enemy_bases` in the summary, the `fight_to` base odds, the role texts'
   `enemy_base` lines.
2. Build the evidence section, the report line, the map tool's boxes, the place entries' known enemy, the spot words.
3. Role texts, rules, default; `docs/heuristics.md` (H-HANDS-ENEMY-EVIDENCE; H-MAP-ENEMY-CLUSTER's suspension
   reworded), `docs/knowledge/maps.md` (K-maps-barb-starts-where-the-guesser-puts-it, with the 86-game table) and
   `docs/knowledge/jev.md` (K-hands-presumed-point-reads-as-a-lead).
4. `run/floor.py`: `look_min` from the truth, `fac_min`.
5. Rerun the bombers and Bulls objective games (Comet easy, Opus with Jev), judged by `fac_min`, `look_min` and
   whether the flight receives a target.

## Status (2026-09-22 evening)

Built (8e7c8e0, c1f41f4, e420a6c). evidence-1-bombers: the opponent found by the player's own scouting (a scout
plane at 6:06, the bombers "flying to spot_12 to find them"; its plant and commander in the picture at 13:12, where
roster-1 never found them); lost on the air hold cancelling every strike (K-hands-air-hold-cancels-the-strike, the
domains design's next pass). evidence-2-bulldogs was invalid: the builders' free-spot rule keyed on the words "free"
and the new "never in our sight" words failed it (fixed e420a6c). evidence-3-bulldogs, the valid rerun: WON with
the objective met, the opponent found by sweeping "the far corners" and "north through their box" from the
never-looked list, its base marked from its buildings and its commander found at G2 before the Bulls went. Across the
two valid games the player used the evidence as intended and asked for nothing the picture lacked. What the games
left: energy (empty 16 % and 42 %) and the air hold (the domains design's next pass).

## Status (2026-10-17): the rule extended to his economy and army

The user, reading docs/studies/2026-10-17-wrong-post-mortem.md: "I was not aware the harness was trying to say things
like 'we are ahead' -- that's one kind of thing the harness has almost no authority to say. Opus should be
responsible for predicting enemy game state in both economy and army from what it has seen." Removed the same day:
- the `his spending seen` line (`report.rs`, from the posing changes of 2026-09-27: metal of his seen dying plus
  soldiers and buildings seen, "at least N a second, about K extractors' worth"; it read 7 extractors' worth against
  a true 28 at 19:58 of levers-player-1) and `Score::enemy_buildings_metal`, which fed nothing else;
- "the opponent is known to hold N" on the score line: each of his extractors is now a sighting with the age of the
  look ("#40 E4 (0:14 ago)"), nearest our start first, with no count (`Score::enemy_spots` carries the seconds);
- the clause "its army is at least that and may be much more" after the soldiers seen in the last three minutes;
- the map sheet's "where his start may be" rows, which priced the bot's own base guess (`bases.rs`, refined by
  sightings) against decision 1 above: the sheet now prices his start as the lobby gave it (the script's position
  when the lobby fixed the starts, else the centre of his box while the boxes are honoured, else no row), and the
  not-honoured note no longer says "his start is guessed as the mirror of ours";
- the passages note's "whoever holds one decides who crosses".
Left where they were, by the user's word ("the third group can stay for now"): the hands' fight words on units in
sight ("we outweigh it heavily", "it outclasses us"), the shelling place estimate labelled as one, the direction
words of the recently-seen parties, and the menu's "a walk back toward our side" that reads the bot's guess for
the direction. The prompts' sentences of the same kind (the commander's "compare the sides in the report's own
numbers ... extractors against what he is known to hold", the player prompt's "the opponent keeps its army at home
as one block") are removed with the brief's amendments, after finish-player-1 ends (prompts are read at run time).
Test `sightings_tests` in `report.rs`; unmeasured in a game.
