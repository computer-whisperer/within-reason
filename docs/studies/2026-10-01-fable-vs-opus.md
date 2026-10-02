# Fable 5.1 against Opus 5.5 as the player, 2026-10-01 (fable-2-medium beside player-33-hard)

**Checked by the main session, 2026-10-01** (the study below is a subagent's, on Opus). Read again from the logs and as
written: Fable's packet first carries "no detachment of one or two" in the orders in force from 4:27, and the hunts'
forbidden noul is a median 0.66 before it (n = 44) and 0.85 after, to 8:00 (n = 238); turrets finished by 8:00 are 3 in
the Fable game and 8 in player-33 (the study says 9); the commander was over 600 from the plant in 166 of 361 samples
from 4:00 to 10:00 in the Fable game and in none in player-33; extractors at 8:00 in player-27 and player-28 are 17 and
15 in the ledger. Not read again: the share of group asks matching a group's own paragraph (52% against 27%), the
turn-by-turn tool mix, and the shares of the gap, which rest on one game a setup.


The question (the user): why did `fable-2-medium` (Fable 5.1, BARb medium, won 27:57) fall behind `player-33-hard`
(Opus 5.5, BARb hard_aggressive, won 24:34) in economy, army and tempo, against an opponent a tier lower? The
suspicion to test: the role text, brief, tools, report and the code that reads the packet are tuned to how Opus fills
gaps, so a different model gets worse behaviour from the same harness.

Both games: commit be75a69, Comet Catcher, seed 1, our start B8, his G1, think penalty 1 cap 7, hands lean. The
closer comparison for the opponent is Opus against BARb medium with a vehicle opening on older code:
`player-27-twostage` (e144bb4, called at 14:53, behind) and `player-28-budget` (a969cf2, won 20:46). The code between
them and today: the forbidden mark on hunts (3d4a929), the opening turn before the game (3748b72), produce as a
sequence (80537d8), the long-form gate restored (c400b85), escort, list replacement (be75a69). `player-26-joint` drew a
bot lab from BARb medium and is not a vehicle comparison. Everything below is one game per cell; read every share as
an estimate from n=1.

## Verdict

- **About half the gap is the opponent, more in the early game.** At 8:00 we held 22 extractors against hard
  bot-lab BARb, 15-17 in the two Opus games against medium vehicles, and 10 in the Fable game. At 12:00 the counts
  were 31, 22-25 and 16. Medium vehicles carried 2-3x the army of hard bots in minutes 4-8 (550/880/1,052/1,486
  against 162/486/486/756). Its raiders were Blitzes and Rascals, which no tier-1 vehicle of ours catches; the
  Pawns could be caught. The hard opponent spent on builders instead (5-9 at 5-8 min against medium's 1-2).
- **The player model's reading of the harness explains little or none of the gap that is left.** I tested every
  place the suspicion named, and each came back clean or favoured Fable. Jev rated Fable's hunts "forbidden"
  correctly: from 4:19 Fable's packet said "no detachment of one or two". Opus wrote the same doctrine against the
  same opponent in player-28 ("never split and sends no detachments", 5:10-5:56). Fable's packets matched the
  group-paragraph convention more often than Opus's did (52% of group asks against 27%). Both models used "advanc" and
  `spot_N`. Neither had a refused call. The tool mix was nearly identical, and Fable took turns a little faster.
- **The rest is player judgement plus opponent variance that I cannot separate, about a third of the gap.** Three
  things set this game apart. (1) Thirteen extractors were lost in minutes 4-8, against 3-4 in player-27/28. Four of
  them died to Rascal pairs, a raider the player-27/28 opponent never sent (that is variance). (2) Turrets came late
  and many frames died: 3 finished by 8:00 against 9 for player-33 and 9 for player-28. This is partly the player:
  Fable put the turret fifth in its first constructor list, where Opus put it second. (3) The commander was off the
  plant 4:27-5:51 and 6:10-9:29, against the brief's "out only for one solar or one extractor at a time". This is a
  choice made against explicit text, not a gap Opus happened to fill.
- **One harness bug hit both games about equally:** list steps "taken as done". It ate 5 genuine steps in the Fable
  game before 5:00 (2:33-2:34, 4:47-4:48) and 4 in player-33 (4:36-4:37, 7:37, 9:00). c888b53 has fixed it since.
  Its cost was small and the same for both.
- **Fable's game is inside Opus's spread against the same opponent type.** At 12:00 its army was level (4.3k/4.2k);
  player-27 was behind (3.6k/4.8k) and player-28 ahead (6.1k/2.5k). Fable's extractors were lower than both.

Rough attribution of the 8:00 extractor gap (22 against 10, 12 extractors): opponent type 5-7; this game's raid and
turret story 4-6, part of it opponent variance (the Rascals) and part player choices (turret order, the commander);
the list bug about 0 net; reading the harness the way Opus does, no measurable share. After 12:00 the gap is mostly
the opponent's army: medium's 7-8k from 15:00 against hard's 3-4k.

## The mechanisms

### 1. The opponent: vehicles early, Blitz and Rascal raiders (opponent)

- **Clock and numbers** (`run/analyze_match.py`): his army at 4-8 min was 550/880/1,052/1,486/1,376 in the Fable game,
  990/770/910/1,585 (min 5-8) in player-27, 990/1,100/440/775 in player-28, and 162/486/486/756/1,425 in player-33.
  His builders at 5-8 min were 1-2 for medium vehicles and 6-9 for hard bots. His extractors at 8:00 were 9 in all
  three vehicle games and 11 for hard.
- **Raiders and our answers.** In the Fable game, hunt lines against Blitzes read "they drive it off ... without
  catching it (101 against its slowest 101)". In player-33, hunts against Pawns read "can catch it (101 against its
  slowest 87)" (`jev-0.jsonl` gate questions, 4:02 and 4:53). Rascals (168) outrun everything we field before Rovers.
- **Our extractors at 8:00 / 12:00:** player-33 22/31, player-27 17/22, player-28 15/25, Fable 10/16.
- **Supports:** the opponent. **Would have falsified it:** Opus against medium vehicles reaching the 8:00 count of
  player-33 (about 22); it reached 15-17. Caveat: player-27/28 ran on older code (no opening turn, produce not a
  sequence, no forbidden mark).

### 2. Raid losses in minutes 4-8: 13 against 3-4 (opponent variance and the player; one instance)

- **Numbers** (`run/raid_ledger.py`, by minute): Fable 13 in 4-8, player-27 3, player-28 4, player-33 7. Fable's
  killers in that window: armflash 8, armfav 4, def-1 1. In player-27/28 the killers were armflash and armstump only:
  no Rascal raids.
- **The answers.** From 4:19 Fable's packet said: "group_C attacks it with the whole group together, no detachment of
  one or two, and keeps attacking while it is in sight". The ledger shows `group_C: attack party_N with the whole
  group`, a median delay of 14 s from first sight (player-33: 2 s), and the 11-Blitz ball chasing single Rascals. For
  example, at 6:09-6:15 (`run/hands_window.py ... 6:00 7:00`) the pick took `group_C: attack party_13 (1 armfav) with
  the whole`, then party_8, then party_13 again. The role text names this exact failure ("they send the whole ball
  after one scout car and it never catches it", realtime-2). Fable's own note at 6:22: "Blitz ball cannot catch
  them".
- **Turrets** (record `created`/`finished`/`destroyed`). Turrets finished by 8:00: Fable 3 of 10 started (frames
  killed at 5:21, 5:58, 6:57, 7:05 by armflash and armfav, three decayed); player-33 9 of 13; player-28 9 (finished
  counts per minute). The turret's place in the first list: Fable's 1:55 list put `armllt spot_63` fifth, after four
  extractors; Opus's 2:15 list was `extractor spot_69, armllt spot_69, ...`.
- **Constructors:** made by 8:00, Fable 7, player-33 10, player-28 9, player-27 13; alive at 8:00, 4 / 9 / 8 / 12.
  Fable lost 3 to raids.
- **Supports:** mostly the opponent's raider mix (the Rascals are variance between BARb medium games), plus the
  player's turret order. **Against the harness-reading story:** the whole-group doctrine is not a Fable reading. Opus
  wrote it, word for word in kind, in player-28 against the same raiders. **Would have falsified "player":** player-28's
  turret order matching Fable's while its losses stayed at 4. I did not check player-28's list order step by step.

### 3. The forbidden mark and the hunts: 25 played against 152 (the packet's own words; a harness change is the amplifier)

- **Numbers** (`run/jev_load.py`): forbidden nouls median 0.86, 93% at or over 0.7 (Fable); 0.67 and 44% (Opus). Hunts
  played: 25 against 152.
- **Time course** (my script over `worlds_gate` flags, 15 s buckets): Fable 0.55-0.68 from 2:45 to 4:14, under packets
  that said "Against a lone raider send one soldier" (1:55) and "It never hunts a party that is out of sight" (3:13).
  It was 0.77 at 4:15 and 0.88-0.90 from 4:45, after the 4:19 packet's "no detachment of one or two". Fable's packets
  then said "never hunts" 37 times, "no detachment(s)" 21 times and "never sends a" 17 times. Opus's said "attacked by
  two of group_R1 sent against it" and "sends two soldiers" (4 times). Every hunt state is a detachment ("N of group_X
  hunt party_M"), so Jev's 0.86 is a correct reading of what Fable wrote.
- **Fable's reason** (note, 4:19): "lone enemy Blitz killed two of ours piecemeal". Against an equal-speed raider, a
  detachment of one or two cannot catch it and loses half the time.
- **Supports:** the player's deliberate doctrine, driven by the opponent. The harness part: the forbidden mark
  (3d4a929, 2026-09-30) did not exist in player-28, where the same doctrine left the hunts playable (player-28: 13
  hunts killed their target, 32 hit the leash, 6 lost their quarry). Today the mark makes that doctrine bind. That is
  a harness change acting on any model in the vehicle matchup; the Pawn games never exercised it. **Would have
  falsified it:** a high forbidden noul under the 1:55 packet that allowed one-soldier answers. It was 0.55-0.68
  there.

### 4. The commander off the plant (player judgement against the brief)

- **Clock** (jev `played` rows for `commander`): Fable's commander left at 4:27 for spot_50 and spot_45 (about
  1,900-2,270 away) and a turret, assisted from 5:51, left again at 6:10 for spot_73, turrets, solars and spot_69,
  and was back on the plant at 9:29. Opus's commander assisted from 3:30 and left only for solars, 5:06-5:28 and
  6:57-7:26.
- **Quoted:** Fable at 4:19: "6 mex, metal 0: assisted plant starves the constructors' extractors. Commander goes to
  spot_50/45 ... instead of assisting". The brief says the commander is "on the plant from the second it stands and
  out only for one solar or one extractor at a time" and "when the metal store reads 0 in the first three minutes the
  commander's own build is the thing to drop". The role text says "the store never reads 0; when it does, the plant
  is starving".
- **Cost:** smaller than I expected. Plant output in metal through 8:00 was Fable 4,063, player-28 4,240, player-27
  4,424, player-33 4,898 (record `created` by the plant, priced from `unit_defs`). Minutes 5-7 dipped to 355/470/360
  against 448-795 in the Opus games.
- **Supports:** the player model's judgement, not a gap Opus fills differently. The text is explicit. The one opening
  for a literal reader is that the brief's rule is scoped to "the first three minutes", and Fable acted at 4:19.
  **Would have falsified "a cause":** plant output equal to the Opus games in 5-8. It was 25-45% lower in 5-7 and
  level over 1-8.

### 5. The energy stall 7:00-8:40 (player; and against the suspicion)

- **Clock:** energy at 0 for 39 s in minute 7 and 21 s in minute 8 (record samples, `e[0] < 30`). Fable's
  constructor_16248 was ordered `armnanotc, armadvsol, ..., armvp` with a 900-1,280 bank at 6:12-6:52. Advanced
  solars cost energy to build, which the brief warns about. At 7:27 Fable noted: "Energy at 0 with 1080 metal
  banked: the real fault is energy (5 solars)".
- **Against "Fable is worse":** Opus stalled more. Energy-empty seconds before 12:00 were 2/32/3/52/8/35 in minutes
  1-6 and 43/28 in minutes 10-11, against Fable's 19 (min 2) and 60 (min 7-8). `floor.py` e0%: Opus 20.1, Fable 5.7.
  The brief's solar rules contradict each other ("a solar only when the energy words say stalling", "16 by 8:00",
  "budget a solar per Blitz"), but the two models erred in opposite directions and Opus erred more.

### 6. Harness bug that hit both: list steps "taken as done" (harness; fixed in c888b53)

- **Numbers** (report lines in the `turn` prompts): Fable 16 "taken as done", of which 5 genuine steps before 5:00
  (2:33-2:34 spot_62/spot_54; 4:47-4:48 spot_64, spot_59, `armllt spot_76`). The rest consumed Fable's own
  deliberate duplicates (5:30-5:31, 6:20-6:22). Player-33: 9 (4:36-4:37 spot_56/spot_58, 7:37, 9:00, then late).
  Player-28: 6.
- **The player's read:** Fable diagnosed it at 2:46 and 4:49 ("a list given to a constructor that is walking to a
  build takes its first TWO steps as 'the build already under way'") and worked around it by repeating the first step.
  Opus did not remark on it. This is evidence of Fable using the harness well.
- **Supports:** a harness bug affecting both about equally. It is not a cause of the difference.

### 7. Things the suspicion predicted that did not show

- **The group paragraph** (`diet::paragraph`, the head before `:` or `(` must equal the group's name). Over the Jev
  calls, the group asked had its own paragraph in 52% of asks in Fable's game and 27% in Opus's; 24% against 49% were
  groups the packet did not name at all. Both wrote heads that miss some groups: Fable "every other group of armflash
  (group_G, group_I, new ones): joins group_C"; Opus "group_R1 (and group_D2, group_F2, group_E2, which join
  group_R1)", whose head matches only R1, and "New Stouts and Janus from the plant: group_S". Without a match the code
  falls back to the whole packet, which offers more, not less.
- **"advanc" for fighting legs** (`plan.rs` 897): Fable wrote "advance(s/ing)" 80 times, Opus 53. Advance states were
  asked 22k times against 15k walk (Fable) and 47k against 25k (Opus): the same kind, a similar ratio.
- **Spots named only as `spot_N`:** neither model used shorthand (`spot_50/45` appears only in Fable's notes, never
  in a packet). **Escort** (paragraph names a builder): only Opus's packet produced an escort state
  (`escort_commander`, 257 asks), and it was played 0 times in both games.
- **Tools** (`strategist-0.jsonl` `tool_call`): Fable queue 56, produce 20, instruct 60, lane 23, mark 1; Opus queue
  58, produce 21, instruct 52, lane 14, mark 5, remove 3. Zero refused in both (no "refus" in `bot.log`). Packet median
  length: 2,456 characters (Fable) against 2,126.
- **Cadence** (`run/wake_read.py`): 5.68 turns a minute at a 6.9 s median against 5.26 and 7.9 s. Orders were in
  flight 91% against 73% of game time, a consequence of more turns at the same cap rather than a deficit. Fable
  switched the `extractor_lost` wake off for 1,334 s and `enemy_near_extractor` for 1,430 s; Opus kept them on. Fable
  still rebuilt 51 of 66 lost extractors (median 136 s) against Opus's 23 of 42 (140 s).
- **Where his base is:** Fable had his commander and plant at G1-G2 at 2:56. Two Rovers came within 560-667 of his
  start because they roved there. Opus's single Rover died to a turret at an H3 outpost where the hard opponent's
  commander stood at 3:12, and Opus took that for the base until 23:51. The opponent's layout differed; this is
  not evidence about reading the report, and it cut in Fable's favour.
- **Factory plays from lists against the pick** (134/58 against 118/147): with produce as a sequence, a plant follows
  the counted entries and the pick plays only after them. Opus's plant made more units, so it ran past its counted
  entries more often. This is a throughput effect, not a reading.

## Passages and code checks that depend on Opus's habits

I found no check that fails on Fable's wording where it passes on Opus's. Three passages are tuned to the Pawn and
Tick opponent the recent Opus games faced, not to Opus's wording. Each would mislead any model against vehicles:

1. **Role text, raid answers** (`player.md`, "Holding ground and attacking": "a single Tick or scout car at an
   extractor is met by one soldier from the nearest group ... the ball never chases a lone raider"). Against
   equal-speed Blitzes a single soldier neither catches nor reliably beats the raider. Fable followed the text at
   1:55, saw it fail, and switched to whole-group-only at 4:19. Opus did the same in player-28. Counter-example to
   the suspicion: the drift is the opponent's, not the model's.
2. **The forbidden mark** (`plan::FORBIDDEN`, `threats::gate_questions`). It was built and set (0.7) on player-29's
   Pawn game, where the packet forbade the ball's hunts while pickets hunted. Every hunt is a detachment, so a packet
   that forbids detachments for its only raid-answering group leaves Jev no catching answer but the whole ball. The
   mark is working as designed. What is untested is the vehicle matchup, where "no detachments" is the doctrine both
   models reach for.
3. **The brief's metal-0 rule** ("when the metal store reads 0 in the first three minutes the commander's own build
   is the thing to drop") is scoped in time. Fable inverted it at 4:19 ("assisted plant starves the constructors'
   extractors"). That contradicts the role text's unscoped "when it does, the plant is starving", so I count it as
   judgement, not literal reading.

## Cheap discriminating tests

1. **Opus 5.5 against BARb medium with `--opponent-opening vehicles` on today's code** (seed 1, `--max-minutes 30`).
   This separates the opponent from the player directly. If extractors at 8:00 come out near 10-12 and hunts are
   forbidden from the first "no detachments" packet, the gap is the matchup plus the forbidden mark. If they come out
   near 15-17, the remaining gap is Fable's judgement.
2. **Fable 5.1 against hard_aggressive with `--opponent-opening bots`**, seed 1. Fable on the matchup the harness was
   tuned on. Compare to player-33's 22 extractors at 8:00 and to player-29 to 32.
3. **Offline, no game:** replay fable-2-medium's recorded gates from 4:19 to 8:00, with Fable's 4:19 packet rewritten
   to "a party of Blitzes is met by four or more together; a lone Rascal is left to the turrets". Does the forbidden
   share fall, and do the hunts offered catch anything? This tests whether the mark or the doctrine is what took the
   hunts away. `run/jev_forbid_ab.py` asks the question but has no packet override today; it needs a small
   `--packet` option, or use `run/jev_ab.py`'s variant mechanism.
4. Check (no game): player-28's first constructor lists, for where the turret stood in the list, before blaming
   Fable's turret order.

## Least sure

- Every share above rests on one game per cell, and BARb medium's raider mix varies between games (Rascals here,
  none in player-27/28).
- player-27/28 ran 23 commits earlier. The opening turn and produce-as-sequence change the first four minutes, so
  "Opus against medium vehicles" is not a clean control.
- Whether the commander's absence cost anything net: plant output over 1-8 min was level with the Opus vehicle
  games, while 5-7 min dipped.
- The decayed frames (killer `xmasball2_6` in the record, read here as our own decay or cancel) are an inference from
  the def name.

## Commands run

From the repository root, for M in `run/matches/1790899828-player-33-hard/00` and
`run/matches/1790899839-fable-2-medium/00`:
`python3 run/minutes.py M`, `run/analyze_match.py M`, `run/floor.py M`, `run/raid_ledger.py M`, `run/wake_read.py M`,
`run/jev_load.py M`, `run/pro_baseline.py M` (no carded side for B8/G1 as arm), `run/commander_turns.py M --told
--calls`, `run/commander_turns.py M --notes`, `run/hands_window.py run/matches/1790899839-fable-2-medium/00 6:00 7:00
plant_5440 commander`. For player-26/27/28 (`.../00`): `minutes.py`, `analyze_match.py`, `raid_ledger.py`,
`wake_read.py`. The rest came from throwaway scripts over `record-0.jsonl` (plant output and constructors by minute,
buildings' fates, energy-empty seconds), `strategist-0.jsonl` (packets, tool counts, wake settings, "taken as done"
report lines) and `jev-0.jsonl` (forbidden nouls by 15 s, hunt state words, commander plays, group paragraph matches).
