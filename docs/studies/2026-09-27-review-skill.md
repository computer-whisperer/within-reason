# The review skill: what the high-OS replays do, minute by minute, against our better and our bad games

Written 2026-09-27 by Claude (Fable 5.1) from the replay cards and our match records (no game run, no model call).
The user's ask: a reviewer skill that a subagent follows after one of our games to say what an experienced player
reviewing the replay would say, built from the difference between the pros' games and ours, and made to grow with
every packet of player feedback. The skill is `.claude/skills/bar-review/SKILL.md`; its numbers are in
`.claude/skills/bar-review/baselines.md`; this is the study behind both. Findings here are observations; nothing in
the bot changed, so no heuristic is registered.

## Sources

**The pros.** The replay survey's cards (`run/matches/*-replay-*/card.json`, `card.md`; the manifest
`run/data/replays/manifest.jsonl` with each player's OS; `docs/knowledge/replays.md` indexes them). 78 replay match
directories exist; 71 have cards; 68 of those are Comet Catcher Remake 1.8 duels (the other three: two Full Metal Plate,
one Gasbag Grabens, not used). Of the 68, **40 have both players at OS 40 and above, and those 40 are the baseline**
(the experienced players, 2026-09-24: 25 is too low to learn from, 40 and above is where the lessons are); no game in
the pool has both players at 45+. The 29 duels at OS 25+ carded on 2026-09-23 stand behind the
`K-map-comet-catcher-remake-1-8-*` claims in `docs/knowledge/maps.md`, `openings.md`, `army.md` and `economy.md`; they are
quoted where the 40+ pool has not been recounted, and the recounts here (`run/replays/recheck.py`,
`run/pro_baseline.py`) agree with them in direction everywhere they overlap. The fight numbers are
`docs/studies/2026-09-24-pro-fight-shapes.md` (the same 40 games, 693 fights).

Per start and faction the pool is small: Armada from B5 or G4 (our arena start is B5 in most player games) 18 sides
of 17 games, 9 winners; Armada from A4 or H5 20 sides of 17 games, 14 winners. `run/pro_baseline.py` takes our
start cell and its half-turn mirror and our faction by default; `--faction any` and `--all-starts` widen it.

**Spot numbers.** The brief for this work warned that the cards' 80 spots are not the bot's 75. Checked: the header
of `onepass-player-fable-1`'s record and a replay record (`1790229111-replay-596ca96a...`) both carry 80 spots, every
one within 2 elmos of the other's in the same order (`docs/design/2026-09-23-replay-survey.md`, the 2026-09-24 note,
holds; the 14:45 note it corrects and the sentence in `docs/briefs/player.md` "Spot numbers there are the game's 80-spot
list, not yours" are stale). Spot numbers in this study and in the skill are the same for both.

**Ours.** Fourteen recorded games, all `run/matches/<dir>/00` with a truth file, all Comet Catcher against BARb from the
west, Armada mirrored, the Opus player over Jev unless said:

| game | result | what it is |
|---|---|---|
| `1790132331-escalate-7-hard-aggressive` | won 16:00 | the series' first hard_aggressive win, A4 start |
| `1790171743-held-1-hard-aggressive` | won 11:11 | the fastest hard_aggressive win (their commander walked out to D2), B5 |
| `1790445854-onepass-player-6-nopenalty` | won 20:18 | Opus 5.5 medium, think penalty 0, B5 |
| `1790454920-onepass-player-8` | lost 32:31 | Opus 5.5 medium, penalty 1 capped at 7 s, the current target; finished during this work |
| `1790452607-onepass-player-7` | lost 25:42 | lost after leading, uncapped penalty |
| `1790444856-onepass-player-5` | lost 23:00 | penalty 1, the list-step fault (92d59cc) |
| `1790455648-onepass-player-fable-1` | stopped 10:01 | Fable 5.1 as the player, a 10-minute batch |
| `1790447448-models-medium2-*` (opus55-low, opus5, sonnet5, gpt56-luna, gpt56-terra, gpt6-astra) and `1790447619-models-medium2-sonnet5-nothink` | all lost, 15:00-38:24 | seven models against BARb medium |

`1790140434-quick-1-hard-aggressive` (won 14:39) is on Quicksilver, which has no cards; it was read for the tools' tests
and is not in the tables.

**Tools.** Existing: `run/analyze_match.py`, `run/floor.py`, `run/wake_read.py`, `run/fire.py`, `run/jev_audit.py`,
`run/commander_turns.py`, `run/replays/check.py`. Written for this study and cited by the skill: `run/pro_baseline.py`
(our game beside the pool's quartiles, milestones and curves), `run/raid_ledger.py` (one row per extractor lost),
`run/hands_window.py` (the hands second by second in a window), `run/batch_read.py` (one row per game);
`run/commander_turns.py` gained `--told` (what the player was shown above what it ordered). Our curves are counted the
way `run/replays/card.py` counts the pros' (units in `own` at the minute's sample, frames included; soldiers are the
def table's class `army`), so the columns compare like with like.

## 1. The opening, 0:00 to 2:00

| | pool (B5/G4 Armada, 40+) | our wins (escalate-7, held-1, player-6) | our losses (player-5, -7, -8, fable-1, opus55-low, sonnet5-nothink) |
|---|---|---|---|
| plant started / standing | 0:34 / 0:53 (A4 pool: 0:58 / 1:17) | 1:08, 0:48, 0:48 / 1:07 | 1:05, 0:51, 0:48, 0:48, 0:51, 0:41 |
| solars before the plant | 2 | 3, 3, 3 | 3 in all but sonnet5-nothink (2) |
| the plant's first unit | a Rover at 0:53 | a Rover at 1:29, 1:09, 1:09 | a Rover at 1:09-1:12 in player-7 and -8; a constructor at 1:09 in fable-1, then two more before any soldier |
| first soldier | 0:53 | 1:29, 1:09, 1:09 | fable-1 2:37 |
| factory units by 4:00 | 18.5 (p25 17) | 8, 8, 8 | 6-10 |
| commander guarding the plant in the first 4:00 (`assist4`) | 50 s (thebluegecko 140) | 0, 0, 19 s | 21, 2, 12, 0 s; opus55-low 92, sonnet5-nothink 107 |
| store below 20 in 1:00-4:00 (`stall4`) | 9% | 0, 64, 1 % | 0, 0, 0, 12, 0, 10 % |

The plant is 14-17 s late in every game because the brief's list has three solars where the pool builds two, and the
plant then makes half the pool's units by 4:00 because the commander walks off to solars, turrets and far extractors
instead of guarding it (`K-open-comet-our-plant-starves` measured this on six games in September; it holds on all
fourteen here). In fable-1 the hands' pick took the constructor world over the Rover world three times running
(0.79-0.84 against 0.74-0.78, `run/hands_window.py 1:00 2:40 plant_22773`) against an allowance that listed Rovers first:
the allowance is a count, not an order, and the pick does not read the order. **Checklist items O1-O5.**

## 2. Expansion, 2:00 to 8:00

Extractors, income and army metal by minute (ours from `run/pro_baseline.py`; the pool's median with the winners'
median in brackets):

| minute | pool extractors | wins: esc-7, held-1, p-6 | losses: p-5, p-7, p-8, fable-1, opus55, s5-nothink | pool income | wins | losses |
|---|---|---|---|---|---|---|
| 4 | 7 (8) | 9, 8, 8 | 5, 12, 12, 7, 8, 6 | 18 (18) | 17, 17, 18 | 10, 21, 22, 12, 16, 13 |
| 6 | 13 (14) | 17, 18, 19 | 10, 15, 18, 4, 16, 10 | 30 (32) | 32, 29, 35 | 22, 27, 33, 10, 33, 19 |
| 8 | 17 (18) | 18, 23, 24 | 11, 21, 14, 5, 29, 12 | 36 (41) | 39, 46, 44 | 24, 38, 29, 12, 51, 28 |

To 8:00 the economy does not separate our wins from our losses: player-7 and opus55-low were at or above the winners'
curve and lost; fable-1 and player-5 were under the pool's p25 (6 / 10 / 12) and their story is raids. Constructors:
the pool 3 at 4:00 and 5 at 8:00 (p75 6); ours 7-10 by 8:00 in five of the nine games, idle 14-22% of their time in the
losses (`run/floor.py`), which is what the experienced players said on 2026-09-20 ("too many construction bots, made
and kept out on the map"). Turrets by 8:00: the pool 9.5 (winners 13); ours 4 and 6 in player-7 and player-5, 9 in fable-1, 11-18 in the rest.
Solars by 8:00: the pool 16; ours 17-24 in escalate-7, held-1, player-6, -7, -8 and opus55-low, 6-15 in the rest (player-5 7,
fable-1 11, the models bundle 6-15, where `e0%`, the energy store empty, ran 10-58% of the game). **Items E1-E6.**

## 3. Raids

The pros lose extractors all game: 19.5 a side (median; p75 36) over a 12:39 game, 1.85 a game-minute, the first at
3:26 (B5/G4 Armada 40+), and re-take 7.5 spots a game (`run/data/replays` cards, `spots_taken`). They meet raids
with light turrets at the extractors and the scouting swarm, not by holding the army home
(`K-map-comet-catcher-remake-1-8-raids-met-by-turrets`).

`run/raid_ledger.py` over ours (a loss is undefended when no soldier of ours stood within 900 and no turret within 700
ten seconds before; answered when the hands played a hunt or attack against a party within 900 of the spot):

| game | lost (a minute) | undefended | with a turret | answered within 60 s | raiders killed after | rebuilt (stood) | rebuild delay median |
|---|---|---|---|---|---|---|---|
| escalate-7 (won) | 9 (0.6) | 2 | 6 | 8 of 8 | 6 | 7 (5) | 57 s |
| held-1 (won) | 12 (1.1) | 6 | 3 | 10 of 10 | 9 | 3 (3) | 13 s |
| player-6 (won) | 15 (0.8) | 4 | 6 | 13 of 13 | 10 | 8 (7) | 134 s |
| player-5 (lost) | 44 (2.0) | 14 | 19 | 33 of 33 | 17 | 19 (14) | 103 s |
| player-7 (lost) | 47 (1.9) | 19 | 16 | 38 of 38 | 17 | 19 (15) | 161 s |
| player-8 (lost) | 70 (2.2) | 10 | 47 | 57 of 59 | 41 | 28 (22) | 101 s |
| fable-1 (stopped) | 19 (2.1) | 14 | 3 | 16 of 17 | 5 | 6 (3) | 109 s |
| opus55-low (lost) | 89 (2.3) | 19 | 52 | 81 of 83 | 66 | 43 (36) | 120 s |
| sonnet5-nothink (lost) | 28 (1.2) | 0 | 27 | 25 of 27 | 21 | 6 (5) | 88 s |

The answer is always fast (median 1-16 s from first sight) and mostly useless: in fable-1 it was a hunt of one to four
Blitzes after a Tick in 12 of 19 rows and no raider died after 3:10; in player-7 and -8 "1 of group_A hunt" after Pawns.
The loss rate, not the answer delay, separates the wins (0.6-1.1 a minute) from the losses (1.9-2.3), and behind it
the cover: 14 of fable-1's 19 and 19 of player-7's 47 losses had nothing of ours in reach, against 2 of 9 and 4 of 15
in escalate-7 and player-6. Player-8 and opus55-low lost 47 and 52 extractors with a turret in reach: a light turret
does not stop a party of four Pawns (player-8 7:33, spot_28 and spot_30). **Items R1-R5.**

## 4. The window that decides the game, 8:00 to 12:00

The 29-duel survey: winners add 11 extractors from 8:00 to 12:00 and losers 1; winners spend (under 500 banked at 10:00
in 15 of 22). The 40+ B5/G4 pool: extractors 16 / 20 at 10:00 / 12:00 (winners 20 / 26), income 43 / 48 (53 / 62), army
metal 2,654 / 4,163 (3,008 / 5,601).

Ours at 12:00: player-6 31 extractors, income 58, army 8,132; player-8 36 / 66 / 7,135; player-7 21 / 39 / 6,834;
opus55-low 23 / 45 / 6,328; escalate-7 21 / 45 / 9,031. **Our Opus games are on or above the winners' curve at 12:00 in
every column**, and three of those five lost. Income per extractor from 12:00 is the one economic column where the
pros stay ahead: 117 income from 25 extractors at 16:00 in the pool (4.7 each) against our 59-62 from 30-32 (2.0):
wrecks reclaimed and resurrection bots (24 of 58 sides, `K-eco-comet-pros-rez-bots`), which we never build (0 rez bots by
8:00 in all fourteen games; player-8 was told of 2,351 metal of wrecks on held ground at 13:20 and reclaimed none).
The second factory: the pool 7:20, ours 5:38-7:16 in the Opus games (player-5 8:53). The store: full 13.8-29.8% of the game in the
lost Opus games (player-7 28%, player-8 14%, opus55-low 26%), 0.6-1.0% in the wins. **Items W1-W4.**

## 5. Fights and closing

**Closing.** Computed 2026-09-27 from the 40 games' cards (`curves`, per team): the games last 12:39 (median; p25-p75
9:17-16:07). The eventual winner first holds twice the loser's army metal at minute 7 (median over the 27 games where
it happens; p25-p75 6.5-13) and the game ends 3.4 minutes later (median; p25-p75 1.2-5.4; over six minutes in 5 of 27).
The extractor lead reaches five at minute 7 (33 games). At the end the winner holds 27 extractors to 14.5 and 1.8 times
the army. Ours:

| game | first minute our army value was twice the truth's (with at least 1,000 of ours; `run/analyze_match.py` curves, soldiers standing) | minutes from there to the end | result |
|---|---|---|---|
| held-1 | never (5,770 against 3,314 at 10:00, 1.7x) | - | won 11:11: their commander walked out to D2 |
| escalate-7 | 11 (6,418 against 2,150); 8.3x at 12:00 | 5 | won 16:00 |
| player-6 | 6 (1,052 against 519); 3x at 10:00, 4.7x at 12:00 | 14 (the push began 11:32 on 7.8k against 1.8k seen; the commander was first seen 19:08) | won 20:18 |
| player-7 | 13 (6,484 against 729); 9x at 14:00 | lost at 25:42 | the base push at 15:25-16:34 failed on a beamer and unseen shelling |
| player-8 | 8 for a minute (1,461 against 432), then from 13 (7,035 against 1,388); 6x at 14:00 and 16:00 | lost at 32:31 | the north-edge strike 14:13-20:39 into Gauntlets and a Fatboy, 15.9k lost for 5.6k |
| opus55-low | 9 (3,203 against 1,237); 2.9x at 16:00 | lost at 38:24 | the army spent on turret lines out of sight while Pawn raids bled the extractors | This is the largest difference
between the pros and our Opus games: the pros close within four minutes of holding twice the army; we mass on for
five to twenty. The wins closed in 3-4 minutes when the enemy commander came out (held-1) or the push went at once
(escalate-7), and in 11 when the base had to be found (player-6). **Item F1 is the checklist's costliest item.**

**Where the metal is lost** (`run/analyze_match.py`, "metal lost by place"): escalate-7 lost 7,692 in their half and
2,677 in ours; player-6 19,170 in theirs and 3,511 in ours; player-7 27,799 in ours and 7,551 in theirs; opus55-low
54,335 in ours and 12,525 in theirs; player-8 68,307 in ours and 17,552 in theirs. The pros: from minute 6 the deaths
are in the loser's half. **Item F2.**

**Static defence.** Player-8's engagements #15, #18 and #19 (16:26-17:57) had 2,840-3,200 of enemy turrets on the spot
against our fighters alone (#19: 2,585 lost for 0); its notes name a Gauntlet (1,220 range, found 16:23 by walking into
it), a Dragon's Claw (18:39) and a Fatboy (20:08). Player-7's base push at 15:25-16:34 failed the same way (a beamer and
unseen shelling). Player-6 broke an HLT nest at 12:40 by rushing it with 24 Stouts together. The pros' games are
decided on tier 1 before 13:00 (tier 2 in 4 of 58 sides, never before 12:51): the defences that stop a tier-1 army
exist because the game ran on. **Items F3, F7.**

**Shape.** `docs/studies/2026-09-24-pro-fight-shapes.md`: bodies of 6 (ours 13), a friend on the line of fire for 22%
of units in reach (ours 43%, Stouts 53%), spacing 67-70 elmos (ours 25-40), mixed raider-and-line bodies 10% (ours
61%), one point per unit in 70% of orders (ours 21%). `run/fire.py` on player-8: 12% of in-reach seconds muzzled, 9%
of damage dealt on our own units. **Item F4.** Reinforcements walking one by one to a group at the front (player-8's
19:39 note; engagement #22, 445 of ours against 2,940) is **F5**.

**Scouting.** `run/floor.py`'s `look_min` (the minute one of ours first stood within 1,500 of the enemy commander's
true start): 16.0 (player-8), 14.3 (player-6), 15.9 (escalate-7), 3.6 (held-1); `fac_min` (an enemy factory seen in the
picture) never in player-8, fable-1, player-5; `known%` 17-49 in every game. The cards cannot give the pros' number (a
replay sees everything); their vehicle plants make five or more scout cars by 3:00 in 38 of 45 sides. **Item F6.**

## 6. Endings

The pros: 12:39; the loser's commander and factories in its start cell in 28 of 29 surveyed games; the winner at 27
extractors to 14.5 and 1.8x the army. Our wins end at 11:11, 16:00 and 20:18 by killing the commander after it was
found; our losses run 23-38 minutes and end with our base razed by tier 2 (player-8: Razorback, Fatboy, Marauder led
the damage on us) or with the extractors bled to nothing (sonnet5-nothink: 3 left at 21:33 with a 47-62 Blitz ball at
home). **Item X1.**

## 7. The player and the hands

Numbers the skill asks for, from `run/wake_read.py`, `run/floor.py`, `run/batch_read.py`:

| game | turns a minute | wall median (max) | abandoned | in flight | known% | idle% | mfull% | stuck s | refused |
|---|---|---|---|---|---|---|---|---|---|
| escalate-7 | 4.1 | 7.2 (19) | 0 | - | 33 | 1.0 | 0.7 | 227 | 1 |
| player-6 | 6.5 | 5.5 (18) | 0 | - | 25 | 18.1 | 0.6 | 650 | 0 |
| player-7 | 5.4 | 5.1 (21) | 0 | - | 24 | 19.4 | 28.3 | 280 | 0 |
| player-8 | 5.7 | 5.3 (17) | 0 | 57% | 28 | 17.8 | 13.8 | 1,083 | 0 |
| fable-1 | 4.1 | 6.4 (42) | 0 | 67% | 17 | 9.5 | 1.2 | 52 | 0 |
| opus55-low | 8.1 | 3.6 (11) | 0 | - | 49 | 22.4 | 25.6 | 1,370 | 1 |
| sonnet5-nothink | 4.5 | 5.6 (24) | 0 | - | 31 | 7.0 | 29.8 | 1,315 | 2 |
| sonnet5 | 3.3 | 12.3 (45) | 3 | - | 35 | 15.9 | 1.0 | 1,090 | 2 |
| opus5 | 2.6 | 21.6 (45) | 2 | - | 36 | 20.7 | 9.4 | 64 | 0 |

The player's cadence is not what separates the wins from the losses among the Opus 5.5 games (5.1-7.2 s a turn in all
of them). What the trial reviews found the player doing: massing past the pros' closing moment (P4), committing without
scouting (F6), noticing the plants idle four minutes after the store filled (P5), and diagnosing correctly one turn
after the damage (P7, fable-1's notes at 2:13, 5:04, 5:44, 9:44). What they found the hands doing: constructor-first
production against a Rovers-first allowance (H1), hunts of one Blitz after a Tick and hunts ending "no hunters" (H2),
the commander's `attack party_N` played beside its list step every second at 2:13-2:22 in fable-1 with the pick still
choosing the attack after the standing rule was cleared (H3, Z4), the advanced plants' worlds offered every second and
never picked under a full store in player-8 at 25:05-28:38 (H1, Z2, `K-hands-the-pick-dilutes-over-single-change-worlds`).

## The differences that became checklist items

| difference | numbers | items |
|---|---|---|
| the plant is late and starved | 0:48 against 0:34; 8-10 units by 4:00 against 18.5; the commander guards it 0-21 s against 50; three solars against two | O1, O2, O3, O4 |
| no turret for the scout cars and Ticks | first LLT 3:19 in fable-1 after five losses | O5, E4 |
| constructors above the pool and idle | 7-10 by 8:00 against 4.5; idle 14-22% | E2, H4 |
| extractors lost uncovered, not rebuilt | undefended 14 of 19 (fable-1) against 2 of 9 (escalate-7); rebuilt 6 of 19; the pros re-take 7.5 a game | R2, R4 |
| the raid answer cannot catch | Blitz (101) after Tick (140), 0 killed after 3:10 in fable-1; a 47-Blitz ball at home in sonnet5-nothink | R3, R5, H2 |
| the store full while the plants idle | mfull 14-30% in the Opus losses, 0.6-1.0% in the wins | E6, H1, P5, Z2 |
| income per extractor half the pros' from 12:00 | 2.0 against 4.7 at 16:00; 0 rez bots; wrecks named and not reclaimed | W3 |
| the lead not converted | the pros at 2x by minute 7 and done 3.4 minutes later; player-8 at 2x from 13:00, 6x at 16:00, lost at 32:31 | F1, P4, X1 |
| the army walked into static defence | 2,840-3,200 of turrets on the spot; 15.9k for 5.6k in player-8's strike; Gauntlet 1,220 against Stout 350 | F3, F7 |
| the fighting at our base, not theirs | wins lose most metal in their half, losses at our base | F2 |
| bodies twice the pros' size, half the spacing, friendly fire | 13 against 6; 43% against 22%; 9% of damage on our own units | F4, F5 |
| the enemy's ground unscouted | look_min 14-16; fac_min never; known 17-49% | F6, P3 |
| the commander chases what it cannot catch; two plays a second | fable-1 2:13-2:27 | H3, Z4 |

## What was done with `docs/harness/verdicts.md`

The verdict pass (one analyst per lost match, `verdict.json`, `run/tally_verdicts.py`) and the review are one procedure
now: the skill's step 1 is the verdict pass's procedure (`run/analyze_match.py` first, the last level minute, the
decisive moment, the earliest sufficient cause) with the pro baseline and the raid ledger beside it, and its step 4
writes the same `verdict.json` with a `review` object added (the ranked findings by item id, the stronger player's
orders, the player's and the hands' numbers, the harness faults), so `run/tally_verdicts.py` reads the new files
unchanged. `verdicts.md` keeps the file's shape and the tags (two added: `lead_not_converted`, `opening_starved`; and
`won` / `stopped` for games that are not losses) and points at the skill for the procedure. The two trial reviews
(`run/matches/1790455648-onepass-player-fable-1/00/review.md`, `run/matches/1790454920-onepass-player-8/00/review.md`,
each with its `verdict.json`) were tallied with `run/tally_verdicts.py` to check the shape.

## Not measured, and what a packet of feedback should bring

- The pros' raid answer and rebuild delay: the cards carry re-takes (7.5 a side) but not who answered a raid or how
  fast; a replay record has both sides' units and could give the ledger's columns for the pros (`run/raid_ledger.py`
  reads our records only).
- The pros' scouting (a replay sees everything): the number of scout cars and where they walked is in the records'
  `own` positions and could be counted.
- Where the pros' turrets stand relative to the extractors (the cards have no position for turrets).
- Our Quicksilver games have no baseline: no Quicksilver replay has been carded.
- The A4 Armada pool at 40+ is thebluegecko's three-solar line in 13 of its 17 games: two players' habits, not a
  population.
- Every "our games" cell rests on one game per condition; the fourteen games share the brief's opening and the
  differences among them are the player's choices and the harness's faults of the day.

## Stale text and rot noticed

- `docs/briefs/player.md` (the Comet section, "Spot numbers there are the game's 80-spot list, not yours"): stale
  since 2026-09-24; left alone because the brief is compiled into the bot and a change there is a brain change.
- `run/jev_audit.py`: every section but `view` and `turns` crashed on the one-pass logs (`KeyError: 'played'`: version-2
  `call` lines carry no `played`); the lookups were made tolerant (`c.get("played") or []`) so the sections run and
  print zeros where the older menu logs would have had counts. The menu sections are of the retired menu era; `view`
  and `turns` are what the skill cites.
- `run/army_use.py` has no docstring and reads batches by glob; not touched, not cited.

## 2026-09-28: cause before symptom (the second iteration)

**The fault.** The user, after player-9's review (2026-09-28): the review feedback from the models "is still rather
poor in BAR"; "in many cases the visible symptom of a lost fight is a consequence of many things minutes ago"; a text
reader can "struggle to see spatial and temporal patterns unless they are called out"; "trust the feedback the humans
give you over the feedback opus gives you." Player-9's review had opened on the 2x lead held from 12:00 and his base
never found. The user's read, reached only after "look even earlier -- 6:00": the game left the winning path at
3:00-7:00, when a 2.3-3.0x lead was walked round our own half (the 5:59 raid loop through six of our own spots), the
plant made constructors, and the one scout to reach his side died 1,085 short of his lab at 5:37; his tier-2 lab
(10:32, never seen) was the window a duel is won in, not a cue to match him.

**The change.** `run/minutes.py`: the game minute by minute (both armies' mass by cell and the share of ours in his
half, what of his was in sight, what the plants made, the player's tools, the losses and where, what he started
building from the truth file), then WINDOWS (every run of minutes at 1.5x his army or more, where our mass stood
through it, how it ended), HIS TIER 2 (his advanced lab's start, its first unit, the six minutes after), HIS BASE IN
THE PICTURE, and UNITS OF OURS SENT TO HIS SIDE (how near his start each got, whether it turned back, what killed it).
It is step 2 of the skill and is read before any checklist item; the reviewer writes down the first minute the game
left the winning path before opening anything else. The skill's preamble carries the ruling; findings are ranked by
root (the earliest moment still avoidable) with a chain to the symptom, not by the symptom's size, and only among
findings on the chain to the result; each says whether its "because" comes from a person or the reviewer; a source
tag naming a person outranks one naming our games. Items: E8 (the early lead spent in our half) new; F1, F6 and F7
rewritten around the tool (F7: his tier 2 is our window, with the user's calculus by scenario). Verdict: `root_minute`,
per-finding `root`, `chain`, `because_from`; tag `lead_spent_in_our_half`. `analyze_match.py` no longer counts an
abandoned frame decaying to nothing as a death in a fight (player-9's engagement #8, 2,910 "lost to Pawns" at 13:56).

**The test.** An Opus 5.5 reviewer given a copy of the revised skill with every player-9 number removed from its
worked examples (the repository's copy carries them), told not to open the earlier `review.md`, reran player-9
(`00/review-2.md`, `00/verdict-2.json`). Its opener: "You were at three times his army at 5:00 and had his base's
doorstep in sight at 5:37. Then at 5:59 you sent the twelve Blitzes on a loop of your own middle spots and back, and
filled the plant with constructors. By 8:00 he had the bigger army." `root_minute` 5, `primary_cause`
`lead_spent_in_our_half`; his tier-2 lab at 10:32 ranked as the missed window; the 12:00-24:00 lead as the same
mistake a second time. Checked against the record: the 5:59 orders (`station "spot_49 spot_46 spot_40 spot_55 spot_58
spot_64"`, `armcv:2`), the Rover's death at 5:37, the lab at 10:32, and the decayed advanced-plant frame it flagged as
a tool fault all hold. Caveat: it read the ledger row, which carried the first review's summary. The first review,
same model and game, had ranked the opening eighth and the 12:00 lead first. One rule it exposed: ranking by root put
the opening (root 0:00, which the game recovered from: 2.6x at 3:00) at rank 1; the skill now ranks by root only
among findings on the chain to the result.
