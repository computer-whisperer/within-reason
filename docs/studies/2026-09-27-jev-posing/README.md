# How the game is posed to Jev and the player: the mining of the games with people, 2026-09-27

The user, after the last human match of the day: "Let's begin sending opus subagents for mining the replays for
major issues. I expect most of the gap is how the game is posed to jev, missing scenarios and actions, unfortunate
interactions, etc. Don't start major changes yet, since any changes we make will invalidate the applicability of the
information we get. Just stockpile reports from players and clear observed wrong moves from jev. The answers almost
certainly won't be hard limits or behavior enforcement, but situations where the players and jev calls were never
presented with the right information needed to make the call."

Each game's report is `run/matches/<game>/00/posing-review.md` (written by an Opus reviewer from the record, the
jev logs and the player's turns, read-only; the reports are data, verified before use). The games mined:

| game | opponent | result | report |
|---|---|---|---|
| 1790471259-bluegecko-3v1-comet-catcher-3 | thebluegecko | lost 26:00 | posing-review.md |
| 1790473186-bluegecko-3v1-comet-catcher-4 | Irishstud14 | lost 20:30 | posing-review.md |
| 1790475913-bluegecko-3v1-comet-catcher-6 | Infern8 | lost 17:00 | posing-review.md |
| 1790477174-bluegecko-3v1-comet-catcher-7 | u6bkep | won 14:11 | posing-review.md |
| 1790479031-bluegecko-3v1-comet-catcher-8 (the ninth game) | thebluegecko | lost 20:16 | posing-review.md |
| 1790481450-bluegecko-3v1-cape-violet | thebluegecko | lost 18:18 | posing-review.md |

The players' own words from the games' chat: `docs/knowledge/_inbox/players-chat-2026-09-27.md`. The user's
replay-review packet on onepass-player-8: `docs/knowledge/_inbox/player-notes.md`.

## The consolidated findings

(Filled in from the reports once they are verified: one entry per gap class, MISSING INFORMATION, MISSING ACTION,
MISLEADING WORDS, UNFORTUNATE INTERACTION, each with the games and clocks it was seen at. Nothing here is a change
made; the code, the rules and the brief stand as they were at commit 6151a89 until the mining is read.)

## Reports as they land, with what was verified

### Game 7 (u6bkep, won 14:11) — verified: the teal commander died at 6:12 to a Blitz (record-2), ten units died to an unseen source in the five seconds round 13:59 (record-0), the pick at 5:50 was "walk to spot_42" (jev-2)
- MISLEADING WORDS: the odds weigh metal alone. The commander (38 speed) was stepped away from Blitzes (101) at 5:50 and
  fell from 100 % to 29 % on the walk, then at 6:10 at 13 % was offered "attacks party_20_t3 ... alone, we outweigh it"
  as the default and the pick took it; it died at 6:12 and its death blast took 755 of teal's own base. The commander's
  line said "D-guns anything at 262" while no state fires the D-gun. At 7:26-8:11 sixteen Incisors died at the enemy
  commander under "789 away: we outweigh it heavily; it is hitting this group now", eleven to its D-gun; the hunt report
  said "the quarry is dead" while it lived. At 13:59 thirteen Brutes died to the enemy commander's death blast (350
  radius, 5,000 damage) in a fight the picture called "an even fight": nothing says a dying commander clears 350 around it.
- MISSING INFORMATION: a group's `near: None` while its Incisors ran 700+ ahead of its Brutes (the line measures from the
  centroid) at C5 11:04-11:53; "outruns this group at 101 against 72" from one Blitz in a party of Stouts and a Janus
  that outrange Incisors; the advanced plant's progress never shown (45 % at 13:00, the player planning for "when it
  finishes"); the score and eco extractor counts disagreeing all game; a wake saying "lost 2 of its 24" when 8 had died.
- MISSING ACTION: against the Blitz raids of 9:33-10:47 (8 extractors, 3 solars) every offered answer was a chase the
  words said could not catch; no "stand at the next extractor on its path", no turret state. The threat slot on teal's
  base 5:15-5:42 held "nobody moves" alone with teal's group 2,400 away.
- UNFORTUNATE INTERACTION: builders on the player's construction-turret lists were never offered a solar through 120 s
  of STALLING (8:00-10:00; the draining-store default of 9b4ad85 covers free builders only); the fall-back rule sent
  the Incisor group to spot_11, 3,845 away, and five more died on the walk; the player's 11:18 "advance together"
  landed after the 11:21 pick had sent t2 back.
- Tools: hands_window, analyze_match, raid_ledger and floor read seat 0 only (the reviewer symlinked the other seats);
  raid_ledger said "answer none" for raids jev-0 shows answered five times.

### Game 3 (thebluegecko, lost 26:00) — verified: the three Blink scouts never left the G1-G2 corner from 8:21 to 25:41 (record-0 positions); the refused `engage_party party_46_t2` orders have a traced cause: the standing tool checks a party name against the merged hands' `enemy.in_sight`, which carries the lead seat's parties only (mcp.rs, the standing branch), so a party seen by another seat is "not a party in the picture" while that seat's picture carries it every second
- MISSING INFORMATION: a Mauser (range 820) shelling group_A_t2 at spot_41 from out of sight 19:30-20:02 appeared only
  on the `under_fire` line, never in the odds ("we outweigh it" / "an even fight"), and the group lost 37 of 48 (5.1k)
  standing there. A group described by its centroid: lab_11012's Thugs joined group_A_t2 one at a time and strung
  it from E4 to G8, the picture put it "at spot_37 (E4), stalled, party_46_t2 1224 away" while its 17-Thug front
  stood 241 from the party (11:34-12:14, 35 soldiers to 21). Seat 3's picture never mentioned group_A_t2 or any
  ally, so "advance together, never split" had nothing to hold to (11:00-11:27, group_A_t3 alone at C3). The Blinks
  were in no group and no actor entry, so the packet's scouting route had nothing to act on; no enemy factory or
  commander was seen all game. The player's wakes said "lost 2 of its N" while Jev's own picture said "lost 12 of
  its 36 ... losing this fight".
- MISSING ACTION: at spot_41 both the station and the fall-back offered were spot_41 itself ("right here"); nothing
  took the group out of the Mauser's reach or sent it in as one body. commander_t3 stuck at home with Bulls
  closing from 779 to 462 had no step-away until 24:07. commander_t2 at 21:54 was picked to "attack party_215_t2
  (3 unidentified), we outweigh it" with 4 Bulls already typed there in the record, its only escape spot_60 along
  the Bulls' flank.
- MISLEADING WORDS: ground groups offered and played against Banshee parties with "3149 metal against its 135, we
  outweigh it heavily" (14:21-15:40; a Pawn offered to hunt a Falcon); air and ground enemies pooled in one party
  that took a new name at each sighting; the same block named party_46_t1, _t2 and _t3 in one prompt; "outruns this
  group ... a chase drives it off" of a party holding ground and shelling.
- UNFORTUNATE INTERACTION: "advance to shelling" played four times (rule and plan) with the group's task coming back
  as `hold` each time, read by the player as the mark flapping; the fall-back choices at 11:13 were base or spot_42
  while the player's own fall-back point was set only at the next turn.
- Tools: raid_ledger's answer regex cuts `party_46_t2` to `party_46` (answer none for all 66 losses); the seat-0-only
  readers as above; commander_turns --told shows prompt headers only.

### Game 6 (Infern8, lost 17:00) — verified in code: the report's trade line sums each seat's `traded` (seats.rs, the field merge), and every seat records the same enemy deaths, so the "destroyed" side of every trade figure of a three-seat game is counted up to three times (every ledger row's trade tonight is inflated on that side); the per-seat picture carries no allied units (picture.rs has none), so each seat's Jev prices its own group alone. Corrections to my own record: the opponent played Cortex (the lobby side said Armada); there was no lone purple attack at 7:38 (Jev turned the attack down every second; five units died 700-900 ahead of the group on per-unit fight orders)
- MISSING INFORMATION (the biggest of the night): no seat's Jev sees the other seats' groups, so "advance as one body" is
  unpriceable: at D4 8:24-9:31 the three groups stood together 72 strong, then teal attacked at 8:28, purple at 8:35
  and dark blue went by another road, 9.6k lost for 3.5k. The trade line said "traded even" at 9:08 when the truth was
  4.0k for 11.2k, and "raid beaten 31.8k vs 18.8k" at 12:31 when it was 10.6k for 19.0k against us; "the opponent is
  known to hold 8" extractors never changed from 6:45 and carried no age; the block that took purple's home left
  sight at 15:00 and the report gave no last-known position; a party's weight jumped 2,135 to 1,075 to 5,510 to 2,590
  within seconds as the enemy commander came in and out of sight; "no growth: 12 extractors" was one seat's count
  beside a side score of 25.
- MISSING ACTION: the packet's "advance as one body" and "attack the enemy commander on sight" were dropped by the
  decompression into standing orders (no rule in the vocabulary holds them); the player's escape route for purple's
  commander was cut the same way, only its "never go to" spots surviving; dark blue's commander's escapes were "go
  home" (it was home, where the raid was heading) and steps to spots 1,100-1,285 away "out of its reach" with Brutes
  at 87 speed against its 38; no time-to-contact anywhere.
- MISLEADING WORDS: "attack 2 unidentified, we outweigh it heavily" with 7 Brutes 292 away; "shelling" named D4 early
  and G2 at 16:14, and the rule's step-away target at 16:14 was "shelling" itself, so the commander walked toward the
  unseen Brute and died; "out of its reach" for a step 1,200 away from a faster raider.
- UNFORTUNATE INTERACTION: a 9:52 "fall back home" pick stood 38 s while the player's newer station was offered only
  as an alternative and the report showed the station as set; a hunt "ended after 0 s: the leash of 900 reached"; the
  16:42 refusal on party_59_t3 (gone by the time the order landed) took the group's other three rules with it; the
  1:05 produce refusal (fixed since).

### Game 4 (Irishstud14, lost 20:30) — verified by the game-6 findings (same mechanisms); the report's own numbers accepted where they match the ledger
- MISSING INFORMATION: two seats attacked the same three Pounders under two names (party_87_t3 and party_87_t2, the
  same unit ids) with 8k of metal, 13:22-14:45, while 15-18 Incisors took eleven extractors; nothing said the parties
  were one. The centroid: at 7:27 "together at D3" while 15 Blitzes fought at C3 and 7 trailed to G2; `enemies_near`
  named one Brute while eleven Incisor hits landed; at 8:36-8:52 the Pounders killing the group stood 1,078 from the
  centroid, beyond the 600 alarm reach, so no threat slot and no engage state existed. "A large share: it is losing
  this fight" stayed 15-30 s after the fight ended, and group_A_t1 walked home "without fighting" past a raid on its
  own extractors it outweighed (15:44-16:32). The enemy's Graverobbers raising our wrecks were never said (the player
  wrote "captured Stouts"). Wakes: "lost 2 of its 18" against 7 of 22 in the hands' picture.
- MISSING ACTION: against the Incisor swarm the menu was chase or nobody moves; no "stand at the next extractor on
  its heading", no heading shown; a retreat home walked 11 Blitzes into five Pounders seen on the route 25 s
  earlier (9:02-9:19), the walk home not fighting back, and under fire the only state was "nobody moves".
- MISLEADING WORDS: "attack party_156_t3 (1 armstump), we outweigh it" as the default of `attack_raiders yes` when the
  Stout was the edge of a nine-unit ball 204-250 away: each small party's odds are priced as if alone (commander_t3
  died on it at 19:10-19:18).
- UNFORTUNATE INTERACTION: `raiders_party whole_group` made the attack the default in every threat slot, so one pick
  played two or three whole-group attacks for one group at once; `hold_line yes` pruned every way back because the
  centroid's odds never read "against"; two of three engage orders on the enemy commander refused for a rename
  (party_83 to party_86) between the report and the landing (12:35, it escaped at 13:27).
- Tools: raid_ledger's regex again (with the tag accepted the answered counts are 10/15, 12/17, 22/24 by seat); the
  seat-0-only readers; hands_window prints no state words. The reviewers shared one scratchpad and overwrote each
  other's scratch files (each says it rebuilt from its own game's data).

### Game 9 (thebluegecko, lost 20:16) — verified: `NEAR` in picture.rs is the reach within which a party makes a group's `enemies_near` line, measured from the group's centroid; the trade and allied-unit findings of game 6 apply here too
- MISSING INFORMATION: a "group" was a line of units 2,000-2,900 long and everything was measured from its centre:
  the Pounder ball 15:44-16:05 had no `enemies_near` line while Bulls stood 669-936 from its nearest Pounder
  (1,264-1,488 from the centre, past NEAR); 11-13 Pounders stood inside Bull reach while the attack option read
  "1238 away, 31 s of walking ... a chase drives it off"; "health: full on average" at 22 of 39 lost; the Stouts'
  group "advancing to spot_59" while the distance to go grew from 1,462 to 2,119 with 6-7 of them in Beamer reach.
  Bulls were a metal price only: "an even fight" for 13 Stouts (2,925) against 3 Bulls (2,850); a hunt worded "drive
  it off in 4 s ... can catch it" against a 950-metal tank; "we outweigh it heavily" of four radar blips with the
  Bulls unmentioned; the commander lines carried the Bull's reach (460), the group lines did not. The Bull switch was
  invisible: nothing scouted after 2:35, "factories_seen: none, ever" at 15:49, "known to hold 7" extractors while
  the 37k of his we had seen die by 13:24 implies about 18 extractors of income. The advanced plant showed "ordered
  200 s ago" with no build percentage: believed standing at 14:45, finished 16:12, on a seat with 0 metal while
  another seat banked 759 (no way to move metal between seats).
- MISSING ACTION: no state let a group close on unseen shooters as one body, pull out of their reach, or gather
  before contact; nothing used the four Shellshockers (reach 710) in the nest fight. A usable marker for a missing
  action: seconds where the gate wanted a change (up to 0.88) and no offered state rated above 0.36: 42 for F_t2,
  29 for E_t3, 19 for E_t1.
- MISLEADING WORDS: "moving away from us" of the Bulls at spot_33; 22 hunts and attacks named 20 different Bull
  parties with at most 15 Bulls in sight, the fresh names hiding that the last hunt had failed; 16 hunts of 5-11
  tanks against one or two Bulls.
- UNFORTUNATE INTERACTION: four reversals in 45 s (the player's advance landing at 16:07, Jev's fall back at 16:08,
  then hunts, an attack, a hold, an advance, a retreat); three groups told to "fight the Bulls together" at E7 each
  flipped by its own fall-back and pick (15:24-15:42) and never stood together; "fall back to spot_10" picked five
  times with spot_10 the plant the Bulls were entering; a standing rule walking commander_t1 to `shelling`. The
  player's orders ran a median 12.6 s behind the picture, in flight 84 % of the time. The queue call "corsolar
  spot_25" was accepted silently for a place not in the picture.
- The Pounders took 38,780 damage from Bulls and had an enemy in their own reach for 160 soldier-seconds all game.
- The user's own read of the replay (2026-09-27, after the reviews), verified in the records: (a) the first
  constructors of the north and middle seats drove west at 1:29-1:37 to spot_2, spot_36 and spot_45 (the person's
  start), 107-115 s of walking, because the hands offer the free spots the packet names and the packet wrote the
  seats' spots in the brief's shorthand ("spot_10, 5, 6, 12, ...", so only spot_10/34/72 matched, all three the
  commanders' list steps) while the scouting sentence named "spot_36, spot_19, spot_45, spot_2" in full; the `job
  expand` default took the nearest named by rule (jev-0 f2880-2910, jev-2 f2640-2850). Nothing is hardcoded to a
  side: every Comet game with people had us east and no earlier game sent a constructor west (all nine records
  scanned). (b) The commander's solar of 0:19 (Jev's pick, f585) was re-issued by the list's `corsolar` step landing
  at f600, before the first had started; the fresh order took a fresh site 96 elmos away and the first nanoframe
  decayed at f909. Both in the changes doc, 7.5 and 11.3. (c) At 5:58 the north and middle groups took no
  initiative against his unguarded buildings: a group has no state that names a building or sweeps the unseen
  (all group slots of the game listed: walks, splits, station, fall back, retreat, join, one-soldier scout, party
  hunts and attacks); the north group held at spot_9 with 27 of his buildings unguarded within 3,000 (truth-1 of
  the replay) until the player moved its station by hand; the packet's raid route "spot_9, then spot_2, spot_7,
  spot_14" became one station. Changes doc 6.6.

### Cape Violet (thebluegecko, lost 18:18) — verified in code: the water note's `amphibious_of_ours` filters our units by the lobby side's first three letters, so a seat on side Random ("ran") lists none, and the note's text is stale ("only the bot lab and the advanced bot lab can be built; the plants that make amphibians are not offered") from before the full roster
- MISSING INFORMATION: the sea was a wall both ways: the E3-E5 ford (6.4 % of the map within 20 below water, the only
  land route between the bases, height -14 where the Welders came) was drawn as deep sea, while the deep sea with his
  construction ships, under-water extractors and Liches was offered to tanks as ground. His economy at sea was posed as
  raiders: unidentified contacts on under-water spot_20 from 8:23 and spot_22 from 8:44 shown as "2 unidentified,
  standing still", named after the nearest land spot, with Blitz hunts offered and played against them (12 across the
  seats). The centroid again: at 8:35 group_A_t1 "at 348 from spot_31, advancing to spot_26, 1953 to go" with no
  `enemies_near` line while 17 of its 48 were in B3 under two Beamers and Welders and the rest strung to H7; the two
  stalls misreported ("advancing to d2_gate, stalled" while only 3 of 17 had the fight order and 14 held formation
  slots at home; "walking to d2_gate, 880 to go" while 19 stood at the gate and 5 sat in the ford). The water note
  gave no count of under-water spots and nothing of what an enemy does from the sea. A wake's loss count restarts
  each turn ("2 lost since your last orders" against 3,252 metal dead). "148 hits in 15 s" of one Blitz.
- MISSING ACTION: against the Liches (named at 17:02) the only answers were Blitz groups ("110 metal against its
  2550, a chase drives it off"); no anti-air state, no evasion; commander_t1 kept "build a armvp" from its list until
  the step-away appeared at 17:17 and died at 17:19 while the player's 20 s turn was still out. No sea domain at all
  in the states (ships, subs, coastal weapons).
- MISLEADING WORDS: "we outweigh it heavily" 350 times against Welders (tier 2, priced by metal); "it outruns this
  group at 87 against 75" from an unidentified contact priced as a Pawn (a Welder does 48); "not free" meaning three
  things (unreachable, held by the enemy, covered by wrecks); "Our bots and vehicles stop at the shore" with a
  Shipyard, a Hovercraft Platform and Beavers on the same page.
- UNFORTUNATE INTERACTION: the gate walk re-issued at 8:05 after the station had become spot_26, so the group missed
  the push; the player's switch to raw re-sent nothing (27 stops).

## The consolidated findings

Read across the six games (four losses to OS-48 players, one loss to a skill-25 player, one win). Nothing below is a
change made: the code, the rules and the brief stand at 6151a89. Each line names the games it was seen in.

### MISSING INFORMATION
1. **A group is a point.** Every distance, odds line, `enemies_near` line and "N to go" is measured from the group's
   centroid, and a group is often a line 700 to 2,900 long because a plant's output joins it one unit at a time.
   The front is in contact while the line says "1,224 away" or says nothing (game 3 11:34, game 4 7:27 and 8:36,
   game 7 11:04, game 9 15:44-16:20, Cape Violet 8:35). `NEAR` (800) from the centroid decides whether a party is
   named at all. What was needed: the group's front and tail, who of it is in contact and with what, and the odds of
   the part that is fighting.
2. **No seat sees the other seats.** The per-seat picture has no allied units, so "advance as one body" and "fight
   the Bulls together" cannot be priced or held: three groups standing together attacked at 8:28, 8:35 and by
   another road (game 6); two seats attacked the same three Pounders under two names (game 4); each south group was
   flipped by its own fall-back while the others stood (game 9). What was needed: the other seats' groups in every
   seat's picture, one name for one enemy party across seats, and the combined odds of what stands together.
3. **The trade is triple-counted.** The merged score sums each seat's destroyed metal and every seat records the
   same deaths (seats.rs): "traded even" was 4.0k for 11.2k; "raid beaten 31.8k vs 18.8k" was 10.6k for 19.0k (game
   6). Every "trade in our favour" said tonight, and in the ledger rows, is wrong on the destroyed side. The
   opponent's extractor count ("known to hold 7 or 8") never changed and carried no age while 37k of his had died
   (games 6, 9). What was needed: one count of enemy deaths across seats, and an income estimate from what died.
4. **What shoots is not in the odds.** A Mauser out of sight on the `under_fire` line only (game 3 19:30); Bulls and
   Welders priced by metal with no reach, speed or tier (games 9, Cape Violet: "an even fight" for 13 Stouts against
   3 Bulls, "we outweigh it heavily" 350 times against Welders); the enemy commander's D-gun and death blast (game 7:
   eleven Incisors to the D-gun under "we outweigh it heavily", thirteen Brutes to the blast in "an even fight");
   a party's speed from its one fast member (game 7, Cape Violet). Turrets covering a party count three times their
   metal but their reach is not said to groups (commanders' lines carry it). What was needed: reach, speed and
   tier beside the metal, the unseen shooter's likely place in the odds, and the commander as a fighter with its
   D-gun and blast radius. (The reach line added after game 9, d265666, covers part of this; it was not in any
   mined game.)
5. **Where the enemy is and what he has.** Nothing scouted after 2:35 (game 9), the Blinks never left home (game
   3, in no group and no actor entry); "factories_seen: none, ever" at 15:49; the block that took a home left sight
   at 15:00 with no last-known position (game 6); the Bull switch and the Liches unnamed until they struck; the
   enemy's Graverobbers raising our wrecks unsaid (game 4). What was needed: last-known positions with age, the
   scouts as actors with a route, an income estimate, and a line for tier-2 and air the moment one is seen.
6. **Our own state.** The advanced plant's progress never shown ("ordered 200 s ago", believed standing 90 s early:
   games 7, 9); a seat's metal banked while another has none, with no way to move it; the score and eco extractor
   counts disagreeing in every game; wake lines saying "lost 2 of its N" while the hands' picture said 7 of 22 or 12
   of 36; the loss count restarting each turn; "health: full on average" at 22 of 39 lost; a stall reported for a
   group most of whose units held formation slots at home (Cape Violet).
7. **The map.** The sea drawn as a wall both ways (a wadeable ford as deep sea, the deep sea as ground to chase
   across); the water note's amphibious list empty for a seat on side Random and its text stale; no count of
   under-water spots; "not free" for three different reasons (Cape Violet).

### MISSING ACTION
1. **A raid can only be chased or left.** Against Incisor swarms and Blitz raids every offered answer was a chase the
   words said could not catch, or "nobody moves"; no "stand at the next extractor on its heading", no heading
   shown, no turret state (games 4, 7; the threat slot on a base held "nobody moves" alone with the group 2,400 away).
2. **No way to fight as one body.** No gather-before-contact state, no "close on the unseen shooter together", no
   "pull out of its reach", nothing that used the Shellshockers' 710 reach in a nest fight (game 9); the station and
   the fall-back both "right here" under a Mauser (game 3). Marker: seconds where the gate wanted a change and no
   offered state rated above 0.36 (42, 29, 19 for three groups in game 9).
3. **The packet loses its shape in the decompression.** "Advance as one body", "attack the enemy commander on
   sight", the commander's escape route: dropped, only the "never go to" spots surviving (game 6). What was needed:
   rules for those, or the packet read at the moment instead of a fixed vocabulary.
4. **The commander.** Its D-gun is on its line and in no state; its escapes are "go home" when home is where the
   raid is heading, or a step 1,200 away from a raider twice its speed; no time-to-contact (games 6, 7). Against
   Liches, no anti-air state and no evasion (Cape Violet).
5. **The sea.** No ship, sub, coastal-weapon or air-over-water state; under-water spots not places at all (Cape
   Violet; the places part fixed after, b9a43ff).

### MISLEADING WORDS
1. "We outweigh it" from metal alone (every game). 2. A party's speed from one member; "outruns this group ... a chase
drives it off" of a party holding ground and shelling (games 3, 7, Cape Violet). 3. A fresh party name at each
sighting (22 hunts naming 20 Bull parties, game 9; one block under three seat tags, game 3), hiding that the last hunt
failed and making engage orders fail on a name. 4. "shelling" as a place: a step-away target (game 6 16:14, game 9
18:03), an advance that comes back as hold four times (game 3). 5. Ground groups "outweighing" aircraft and offered
against them (game 3 14:21-15:40). 6. "Out of its reach" for a step 1,200 away from a faster raider; "moving away
from us" of Bulls at spot_33; "the quarry is dead" of a live commander.

### UNFORTUNATE INTERACTIONS
1. The player's order and the pick crossing about 10 s apart: an advance landing at 16:07 and a fall back picked at
   16:08, four reversals in 45 s (game 9); "advance together" landing after the pick had sent a group back (game 7);
   a 9:52 "fall back home" pick standing 38 s over a newer station shown as set (game 6).
2. Defaults stacking: `raiders_party whole_group` making the attack the default in every threat slot, so one pick
   plays three whole-group attacks for one group (game 4); `attack_raiders yes` making "attack 1 armstump, we
   outweigh it" the default with the Stout the edge of a nine-unit ball (game 4); `hold_line yes` pruning every way
   back because the centroid's odds never read "against" (game 4).
3. Fall-backs into the next fire: "fall back to spot_10" five times with the Bulls entering spot_10 (game 9); a
   retreat home crossing Pounders seen on the route 25 s before, the walk home not fighting back (game 4); the
   fall-back rule sending a group 3,845 away with five more dying on the walk (game 7).
4. Refusals for formalities: an engage order on a party another seat saw (the standing tool checks the lead seat's
   sightings only: games 3, 4, 6, 9); a rename between the report and the landing (games 4, 6); a refusal taking the
   actor's other three rules with it (game 6); a whole `produce` call refused for one plant going up (games 6, 8;
   fixed since); a place accepted silently that was not in the picture (game 9).
5. Lists and rules: builders on a list never offered a solar through 120 s of STALLING (game 7); a gate walk re-issued
   after the station changed (Cape Violet); a hunt ending "after 0 s: the leash of 900 reached" (game 6); a list step
   re-issuing the build a pick had ordered 15 frames before, the first nanoframe left to decay (game 9, 0:19).
6. The packet's words read for what they are not: every `spot_N` token in the packet is an extractor candidate,
   whatever the sentence was about, so a scouting sentence sent two seats' first constructors across the map while
   the seats' own spots, written in the brief's shorthand, matched nothing (game 9, 1:29-1:37).

### Tool debt found by the reviewers (analysis tools, not the bot)
`run/raid_ledger.py`'s answer regex `party_\d+` drops the `_t<team>` tag (answered 0 on every row of every three-seat
game; with the tag the counts are 10/15, 12/17, 22/24 by seat in game 4); `analyze_match.py`, `hands_window.py`,
`raid_ledger.py`, `floor.py` and `fire.py` read the first `record-*`/`jev-*` file only; `hands_window.py` prints no
state words; `commander_turns.py --told` prints headers only; the reviewers shared one scratchpad and overwrote each
other's scratch files. The games with people have no truth file, so every enemy number is a floor.

### The topology review of Cape Violet (`run/matches/1790481450-bluegecko-3v1-cape-violet/00/topology-review.md`) — verified: the 11:17 whole-group attack on a floating radar "938 away, 13 s of walking" with 10 Welders at spot_25 left to nobody (jev-2 f20310-20355, played by plan); group_A_t1 "walking to spot_19, 655-763 to go" for 46-106 s at 10:00-11:00 with station_spot_23/25 shown as the default 59 times and never played (jev-0 f17400-20300); the Welder is bot class, depth 20 (record-0 unit_defs), a wader, not amphibious as the map note said
- The observation holds (12 unit-minutes short of unreachable targets, 12 on unstandable cells, a 98 s and a 90 s
  huddle while the third seat fought alone), but the cost sits in newcomers sent to the group's centre, attacks at
  sea parties, slots on cliffs, and arrival judged by the centre, more than in the straight-line words. Four
  pocket spots and two islands named. Folded into the changes doc as the measured note under 9b and 9b.6-9b.8.

### The decompression audit (three Opus reviewers, per game `decompression-review.md` in every human game's directory) — verified: the re-read bug in code (the "packet" event cleared only after a pass asks) and in game 10's logs (53/60/46 decompressions per seat, one per second, for five packets); the tool's null/false/"no" clears only a tool rule (`check_tool`); the two commander kills of game 10 (record-0 5:04, record-2 6:24 with the commander's `dealt`); game 4's commander_t3 attack picks at 19:10-19:11 and its 19:18 death blast taking three constructors, an advanced solar and a turret; "No energy converters; a solar only when STALLING" read as `solar never` three times; comet-catcher-3's `raiders_lone ignore` playing "leaves and holds" over Jev's hunt and attack picks at 21:50-21:57 with `stop` to all 11 Stouts and four Stouts lost to the Bull by 22:01
- Verdict across eleven games: the stage helped in a handful of plays, all rules the `standing` tool carries and
  that the player set by the tool in the same or next turn; it hurt through switch-offs that cannot work, pruning
  lifted from other actors' sentences or with conditions dropped, one-seat packets read into every seat, and a
  re-read bug costing up to 15 % of a game's Jev tokens. It was on only because run/human_game.sh omits
  `WITHIN_REASON_RULES=off`. Decision recorded in the changes doc 13b: drop it.

### The expansion study (`expansion.md`) — verified in code: `Brain.spot_claims` is read but never filled and `team_mates` is written but never read (the team board shares no spot claims while H-TEAM-BOARD says it does); `free_spots` counts only our own seat's extractors and orders; a list step popped by `next_list_step` is lost without a note when `execute_builder` returns None (lists.rs)
- The hypothesis measured: constructors going home is real but small (retreat and step-away 2 % of constructor
  time, 77 plays in 24 seat-games, nearly all under raids 3:20-5:56; home between consecutive extractors in 9 of 69
  intervals against his 0 of 52). The bigger return is "help the plant": 11 % of the time, 87 picks at a median
  confidence of 0.32 with an extractor on the menu, the extractor state's words saying nothing of what it returns.
- The thrash is between seats: 114 extractor orders walked toward and never built, 57 of them at spots another seat
  of ours took (27 already started when the order went), 2,630 constructor-seconds (7.6 %); a median 800-1,640
  elmos and 54-185 s between consecutive spots against his 500 elmos and 30-39 s. The words never mention the ally
  ("builds a metal extractor at spot_46 (14 s of walking, ground ): our 9th").
- Each seat is a third of him and the side was not behind on extractors: 20-35 for the side against his 18-27 at
  8:00, level or ahead through 14:00 in games 2, 3 and 9, the side's income ahead until 6:00 (game 3) or 12:00
  (games 2, 9); after 14:00 his income per extractor pulled ahead on tier 2 (112-132 a second from 32-40 extractors
  against our 60-76 from 27-37).
- Too few constructors and a ceiling at the midline: the allowances (`armcv:1` or `:2`, then raiders; no constructor
  at all in game 3 at 1:31) kept most seats at 2-4 constructors against his 4 by 4:00 and 9-13 by 10:00; seat-games
  with 6+ constructors held 15-21 extractors at 10:00, those with 4 or fewer 7-13; the side never held more than 5
  spots west of the middle with 16-23 free there at 8:00 ("never go to the enemy's strip"; each seat's named list
  ran out at 8-11 spots; the middle seat stalled at 7-9 every game).
- The first lists: a packet's "spot_34, 29, 35" names only the first as a spot (`diet::names` takes literal `spot_N`),
  two first constructors were sent by the rule toward the person's base (108 s of walking), and 39 of 139
  constructor lists' extractor steps vanished unplayed and unreported.
- His method: one plant making a constructor about once a minute (8 by 8:00); field constructors running strips
  outward with a turret every one or two spots and never going home; one or two constructors on solars and nanos at
  home; the same share of time on turrets and solars as ours.

