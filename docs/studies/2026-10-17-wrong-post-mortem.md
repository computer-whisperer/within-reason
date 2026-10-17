# Why the player's post-mortem names the last fight (2026-10-17)

The user's observation (2026-10-17): in games with people the player asks "what would you have used against your
X?" when the game was lost minutes earlier, in the economy. The user's reading: the question is a symptom; the
thing to find is what gives the player the wrong idea of what went wrong. thebluegecko said the same in human-33 at
21:54-22:19: "I think you are treating the symptom ... it isn't just unit choice, it is eco. I have more money, so I
have more units."

Read for this study: the four losses with people since human-24 whose commander spoke at the end (human-28, a 1v1 on
Comet Catcher, lost 15:20; human-31, 3v2, lost 19:50; human-32, 3v2 on Flats and Forests, lost 27:20; human-33, 4v2
on Gecko Isle, lost 22:45) and the two recent 1v1 losses to BARb hard_aggressive with a truth file (levers-player-1,
34:20; nuke-2-hard, 32:22). The instruments: the transcripts (`strategist-N.jsonl`, `commander.jsonl`; the dump
script is in the session scratchpad, not the repo), `run/analyze_match.py` (the curves from the truth file),
`run/team_read.py` (our side of a team game), `run/floor.py` (the `known%` column). Nothing here is a verdict on a
single game; it is the same shape six times.

## 1. What the player said, what the record says

| game | the chat line at the end | the last note's cause | the record |
|---|---|---|---|
| human-28 (1v1) | "Your Stout push broke us" | "we lost the 4-8 min Blitz mirror by trickling Blitzes in singly, and extractors sat at 8-9 from 5:00 to 12:00 (pool 16-22). We never had a lead to use." | extractors 8-9 from 5:00 to 12:00 against the pool's 16-22; income 21 against the pool's 39 at 9:00 |
| human-31 (3v2) | "The Tigers broke us ... at ~13:00 when your T2 vehicles showed up, what would you have done in our place?" | "our T2 start at 14:30 too late; 3 split armies through 8-14 min never hit his base" | the trade negative in every 3-minute window from 3:20 (-0.6k, -0.9k, -1.7k, -1.4k, -0.8k, -2.3k, -4.2k ...); extractors 31 at 10:00 for three seats and falling from there; "staying tier 1" chosen at 10:50 |
| human-32 (3v2) | "the Shiva switch at 21 min broke us and we never answered it" | "army stood in our half from 10:30 on; fights fed piecemeal; our T2 started at 19-21 min vs blue's gantry by ~21" | "army 18.9k vs 7.5k seen, none of ours in his half for 6 min" at 16:50; his tier 2 seen 19:50, a gantry at 21 |
| human-33 (4v2) | "your tzars + bombers turned it ... what would you have built instead vs the tzars?" | "too many T1 arty units" (thebluegecko's first answer, taken as the lesson) | four seats at 11-16 extractors and 25-38 income each, flat from 6:30 to 13:00; the trade even to 11:44 and then -3.6k, -7.8k, -8.9k, -16k a window; one advanced plant allowed at 13:14; his T2 at ~11 |
| levers-player-1 (BARb) | (no people) 29:11 "the game is drifting against us (army 10.7k vs >=11.5k his, tier 2/3)" | Fatboys, a Gauntlet, a Rattlesnake, the body strung out, the hands chasing | his builders 18 at 17:00 and 45 at 24:00 against ours 11 and 10; his advanced lab at 9:54 never seen; ours at 24:26; metal banked 1,200-1,300 from 17:00 to 20:00; the army lead 3:1 at 17:00 spent at E3 under turrets and the Gauntlet (7,300 lost in three minutes) |
| nuke-2-hard (BARb) | (no people) "the push collapsed", Tiger and Shiva raids, "unseen losses" | the silo, the fusions, the missiles | his extractors 36 against our 21 at 20:00 and 57 against 12 at 28:00; metal banked 6.6k to 12.2k from 20:00 to 25:00 |

In every game the chat line names the last thing that hurt (a unit type), and the note under it is better: it names
the tech timing or the split armies. Neither names the minutes the user means, because the player did not know it
was behind then. That is the finding: the post-mortem is wrong because the in-game model was wrong, and the in-game
model was wrong in one direction, "ahead".

## 2. What the player believed mid-game, against the truth

levers-player-1, the player's own words against `analyze_match` (ours / his extractors):

| clock | the player wrote | the truth |
|---|---|---|
| 10:00 | "Economy ahead: 24 extractors against his 13 known" | 24 / 17 |
| 11:48 | "we hold 19 extractors to his ~10" | 21 / 13 |
| 13:18 | "win on extractors (23 to his 13)" | 21 / 20 |
| 15:18 | "We win on extractors (25 to his 16)" | 26 / 23 |
| 15:48 | "We out-extract him 26 to 16" | 27 / 26 |
| 18:54 | "34 extractors to his roughly 18" | 34 / 28 |
| 19:58 (the report) | "his spending seen: at least 17 a second over the game, about 7 extractors' worth" | 28 extractors and an advanced lab since 9:54 |
| 20:08 nuke-2-hard | "army 12.3k vs ~1.4k of his seen" | 11,327 / 5,960; extractors 21 / 36 |

`run/floor.py` `known%` (the player's enemy-army metal as a share of the truth, median over its turns past minute
five): levers-player-1 27, nuke-2-hard 26; the wins leg-player-1 25, jev114-player-1 38, prio-player-1 81. The army
floor is close in the minutes a fight is on (levers-player-1 16:00-20:00: "3,500 seen" against 3,039; "27 worth
5,874" against 5,475), because fights are where we see him; the economy floor is never close, because his base is
seen once at 2-3 minutes and not again (the 19:58 report: G1 "seen 17:01 ago", G2 14:22 ago, the lab line "Bot Lab
armlab at G2" as the only factory).

In the team games the sum of three or four seats stands against two people's floors, so the side reads as ahead by
construction: human-31 10:50 "31 extractors against the 18 we know of theirs" is ten a seat against nine or more a
person; human-33 13:14 "48 to his 23" is twelve a seat.

## 3. The causes

**3.1 The opponent is reported as floors, and the player writes the floors as values.** Every number about him in
the report says so ("known to hold", "seen in the last 3 min", "at least", "a floor, since we see little of his
side"), and the player's next sentence drops the word: "3x army lead", "we out-extract him 26 to 16", "we lead
clearly: 37 extractors to his 10 known". The `his spending seen` line was added after game 9 for exactly this
(`report.rs`: "37k of his seen dead by 13:24 implied 18 extractors while the line said known to hold 7") and reads
"7 extractors' worth" at 19:58 of levers-player-1 against 28. A floor that is 2-4x under is not information the
player can use; it is a number it will use anyway.

**3.2 The one instrument that corrects it is absent where the games are lost.** The reference line
(`strategist/reference.rs`: ours against the experienced players' quartiles at this minute) exists for Comet Catcher
only, for one seat a side only, and to 16:00 only ("past it there is no table to stand against"). human-28, the one
loss with the line, has the right diagnosis at 9:20: "we are still behind on economy, and that gap is what loses the
game. We hold 9 extractors against the pool's 16 at 9:00, with metal income 21 against 39." human-31, -32 and -33
had no line (0 of their reports); levers-player-1 had it to 16:00, where it read income 64 against the pool's lower
quartile 70 and constructors 13 against the winners' 18 with army 9,622 against 6,734 and none of it in his half,
and the player wrote "the line holds at spot_59 with about 8,700 metal of army ... we out-extract him 26 to 16".
The player quoted "experienced players" in nearly every turn to 9:20 and never once after (the first session ended
at 9:30; the line stood in every report to 16:00). What stands against the floor, not against the pool, wins the
sentence.

**3.3 The brief has the lesson and two answers, and no way to tell which applies.** The brief says, in the user's
words, "the visible symptom of a lost fight is a consequence of many things minutes ago", "look even earlier --
6:00", and in an opponent's, "not a unit choice problem, a not enough mex problem"; and it says "a lead is spent or
teched, never banked", with BARb's advanced factory at 10:41 in 73 of 74 games. The player reads it: "before his
tier 2 at ~10:40" is in its 6:16 note. The two answers (spend the lead on him before his tier 2; tech) are chosen on
the floor's evidence every time: levers-player-1 13:18 "we need to stop feeding units into tier-2 range and win on
extractors (23 to his 13)" when it was 21 to 20; human-31 10:50 "Staying tier 1"; human-33 13:14 one advanced plant
"since banks are zero". Our tier 2 came at 24:26, 14:30, 19-21 and 13:14 (one seat of four); his at 9:54, about 12,
about 19 and about 11. Nothing in the report said "his is up and yours is not": his advanced lab was never in the
picture in levers-player-1, and the scouting block's "no factory of his has been seen" answers a different question.

**3.4 The report is a fight ledger, and the notes copy it.** The longest and most changing parts of every report
are the 3-minute trade, the fights line and the fights-so-far block; the economy is our own stores and a 6-minute
curve of our own numbers. No line carries his builders (45 against our 10 at 24:00 in levers-player-1), his
extractors as more than a floor, his tech, or our income against anything after 16:00. The notes are what the report
is: trades and unit names ("Fatboy killed", "push_f5 given", "lost 1115 for ~700 vs Pounders"), and the notes are
the whole memory of a replacement session (levers-player-1 ran five sessions, replaced at 9:30, 16:08, 21:52 and
28:21; nuke-2-hard five). The 16:08 takeover prompt opens with the first session's "Economy ahead: 24 extractors
against his 13 known". The floors become facts at every handoff, and the session that writes the post-mortem never
lived the minutes the user means.

**3.5 The chat question is the proximate cause by its form.** It is asked of a person who can answer in a line, about
the thing that was on the screen. The notes under it are closer (human-31: "T2 start at 14:30 too late"; human-32:
"army stood in our half from 10:30 on"). What neither has is the sentence the record supports: "we were behind on
economy from about 6:00 and did not know it".

## 4. What this suggests (the user's call; nothing built)

*The user's ruling, the same day: the harness has almost no authority to say what he has; the player predicts his
economy and army from what it has seen. The first bullet below, which proposed more estimates, is withdrawn by it;
what was removed instead is in docs/design/2026-09-22-enemy-evidence.md's status of 2026-10-17.*

- **A score the player cannot read as a lead.** The floor stated as a floor with its age ("his base last seen
  17:01 ago; 7 extractors' worth seen, the pool holds 27 at this minute"), or an estimate from the clock instead of
  a floor: against BARb the curve is known (`docs/briefs/reference/comet-catcher-remake-1.8-barb-hard_aggressive.md`:
  extractors, army and the 10:41 advanced factory by minute); against people the pool's median at this minute is
  a better guess of a person at OS 34 than anything our units saw.
- **The reference line where it is missing**: a seat against the pool in a team game (the pool is duels, but a seat
  is a seat), and past 16:00 from the frontier tables (`docs/briefs/reference/frontier-*.json`, the late-economy
  arc) rather than "there is no table".
- **A word for behind, the way STALLING is a word for energy.** `report.rs` writes STALLING because the player keyed
  its rule on it; "BEHIND: income 64 against the pool's 70 at 16:00, constructors 13 against 18" is the same device
  for the economy.
- **The whole game's curve, not six minutes**, in the takeover prompt and every few minutes: ours and the pool's by
  minute, so a session that did not live 9:30 sees the slope.
- **The post-game debrief from the curves, not the notes.** `run/debrief.py` asks "what decided the game" over the
  notes and closing sentences, so its DECIDED lines are the fight story (the inbox: "a failed 14:02 north push",
  "lost to a Rattlesnake", "the Shiva switch"); `analyze_match` already prints the candidate causes from the truth.

## 5. Not read

The wins (prio-player-1, jev114-player-1, leg-player-1) for whether the same floors read as a lead there too; the
glitters 8v8s (the memory says the same curve: level at 8:00, his army triples by 16:00); whether a person's economy
in the team games was in fact larger per seat (no truth file in a game with people; the chat says so).
