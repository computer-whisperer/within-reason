# Part b: `docs/briefs/player.md`, passages beginning at lines 601-1204

Reader B, 2026-10-21. Rows in `part-b.jsonl`: 104 rows over 44 passages, covering lines 658-1204 without a gap.
Lines 601-657 are the tail of the Comet opening bullet that begins at line 581 and belong to reader A; that bullet
carries two bold sub-leads inside it ("Other openings seen on this map, as snippets" at 606 and "The limit in the
first four minutes is metal, not build power" at 643) that reader A should split into parts. The first passage of
this part is the replay-survey bullet at 658.

Facts about games come from the passages, `docs/experiments.md` (rows read in full for every game named) and
`docs/knowledge/`; game dates from the match directories' epochs. Unit numbers were checked against
`docs/glossary/units.csv` by one lookup each.

## Counts

By layer (what the passage is now): mixed 32, episode 22, derived 18, meta 14, tool 11, fact 6, ruling 1.

By fate (what it would become): episode 31, rederive 29, meta 18, tool 13, fact 9, discard 4.

Doctrine: rule-from-one-case 49, none 38, rule-from-ruling 15, rule-from-pool 2. Lens: ledger 76, chat 10, person 9,
model 7, unknown 2. Of the 29 rederive rows, 22 look derivable in full and 7 in part.

The shape of this half of the file: 290 lines of game-by-game accounts (comet-1..5, escalate-1..7, four 3v1 games,
the pianist-player series) in which the lesson and the harness defect of the day are written in one sentence; 60
lines of players' words (the best-sourced meta in the file); three map sections (Cape Violet, SailAway, Great
Divide) each resting on one or two games and one person; and a Great Divide plan built on a conjecture that the one
game with a person there refuted.

## The derivation backlog

Subjects a derivation would own, each with the lessons it must reproduce or explain away (row refs are line
range and part).

1. **Energy in the first six minutes** (the plant's queue against solars and the store). Premises: solar 155 metal /
   20 a second, commander 30, the start store, Blitz 900 energy per 2,000 build time, Stout 2,100, constructor
   vehicle 1,950 per 4,050, the plant's 150 build power plus the commander's 300. Must reproduce: six solars starve
   the opening (707-722/1); energy at zero at 2:00 on two solars, three plus three by 2:00 (765-777/1); a solar per
   Blitz queued (778-796/1); eight solars per running plant (740-750/3); 200 energy a second by 6:00 with a store and
   no converters (925-983/1); the assisted plant's 220 a second on a constructor against 90 (925-983/9); advanced
   solars cost energy to build (816-833/4); three windmills under three constructors stall (1110-1121/3). Must decide:
   two solars (comet-3) or three (the players, reader A's opening). The file has four forms of this rule.
2. **Build power against income** (why a bank forms and what spends it). Premises: constructor 135 metal / 90 build
   power, nano 230 / 200, plant 590 / 150 (25 metal a second assisted), the pool's five constructors by 5:00 and three
   nanos by 6:37. Must reproduce: a bank above 500 with income over 30 is build power missing (816-833/1); the
   second plant at income 30, the third at 40 (723-739/2); plants or nanos, either spends it (751-764/2); the
   three-loss signature at hard_aggressive (834-847/2); build power on a rising store (925-983/6). Must decide: plant
   versus nanos first (comet-4 against escalate-5).
3. **The reach table** (what outranges what, and the approach paid under fire). Premises: units.csv reach by type;
   the square law with reach (H-HANDS-ODDS-REACH). Must reproduce: Blitzes never at turrets, Shellshocker 710 over
   the laser turrets but not the Gauntlet (723-739/3); Bulls 460 and Beamers 490 over line, riot and raider tanks but
   not skirmisher or artillery tanks (925-983/7); the camp at the pass and fire over it (1074-1079, 1080-1094/2);
   Riptide 480 over Dolphin 280 and Ellysaw 470 (1005-1052/6); Janus beats Stouts, artillery behind a screen beats
   Janus (740-750/2); Stouts in front of the D-gun (765-777/4). Both "every tier-1 ..." claims need narrowing.
4. **Raids and what catches them** (speeds, the raid clock, where raiders must pass). Premises: Tick 132, Rover 168,
   Blitz 101, Pawn 87, Stout 75, Mace 46; BARb's raid clock by profile; a light turret's 430 and 85 metal; the
   pool's turrets by 8:00. Must reproduce: a light turret catches a Tick, Blitzes do not (707-722/3); Maces cannot
   catch Flashes, raiders on the passages (1164-1171); a ball of mediums never catches Incisors (865-924/5); pickets
   of two per seat, the ball is not the raid answer (865-924/1); the turret at 0:27 matters more against BARb
   (658-679/4). Must decide the file's three raid answers: pickets (bluegecko-1, escalate-6), the whole home group
   (escalate-2), turrets alone (comet-5).
5. **Gathering and the square law** (why one body wins what three detachments lose). Must reproduce: gather first
   and commit everything (751-764/4); the groups arrive together or the push waits (865-924/3); one line, few groups
   (925-983/4); waves ground down piecemeal (1131-1139); cross with everything once the camp is dead (1080-1094/4).
6. **The tier-2 sprint's timing** (owned by reader A's bullet at 92-130). This part contributes: the sprint by about
   10:00 on Great Divide from saturation at eleven spots (1080-1094/1); the advanced plant by 12:00 whatever the front
   is doing (865-924/3); the frame's exposure on one constructor, 2,600 metal at 90 build power (816-833/2); the
   seat's other plants pause (925-983/6); tier 2 in 4 of 58 pool sides (658-679/3) as the counter-weight.
7. **Water** (what reaches a sea spot and what crosses water). Premises: the commander, Beaver 150, Muskrat,
   construction ship 200 from a 450 shipyard, hover constructors from a 670 platform; sonar-only sight and
   torpedo-only hits under water (engine). Must reproduce: an islet's economy is the sea's (1005-1052/2); the Cape
   Violet plan (992-1001/2), which omits that the commander takes sea spots itself; one hover platform and two
   shipyards, not both per seat (1005-1052/8); the sonar tower by 3:00 (1005-1052/4).
8. **Radar and sight** (the cost of walking blind). Must reproduce: a radar by minute three at the strip's edge
   (680-697/2), a scout car by 1:20 and a radar at the front by 4:00 (751-764/3), a scout plane before planning
   against the camp (1080-1094/3).

## The meta catalogue (observations of what people do, with population, map and date)

- The Comet pool, 29 duels at OS 25+, 2026-09-23 (658-679/1-3): two openings (the 0:34 plant, the turret opening at
  0:25-0:28 with the plant at 1:05, mostly Cortex, the later plant winning 18 of 29 mostly with the higher OS); the
  commander takes the three home spots and the next pair, constructors the flanks; the A4 north strip and the B5
  south-east; 18/23/28 extractors at 8/10/12 for winners, 7 constructors by 10:00; a second factory by 8:30; the
  winners' bank under 500 at 10:00 in 15 of 22; turrets 9 by 8:00, the first extractor lost at 4:18; tier 2 in 4 of
  58 sides, never before 12:51; a radar by 3:00, a nano by 5:42, solars only. Three pools now state the extractor
  curve (this one; reader A's 33 duels at OS 40+; the prompt's 40 duels at OS 40+) with different numbers.
- thebluegecko and Irishstud14, OS 48, Comet Catcher, games 3-5 of 2026-09-27, in chat (925-983/1-5): three solars
  for a fast constructor, the commander on the plant, no idle plant, no converters ("metal heavy map"), an energy
  store, 200 energy a second and nanos by 6:00; raids on naked extractors from 4:00 both corners; Pounders from 7:30
  against a light-tank ball, mediums with three to five Lashers; a lead spent at once as one line, reclaim behind;
  the commander sniped by circling light tanks; bomb or t2; chat discipline.
- u6bkep, OS 19, game 7 of 2026-09-27 (925-983/6): "taking mexes so much faster that 3v1 is about fair"; raids as
  whole lists of stops; build power on a rising store; the sprint seat's other plants pause. Filed under the OS-48
  heading.
- thebluegecko, game 10 of 2026-09-28 (925-983/8): the mix as a message ("adversarial ... you made many lashers, so
  I was safe to go T2"; Brutes for pressure); the commander assists the plant for its first units (925-983/9).
- thebluegecko on water, games 11-12 of 2026-09-27 and human-17 of 2026-10-03 (1005-1052/5): better than BARb at
  water; hovers for harass, not bulk; tier-1 artillery one to three behind a body.
- What thebluegecko did to three seats, Comet, 2026-09-27 (865-924/1-3): raids from 2:06, Stouts and Janus from
  7:47, one block to the nearest seat by 13:00, air from 17:00, a Bull and an air plant by 18:32, Banshees 14:00,
  Mauser 16:00, Bulls 18:30, Razorback 25:00. Irishstud14, game 4 (865-924/5): Pounders at 8:41, Incisor swarms of
  ten to fifteen from 11:00.
- The Quicksilver people, human-1/2 of 2026-09-22 (1110-1121/2): "30 pawns before mace", "about 20 energy for every
  metal", "more than one or two constructors makes defending hard"; a Pawn rush at 2:10.
- BARb by profile, Comet (797-815/2, 723-739/4, 765-777/3, 740-750/1, 680-697/4, 707-722/2): hard_aggressive's tech
  clock (Gunslinger 12:00, Fatboy and Sharpshooters 19:00, Pulsar 24:00, Razorback 25:00; two games agree); medium's
  block at about minute 10, vehicle raids in fives from 6:00, Janus at 9:00; medium opened a bot lab in three games
  and a plant in one, easy a plant; hard's fortress at E3/F3 (778-796/2). BARb on Quicksilver (1154-1163, 1188-1192,
  1199-1204): medium's twelve Flashes by minute 10 and the block at 16; hard's commander sortie at 5:45 and 4:30;
  the commander rebuilding in the far south-east corner (games 7 and 13).
- Great Divide, second-hand (1053-1056, 1074-1079): a person "will fortify the pass"; all-air loses. The one game
  with a person there (bluegecko-2v1-great-divide, 2026-09-27) did not fortify the pass and is not in the section.
  K-opp-thebluegecko records him putting three Beamers and two rocket AA at the pass's north mouth by 5:10 when he
  played the map alone against six BARb (2026-09-26, a replay).

## The episode index

| game | lines | date | era | defects the lesson depends on | lens |
|---|---|---|---|---|---|
| comet-1-easy | 680-697 | 2026-09-22 | barb-easy, pre-rebuild | plant deterred by the draw warning; prose sequences not followed (both replaced by `queue` the same day) | ledger; the user's replay remark (radar) |
| comet-2-medium | 698-706 | 2026-09-22 | barb-medium, pre-rebuild | every `produce` refused; sequences not followed; stale spot names | ledger |
| comet-3-medium | 707-722 | 2026-09-22 | barb-medium, pre-rebuild | the menu's walking-time spot order; the claim's update says the pros take those spots | ledger |
| comet-4-medium | 723-739 | 2026-09-22 | barb-medium, pre-rebuild | superseded opening; the recall at 25% was the player's | ledger |
| comet-5-medium | 740-750 | 2026-09-22 | barb-medium, pre-rebuild | `queue` took no null; five guard groups the player's split | ledger |
| escalate-1-easy | 751-764 | 2026-09-22/23 | barb-easy, pre-rebuild | factory place word dropped; cancelled list did not stop a plant; footwork under a Beamer | ledger + the player's debrief (model) |
| escalate-2-medium | 765-777 | 2026-09-22/23 | barb-medium, pre-rebuild | the home guard in the plant's exit lane; detachments were `send_against` | ledger + debrief (model) |
| escalate-3-hard | 778-796 | 2026-09-22/23 | barb-hard, pre-rebuild | nano asked of the commander gave a plant; old walk kept; "first" behind sent home; artillery split off | ledger + debrief (model) |
| escalate-4-hard-aggressive | 797-815 | 2026-09-22/23 | barb-hard_aggressive, pre-rebuild | two plants' sites chosen while orders walked; commander off the menu by an old list | ledger + debrief (model); K-barb-hard-aggressive-tech-clock |
| escalate-5-hard-aggressive | 816-833 | 2026-09-22/23 | as above | lists restarting at a turret step (not a fault, the ledger says) | ledger + debrief (model); the pickets rule the user's |
| escalate-6-hard-aggressive | 834-847 | 2026-09-22/23 | as above | constructors stuck at the yard; a solar in an exit lane; a list run on from a 1-3% frame; `send_against` split; `armcv:N` read as N more | ledger + debrief (model) |
| escalate-7-hard-aggressive | 848-864 | 2026-09-22/23 | as above | the two produce fixes it measured; five open hands items | ledger + debrief (model) |
| bluegecko-3v1-comet-catcher | 865-879 | 2026-09-27 | people, pre-rebuild | standing rules stored on the wrong seat; paragraphs not reaching tagged groups | model review (00/review.md) + ledger |
| bluegecko-3v1-comet-catcher-2 | 880-890 | 2026-09-27 | people, pre-rebuild | the retreat walked a builder into the party; a queued build blocked a retreat | the player's lessons (model) + ledger |
| bluegecko-3v1-comet-catcher-3 | 890-905 | 2026-09-27 | people, pre-rebuild | the opening refused (one seat's roster); stale party names; `remove` on one seat; exit lanes | ledger + chat |
| human-10/11/12 (filed under the 3v1 bullet) | 905-916 | 2026-10-03 | people, rebuilt hands | internal names before the factions; listless constructors to the lab; the attack sentence with a fall-back clause (measured offline) | ledger |
| bluegecko-3v1-comet-catcher-4 | 917-924 | 2026-09-27 | people, pre-rebuild | first turn with two seats; `[gecko]` read as a section; chat cut at 127 | chat (Irishstud14, thebluegecko spectating) + ledger |
| bluegecko-3v1-comet-catcher-9, -10 | 925-983 (o)-(q) | 2026-09-27/28 | people, pre-rebuild | odds words without reach (H-HANDS-ODDS-REACH after game 9); the lobby's boxes not honoured, every rover goal a row 1-2 spot (game 10) | the user (game 9); chat + Opus review (game 10) |
| player-12-post-human | 975-983 | 2026-09-27 | barb-hard_aggressive, pre-rebuild | the timed assist's first game; the eco line without STALLING; lost 26:28, root opening_starved, which the passage omits | Opus review, verified |
| bluegecko-3v1-cape-violet | 984-1004 | 2026-09-27 | people, pre-rebuild | the picture listed walkable spots only (every Beaver step "not free"); a water-mark site snapped to the builder's ground; no tidal generator on the menu | ledger; K-map-cape-violet (observed) |
| bluegecko-3v1-comet-catcher-11 (SailAway) | 1005-1034 | 2026-09-27 (the passage says 09-28) | people, pre-rebuild | no tidal on the menu; hover constructors uncounted; the snap bug | Opus review + chat (the host, thebluegecko) |
| bluegecko-3v1-comet-catcher-12 (SailAway) | 1034-1052 | 2026-09-27 | people, pre-rebuild | the sonar sentence repeated 2,900 times; islet words; the 4:09 produce list the player's | Opus review + the user's observation |
| Great Divide team games (team-2v2-smoke, -board, commander-team-1) | 1068-1073 | 2026-09-20 | barb-medium, the old heuristic bot | the Sonnet commander never opened `map`; no truth files | ledger |
| pianist-smoke-1..6, easy-1, audit-1 | 1098-1109 | 2026-09-21 | barb-easy, pre-rebuild | the default packet; one scout at a time; the `where` question's fallback | ledger |
| human-1, human-2 | 1110-1121 | 2026-09-22 | people (Quicksilver), pre-rebuild | the lab anchored on a spot; pairs sent against eight Pawns; no builder attack on the menu | ledger + chat |
| pianist-player-1..13 | 1123-1204 | 2026-09-21/22 | barb-easy/medium/hard/hard_aggressive (Quicksilver), pre-rebuild | H-HANDS-STARTED; `fight_to` not committed to turrets; the base guess pulled to the middle; named spots not places; unwalkable marks; the 90 s give-up; H-HANDS-SHELLED; the first-turn hang | ledger (the user watched pp-2) |

## Numbers checked against `docs/glossary/units.csv`

Match: Tick 21 metal / 132 speed, Blitz 101 / 900 energy / 110 metal, Rover 168 / 31, Pawn 87, Mace 46, Stout 350
/ 2,100 energy / 225, Janus 380, Pounder a riot tank (315, speed 40), Incisor 85 against Brute 72, Shellshocker
710, Pillager `armmerl` 1,300, Wolverine 710, Tremor 1,470, Mauser 820, Bull 460, Fatboy 700, light turret 430,
Beamer 490, heavy turret 620, Pit Bull 730, Gauntlet 1,220, Punisher `corpun` 1,245, Pulsar 1,400, Razorback tier 3
/ 3,800, flak 850, Ferret 950, solar 155 (six = 930), advanced solar 350 / 5,000 energy, plant 590 / 150 build
power, nano 230 / 200, constructor vehicle 135 / 90 / 1,950 energy, armavp 2,600, moho 620, radar tower 60 metal,
tidal 90 / 400, Riptide 480, Dolphin 280, Supporter 280, Coral 630, Longbow 1,550, Liche 5,625 against the
commander's 3,700 (one bomb kills), shipyard 450, construction ship 200, Beaver 150, hover platform 670, Anemone a
torpedo launcher, Nettle anti-air, naval sonar tower buildable by the commander, the commander cannot build a nano
turret (its `made by` list), the assisted plant's 25 metal a second on Blitzes and about 217 energy a second on a
constructor, 8 to 14 energy per metal for the plant's units.

Mismatches (ten rows): the radar "sees 2,000" (units.csv 2,100); the Ellysaw 500 (470; other-maps.md:57 repeats
500); "Shellshockers outrange every tier-1 turret" (the Gauntlet is tier 1 at 1,220); "Bulls and Beamers outrange
every tier-1 tank" (Lasher 575, Whistler 550, Shellshocker and Wolverine 710); Welders as "amphibious tanks" (a
tier-2 assault bot that wades, per units.csv and the knowledge claim); "Reapers" as the Bull's twin (Tiger
`correap`; Grim Reaper is a resurrection submarine) and "Tremor" as the Mauser's (Quaker `cormart` 800); Lashers
grouped with Janus as riots (a skirmisher tank); "Phoenix" bombers from the tier-1 air plant (armpnix Blizzard is
tier 2; units.csv's Phoenix is Legion's); Pillager and Punisher under names units.csv does not use (Ambassador,
Agitator); Cape Violet's 28 spots under water (26 plus two islands in the knowledge claim, 26 in cases.md:403); the
Great Divide pass 644 wide here and in the knowledge file, 700 to 840 in other-maps.md:85. Consistent only with a
reading: "three solars' 90" counts the commander's 30. Old names throughout the pianist-player section (Flash,
Stumpy, Janus rocket trucks) and "Mason" for the construction vehicle.

## Conflicts

Within this part and with the files grepped (reader A's half by subject; reader C/D/E's by file and line):

1. **The Great Divide premise against its one game.** 1053-1056 and 1074-1079 expect a person to fortify the pass;
   `docs/experiments.md` bluegecko-2v1-great-divide (2026-09-27, lost 15:28) says "the person did not fortify the
   pass": his commander came through at 4:26 and camped in our half. The section does not mention the game; the
   knowledge claim is marked conjectured.
2. **Four energy rules**: two solars before the plant (707-722/1) against three (925-983/1, reader A's opening);
   eight solars per plant (740-750/3), a solar per Blitz queued (778-796/1), 200 a second by 6:00 (925-983/1).
3. **The bank**: the second plant at income 30 (723-739/2) against nanos and constructors before any second plant
   (816-833/1), with escalate-1's "either spends it" (751-764/2) between them.
4. **Three raid answers**: pickets of two per seat (865-924/1, 834-847/2), the whole home group not detachments
   (765-777/3), turrets alone with one block (740-750/1); and "Blitzes never go at a commander" (865-924/2) against
   "sniped by light tanks circling it" (925-983/5).
5. **Three extractor curves** for Comet: 29 duels OS 25+ (658-679), 33 duels OS 40+ (reader A, 581-657), 40 duels
   OS 40+ (`crates/bot/src/strategist/player.md:276-285`), with different numbers at 4/6/8/12.
6. **The sprint's clock**: "by 12:00 whatever the front is doing" (865-924/3) and "by about 10:00 here" (1080-1094/1)
   against reader A's sprint bullet (92-130: at the first bank or when the spots are taken, not by the clock); and
   the pool's tier 2 in 4 of 58 sides (658-679/3) against the 2026-10-18 ruling that tier 2 is a sprint.
7. **The middle spots**: comet-3's "strip first" (707-722/2) against the knowledge claim's update that the pros take
   spot_43/38/39 first with the commander and lose none.
8. **The commander at sea**: the Cape Violet plan (992-1001/2) sends Beavers and a shipyard; the commander prompt
   (`commander.md:64-73`) and cases.md:409 have the commander take sea spots itself from 0:00.
9. **The commander near home** (1105-1106, from the smoke games) against reader A's opening that sends it to the
   middle spots from 2:00.
10. **Two assist lists for the same opening**: assist 20 / assist 60 with the constructor first (reader A, 581-657)
    against assist 40 / assist 30 with a Blitz and five Rovers first, "his line, never armcv:2" (925-983/9), the
    latter attached to player-12, which lost on it.
11. **The chat-lead rule as applied**: 925-983 (f) "spend the lead at once" sent 6,800 of Blitzes into the B3 Beamer
    camp on Cape Violet (1002-1004, per the ledger): a lesson applied where its premise did not hold.

## Era-bound passages

- Pre-hands-rebuild (before 2026-10-02), where the lesson is against a defect since fixed and would be discarded or
  kept only as the defect's record: 680-697/1, 698-706, 751-764/4, 765-777/2, 778-796/4, 797-815/3, 816-833/3,
  848-864/3, 1105-1106, 1107-1109, 1123-1130, 1147-1153, 1154-1163, 1172-1173, 1196-1198, and the numbered harness
  items inside every escalate and 3v1 account.
- BARb easy and medium only (2026-09-21/22): comet-1..5, pianist-smoke, pianist-player-1..8, human-1/2's BARb
  comparisons; the profile's clocks (block at minute 10, twelve Flashes by minute 10) are medium's and should carry
  the profile.
- BARb hard_aggressive, the first four games at that tier (2026-09-22/23): escalate-4..7 and the tech clock; the
  Apex and bonus-ladder eras are absent from this half of the file.
- People, 2026-09-27/28, the week the seats, the standing tool and the water words were being built: every 3v1,
  Cape Violet and SailAway lesson; the human-10/11/12 items (2026-10-03) are on the rebuilt hands.
- Pool, 2026-09-23 cards at OS 25+: the replay survey bullet.
- Undated: the map facts (Cape Violet, SailAway, Great Divide), which are map data and do not age; the Great Divide
  plan, which was written 2026-09-26/27 before any game and rewritten 2026-10-18 to splice in the sprint.
- Rewritten in place after the fact (blame): 946-948 (the (h) item, 2026-10-18), 1080-1084 (the Great Divide
  economy, 2026-10-18), 988-989 (2026-10-19), 1089 (2026-10-17), 898-916 (2026-10-02/03), 691-696 (2026-09-27).

## Duplicates for the merge pass

Most of this part is restated in reader C/D's files: `docs/briefs/experience/cases.md` (the comet, escalate and
3v1 cases by subject at lines 60, 74-80, 111, 194-204, 240-250, 264-292, 314-346, 375-397, 403-431),
`team-games.md` (the 3v1 games whole, 11-56; the opening with several seats 74-84; the one-sentence commit 89-95),
`other-maps.md` (Cape Violet 38-47, SailAway 50-67, Coast To Coast 70-81, Great Divide 83-98, Quicksilver and
human-1 7-36), `players-words.md` (every quote of 925-983). `docs/briefs/player-experience.md` (uncommitted)
concatenates those files and so repeats them again (its 1241-1320, 1478-1480, 1553-1563, 1653-1655, 1905-1917).
Within reader A's half: the sprint bullet (92-130), the unit-mix bullet (215-232), the bank bullet (247-253), the
reach cases (283-291), the gathering place (292-300), the scouting cases (303-308), the pickets bullet (335-343),
the wins by shape (55-70), and the opening list (581-657). The role prompt (reader E) carries the shelling place
(264-269), the detachment mechanics (345-357) and the pickets sentence (370-371) in their current form.

## What could not be classified

- Whether the hands mechanics named in 778-796/4, 848-864/3 and 1107-1109 survive the 2026-10-02 rebuild: the
  current prompt does not use several of the names (`send_against`, `attack_unit`, `fight_to` with the underscore,
  `start_boxes_note`, `reached_by` have no hits in `crates/bot/src/strategist/player.md`); reader E's files decide.
- The source line of "moving the commander around costs build power" (1110-1121/2): in other-maps.md and not in
  the inbox chat files read.
- The lens of the Great Divide plan (1080-1094): the user's word relayed from experienced players per the knowledge
  claims, rewritten 2026-10-18 by the session; marked unknown where no person or game is named.
