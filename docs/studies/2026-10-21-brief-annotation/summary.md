# The brief annotation: merge pass (2026-10-21)

The join of the five readers' parts (`part-a.jsonl` to `part-e.jsonl`, 700 rows; `part-a.md` to `part-e.md`). Every
item below traces to a row (`file:lines`, with `part N` where a passage was split) or to a line of a reader's summary;
nothing was re-annotated. Readers A and B split `docs/briefs/player.md` at 657/658, C had `experience/intro`,
`winning`, `cases`, `late-game`, D the other experience files and the reference tables, E the role prompts and the
hands' rules. Cross-file references that a reader gave by file and subject were joined here to the other reader's
row by its line range; where the join needed the brief text, the lines were read (`player.md:818-822`, `108-112`,
`late-game.md:170-176`, `commander.md:19-22`). The companion file is `backlog.md` (the derivation backlog, the meta
catalogue, the episode index, the fact index).

Paths are shortened: `player.md` is `docs/briefs/player.md`, `experience/x.md` is `docs/briefs/experience/x.md`,
`reference/x.md` is `docs/briefs/reference/x.md`, `strategist/x.md` is `crates/bot/src/strategist/x.md`,
`pianist/x.md` is `crates/bot/src/brain/pianist/x.md`, `player-opus.md` is `docs/briefs/player-opus.md`.

## (a) Totals, recomputed from the JSONL

A Python pass over the five files (every one of the 700 lines parses and carries the schema's keys). Counting rules are
beside each number.

- **Rows:** 700 (a 94, b 104, c 187, d 152, e 163). Games cited: 159 names in `sources.games`, 128 after joining
  aliases (`comet-3`/`comet-3-medium`, `player-9`/`player-9-posing`, the two `Center Command 2v4` spellings, and so on).
  People named most: thebluegecko 86 rows, the user 85, u6bkep 13, Irishstud14 12, Matt 9, VAK 6, computer_whisperer 4.
- **By fate:** tool 177, episode 162, rederive 148, meta 105, fact 84, discard 24.
- **By layer (what the passage is now):** episode 160, mixed 141, tool 135, derived 92, meta 90, fact 70, ruling 12.
- **Derivable, among the 148 rederive rows:** yes 106, partly 42, no 0. (Four rows carry `derivable: no`; all four are
  reader E's `pianist/default.md:2-5`, whose fate is discard, not rederive.)
- **Doctrine** (a rule flagged with its origin): 203 of 700 rows: rule-from-one-case 123, rule-from-ruling 49,
  rule-from-pool 31. The density differs by reader: A 38 of 94, B 66 of 104, C 24 of 187, D 15 of 152, E 60 of 163;
  see (d).
- **Era** (multi-valued; 246 rows carry more than one): undated 213, people 200, pre-hands-rebuild 186,
  barb-hard_aggressive 146, barb-medium 94, pool 87, apex 32, barb-easy 22, barb-hard 7.
- **Rows whose `defects` names a defect:** 258 (441 say `none`, 1 `unknown`). Rows whose `defects` is only the era
  marker (reader C's "hands rebuilt 2026-10-02; before rove as a group task ... the standing transfer", naming no defect
  the lesson rests on) are counted among the 258; the episode index in `backlog.md` separates them.
- **Conflicts:** 152 rows carry at least one, 201 conflict items in all (a 28, b 45, c 23, d 21, e 35 rows).
- **Duplicates:** 439 rows carry at least one, 802 duplicate items in all (a 70, b 97, c 82, d 105, e 85 rows).
- **Numbers checked:** 182 rows carry a check string; 158 report a match (or that the number is not a glossary column:
  map data, weapon files, the reference sheet) and 24 name a mismatch (a 4, b 10, c 7, d 3, e 0). Reader E's three
  failures against the reference sheet and the one against speeds ("Blitzes against Ticks") are not glossary
  mismatches and are listed separately in `backlog.md` (d).
- **Lens:** ledger 360, unknown 208, person 68 (one row says "person for player-50; ledger for player-54"), chat 38,
  model 26. `unknown` is concentrated in E (99 of 163: tool and fact rows), C (53) and D (51).
- **Confidence:** high 499, medium 193, low 8. **Grounding** (multi-valued): our-game 344, code 277, engine 151,
  person 128, pool 117, model-review 29, chat 15, unknown 10.

## (b) The conflicts, joined across files by subject

Each cluster lists every statement the readers flagged, with its passage, its source and date as the row gives them.
The user rules; nothing here is resolved.

### 1. The anti-nuke's timing (a reversed ruling, both versions live)

- `player.md:168-182` (A, layer ruling): once his tier 2 stands, an anti-nuke under every home is due diligence, not a
  reaction; the user, 2026-10-19.
- `experience/late-game.md:172-188` (C, layer ruling): the anti-nuke goes up on the opponent's clock, not on his tier 2;
  the user, 2026-10-20: "we have definitely been building the anti-nuke too early". Lines 172-183 carry both rulings
  in sequence (E's conflict 7).
- `strategist/commander.md:43-55` (E, meta): the timing is the opponent's habit; people build a silo very late or never
  (thebluegecko, u6bkep, computer_whisperer: Center Command, Cape Violet, 2026-10-18..20), BARb hard_aggressive's
  anti-nuke at 21-25, Apex +60 a silo at 24:28 once and none the next game; the user 2026-10-20.
- `reference/frontier-great-divide-v1.md:16` and `frontier-quicksilver-remake-1-24.md:16` (D): BARb's 21-25 window,
  measured on Comet (K-barb-anti-nuke-by-minute), applied to Great Divide and Quicksilver without a measurement there.
- `experience/late-game.md:147-170 part 2` (C): BARb medium's anti-nuke "in one long game of six, at 25" against the
  claim's 135 of 815 long games (median 25); denominator unexplained.
- The episode behind the 2026-10-20 ruling: apex60-1, the first silo seen at 33:19 with its missile landed at 33:12
  (`late-game.md:172-188 part 2`, C; `player.md:168-182 part 2`, A), before H-REPORT-BLAST-CAUSE existed.

### 2. BARb's tier-2 clock

- `player.md:369-372` (A, fact): BARb's second factory is a tier-2 lab by 15-20 at the latest, "on that clock whatever
  we do" (K-barb-tier2, conjectured for Quicksilver medium, 2026-09-19); a duel should be decided by then.
- `player.md:16-29` (A): a Comet duel is decided inside fifteen minutes (the user, 2026-10-18) against 369-372's clock.
- `player.md:581-657 part 4` (A, meta): hard_aggressive's advanced lab near 11:50 (K-barb-hard-aggressive-tech-clock,
  2026-09-22).
- `player.md:797-815 part 2` (B, meta): hard_aggressive's tech clock from escalate-4/5 (Gunslinger 12:00, Fatboy and
  Sharpshooters 19:00, Pulsar 24:00, Razorback 25:00; two games agree).
- `experience/cases.md:175-179` (C, meta) and `experience/winning.md:21-23` (C, discard proposed): an advanced factory
  at 10:41 median (9:55-11:44) in 73 of 74 games; the army flat 12:00-16:00, doubling every five minutes from 20:00.
  Profile unstated in both; C could not find the source claim in `docs/knowledge`.
- `reference/comet-catcher-remake-1.8-barb-hard_aggressive.md:1-7` (D, meta): 9:57 / 10:44 / 11:45 in 83 of 110 of our
  games to 2026-10-03.
- `experience/opponent-barb.md:17` (D, fact): "techs on a clock, whatever we do" against K-barb-tempo-is-half-clock-
  half-sighting and K-barb-tier2's income trigger (our raids on its income can move the clock).
- `experience/late-game.md:79-83` (C): his plant 10:41 and moho 15:12, the profile named.

### 3. What the bank buys first

- `player.md:247-254` (A): a bank means build power missing; constructors and nanos the turn it appears, a second plant
  only after (escalate-5/6/7, comet-4; thebluegecko and Irishstud14; 2026-09-22/23).
- `player.md:374-385` (A): a bank goes to a second lab or a construction turret, not converters
  (K-eco-production-is-the-bottleneck, K-eco-expansion-before-conversion, 2026-09-19/21).
- `player.md:543-547` (A): a constructor a minute per plant, four by 4:00 and eight by 8:00, "the person's count"
  (unattributed), above the pool's 3/5/8 at 5/8/12 (`experience/cases.md:282`).
- `player.md:723-739 part 2` (B): the second plant at income 30, the third at 40; "a plant pays for itself in twenty
  seconds" (comet-4, 2026-09-22; B's check: 590 at 25 a second is throughput, not a return).
- `player.md:751-764 part 2` (B): at 5:00, plants or nanos, either spends it (escalate-1).
- `player.md:816-833` (B): a bank above 500 with income over 30 is build power missing, not a plant missing; `armcv:4`
  or 5, nanos from 5:00, only then a second plant (escalate-5/6; the pool's five by 5:00 and three nanos by 6:37, VAK).
- `strategist/player.md:429-430` (E): a bank is spending behind, "never that you are ahead": a second factory, soldiers
  or constructors that turn. E's check against the sheet: winners bank more than losers at 8:00-9:00 (1,148 against
  614) before the second factory at 8:11.
- `strategist/commander.md:121-129` (E): a climbing bank with little power in reach wants construction turrets on the
  factory, not another factory.
- The pool (`experience/cases.md:280-284`, C; `reference/comet-catcher-remake-1.8-arm.md:148-154`, D): constructors
  3/5/8 standing at 5/8/12, store 260/837/410, the first construction turret at 5:56 before the second factory at 8:11.
- Partly a defect: escalate-5/6's bank was in part the plant choosing a constructor 1 of 17 times and `armcv:2` misread
  (fixed before escalate-7; A's row 247-254).

### 4. Solars before the plant, and the energy rules

- `player.md:548-552 part 2` (A): a plant wants about eight solars, built beside the commander before and while the
  plant goes up (2026-09-22).
- `player.md:581-657 part 1` (A, the pool 2026-09-24): three solars, never a fourth before the plant; the fourth at
  about 1:30, the fifth by 3:00; part 5: five solars at the 4:00 check.
- `player.md:255-281 part 3` (A, player-12, thebluegecko 2026-09-28): a solar with energy not STALLING is three
  extractors spent on nothing; the bound is metal.
- `player.md:707-722` (B, comet-3): two solars before the plant; six starve the opening.
- `player.md:765-777` (B, escalate-2): three before the plant and three more by 2:00.
- `player.md:740-750 part 3` (B, comet-5): about eight solars per running plant.
- `player.md:778-796` (B, escalate-3): a solar per Blitz queued.
- `player.md:925-983 (a)-(c)` (B, thebluegecko and Irishstud14 2026-09-27): three solars for a fast constructor, no
  converters, an energy store, 200 energy a second and nanos by 6:00.
- `player.md:1110-1121 part 3` (B, Quicksilver): three windmills under three constructors stall.
- `strategist/player.md:437-439` (E): a basic solar when STALLING and 150 metal (the 150 has no source) against
  `player.md:209-212` (every later generator an advanced solar, basic only when STALLING or the store is empty) and
  `pianist/default.md:2` (wind until banking).
- `pianist/rules.md:1 part 2` (E): energy only keeps the store from emptying, against `experience/late-game.md`
  (energy binds the tier-2 economy; apex50's debrief) and `player.md:165`.
- Two assist lists for the same opening: `player.md:581-657` (assist 20 / assist 60, the constructor first) against
  `player.md:925-983 part 9` (assist 40 / assist 30, a Blitz then five Rovers first, "his line, never armcv:2"),
  the latter attached to player-12, which lost on it (B).
- A fact disputed inside the arithmetic: "the plant alone draws 48 a second" (`player.md:255-281 part 2`,
  `experience/cases.md:304-306`) against 72 from armvp's 150 build power (A, C); "a lab draws about 80"
  (`player.md:486-492`, `experience/units.md:62`) is the vehicle plant's 82, the lab's is 57-63 (A, D).

### 5. The commander's place

- `player.md:312-321` (A): fights at home, builds at the plant, never a walker; leaves for the neighbouring seat when a
  block is coming (comet-4, 3v1-2, human-1, human-8, escalate-4; the user). Two of the five cases were harness faults
  walking it (A).
- `player.md:581-657 part 5` (A): from 2:00 the commander can walk toward the middle as VAK's did; part 1: on the plant
  from the second it stands, out only for one solar or one extractor.
- `player.md:92-145 part 2` (A): reclaimable for 2,700 when the side keeps another commander.
- `player.md:1105-1106` (B, discard): keep it near home after the opening (the smoke games' far walks, a 2026-09-21
  defect).
- `experience/cases.md:390-394` and `player.md:723-739 part 4` (C, B): comet-4's commander killed at home / the
  withdrawal at 25%, the player's own order.
- `strategist/player.md:76-83 parts 2-3` (E): the commander is the early army; later never toward soldiers that
  outreach its D-gun or into unseen ground; its places are the packet's (wake-4).
- `pianist/rules.md:1 part 12` (E): never toward a party that outreaches or outweighs it; away from home it walks to a
  named place before it is hit.
- `pianist/rules_gate.md:1` (E): "The commander never walks toward a fight" (stricter than either).
- `pianist/default.md:2` (E): "Walk home (go) at once when under fire" (against the 2026-10-01 ruling on fall-back
  places, `strategist/player.md:123-126`).
- `experience/players-words.md:12` (D) and `experience/cases.md:322` (C): thebluegecko, the commander assists the lab at
  birth (2026-09-28).
- At sea: `player.md:992-1001 part 2` (B) sends Beavers and the commander to a shipyard; `strategist/commander.md:64-73`
  (E) and `experience/cases.md:409` (C) have the commander take sea spots itself from 0:00.
- BARb's commander: `experience/opponent-barb.md:15-16` (D) "builds at home, never attacks" is medium's rule; on hard
  it roams to 2,000 elmos until minute 7 (K-barb-commander).

### 6. Who answers a raider (the raid answers)

- `player.md:30-55 part 3` (A, the user 2026-10-03): the first raiders go to his nearest spots; the picket for our
  extractors is the turret beside each, not the first army.
- `player.md:334-343 part 2` (A, the user 2026-09-22, escalate-4): pickets of one or two Blitzes at each outer spot
  cluster from the first Blitzes, a light turret beside each pair.
- `player.md:234-246` (A): a unit that can catch the raider, standing across the front before the raid (a Rover at 168
  catches a Tick at 132, a Blitz at 101 never); a turret beside each outer pair; never the ball.
- `player.md:493-500` (A, human-8, the user): on Quicksilver the first Grunts are the guard beside the commander.
- `player.md:581-657 part 2` (A): two Rovers stay home from 2:30 against Ticks; the rest scout.
- `player.md:740-750` (B, comet-5): turrets do the guarding; one block, never five guards.
- `player.md:765-777 part 3` (B, escalate-2): the whole home group, not four-Blitz detachments.
- `player.md:865-924` (B, 3v1-1) and `834-847 part 2` (escalate-6): pickets of two per seat; one group for the first
  Pawn wave; the ball is not the raid answer.
- `player.md:1131-1139` (B, pianist-player-2): a named raider-hunting group beside the ball.
- `player.md:1164-1171` (B, pianist-player-6): raiders of our own on the passages, not turrets alone and not Maces.
- `strategist/player.md:370-373` (E, the user's 2026-09-22 observation of BARb): soldier pickets at each outer cluster
  from the first Blitzes, against `strategist/player.md:433-436` (the picket is the turret). Both stand in the role
  prompt.
- `strategist/player.md:431-432` (E): raiders answered by units that can catch them ("Blitzes against Ticks": fails,
  101 against 132), never Hammers, Maces or Stouts alone; against `pianist/rules.md:1 part 5` (the user, 2026-09-27):
  a chaser slower than the raider still drives it off.
- `pianist/rules.md:1 part 10` (E, the 600 rule, 2026-09-22): a party within 600 that the group outweighs is fought
  now, "whatever the instructions call it", against `strategist/packet-example.md:42` ("a single armfav is NEVER
  answered by Blitzes or Stouts"), `player-opus.md:25-27` ("never the body's") and the menu's 4x withholding
  (`strategist/player.md:345-360`). E: the mechanism under player-57's thrash is written into the shape players copy.
- `strategist/commander.md:130-137` (E, human-21): do not send a body back out after a raid.
- `player.md:865-924 part 2` (B, 3v1-2) "Blitzes never go at a commander" against `925-983 (g)` "his commander is
  sniped by light tanks circling it".
- BARb's squad: `player.md:356-359` (A) and `experience/opponent-barb.md:6-8` (D) "about four" is medium's figure
  (the quota is 9 on easy/medium, 6 on hard); `pianist/rules.md:1 part 3` states BARb's raid clock to the hands as
  every opponent's nature, against the people's shapes (`experience/team-games.md:9-26`, human-23's gunship rush) (E).

### 7. Going home, going, and arriving together

- `strategist/commander.md:155-160` (E, the user 2026-10-04; human-23 x2, packet-example-1, going-1): the army goes
  now, to his outer extractors and constructors; it does not come home; never a place in our half, a count or a clock.
- `strategist/player.md:318-325` (E, undated, the early arcs): the raid answer stands before the ball leaves; a
  crashing economy is survivable only if the kill is sure, else the ball comes home and the economy is rebuilt first.
- `strategist/player.md:380-382` (E) and `player.md:299-300`, `893-894` (A, B): one gathering place, the groups arrive
  together or the push waits; new units join at the gathering place, not at the front.
- `pianist/default.md:5` (E): a group of four advances to passage_1 and holds; an outweighed group walks home.
- `strategist/player.md:123-126` (E, the user 2026-10-01): the step-back place is the player's; the hands have none.
- `player.md:56-91` (A): a lead decays unless the ball is in his half within minutes, against `92-145` (the lead can
  also be spent as the sprint).
- `player.md:925-983 (f)` (B, thebluegecko): spend the lead at once as one line; applied on Cape Violet it sent 6,800 of
  Blitzes into the B3 Beamer camp (`player.md:1002-1004`, per the ledger).
- A lens mismatch, not a text conflict (E): human-23's second game is cited for the going doctrine
  (`strategist/commander.md:149-151`) while the ledger's person-ranked reading is "poor defensive control".
- The players' words: "your armies keep not fighting together" (`experience/cases.md:352-353`,
  `players-words.md:43-47`) and K-army-piecemeal-midmap stand as the counter-premise E's derivation must bound.

### 8. Constructors by the clock

- `player.md:543-547` (A): four a seat by 4:00 and eight by 8:00 ("the person's count", unattributed).
- `player.md:247-254` and `816-833` (A, B): the pros' five constructors by 5:00 (VAK's game).
- `experience/cases.md:280-284` (C, the pool): 3/5/8 standing at 5/8/12.
- `strategist/player.md:65-73` (E, the sheet): first by 1:13/1:30/1:37, three standing at 5:00 (four at the upper
  quartile), four to seven made by 8:00.
- `strategist/commander.md:110-120` (E): "about five by 5:00" and the rule "five constructors a seat by 5:00, then one in
  every four or five units" (E: does not match the sheet).
- `strategist/player.md:291-304 part 2` (E): "four to six constructors on expansion" (no source).
- `pianist/default.md:4` (E): one constructor first, raiders until a group, a second only then (the cadence human-7
  and human-22 name as the failure; `commander.md:110-120` forbids an opening count).
- Quicksilver: "two constructors is very greedy" (`player.md:486-492`, `experience/players-words.md:13-15`,
  `other-maps.md:10-12`) against Matt's second constructor at 1:45 (`player.md:520-530`) (A, D).

### 9. What ends a game

- `experience/winning.md:75-77` (C, fact): his commander or his ability to produce.
- `experience/cases.md:168-169` (C): a side loses only when it has no commanders left (the game-end rule).
- apex60-1's row (C): the game needs a living commander (Lazarus raised two).
- `pianist/rules.md:1` and K-rules-commander-death-ends-game (E's premise; cited by `player.md:399-422 part 3`, A).
- `experience/winning.md:7-13` (C): economy is a means; only the army wins (rests on the rule once settled).

### 10. The moho frame time

- `experience/late-game.md:18-24` (C, fact): about 70 s an upgrade on one advanced constructor (14,900 / 210 = 71 s).
- `experience/cases.md:160-162` (C): 90 to 135 s on one advanced constructor, measured on Center Command with the
  store empty; C's check: cormoho 14,100 / coracv 310 = 45 s alone, so the measured time includes the store.
- `strategist/player.md:35 part 6` and `commander.md:32-38 part 4` (E): 19-33 s each with helpers, 90-135 s alone
  (the replay's numbers).
- `experience/late-game.md:84-88` (C): player-32's fusions at 274-276 s against 186.
- `experience/units.md:93-96` (D): 3,150 metal at 9 a second is 350 s (5:50), not "three and a half minutes".

### 11. The Great Divide pass

- Width: `player.md:1057-1061` (B) and `docs/knowledge/maps/great_divide_v1.md:29` say 644; `experience/other-maps.md:85-88`
  (D) 700 to 840; `experience/cases.md:336-337` (C) 700, and "1,700 wide at the median" not found anywhere.
- The camp: `player.md:1053-1056` and `1074-1079` (B), `experience/other-maps.md:92-94` (D, the user 2026-09-27,
  second-hand), `experience/cases.md:336-337` (C, "the players expect a camp at the pass", no person found): against
  the ledger's bluegecko-2v1-great-divide (2026-09-27, lost 15:28): "the person did not fortify the pass", two light
  turrets at 7:05, his commander through at 4:26 and camped in our half. None of the three sections mentions the game.
  K-opp-thebluegecko records him putting three Beamers and two rocket AA at the north mouth by 5:10 when alone against
  six BARb (2026-09-26 replay) (B).
- Our games there: `experience/other-maps.md:95-96` (D): "three team games of the old bot sat at home on five
  extractors for 35 minutes" against the ledger's rows 113-118 (2v2s at 20- and 30-minute caps; commander-team-1 held
  10 extractors at its 35-minute cap); "the games with people since have not been read" is stale.
  `experience/cases.md:336` "no game of ours has met it yet" against the ledger (C).
- The nest: `experience/other-maps.md:97-98` and `cases.md:328` say E3; the knowledge entry
  K-map-great-divide-v1-the-choke-is-the-plan says F3 (D).
- The sprint there: `player.md:1080-1094` (B) fixes "by about 10:00", which the sprint bullet (`player.md:92-130`) says
  not to do; lines 1080-1084 were rewritten 2026-10-18 to splice the sprint in.
- `player.md:1068-1073` (B): a radar and two turrets at the north mouth cover everything, against the same game (two
  turrets at the mouth did not cover the spots behind them).
- `experience/other-maps.md:93` (D): "an experienced player's word: an all-air side loses", no source found.

### 12. The sprint's start, and how rare tier 2 is in the pool

- `player.md:108-109` (A): "the sprint starts now" once the three questions answer long, covered, none; against
  `110-111`: at the first bank or when the side's spots are taken, whichever comes first (the user, 2026-10-18).
- `player.md:865-924 part 3` (B, 3v1-3): the advanced plant by 12:00 whatever the front is doing; `player.md:1080-1094`
  (B): by about 10:00 on Great Divide; `strategist/commander.md:17-22` (E): first bank or spots taken, "not when his
  tier 2 is seen".
- The pool's count: 4 of 58 sides (`player.md:658-679 part 3`, `experience/winning.md:27`, `cases.md:125`) against
  4 of 80 (`experience/late-game.md:3-4`) against 1 of 41 Armada and 3 of 39 Cortex (`reference/*-arm.md:148-154`,
  `*-cor.md:151-157`, D: "the games end before"); never before 12:51 (the knowledge file). The brief carries the
  2026-10-18 ruling that tier 2 is a sprint beside the pool's rarity without reconciling them (B, D).
- `player.md:92-145` (A): "the first moho at 6 to 9" against the frontier's first tier-2 extractor at 8:14 on Comet
  (`experience/late-game.md`).
- `player.md:816-833 part 2` (B, escalate-5): the tier-2 plant beside the home turrets with a Blitz pair, started while
  the bank is high, against the later placement and feeding rules.
- The advanced constructor's cost: "410 to 470" (`player.md:92-145`, `experience/cases.md:118`) is the bots' range (and
  410 is Legion's); the vehicles are 550/580; `cases.md:148` says 430 to 580 (A, C).

### 13. Energy for the mohos and the late ladder

- `player.md:157-167` (A): mohos, then fusions, the advanced fusion, advanced converters (K-t2-post-moho-scaling).
- `player.md:183-200` (A): one advanced solar per four mohos beside them (player-42, human-10).
- `player.md:374-385` (A): extractors beat converters while free spots remain; a bank goes to a second lab, against
  `183-200` (past the spot count the bank goes to mohos).
- `experience/late-game.md:52-77 part 2 and part 4` (C): the user's recollection, fusions as the transition then heavy
  advanced converters (2026-10-18).
- `experience/late-game.md:31-35` (C): "on Comet the experienced players build no converters" against
  K-eco-energy-storage-is-the-cheap-buffer's 10 of 80 sides.
- One episode, two numbers (A): the +50 Altored converter wasted 820 energy a second for thirty minutes
  (`player.md:374-385 part 3`) or banked 2,000 a second for sixteen minutes (`player.md:157-167`).

### 14. Expansion: steps, the midline, the middle spots

- `player.md:386-387` (A, K-army-verdicts-v18 2026-09-19): expand in steps the army has covered.
- `player.md:543-547` (A): the midline is not a wall; free spots past it by walk, nearest first.
- `player.md:581-657 part 5` (A): the commander toward the middle from 2:00.
- `player.md:707-722 part 2` (B, comet-3): the strip first, against the claim's update (the experienced players take
  spot_43/38/39 first with the commander and lose none).
- `strategist/player.md:291-304 part 2` (E): a turret with every middle spot; `285-290`: tempo, not schedule.

### 15. Light posts, static defence and reach

- `player.md:30-55 part 4` (A, player-50): a light post dies to the handful arriving as one body; against
  `player.md:282-292` (static defence is out-ranged or walked round, never walked into); reconciled only by "a nest it
  outweighs".
- `player.md:723-739 part 3` (B): "Shellshockers outrange every tier-1 turret" against the Gauntlet (1,220, tier 1 in
  units.csv) at `player.md:1074-1079`.
- `player.md:925-983 part 7` (B): "Bulls and Beamers outrange every tier-1 tank" against the Lasher 575, Whistler 550,
  Shellshocker and Wolverine 710.
- The odds model: "a turret counts about three times its metal" (`player.md:350-354`, `experience/winning.md:60-62`,
  `units.md:34-43 part 3`, `strategist/player.md:380-382`, `pianist/rules.md:1 part 4`) is a 2026-09-19 fit on the
  heuristic bot's easy/medium games (A) or a code constant whose source no text gives (E) or a nine-in-ten simulator
  result (D); the origin is unknown to all three.
- Unseen fire: `strategist/player.md:263-269` (see the shooter first; never make shelling a standing destination)
  against `pianist/rules.md:2 part 2` (close on the shooter when the estimate says we outweigh it) (E).

### 16. The pool's extractor curve, three spellings

- `player.md:658-679` (B): 29 duels at OS 25+ (2026-09-23): winners 18/23/28 at 8/10/12.
- `player.md:581-657 part 2` (A): 33 duels at OS 40+ (2026-09-24): 7, 13, 17 at 4:00, 6:00, 8:00 (E: the sheet's
  medians are 7, 11, 15; 17 is the winners' median at 8:00 and 13 is nobody's).
- `strategist/player.md:278-284` (E): 40 duels at OS 40+: 7 / 15-16 / 23 at 4/8/12 (the sheet: 22 at 12:00).
- `experience/cases.md:15-20` (C): "78 of 280 at 3:00" is in neither reference table (the arm table has 93 of 272).
- `player.md:658-679 part 2` (B): K-eco-winner-pulls-away-8-to-12's caveat (the losers' bank at 10:00 may be a killed
  factory) is not carried by the brief's reading as a spending habit.

### 17. Unit facts against the glossary (the readers' mismatches with a doctrinal consequence)

- "A Stout beats any tier-1 bot head on" (`player.md:570-580 part 2`) against units.csv's sim column (loses to the
  Mace at -0.34) (A).
- "Pawn beats only Grunt and loses badly to Thug" (`player.md:328-331`) against the csv (beats seven types, not
  Grunt; -0.32 to Thug) (A).
- "A Blitz is two Pawns' speed" (`player.md:570-580 part 2`, `experience/units.md:38`): 101 against 87 (A, D).
- "Six Stouts beat one Fatboy (+0.12)" (`experience/cases.md:105`) against +0.52 after the area-of-effect fix
  (`docs/knowledge/units.md:209`) (C).
- The pros' spacing 67-70 (`experience/units.md:57-59`) against `docs/knowledge/formations.md:16`'s median 64 (D).
- The rest of the glossary mismatches are in `backlog.md` (d).

### 18. Attributions, names and dates that disagree with the ledger or the knowledge

- thebluegecko "about OS 48" (`experience/players-words.md:1-6`, 3v1-10) against 34 OS in the human-10 and Center
  Command rows (D).
- "Not a unit choice problem, a not enough mex problem" attributed to "the opponent, human-10"
  (`experience/players-words.md:17-22`, `cases.md:224-225`); the human-10 row names thebluegecko at 22:38 (D).
- `experience/cases.md:240-241` attributes games 3-5 to "a person"; the ledger has thebluegecko, Irishstud14 and
  Infern8 (C).
- `experience/cases.md:10-13` says human-N games are on Quicksilver unless said; human-10 was Gecko Isle, human-18 All
  That Glitters, human-23 Comet, and none of their passages says so (C).
- pianist-player-3 "against BARb medium" (`cases.md:77`); the ledger says easy (C). pianist-player-6: Pawns on the
  passages stopped the raids (`cases.md:270-272`); the ledger says the player built no Pawns (C, D).
- `experience/hands.md:36-41 part 2`: a roving Rover "killed nothing (player-20, player-21)"; the player-20-live row:
  the roving Rovers killed about 420 by 4:36 (D).
- `experience/units.md:141` "nothing below has been measured in a game of ours" beside lines 146-148's leg-player-1
  (WON 25:54 in the ledger) (D).
- `experience/team-games.md:80-82` and `88-95` are headed "several seats"; human-11 and human-12 were 1v1s (D).
- `experience/team-games.md:25`: the 3v1 trades quoted at 15-22 predate the ledger's 2026-09-27 evening correction
  (seen figures summed the seats and counted deaths twice) (D).
- `player.md:1005-1052` (B): SailAway game 11 dated 2026-09-28; the match directory's epoch is 2026-09-27.
- Cape Violet's spots under water: 28 (`player.md:987-991`) against 26 (`experience/cases.md:403`) against 26 plus
  two islands (the knowledge claim) (B, C).
- The Welder: "amphibious tanks" (`player.md:992-1001`) against the knowledge claim's wading tier-2 bot (B).
- "Phoenix" bombers from the tier-1 air plant (`player.md:1080-1094 part 3`): armpnix Blizzard is tier 2 (B).
- Names not in units.csv: Pillager (the 1,300 range is the Ambassador's: `player.md:282-292`, `cases.md:328-330`,
  `other-maps.md:89-91`, `player.md:1080-1094 part 2`), Punisher (Agitator), Rascal for armfav at `player.md:245`
  (the csv's Rover; the Rascal is corfav at 153), Tumbleweed, Bulldog, Instigator, Leveler, Slasher, Coyote, Reapers
  (Tiger), Tremor as the Mauser's twin (Quaker), Mason (A, B, D).

### 19. The role prompt against itself and against the knowledge

- "A never sentence holds" (`strategist/player.md:153-177`, `442-443`) against K-jev-never-is-not-heard (a prohibition
  does not remove the option, chosen at 0.4-0.8 when its words fit); which is current is unknown from the texts (E).
- `strategist/player.md:385` "The game is paused while you take a turn" against `10-18` (with people it runs on) (E).
- `pianist/default.md:2-5` (E): a wind opening, a reactive turret, "one constructor first, then raiders", a hold at
  passage_1 with four, walk home when outweighed; live in pianist-alone games and in any second without a packet,
  against `strategist/player.md:437-439`, `305-313`, `65-73`, `123-126`, `commander.md:110-120`, `155-160`.
- `pianist/rules_gate.md:1` against `rules.md:1 part 12` on the commander (any fight / a party that outreaches it).
- The hands' mechanics named in `player.md:778-796 part 4`, `848-864 part 3`, `1107-1109` (B): `send_against`,
  `attack_unit`, `fight_to` with the underscore, `start_boxes_note`, `reached_by` have no hits in
  `strategist/player.md`; whether any survives the rebuild is for E's files, which do not use the names.

### 20. Smaller clusters

- The rematch plan after escalate-4 (`player.md:797-815 part 4`) stands beside its refutation in escalate-5
  (`816-833`): "two plants plus nanos" was not what failed (B).
- The sonar-by-3:00 rule (`player.md:1005-1052 part 4`) stood while game 12 ordered the sonar at 12:40 (`1034-1041`) (B).
- The 2026-10-18 rewrite of `player.md:946-948` ((h)) points at the sprint bullet and no longer records what
  thebluegecko said on 2026-09-27 (B).
- Quicksilver's "twenty energy for every metal" (`experience/players-words.md:13-15 part 3`) beside "8 to 14 energy
  per metal" (`units.md:62-63`): a wind-map rule of thumb against a vehicle's cost, different quantities (D).
- `player.md:1183-1187` (B): "commit at 5:1" is a BARb-curve ratio the 2026-10-03 pressure ruling
  (`player.md:36-45`) rejects.
- `experience/cases.md:92-94` against `player.md:56-91`: "within two minutes" (C: situational) against the pool's
  3.4-minute median.

## (c) The duplicate clusters (one subject stated in several files)

Locations as the readers gave them; the uncommitted `docs/briefs/player-experience.md` concatenates the experience
files and repeats every cluster below again (B: its 1241-1320, 1478-1480, 1553-1563, 1653-1655, 1905-1917); it was
not annotated.

1. **The tier-2 sprint** (nine files): `player.md:92-145` (the bullet), `816-833 part 2`, `865-924 part 3`,
   `925-983 part 5 (h)`, `1080-1094`; `experience/winning.md:25-39`; `experience/cases.md:115-131`, `133-145`,
   `147-173`, `175-179`, `181-205`; `experience/late-game.md:1-7`, `18-24`; `experience/team-games.md:51-70`;
   `experience/players-words.md:49-57`; `experience/units.md:67-71`, `93-96`; `strategist/commander.md:17-42`;
   `strategist/player-under-command.md:31-39`; `strategist/player.md:35 part 6`; `docs/knowledge/tier2.md`.
2. **The bank and build power**: `player.md:247-254`, `374-385`, `543-547`, `723-739 part 2`, `751-764 part 2`,
   `816-833`, `834-847 part 2`; `experience/winning.md:72-73`; `experience/cases.md:280-297`;
   `experience/players-words.md:17-22`; `experience/late-game.md:9-16`; `strategist/player.md:429-430`;
   `strategist/commander.md:121-129`.
3. **The first handful / early pressure**: `player.md:30-55`, `925-983 (d)`; `experience/cases.md:15-38`;
   `experience/winning.md:49-51`; `experience/players-words.md:8-11`, `24-27`; `strategist/player.md:433-436`.
4. **The raid answer and the pickets**: `player.md:30-55 part 3`, `234-246`, `332-333`, `334-343`, `658-679 part 4`,
   `740-750`, `765-777 part 3`, `834-847 part 2`, `865-924`, `1131-1139`, `1164-1171`; `experience/cases.md:256-279`;
   `experience/winning.md:67-70`; `experience/opponent-barb.md:6-10`; `experience/hands.md:23-27`;
   `strategist/player.md:305-313`, `361-369`, `370-373`, `378-380`, `431-432`, `433-436`;
   `strategist/commander.md:130-137`; `strategist/packet-example.md:20-22`, `38`, `42`; `player-opus.md:22-29`;
   `pianist/rules.md:1 parts 3, 5, 10`; `pianist/rules_gate.md:1`; `pianist/default.md:3`, `5`.
5. **The odds, the turret at three times its metal, arriving together**: `player.md:282-292`, `293-301`, `350-354`,
   `925-983 part 7`; `experience/winning.md:60-62`; `experience/cases.md:324-354`; `experience/units.md:34-43 part 3`;
   `strategist/player.md:380-382`; `pianist/rules.md:1 part 4`; `pianist/rules_gate.md:1`.
6. **The Comet opening line**: `player.md:255-281`, `581-657`, `658-679`, `707-722`, `723-739`, `925-983 (a)` and
   `part 9`; `experience/cases.md:299-306`; `reference/comet-catcher-remake-1.8-arm.md:8-31`, `56-84` and `-cor.md`
   the same; `strategist/player.md:25-32`, `35`, `437-439`; `strategist/packet-example.md:30`, `54`;
   `pianist/default.md:2`; `docs/knowledge/openings.md`.
7. **The pool's extractor curve**: `player.md:581-657 part 2 (614)`, `658-679`; `experience/cases.md:256-259`,
   `280-284`, `388`; `reference/*-arm.md:86-136`, `*-cor.md:86-139`; `strategist/player.md:278-284`, `425-428`.
8. **Constructors by the clock**: `player.md:247-254`, `543-547`, `816-833`; `experience/cases.md:280-284`;
   `strategist/player.md:65-73`, `291-304 part 2`; `strategist/commander.md:110-120`; `pianist/default.md:4`.
9. **The commander**: `player.md:312-321`, `493-500`, `581-657 parts 1 and 5`, `723-739 part 4`, `1105-1106`;
   `experience/cases.md:390-400`; `experience/players-words.md:12`; `strategist/player.md:76-83`;
   `pianist/rules.md:1 part 12`; `pianist/rules_gate.md:1`; `pianist/default.md:2`.
10. **thebluegecko's words**: `player.md:215-233`, `925-983`; `experience/cases.md:96-97`, `240-242`, `252-254`, `297`,
    `322`, `352-353`; `experience/late-game.md:9-16 part 2`; `experience/players-words.md` whole;
    `experience/other-maps.md:78-79`; `experience/team-games.md:86`; `docs/knowledge/_inbox/players-chat-2026-09-27.md`,
    `-28.md`. Inside this: hovers for harass (`player.md:1005-1052 part 5`, `players-words.md:39-41`,
    `other-maps.md:78-79`), u6bkep's "3v1 is about fair" (`player.md:925-983 part 6`, `players-words.md:21`,
    `team-games.md:86`), the Lashers/Brutes exchange (`player.md:215-233`, `925-983 part 8`, `cases.md:201`, `242`,
    `players-words.md:29-34`).
11. **Quicksilver's energy words and Matt's line**: `player.md:480-530`, `1110-1121`; `experience/cases.md:308-313`;
    `experience/other-maps.md:6-37`; `experience/players-words.md:13-15`.
12. **BARb's code and clocks**: `player.md:355-372`, `723-739 part 4`, `765-777 part 3`, `797-815 part 2`;
    `experience/winning.md:21-23`; `experience/cases.md:175-179`; `experience/late-game.md:147-170 part 2`;
    `experience/opponent-barb.md` whole; `reference/comet-catcher-remake-1.8-barb-hard_aggressive.md`;
    `strategist/player.md:370-373`; `pianist/rules.md:1 part 3`.
13. **The 3v1 games against thebluegecko**: `player.md:56-91`, `234-246`, `282-292`, `293-301`, `312-321`, `865-924`,
    `925-983`; `experience/cases.md:111`, `194-205`, `239-246`, `266-269`, `331-332`, `344-347`, `382`, `395`;
    `experience/team-games.md:9-56`.
14. **Water**: Cape Violet (`player.md:984-1004`; `experience/cases.md:333`, `401-425`; `experience/other-maps.md:38-47`;
    `experience/team-games.md:45-49`; `strategist/commander.md:65-74`), SailAway (`player.md:1005-1052`;
    `experience/cases.md:204-205`, `249-250`; `experience/other-maps.md:50-68`; `experience/team-games.md:57-58`,
    `83-85`; `experience/players-words.md:35-41`), the under-water builders (`experience/cases.md:408-411`;
    `experience/units.md:116`; `player.md:992-1013`; `strategist/commander.md:68-69`).
15. **Great Divide**: `player.md:1053-1097`; `experience/cases.md:336-337`; `experience/late-game.md:103`;
    `experience/other-maps.md:83-98`; `docs/knowledge/maps/great_divide_v1.md`.
16. **Center Command 2v4**: `experience/winning.md:30-35`; `experience/cases.md:133-173`;
    `experience/late-game.md:172-188`; `experience/team-games.md:59-70`; `experience/units.md:67-71`, `93-96`;
    `strategist/commander.md:17-55`; `strategist/player.md:35 part 6`; `strategist/player-under-command.md:31-39`.
17. **The nuke and the anti-nuke's numbers**: `player.md:168-182`; `experience/late-game.md:43-50`, `147-188`
    (and 111-136 retold at 147-170 part 4); `reference/frontier*.md:16`; `strategist/player.md:210-218`;
    `strategist/commander.md:43-55`.
18. **The moho and advanced solar arithmetic**: `player.md:157-214`; `experience/cases.md:207-232`;
    `experience/late-game.md:18-38`, `52-77`; `experience/units.md:73-96`, `123-129`;
    `experience/players-words.md:52-57`; `strategist/commander.md:30-38`; `docs/knowledge/tier2.md`. The sentence
    "advanced solars started at 18:22 into an empty store" stands at `experience/cases.md:229`, `player.md:196` and
    `player.md:831` (C).
19. **Light posts, splash and body size**: `player.md:30-55 part 4`, `56-91 part 3`, `555-569`;
    `experience/cases.md:40-62`, `99-107`; `experience/units.md:45-59`; `docs/studies/2026-09-28-body-sizes.md`.
20. **Energy per metal and "a factory draws about 80"**: `player.md:486-492`, `548-552`, `740-750 part 3`;
    `experience/cases.md:304-306`; `experience/units.md:61-65`; `experience/other-maps.md:10-12`.
21. **Wrecks and resurrection bots**: `player.md:388-398`; `experience/late-game.md:39-40`; `experience/units.md:131-135`;
    `strategist/player.md:134-144`.
22. **Air and human-9's gunships**: `player.md:344-349`, `1040`; `experience/cases.md:355-370`;
    `experience/units.md:137-139`; `experience/team-games.md:45-49`; `strategist/commander.md:143-148`.
23. **The hands' tool text**: `player.md:423-475`; `experience/hands.md` whole; `strategist/player.md:107-152`.
24. **The going doctrine**: `strategist/commander.md:56-58`, `138-174`; `strategist/packet-example.md:17-19`;
    `strategist/player-under-command.md:9-14`; `strategist/player.md:382-384`; `experience/cases.md:63-98`;
    `player.md:56-91`.
25. **Several seats and chat conventions**: `player.md:399-422`; `experience/hands.md:43-53`;
    `experience/team-games.md:88-95`; `strategist/commander.md:9`, `27-61`, `169-214`;
    `strategist/player-under-command.md`.
26. **The scenario and the map scale**: `player.md:16-29`; `experience/intro.md:13-21`.
27. **comet-4's 22 Blitzes into turrets**: `player.md:723-739`; `experience/cases.md:60-61`, `324-327`.
28. **human-10/11/12's harness lessons**: `player.md:865-924 part 4`; `experience/team-games.md:74-95`;
    `strategist/player.md:44-61`.
29. **The frontier**: `reference/frontier*.md`; `experience/late-game.md:138-146`; `experience/cases.md:115-131 part 3`.
30. **Radar by 3:00 / scouting**: `player.md:302-311`, `680-697 part 2`, `751-764 part 3`, `454-475`;
    `experience/cases.md:372-388`; `experience/winning.md:64-65`.
31. **The unit tables**: `player.md:322-327`, `570-580`; `experience/units.md:5-28`.

## (d) Where two readers classified the same subject differently

Joined by the readers' own duplicate and conflict links to the other reader's row: a link resolved to the row whose line
range contains the cited line, kept when the two rows' fates differ and their subjects share three content words or a
quarter of their words. The join found 54 such pairs; most are a tool or episode row linked to a fact row on a
neighbouring subject and are not disagreements. The ones that are:

| subject | one reader | the other | note |
|---|---|---|---|
| BARb's tier-2 clock | A `player.md:369-372` fact; D `experience/opponent-barb.md:17` fact | B `player.md:797-815 part 2` meta; C `experience/cases.md:175-179` meta, `winning.md:20-23 part 2` discard; D `reference/...barb-hard_aggressive.md:1-7` meta | a measured clock of one profile: fact of BARb's code or meta of its behaviour |
| the pickets rule | A `player.md:334-343 part 2` rederive (layer ruling) | E `strategist/player.md:370-373` meta | the same 2026-09-22 observation of BARb |
| constructors by the clock | A `player.md:543-547` rederive | E `strategist/player.md:65-73` meta; `commander.md:110-120` meta | A's row carries the unattributed count, E's the sheet |
| the anti-nuke's timing | A `player.md:168-182` rederive; C `experience/late-game.md:172-188` rederive | E `strategist/commander.md:43-55` meta | C and E read the same 2026-10-20 ruling |
| the commander assists the lab at birth | C `experience/cases.md:322` meta | D `experience/players-words.md:12 part 2` rederive | thebluegecko's one line |
| the user on mohos and advanced solars | C `experience/cases.md:231-232` meta | A `player.md:183-200` rederive; D `experience/players-words.md:52-57 part 2` rederive | C keeps the user's word as meta, A and D as a derivation to run |
| BARb's raid squad of four | A `player.md:356-359` fact; D `experience/opponent-barb.md:6-8` fact | E `strategist/player.md:370-373` meta, `pianist/rules.md:1 part 3` meta | D's row adds that "four" is medium's figure |
| the light post | A `player.md:30-55 part 4` rederive | C `experience/cases.md:40-53` fact | A took the rule, C the simulator table |
| the frontier | D `reference/frontier*.md` rederive (12 rows, "keep as is") | C `experience/late-game.md:138-146`, `cases.md:115-131 part 3` fact | the same numbers |
| the head-on matchups | A `player.md:328-331` fact | D `experience/units.md:34-43` rederive | the simulator's ranking |
| comet-3's solars starve the opening | B `player.md:707-722` rederive | C `experience/cases.md:314-315` episode | |
| an air plant means gunships soon | A `player.md:344-349` fact | D `experience/units.md:137-139` episode; C `experience/cases.md:368-370` rederive | |
| the under-water direction | B `player.md:992-1001 part 2` rederive; C `experience/cases.md:427-431` rederive | E `strategist/commander.md:65-74` tool | |
| feeding is production | C `experience/cases.md:133-145 part 2` episode | E `strategist/commander.md:22-30 part 2` rederive | the same Center Command measurement |
| the pool's 3.4-minute close | C `experience/winning.md:20-23` meta | A `player.md:56-91` rederive | A folds the number into the rule |
| the opening's bound is metal | E `strategist/player.md:35 part 3` rederive | A `player.md:255-281` episode (the cases), `581-657 part 5` rederive | |
| the sprint's plant beside the home turrets | B `player.md:816-833 part 2` rederive | C `experience/cases.md:199` episode (the sprint botched) | |
| advanced solars into an empty store | C `experience/cases.md:229` discard | B `player.md:816-833 part 4` fact | the same sentence, three places |

A calibration difference rather than a per-passage one: `doctrine` was flagged on 24 of C's 187 rows and 15 of D's 152
against 66 of B's 104 and 60 of E's 163; `lens: unknown` on 99 of E's rows, 53 of C's, 51 of D's, 2-3 of A's and B's.
The totals by reader in (a) should be read with that in mind.

## (e) The 24 discard candidates

| # | passage | was | subject | reader's reason |
|---|---|---|---|---|
| 1 | player.md:5-9 | tool | the brief's provenance paragraph (a hundred medium games, seven hands-alone games on easy) | Provenance of the 2026-09-21 body: medium and easy on Quicksilver, the heuristic bot's games. The opponents, map and hands have all changed since; the one sentence to keep ('evidence, not orders: say in a note when the game contradicts it') is role-prompt text. |
| 2 | player.md:364-368 part 2 | meta | BARb medium's army curve on Quicksilver and easy's column timing in the heuristic bot's games | Medium on Quicksilver against the heuristic bot (measured 2026-09-19, 'a prior for a player's estimate'): an opponent and map no longer played. The hard_aggressive sheet replaces it. |
| 3 | player.md:373-373 | tool | the economy heading with three 2026-09-19 claims | A heading. Its claims are 'observed once', 'conjectured' and 'conjectured' from the heuristic bot's Quicksilver games. |
| 4 | player.md:446-451 | tool | the mapping from the old hands' option names to the new menu's moves | A compatibility glossary for the brief's own stale vocabulary (send_against, raid, sweep, pull_out). Once the pre-rebuild lessons are rewritten or marked as episodes it has nothing to translate. |
| 5 | player.md:723-739 | episode | comet-4's commander list and what it gave (the superseded opening) | The passage announces its own supersession; the numbers (plant 0:41, thirteen Blitzes by 4:07, nineteen extractors at minute 10) stay in the knowledge claim as the measured result of that list against medium. |
| 6 | player.md:778-796 part 4 | tool | three hands mechanics of the time: the old walk kept, 'first' behind the group, artillery splitting off; footwork off for a commit | Tool notes against a harness that no longer exists in that form; discard unless the current prompt still says them (reader E's files). |
| 7 | player.md:848-864 part 3 | tool | escalate-7's open hands items: the walk kept after abandon, lists mid-walk not taken, the engine queue kept, the shelling place under a beamer, plant naming; Stouts when the attack reaches structures | Tool notes against the hands of 2026-09-22; the one strategy line (Stouts when the attack reaches structures, at 12:45 not 14:48) duplicates comet-3 part 4 and comet-4 part 3. |
| 8 | player.md:1105-1106 | episode | the commander's far walks in the smoke games | A lesson against a 2026-09-21 defect; the brief's own opening now sends the commander out. |
| 9 | experience/intro.md:7-12 | tool | the brief's stance: no rules of the form always/never | Describes the current brief's construction (one arguing part, the rest cases and tables). The rework replaces the construction, so the description goes with it; the stance itself (judgement is the reader's) is a preface sentence, not a passage. |
| 10 | experience/winning.md:1-5 | tool | preface to the arguing part | Describes the part's status under the current construction; the rework makes every derived passage argue, so the preface goes. |
| 11 | experience/winning.md:20-23 part 2 | meta | BARb's army curve around its tier 2 | A number from one opponent profile (not named here; cases.md:175 says BARb on Comet, profile unstated) presented in the winning argument as 'this opponent'. Era-bound; keep only in the opponent sheet with its profile and date. Source of the 10:40 and the doubling not found in docs/knowledge by search (unknown). |
| 12 | experience/cases.md:229-229 | episode | advanced solars into an empty store: an unnamed game | No game named, no date; the same sentence stands in player.md:196 and :831. The mechanism (a frame draws its energy as it builds) is a fact already in lines 218-220. |
| 13 | experience/cases.md:324-327 | episode | a mass into a turret line: comet-4 (again) | The same case as lines 60-61, in a second section. |
| 14 | experience/cases.md:399-399 | episode | the commander walking to far spots: the default-packet games | No game, date or packet named; the walk to far spots is a hands behaviour since changed (the fall-back walks removed 2026-10-01, the places layer 2026-10-02). |
| 15 | experience/late-game.md:147-170 part 4 | episode | the two nuke games retold | Repeats lines 111-136 in summary; the lessons (a commander walks out of a blast; remembered buildings may be dead; the field is the target after his anti-nuke) belong with the episodes. |
| 16 | experience/other-maps.md:78-79 | meta | thebluegecko on hovers (again) | The same quote as players-words.md:39-41 and the ledger's K-units-hovers-are-for-harass-not-the-army; a duplicate here. |
| 17 | experience/other-maps.md:95-96 | episode | our Great Divide games: the old bot's team games; the people games unread | The old heuristic bot's 2v2 games against BARb medium (team-2v2-*, commander-team-1, 2026-09-20); the ledger rows run 20/30/35-minute caps and commander-team-1 held 10 extractors at the cap, so 'five extractors for 35 minutes' does not match. The second sentence is stale: bluegecko-2v1-great-divide has a ledger row (lost 15:28, read). A different bot in a different regime: nothing to keep. |
| 18 | experience/team-games.md:23-23 | episode | human-15 unread | A placeholder; no ledger row for human-15 exists. Nothing to keep. |
| 19 | experience/team-games.md:72-72 | tool | pointer to the players' words | A cross-reference line. |
| 20 | experience/team-games.md:86-86 | meta | u6bkep's line on mex speed (game 7) | The same quote as players-words.md:21; a duplicate. |
| 21 | brain/pianist/default.md:2-2 | derived | the default commander opening: two extractors, three winds, the lab, winds until banking, assist; a generator when low, a second lab when banking; walk home when under fire | A Quicksilver-era wind opening (the file is of 2026-09-21, last touched 2026-10-02) that contradicts the pool's Comet opening (two solars then the plant at 0:34), the solar rule, 'the plant's metal is the limit', and the no-fall-back-home ruling. Live only in pianist-alone games. |
| 22 | brain/pianist/default.md:3-3 | derived | the default constructors: nearest spots outward; a turret beside an extractor that has been raided; a radar at six extractors; generators when low; another lab when banking; wrecks when short | The reactive turret rule ('beside an extractor that has been raided') is what player.md:305-313 argues against (the turret is part of taking the spot). Pre-rebuild doctrine kept alive in the fallback packet. |
| 23 | brain/pianist/default.md:4-4 | derived | the default lab: one constructor first, raiders until a group, a second constructor only then, line to raiders two to one | The exact cadence the role texts name as the failure (human-7: three constructors; human-22: no constructor from the first minute to 4:30) and that commander.md forbids ('Never write a factory's order ... as an opening'). The two-to-one mix has no source. |
| 24 | brain/pianist/default.md:5-5 | derived | the default army: one scout at minute three; a group of four advances to passage_1 and holds; a group at home protects only home; detachments for a lone raider; no chase across the map; attack his factory when a real army outweighs it; an outweighed group walks home | Pre-rebuild posture: a place in our half to stand (passage_1) and a count (four) against the going doctrine; one scout at minute three against the first handful by 2:00; 'walks (go) home' against the no-fall-back-home ruling. One sentence ('a group holding at home protects nothing but home') survives in player.md. |

## (f) Every unclassifiable item from the five reports

From reader A:
- "The person's count" behind four constructors by 4:00 and eight by 8:00 (`player.md:543`): unattributed.
- Whether `pianist/rules.md` still carries the detachment sentence "word for word" (`player.md:334`): its first lines
  name the raids, not the detachment; reader E's file.
- Whether the hands still hold no spacing (`player.md:565-566`): true when written (2026-09-27), unknown after the
  rebuild.
- The scout sentence "that held under the enemy commander's eyes" (`player.md:442`): the game is not named
  (also D's `experience/hands.md:28-29`).
- human-26 and human-27 (`player.md:463`, `473`): cited as evidence, no ledger rows.

From reader B:
- Whether the hands mechanics named in `player.md:778-796 part 4`, `848-864 part 3` and `1107-1109` survive the
  2026-10-02 rebuild: `send_against`, `attack_unit`, `fight_to` with the underscore, `start_boxes_note`, `reached_by`
  have no hits in `strategist/player.md`; reader E's files decide.
- The source line of "moving the commander around costs build power" (`player.md:1110-1121 part 2`): in
  `experience/other-maps.md:31` and not in the inbox chat files read.
- The lens of the Great Divide plan (`player.md:1080-1094`): the user's word relayed from experienced players per the
  knowledge claims, rewritten 2026-10-18 by the session; `unknown` where no person or game is named.

From reader C:
- `experience/cases.md:22-23` ("what the first soldiers met there"): population unknown.
- `experience/cases.md:229` (advanced solars started at 18:22 into an empty store) and `399` (the hands' default-packet
  games): no game or date; proposed discard.
- The source of BARb's 10:41 / 73 of 74 / army doubling (`experience/winning.md:21-23`, `cases.md:175-179`): not found
  in `docs/knowledge` by search; grounding unknown.
- "A turret counts about three times its metal" (`experience/winning.md:61-62`): a harness constant whose origin is
  unknown.
- `experience/cases.md:336-337`'s "the players expect a camp at the pass" and "1,700 wide at the median": no person or
  source found.

From reader D:
- `experience/hands.md:28-29`, the scout's sentence "that held under the enemy commander's eyes": which game is not
  said; lens unknown.
- `experience/hands.md:40-41`, "player-9" and "player-10" (three scouts, none arrived; three roving Rovers at his lab at
  3:45): the ledger has pianist-player-9/10, player-10-routes and onepass-player-9 and none carries these lines.
- `experience/opponent-barb.md:15-16`, "in one easy game it walked into our half with a party": the game is not named.
- `experience/other-maps.md:47-48`, human-16's Incisors firing in 2% of their in-reach seconds: no ledger row; whether
  the cause is the weapon or the hands is not said.
- `experience/players-words.md:63-68`, the user's "home guard" (escalate-4) and "light encampments" (player-50)
  readings: the ledger rows for those games do not carry them; the lens is the user's word with no row to check against.
- `experience/other-maps.md:93`, "an experienced player's word: an all-air side loses": no source found.

From reader E:
- The "our games" curve in `strategist/player.md:278-284` (5 / 12 / 16-17) and "the classic failure" (`318-325`):
  which games and which era they summarise is not in the text or the ledger rows read; marked undated and
  pre-rebuild on the shape.
- "A never sentence holds" (`strategist/player.md:153-177`): the role text and K-jev-never-is-not-heard disagree and
  the texts do not say which describes today's Jev (the forbidden mark of 2026-09-30 sits between them).
- The author of the Cape Violet 1v1 review (`strategist/commander.md:65-74`) and of the three unnamed games behind
  "a ball's worth" (`strategist/player.md:263-269`): lens unknown.
- The source of the 3x turret constant, the 150-metal solar threshold, "two to one" (`strategist/player.md:431-432`,
  `pianist/default.md:4`), "four to six constructors" (`strategist/player.md:291-304`): no text, claim or ledger row
  names one.
- human-26 has no ledger row; its facts in `player-opus.md` rest on `docs/studies/2026-10-07-raid-packets.md` (the
  user's review and an offline replay) and nothing else.

## (g) The readers' surprises that matter for the rework (deduplicated)

1. **A reversed ruling lives on in the main brief.** The anti-nuke timing was reversed on 2026-10-20 in
   `experience/late-game.md` and `strategist/commander.md`; `player.md:168-182` still argues the retired version (A, C, E).
   The same shape elsewhere: lines rewritten in place after the fact (`player.md:946-948`, `1080-1084`, `988-989`,
   `1089`, `898-916`, `691-696`, by blame), so a 2026-09-27 quotation no longer records what was said (B).
2. **The Great Divide section's premise was refuted by its only game with a person**, and the section does not mention
   the game (B, D); the pass is 644 wide in two places and 700-840 in a third (B, C, D).
3. **Three pools, three extractor curves** with different numbers at 4/6/8/12, and `player.md:614`'s middle number is
   nobody's median (B, E).
4. **The role texts carry doctrine**: 57 of E's 163 rows; `pianist/default.md` is a complete pre-rebuild doctrine that
   is live in pianist-alone games and in any second before a packet lands, and contradicts the texts above it (E).
5. **The hands' 600 rule overrides the packet prohibitions the example packets teach** ("NEVER answered by Blitzes or
   Stouts"); the mechanism under player-57's thrash is written into the shape players copy (E).
6. **"A never sentence holds"** in the role text against K-jev-never-is-not-heard (E).
7. **The role prompt misreads the sheet**: `commander.md`'s "about five by 5:00" (the sheet: 3, upper quartile 4); the
   pool's winners bank more than the losers at 8:00-9:00, against "never that you are ahead" (E).
8. **cases.md is four fifths episodes**, nearly all 2026-09-22 to 10-04 under the pre-rebuild or early-layer hands;
   `winning.md` is the one arguing part and almost entirely rederivable; `late-game.md` is the best-grounded file, every
   quoted cost reproducing from units.csv (C). The reference tables are the one section already in the rework's shape
   (D).
9. **The game-name key is wrong**: `cases.md:10-13` puts human-N games on Quicksilver; human-10, -18, -23 were not (C).
10. **Numbers with no source**: BARb's 10:41 / 73 of 74 / doubling (C); the 3x turret constant (C, D, E); the
    150-metal solar threshold, "two to one", "four to six constructors" (E); "the person's count" (A).
11. **Lessons against the ledger**: a roving Rover "killed nothing" (the ledger: about 420) (D); "nothing below was
    measured in a game of ours" beside leg-player-1 (D); two "several seats" lessons from 1v1s (D); 3v1 trades quoted
    before the double-counting correction (D); the old bot's Great Divide games misdescribed (D).
12. **opponent-barb.md carries medium's constants** (a squad of four, a commander at home) against the hard_aggressive
    we play (D); `pianist/rules.md` states BARb's raid clock to the hands as every opponent's nature (E).
13. **Lessons applied where their premise did not hold**: the chat-lead rule (f) sent 6,800 of Blitzes into the B3
    camp (B); the rematch plan kept beside its refutation (B); rules in the brief and not followed, which the brief's own
    (q) item observes (B).
14. **Pre-rebuild lessons rest on defects the ledger names beside most of them** (A); the lesson and the defect of the
    day are written in one sentence through 290 lines of game accounts (B).
15. **Unit names differ from units.csv's throughout** (Pillager, Punisher, Rascal/Rover/Tumbleweed, Bulldog, Instigator,
    Leveler, Slasher, Coyote, Reapers, Tremor, Phoenix, Mason) (A, B, D).
16. **A lens mismatch**: human-23's second game cited for the going doctrine while the person-ranked reading is
    defence (E).
17. **Games cited without a ledger row**: human-15, human-16, human-26, human-27, the Azurite Shores and Cape Violet
    games of 2026-10-18 (A, D, E; the ledger check in `backlog.md` (c)).
18. **The anti-nuke window measured on Comet is applied to Great Divide and Quicksilver** without a measurement there (D).
19. **The pool's tier-2 rarity (1 of 41 Armada, 3 of 39 Cortex) stands beside the sprint as the way to win** without
    reconciliation (B, D).
