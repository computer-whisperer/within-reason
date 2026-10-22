# The episode sweep: index

Written 2026-10-22 from the 57 rows in `rows/` (Opus 5.5 readers, one per game; `episodes.jsonl` is the rows concatenated). Outcomes: {'lost': 43, 'won': 12, 'aborted': 2}. Reader confidence: {'high': 10, 'medium': 46, "high for the sequence and the fight (the record and the hands' log cover the whole 3.5 minutes); medium for the ranking, since the energy item is the reader's reading and the root follows the user's": 1}. Mistakes by kind: {'strategy': 199, 'both': 187, 'harness': 60}. 52 of the user's in-game comments carried; 261 open questions.

## The games

| game | date | map | shape | result | root (the earliest minute the game left the winning path) | conf |
|---|---|---|---|---|---|---|
| human-1 | 2026-09-22 | Quicksilver Remake 1.2 | 1v1 (a second person advising in chat) | lost 5:30 (the us | **0:11-0:34** The opening. The 0:11 packet said 'build constructors until we have a couple, then Maces and Pawns about two to one' and sent constructors outward through spot_16/20/21/17 with no soldier or scout; the lab took three constructor p | high |
| human-2 | 2026-09-21 | Quicksilver Remake 1.2 | 1v1 | lost 3:40 (comman | **0:11** The opening put a lab before any generator and three constructors before any soldier. The hands' default text built the lab at 0:12 (Jev pick 0.46, before the player's first packet landed), and the first generator came at 0:45. Th | high |
| human-3 | 2026-09-21 | Quicksilver Remake 1.2 | 1v1 | lost 4:00 | **0:36** The hands put the lab down before any generator, before the player's first turn, and the player's first packet then put constructors first. The store emptied at 0:46 and stayed empty to 1:54. The first unit, a constructor, took 43 | high |
| human-4 (the ledger's label; the batch was named human-2 by mistake, docs/experiments.md:243) | 2026-09-21 | Quicksilver Remake 1.2 | 1v1 | lost 4:26 | **0:11-0:33** The lab was placed at 0:11 with no generator standing: the hands played the default text ('extractor, then the lab, then two generators') before the player's first packet, and the 0:12 packet asked for three extractors before a so | high |
| human-5 | 2026-09-22 | Quicksilver Remake 1.2 | 1v1 | lost 5:03 | **0:53** The lab picked a second constructor while the first was 3 s into its build. The 0:33 packet had asked for 'a second constructor when we have only one', and the lab's words read '0 constructors and 1 more being made'. That construc | medium |
| human-6 | 2026-09-21 | Quicksilver Remake 1.2 | 1v1 | lost 6:10 | **0:59** The lab's second constructor. Jev's question told it 'We have 0 constructors already and 1 more being made', and it picked corck (0.79) against the packet's 'one constructor (corck) first, then raiders'. At 1:25 it picked a third  | high |
| human-7 | 2026-09-21 | Quicksilver Remake 1.2 | 1v1 | lost 6:13 | **0:59** The lab chose a second constructor at 0:59 and a third at 1:24 (a coin flip), against the packet's 'one constructor first, then raiders'. The hands could not count from the words, and produce allowed corck with no cap. The energy  | medium |
| human-8 | 2026-09-22 | Quicksilver Remake 1.2 | 1v1 | lost 3:31 | **2:03** First contact at D4. The hands answered a lone raider with a pair (the opening packet's 'detachment of two or four'), newcomers from the lab formed groups of their own, and from 2:22 the pair group_C chased one Pawn out to G6. The | high for the sequence and the fight (the record and the hands' log cover the whole 3.5 minutes); medium for the ranking, since the energy item is the reader's reading and the root follows the user's |
| human-9 | 2026-09-22 | Comet Catcher Remake 1 | 1v2 | lost 12:55 (the r | **2:00** The economy left the path in the opening. The 0:00 packet put two extractors and five solars on the commander. The default added one more solar in the 22 s before the packet landed, so four stood before the plant (started 1:01; po | medium |
| bluegecko-2v1-great-divide | 2026-09-26 | Great Divide V1 | 2v1 | lost 15:28 | **0:00** The first turn fired at frame 1 with one seat published. The player took itself for the south-east Cortex seat alone and wrote one Cortex opening keyed 'commander', which both seats share. No list step played on either seat before | medium |
| bluegecko-3v1-comet-catcher | 2026-09-27 | Comet Catcher Remake 1 | 3v1 | lost 21:21 | **4:30** The player's (strategy, amplified by the harness): with 22 soldiers (2,104 metal) across the seats against 5 known extractors of his, the 4:30 packet split the army into three roles. group_A_t1 raided the north edge alone, and gro | medium |
| bluegecko-3v1-comet-catcher-2 | 2026-09-26 | Comet Catcher Remake 1 | 3v1 | lost 20:27 | **5:15** The first lead (3.3k of Blitzes and Rovers across the seats against 31 seen at 5:00, the trade still even) was turned on his commander at his plant: 5:15 'THE KILL ... attack_unit the commander with every soldier', the hands' 5:19 | medium |
| bluegecko-3v1-comet-catcher-3 | 2026-09-27 | Comet Catcher Remake 1 | 3v1 | lost 26:18 | **7:54** The player split the side: t2+t3 were 'the ball' and t1's ground army (26 Pawns and Maces, 1.6-2.9k) stayed home 'guarding our northern extractors from spot_16' (said again at 8:48 and 9:32). t1's group then held at F2 from about  | medium |
| bluegecko-3v1-comet-catcher-4 | 2026-09-27 | Comet Catcher Remake 1 | 3v1 (three seats of ours against one person; a second person spectating and advising in chat) | lost about 20:40: | **2:07-4:10 (the spectator saw it at 3:58)** Against an opponent who had announced a greedy opening, the player turned the first soldiers of every seat into pickets on our own extractors (stations from 2:07 and 2:58), barred the commanders from the middle columns (3:13) and  | medium |
| bluegecko-3v1-comet-catcher-5 | 2026-09-27 | Comet Catcher Remake 1 | 3v1 | won 9:35 | **5:00-7:19** The game never left the winning path. The earliest slack on the chain to the result: from 5:00 every seat held 2.7 to 3.7 times the army it saw (3,170 metal against 330 seen, combined), yet the mass stayed in our half until 7:19-7 | medium |
| bluegecko-3v1-comet-catcher-6 | 2026-09-26 | Comet Catcher Remake 1 | 3v1 | lost about 17:05  | **6:45** His commander and a new forward plant were found at D4 just after his Incisors died (6 at 6:35). We had about 7k of soldiers, 60 of 78 of them light, against one person. The player chose to gather all three seats at a place in our | medium |
| bluegecko-3v1-comet-catcher-7 | 2026-09-27 | Comet Catcher Remake 1 | 3v1 | won 13:59 | **7:00** The game never lost the lead (seen floors only); the root is where the quick close was given up. At 7:00 his Blitz army was spent (the player's note: ~40 killed, his commander at B4 with 3 constructors), and the player ordered gro | medium |
| bluegecko-3v1-comet-catcher-8 | 2026-09-26 | Comet Catcher Remake 1 | 3v1 | won 9:13 | **7:01** None. The game never left the winning path: his commander died at 9:13. The earliest costly detour came after purple's lone north raid lost ten Incisors to his C4 block and the Beamer at C1 (6:43-7:02). The hands' fall-back then w | medium |
| bluegecko-3v1-comet-catcher-9 | 2026-09-26 | Comet Catcher Remake 1 | 3v1 | lost 20:16 | **7:10** The first window (7.1k army against 1.3k seen, his raid block broken at C6, our groups in his half at B1 and C6, our half nearly full by 8:16) was held for 'a later push on his base' while every constructor stayed east of the midd | medium |
| bluegecko-3v1-cape-violet | 2026-09-26 | Cape_Violet_V1 | 3v1 | lost 18:18 | **3:56** At 3:56 the player read 'army 2.7k vs 54 seen' and ordered the three groups to stand together at the D2 choke and hold, advancing only once they had gathered. His base had been unseen since 2:41-2:53, when the hands' pick sent eac | high |
| bluegecko-3v1-comet-catcher-10 | 2026-09-28 | Comet Catcher Remake 1 | 3v1 | lost 26:42 | **3:17** The first scouts reached the picture's 'his start' (D2, the lobby box A1-H2, which holds our own G1 home) and found nothing; a Rascal pair drove row 1 to B1 by 3:32 and turned back. From here every rover goal stayed in rows 1-2. T | medium |
| bluegecko-3v1-comet-catcher-11 (the label is wrong: the map was SailAway 2) | 2026-09-27 | SailAway 2 | 3v1 | lost 24:33 | **0:00** The opening packet made hovers the army on a 69%-sea map: a land hover platform in every commander's list, 'Goons and Caymans about two to one', groups 'never north of row 5', and no shipyard, sonar tower or torpedo launcher on an | medium |
| bluegecko-3v1-comet-catcher-12 | 2026-09-27 | SailAway 2 (the direct | 3v1 | lost 14:47 (resig | **2:21-2:34** The side's economy was capped in the opening. Every seat had spent 450 on a shipyard first (1:09-1:23), and its first units were construction ships. At 2:21-2:33 the hands refused all their extractor steps (harness: no ship distan | medium |
| human-10 | 2026-10-02 | Gecko Isle Remake v1.2 | 3v1 | lost 23:45 | **5:50** The raid was beaten and the army stood at 4.9-20.7x what we saw of his. The lead was turned into neither ground nor extractors: the push ordered at 5:50 (spot_42/35/38, then north) stayed in our half until 10:09 and then met the o | high |
| human-11 | 2026-10-02 | Avalanche 3.4 | 1v1 | lost about 12:34  | **0:10** The player's second turn replaced the pre-game list already in force (two extractors, three solars, lab, assist, a third extractor) with seven wind turbines and no third extractor; the commander built turbines and the lab, and the | medium |
| human-12 | 2026-10-02 | Avalanche 3.4 | 1v1 | won 17:33 | **5:08** With the first lead (6.2x his seen army at 5:00, the player: 'He plays air ... with few ground soldiers'), the player's orders barred his base and later gated the push on a count. 5:08: 'never goes to spot_1 or spot_2 ... without  | medium |
| human-13 | 2026-10-03 | Quicksilver Remake 1.2 | 1v1 | won 14:57 (his c | **2:00** The game stayed winnable throughout; 2:00-4:00 is where it left the short path. The first window (2.1-2.4x his army as seen) ended at 4:00 with none of our army in his half. The brief's Quicksilver guard rule kept the Grunts as pi | medium |
| human-14 | 2026-10-03 | Altair_Crossing_V4.1 | 2v2: one seat of ours beside the user's near-idle base, against two BARb medium (the user: 'Your goal is to win the 1v2') | aborted about 11:04. | **3:20-3:37** The first handful had reached the Armada lab at 2:40, the bank stood at 1,400 and income was 12 a second. The player chose build-up over pressure: a third lab (3:20; the second was started at 2:35) and three nano turrets (3:08), a | medium |
| human-15 | 2026-10-03 | Comet Catcher Remake 1 | 3v1 | won 10:20 | **4:32** Nothing ever put the win at risk; this root marks where the game's cost and length were set (the reader's reading, no person commented). With the side's army at about 3.5x his seen, the packet sent each Blitz group to spot_33 (D4) | medium |
| human-16 | 2026-10-03 | Cape_Violet_V1 | 3v1 | lost 26:06 (where | **4:02** The player saw his lab on the B3 passage, the only land way west, and planned to break it by land with tier-1 tanks. The 'shipyard decision at ~2:30' from 0:05 lapsed, although the brief in force said to build a shipyard by 4:00 a | medium |
| human-17 | 2026-10-03 | Coast To Coast BAR v1. | 3v1 | lost 15:51 (the s | **0:05** The plan for the sea map gave each seat its own factory on its own economy: the two Cortex commanders got three winds, a shipyard and no extractor step, though a commander is the only builder that reaches all 24 spots. The sea spo | medium |
| human-18 | 2026-10-03 | All That Glitters Exte | 3v1 | lost 32:19 | **9:28 (inside the 8:00-12:00 window)** Reader's reading, on the user's 5:11 line: with the first lead (seen army 4.8-28.6x, 91 extractors), the 9:28 packet kept every new group home ('never north of row 6') and left denial of his spread to Blitzes aimed at his base sid | medium |
| human-19 | 2026-10-03 | Comet Catcher Remake 1 | 2v1 | won 15:23 | **8:29-8:47** Both seats fought the first in-step fight (7:57-8:34 C4, 665 lost for 1,545), then split. t2 stopped at meet_west at 8:29; t1's 8:47 packet sent its body on to his_base with orders to attack his commander. The commander was in a 6 | medium |
| human-20 | 2026-10-03 | Comet Catcher Remake 1 | 1v1 | lost 11:11 | **2:44** The first six Blitzes (made 2:16-3:06) were turned from the raid into a home picket by the player's 2:44 and 3:03 packets while the hands split them into hunts of one or two after Rovers they cannot catch; the window of 3:00-4:00  | medium |
| human-21 | 2026-10-03 | Comet Catcher Remake 1 | 1v1 | lost 17:44 | **3:51** The player's 3:51 packet (repeated at 4:11) was written with a Blitz at H3 in the report and the commander's 3:20 'Keep a turret or two at home against his Blitzes' in it. It left home to group_D, two Incisors, told the seven-Inci | high |
| human-22 | 2026-10-03 | Isidis crack 1.1 | 2v1 | lost 18:40 (surre | **0:41-3:51** The opening set both seats on a slow, unpressured economy: 'two constructors first, then raiders only' (seat 1 0:41, seat 0 0:56) held two builders a seat until 4:51, and at 3:15 (seat 0: Pawns gather at spot_34 'only when about t | medium |
| human-23 (first game) | 2026-10-04 | Comet Catcher Remake 1 | 1v1 | lost about 11:00  | **4:15-4:22** The pressure was pulled home. The commander had told the Incisors to leave the rove and gather (3:10, 4:08: 'put them with the new Incisors at meet_sw'); the player's 4:15 packet took group_A off rove and told it to 'join group_B  | high |
| human-23 (second game) | 2026-10-04 | Comet Catcher Remake 1 | 1v1 | lost 14:13 (infer | **2:02** His first Rover stood in our base with nothing armed at home but the commander: the player had put every early soldier on rove (1:11, 1:31, against its own 0:00 'two stay home against raiders') under the commander's 'cross the map | medium |
| human-25 | 2026-10-06 | Comet Catcher Remake 1 | 3v2 | won 24:22 | **4:48-5:48** The first lead was spent piecemeal and the plan split the side: go_base at 4:48 sent t3's and t4's Blitzes by B5 without the gather (t2 arriving later from the north), and at 5:48 the commander chose the far base (A3) first with t | medium |
| human-26 | 2026-10-06 | Comet Catcher Remake 1 | 2v2 | lost 10:07 | **1:51** The people's Rover opening reached homes with no turret and with their soldiers sent elsewhere. At t3 the player's rove (ordered 1:51, landed 1:57) took its two Rovers off the first raid; spot_77, a constructor and spot_65 died 2: | medium |
| human-27 | 2026-10-07 | Comet Catcher Remake 1 | 1v1 | lost 13:48 | **4:48** The constructors were tied to home. From the 4:48 packet on, a constructor with an enemy in reach walked to the commander, with his raiders in our half every minute and lone Blitzes parked at outer spots. From 5:56 the player also | medium |
| human-28 | 2026-10-07 | Comet Catcher Remake 1 | 1v1 | lost 15:26 | **4:16** The reader's reading (no person reviewed this game). After the first raid body died piecemeal (2:35-4:17), the player brought every Blitz home as one home guard at spot_11 and gated the raid on eight or more, against the commander | medium |
| human-29 | 2026-10-09 | Comet Catcher Remake 1 | 1v1 | won 18:03 | **8:22** The game never left the winning path; this root marks where the win was delayed by about six minutes (the reader's reading, no person commented). At 8:22 the player made plant_20622's whole output a home picket at spot_18, while t | medium |
| human-30 (no ledger row; the label is shared with the later batch 1791748300-human-30, a different game) | 2026-10-09 | Comet Catcher Remake 1 | 1v1 | won 19:37 | **6:31** After the raid's even Blitz trade at his C1 turret, the player pulled the body back to hold and gather at spot_1 in our half ('does not go west of spot_1') until it had about six Stouts. We had income 32 and estimated his at half  | medium |
| human-30 (the second game of that label, on Eclipsed; no ledger row, the impedance study calls it 'human-30 Eclipsed') | 2026-10-11 | Eclipsed 0.9.8 | 4v2 | lost about 21:45  | **5:15 (its cost 6:12-7:23)** The commander spent the early lead (side army 3,604 against 330 seen at 5:00) by gathering four seats' raiders at meet_d5 (3000,4100), inside the reach of his D4 Beamer and beside his Pawns at C4/C5, with no lever for one seat's g | medium |
| human-31 | 2026-10-11 | Comet Catcher Remake 1 | 3v2 | lost 20:03 (surre | **9:20** The last cheap moment, spent: the base was found, his tier 2 was not yet seen, and the side's army was about 3.7-5.9x what we saw of his. The commander named it ('Now is the moment, before his tier 2 comes') but sent only t4's 7 B | medium |
| human-32 | 2026-10-11 | Flats and Forests v2.2 | 3v2 | lost 27:20 | **7:50 (the window opened at 6:00)** The first lead went west late and in pieces. From 6:00 the side held 2-9x the army it saw, at home against the Rovers. The commander's go_west at 7:50 sent bodies on three routes: t4's chased Incisors in our half (17 to 4, first s | medium |
| human-33 | 2026-10-12 | Gecko Isle Remake v1.2 | 4v2 | lost 22:45 (the c | **8:00-8:44** The side's extractors stopped at about 48-50: his armed extractors and towers held the middle, and the Grunt body held t3's lane. The side had a 1.7k bank and spare energy, and nobody teched. The commander expected his tier 2 at a | medium |
| human-8v8-1 | 2026-10-18 | Altored Divide Bar Rem | 5v5: our 2 seats + 3 Apex v0.3.0 seats (all five at +50% income) against 5 people | won 57:38 | **4:30** This is the earliest minute the short win was lost. t9 had 6 extractors and 11 constructors, and the Apex allies held the spots nearby. It stopped making constructors, put the spare ones on wind and reclaim, and switched both labs | medium |
| human (Azurite Shores 3v2; no ledger row) | 2026-10-18 | Azurite Shores 1.0.2 | 3v2 | lost 39:42 | **12:55-15:47** The first big lead in his half was spent on his outer spots and not on his base: go_north (12:55) took B4 and A4 (13:26-15:18) as the plan ordered, his tier 2 came out at A3-A4 (advanced constructor 14:22, Sumos 14:46) and at 15:4 | medium |
| 1792369310-human (no ledger row; the second Azurite Shores game of 2026-10-18 evening, after 1792366754-human) | 2026-10-18 | Azurite Shores 1.0.2 | 2v1 | won 18:32 (he re | **9:30** The game never left the winning path (a win); the earliest avoidable delay of the win (reader's reading): with his tier 2 just seen and many of his Garpikes on our shore, t2 sent its 7 Pincers onto his commander at 9:27 before t1' | medium |
| 1792370678-human | 2026-10-18 | Cape_Violet_V1 | 2v1 (his side also seated an InactiveAI) | aborted 3:15 | **before 0:00 (launch, 20:44:38 EDT)** The game was launched on the ~/.claude2 account with no five-hour room left. The Azurite players' last rate events (20:44:01-05) read allowed_warning at 0.99, the +40 Apex rung running beside it since 20:01 was refused at 20:44:38 | high |
| 1792372195-human | 2026-10-18 | Cape_Violet_V1 | 1v1 | lost 33:31 (the A | **0:10** The player put off the under-water spots and never came back to them. Neither it nor the commander agent ordered a sea extractor, Muskrat, hover constructor or shipyard. Our count stopped at the land ceiling, though 14 spots no bo | medium |
| 1792459018-human (no ledger row; a 4v3 lobby game on Gecko Isle, 26 minutes before the Center Command 2v4 on the same build) | 2026-10-19 | Gecko Isle Remake v1.2 | 4v3 | lost 21:48 | **9:35-12:50** Our tier 2 stood first and the side did not turn to it, nor spend its tier-1 lead before his tier 2 came. t4's advanced plant stood at 9:35, before any sign of his (advanced constructor seen 12:20); it made seven advanced construc | medium |
| human (Center Command 2v4) | 2026-10-19 | Center Command BAR v1. | 4v2 | lost 21:05 | **4:15** The tier-2 sprint was starved from the minute it was placed. t5 put its advanced vehicle plant down at 4:15 on 4 extractors (9.4 metal a second) while the three feeding seats spent everything on tanks (stores at 0-50) and were ask | medium |
| human (Gasbag Grabens 3v2; no ledger row of its own: the SD-129 row names it as 'lost about 16:40, not reviewed') | 2026-10-20 | Gasbag Grabens 1.1.1 | 3v2 | lost about 16:28- | **3:40 (chosen at 0:02; cost 4:39-9:56)** The side went for the tier-2 sprint at t4 with no cover. The advanced bot lab was placed at F3, 800 west of t4's start and toward the Grunts' E3 path, the minute the first raid had failed in the middle. The commander paused t2's a | medium |
| human (SD-129 2v3) | 2026-10-20 | Project SD-129 v1.0 | 3v2 | lost about 15:01  | **0:01 (the commander's plan; still avoidable until 4:43, when the feeders' plants went idle with his Rascals already on t3's spots)** The plan sprinted t3 on a clock ('around 4-5 min'), not on the brief's trigger (first bank or the side's spots taken: t3 banked 56 at 4:00 and 43 spots in our half were free at 3:31), paused both feeders' plants by the brief's pro | medium |

## Lessons the games bear on (by L- id, with the relation counts)

| lesson | supports | contradicts | illustrates |
|---|---|---|---|
| L-raid-first-handful | 15 | 0 | 33 |
| L-lead-together | 35 | 0 | 10 |
| L-lead-use-it | 20 | 1 | 16 |
| L-mex-half-the-map | 21 | 1 | 11 |
| L-reclaim-field | 15 | 0 | 16 |
| L-mex-game-over | 15 | 1 | 11 |
| L-snipe-commander | 3 | 0 | 20 |
| L-other-u6bkep-nests-and-air | 16 | 0 | 5 |
| L-opening-solars | 7 | 0 | 12 |
| L-mix-pounders-mediums | 11 | 0 | 8 |
| L-raid-naked-mexes | 5 | 0 | 13 |
| L-opening-commander | 4 | 0 | 13 |
| L-tier2-tier1-under-tier2 | 7 | 0 | 9 |
| L-energy-nanos | 13 | 0 | 3 |
| L-energy-converters | 3 | 1 | 9 |
| L-other-infern8-defences | 6 | 0 | 7 |
| L-mix-adversarial | 2 | 0 | 10 |
| L-tier2-go | 6 | 0 | 5 |
| L-other-quicksilver | 7 | 0 | 3 |
| L-placement | 1 | 0 | 9 |
| L-opening-idle-lab | 4 | 0 | 4 |
| L-raid-keep-moving | 3 | 0 | 4 |
| L-water-tidals | 1 | 0 | 6 |
| L-raid-rovers | 1 | 1 | 5 |
| L-mix-arty-not-the-body | 4 | 0 | 2 |
| L-chat-no-faction | 0 | 0 | 5 |
| L-other-infern8-transition | 2 | 0 | 2 |
| L-mix-hovers | 2 | 0 | 2 |
| L-opening-solar-cap | 2 | 0 | 1 |
| L-energy-priority | 0 | 0 | 2 |
| L-reclaim-map | 1 | 0 | 1 |

## Backlog subjects the games bear on

- 43 Going and the lead: a lead decays, one body from a gathering
- 37 The raid answer: who answers a raider, pickets, the turret w
- 35 The commander's conduct: fights at home, builds at the plant
- 35 Pressure from the first units: the first handful crosses the
- 34 Reach, turret lines and nests, the odds model (a turret at t
- 30 Expansion and income: constructors by the clock, the midline
- 27 Energy in the opening: solars before and with the plant, the
- 24 Unit matchups and mixes: counters, the Blitz and the Stout, 
- 22 The bank and build power: what a bank means, what spends it 
- 22 The tier-2 sprint: trigger, conduct, feeding, the team proce
- 22 Ahead on the trade and lost: the 3v1 pattern
- 20 Answers that take minutes start on earlier signs (air)
- 11 Water maps: what reaches a sea spot, the islet's economy, th
- 9 The Quicksilver packet from Matt's line
- 5 The nuke and the anti-nuke: the silo's clock, the anti-nuke'
- 2 The scenario variables and scaling by the map
- 1 Great Divide: the pass, the camp, fire over it, air, the cro
- 1 Base layout: generators in the back field, plants apart

## Harness defects named, by frequency

- 1 The lab asked three times in six seconds with counts that left out queued units:
- 1 Our own chat lines came back as chat from player 3 (the AI's host) and woke the 
- 1 The lab was built beside the commander 102 from spot_10, leaving no site for its
- 1 The lobby's start boxes put the enemy start beside our home (E3, 579 away), so t
- 1 send_against offered against parties of five to eight Pawns: pairs sent four tim
- 1 The lab before any generator, and generators one pick at a time (no queue-ahead)
- 1 Our chat lines cut at 127 characters by the engine: 'What would you build next?'
- 1 A builder had no attack on the hands' menu: the commander could not be sent agai
- 1 The lab was asked again while an order waited, and its words did not count const
- 1 The hands' default text built the lab before any generator, ahead of the player'
- 1 There was one generator option, wind, chosen by the map's average: the player as
- 1 The eco line showed only the net energy flow, with no gross make or draw, so the
- 1 The lobby's start box put the enemy guess 581 from our own start: ground beside 
- 1 The lab was anchored 98 from spot_10, leaving no site for its extractor.
- 1 Three Jev answers were dropped in a burst: the worker answered its backlog in or
- 1 The commander abandoned a fresh extractor frame for a windmill pick (the hands' 
- 1 The opening before the player's first turn was the hands' own. The commander's p
- 1 Era U/E: the lab was asked three times in six seconds while its words said 'cons
- 1 The player's produce whitelist waited behind the queue: Grunts only at 2:13, the
- 1 There was one `generator` option, wind by the map's average. The player asked fo
- 1 Era A, the old hands: the commander's attack option (new in this build) chased a
- 1 The hands' default text played before the first packet and put the lab before an
- 1 One `generator` option, chosen by the map's average wind, and Jev's rules said '
- 1 The current wind was not in the picture. The player told the person 'No, I can't
- 1 The lab asked again 3 s after its first pick and queued a second constructor aga
- 1 No wake on an empty energy store. The player's 90 s timer ran through the stall,
- 1 The report printed 'unclaimed soldiers stand at: A1' in every turn although we h
- 1 The record's header recorded no people, and the archive has no script.txt, so th
- 1 Produce without counts (eras group E): the lab picked armck at 0:50 and again at
- 1 Generators one pick at a time (eras group U): commander wind 1:10, 1:18, 1:38, 1
- 1 The commander was not helping the lab: the hands rated assist_lab 0.15-0.20 and 
- 1 Old hands' detachments (eras groups U and A): newcomers in groups of one to thre
- 1 Old hands' chase (eras group A): a group chased a party to F3 while another part
- 1 Own chat as chat: the banner and the hands' line reached the player as 'player 1
- 1 A phantom in the picture: 'unclaimed soldiers stand at: A1' appeared in every re
- 1 The lab could not count words against what was being made. Allowed [corck, corak
- 1 Helping the lab was withheld from the queue-ahead asks (the guard order cannot b
- 1 The commander's attack row was offered against a party 375 away, with no limit o
- 1 The lab's words read 'building ' with nothing named, while a unit was in progres
- 1 The corak-only produce at 1:43 did not drop the constructor already waiting in t

## Open questions

- human-1: Who were player 0 (the opponent) and player 1 (the advisor), with ratings? The record and the start script carry no names.
- human-1: With the one-deep lab queue (0e6f54f) alone, the first soldier would have started at about 1:38 instead of 2:24, against Pawns at 2:07: would that have saved the field builders, or was the constructors-first packet the loss either way? Read
- human-1: L-other-quicksilver dates the first games with people 2026-09-19/20; this record's wall clock is 2026-09-21 22:34 EDT (2026-09-22 UTC). Should the block's dates be corrected?
- human-1: The player's question 'What would you build next?' (3:35) was cut by the 127-character limit; the advisor never answered it. Was any other advice lost the same way in the early lobby games?
- human-2: Who was the opponent and what is his rating? The header recorded no one; is he human-1's opponent or its advisor?
- human-2: L-other-quicksilver dates the first Quicksilver games 2026-09-19 and 20, but this one ran 2026-09-21 at 22:50 EDT (the batch timestamp; fixes a45186a and 36cbfd7 the same night). Should the lesson's dates be corrected?
- human-2: Would the commander, given an attack at 2:10, have held the first two Pawns while the lab finished? It killed six in 11 s at 3:26-3:37 while walking away, which suggests yes; unmeasured.
- human-2: Which fix ended the commander abandoning a fresh extractor frame for a windmill (1:27-1:36)? Not traced.
- human-2: Was the net-only energy figure in the report replaced by gross make and draw before the spending line of 2026-10-17? Not traced.
- human-3: Who was player 1, the opponent or a spectator? The human-2 ledger row calls the same advice 'the opponent's', and the 3:18 line 'less e trouble this game, nice improvment' shows the speaker had seen the previous game.
- human-3: Did the commander fire its D-gun at 3:37-4:00? The store fell from 685 to 6 between 3:40 and 3:55, but the record has no weapon events. If the store was empty when the last Pawns closed in at 3:55-4:00, it could not fire.
- human-3: Would the commander standing at the lab at 2:40-3:03 have held it against five Pawns? It and two turrets killed nine in 25 s later on, which suggests so (reader's reading).
- human-4 (the ledger's label; the batch was named human-2 by mistake, docs/experiments.md:243): The date: the files and commit 0e6f54f are 2026-09-21 local (23:10-23:16, -0400). docs/knowledge/jev.md dates human-4 2026-09-22 (UTC), and L-other-quicksilver's sources say 2026-09-19 and 20 for these games. Which convention does the episo
- human-4 (the ledger's label; the batch was named human-2 by mistake, docs/experiments.md:243): L-other-quicksilver's games key names 'human-2'. Two batches carry that name (1790046088-human-2 = ledger human-3, 1790046706-human-2 = ledger human-4, this game's quote). Should the key use ledger labels or batch ids?
- human-4 (the ledger's label; the batch was named human-2 by mistake, docs/experiments.md:243): K-hands-one-generator-option-was-wind says the packets at 0:12, 1:49 and 2:03 say 'solar generator'. The reader finds 'solar' only at 0:12 and 1:49; the 2:03 packet says 'generators'. A small correction to the claim's evidence.
- human-4 (the ledger's label; the batch was named human-2 by mistake, docs/experiments.md:243): Who was the opponent? The ledger says the user's lobby, and the lessons call these players unnamed. If the user remembers, the episode's people field can be filled.
- human-4 (the ledger's label; the batch was named human-2 by mistake, docs/experiments.md:243): Is there still no wake on an empty energy store, or does the queue tool make the question moot for openings?
- human-5: Who was the opponent, and what is his rating? Was he the same person as human-6's 'more grunts. 2 cons is very gready on this map'?
- human-5: Would Matt's line (second constructor after six Pawns, commander on the lab from 1:36, generators as one batch) have held against one constructor and all Pawns? This game never played that line.
- human-5: What does the ledger's 'the store never below 700' measure? Neither store in the record stays above 700.
- human-5: Would the commander fighting beside the first Pawns at home from 2:12 (K-hands-grunts-die-away-from-the-commander, human-8) have won the D4/E3 exchanges? Untested here.
- human-6: Does a new produce allowance drop a unit already waiting in a lab's queue today? Here corak-only at 1:43 did not stop the constructor picked at 1:25.
- human-6: Who was the opponent, and what was his rating? The header recorded nobody and the chat names nobody.
- human-6: With the cap and the commander on the lab (human-8's opening: Grunts from 1:20), would this raid have been held? human-8 lost anyway, at 3:30, on Grunts fighting in ones and twos, so the opening faults were not the whole story.
- human-6: The player at 3:28 and 3:43 believed the turrets cannot fire with the energy store empty. Not checked against the Guard's weapon data.
- human-7: Who was the opponent, and at what rating? The same person as human-6 (who said 'more grunts. 2 cons is very gready on this map' an hour earlier)?
- human-7: Does a produce change in today's hands take back a lab order already waiting? Here the third constructor came at 1:45, 8 s after produce allowed only corak.
- human-7: Is 'unclaimed soldiers stand at: A1' (the station placeholder [0, 0]) a picture fault that lasted beyond this build?
- human-7: Did the 4:27 packet's naming of spot_8 as ground to fight for pull group_B west, or was it the pick's own choice of the nearer party? Both read the same in the plays.
- human-8: Should the energy shortfall from 1:16 (four generators by 1:30 against Matt's ten, 451 metal banked at 2:01) rank as the root instead of the 2:03 fight? The user named the fight and the counts were level (ten Grunts by 2:43), but about ten 
- human-8: Did the commander fire its D-gun at 3:25-3:31? The record has no weapon events; four Pawns died beside it before it fell.
- human-8: Who was the opponent, and at what rating? Neither the header nor the chat names him.
- human-8: Would the rules added that night (newcomers walk to the largest group within 1,500; the commander's row at 500) have held E3 against 6-9 Pawns? No game on Quicksilver against a person has tested them since.
- human-8: Should the wake carry an energy-store condition? The player slept from 0:37 to 2:07 on a 90 s timer through the stall, and crates/bot/src/brain/wake.rs has none today.
- human-9: How many opponent seats were there, and who played them? Neither the header nor the chat names anyone. The units show one Armada and one Cortex seat, and the ledger says the user's lobby.
- human-9: Was this winnable for one seat against two people from a corner with a pool-shaped opening (7 extractors by 4:00)? With no truth file, their economy and army are unknown.
- human-9: Was the 'unclaimed soldiers stand at: A1' line in the picture (seen with no soldiers) fixed, and when?
- human-9: The spot_5 extractor stood 113 elmos from the table point (-72, +88), while the ledger measured BARb's extractors at (+40, +72) from the table at this start. Do the two offsets agree with the centroid fix, or did our placement differ from t
- bluegecko-2v1-great-divide: How big were his economy and army (no truth file; 12-20 contacts at D3-E4 never identified)? Was our 2.3-2.6x of the seen army in minutes 6-10 real?
- bluegecko-2v1-great-divide: Would both groups and both commanders converged at 4:30-5:30 have killed his commander (3,700 hp and a D-gun, with 1-3 escorts)? The lowest we saw was 2,567 at 7:29.
- bluegecko-2v1-great-divide: The backlog's episode index (backlog.md section c, group X 'the old heuristic bot') lists this game, but it was a pianist game (Opus 5 over Jev 1.13). The tag is an over-tag through a passage citing team-2v2 games, and its row 809 note shou
- bluegecko-2v1-great-divide: Seat 0's group_A held under shelling at 4:26-4:38 ('leaves party_4 and holds [rule]' while hit) before any fall-back. Was that the rule's hold beside a party with a commander? No eras-list entry names it.
- bluegecko-2v1-great-divide: When did his advanced lab start? It was never seen; the Fatboy at 14:14 bounds it.
- bluegecko-3v1-comet-catcher: Did the packet's prose move group_B_t3 at 13:01? The 12:31 trigger list named spot_18 (F3), and the block reached F3 at about 12:57. Or was it the hands' own threat answer? The decompression review says no packet rule reached a tagged actor
- bluegecko-3v1-comet-catcher: His economy and start are unknown (no truth file). Was the game already lost on extractors before 11:30, as L-mex-game-over would say? A replay read (run/replay_match.py on the game's .sdfz, if one survives) could settle his extractor curve
- bluegecko-3v1-comet-catcher: Was the north raid's 70 s stop at C1 (6:39-7:49) a light turret at spot_9 that the packet forbade attacking, or the old hands' fight_to failing to advance?
- bluegecko-3v1-comet-catcher: raid_ledger reads 'answered 0' on all three seats while floor gives react_s 2-3 s: is that the tool missing tagged party names, or were the raids truly unanswered?
- bluegecko-3v1-comet-catcher-2: His economy: 11 of his extractors seen, none after 10:12, his advanced plant never seen; were our three seats' 35 extractors and 75 income at 14:00 ahead of his or level?
- bluegecko-3v1-comet-catcher-2: Did the triple-counted trade (13:58 '33k:19.7k', 16:16 '56.7k:33.5k') shape the choice to park and rebuild rather than push? The notes cite it at both turns; only the transcript's reasoning could tell.
- bluegecko-3v1-comet-catcher-2: Was the stale march goal (members fighting toward an old station under new ones) fixed specifically, or only removed with the hands rebuild of 2026-10-02?
- bluegecko-3v1-comet-catcher-2: games.jsonl gives our_seats 2 for this batch while the header has three seats of ours; other games of the era may be undercounted the same way.
- bluegecko-3v1-comet-catcher-3: Why were engage_party orders refused on party names the hands carried live (11:34, 11:45, 17:31)? Not traced in either earlier review.
- bluegecko-3v1-comet-catcher-3: Did `constructors solar: never` (6:39-16:15) bind every seat? t2 built no solar 6:31-17:09 and stalled, while t3 and t1 kept building solars; was t2's stall that rule or its constructors' jobs?
- bluegecko-3v1-comet-catcher-3: His true army and economy at 10:45 (no truth file): with the deduplicated trade level, was the 9.4k against ~1k seen a lead at all?
- bluegecko-3v1-comet-catcher-3: Would t1's 2.0-2.9k at D4/E4 in 11:34-14:00 have turned those fights, as the spectator says? Untested.
- bluegecko-3v1-comet-catcher-3: Did the hands rebuild of 2026-10-02 make an air scout an actor with a route?
- bluegecko-3v1-comet-catcher-4: How many extractors did Irishstud14 hold at 4:00-8:00? There is no truth file; thebluegecko's 3:58 line is the only measure.
- bluegecko-3v1-comet-catcher-4: Why did group_B_t3's raid stand at F4 from 5:13 to 6:28 under 'fight_to spot_39': attack-move on the Rascals, the group waiting for the units the plant fed it, or the route?
- bluegecko-3v1-comet-catcher-4: Would the 5:53 army (3.3k against about 260 seen) sent at the two corners, as advised at 6:45, have broken his economy before the Pounders came at 8:10?
- bluegecko-3v1-comet-catcher-4: Did the triple-counted trade line (57k against 23.5k at 14:30) push the player to commit all three groups to the commander at 11:59 and accept the south's risk?
- bluegecko-3v1-comet-catcher-4: Who typed 'still 84 apm, to do this' (20:24), and about whom?
- bluegecko-3v1-comet-catcher-5: Was thebluegecko's 'You are down in metal income' (6:49) per seat or for the team? Our report showed 53 income from 24-26 extractors across the three seats; his count is unknown without a truth file.
- bluegecko-3v1-comet-catcher-5: What held the strike forces at 6:29-7:02? Seat 2's group read 'advances to its station spot_39' while only two members got fight orders at 6:49. The ledger says a stuck straggler; the march rule is the reader's guess.
- bluegecko-3v1-comet-catcher-5: Would a push at 5:13 (3.6k against 330 seen) have found D4 before its turret nest stood? The first of his turrets seen in the minute rows is at 8:00.
- bluegecko-3v1-comet-catcher-6: How many extractors did he really hold? Only 9 were ever seen, and the posing review estimates about 24k of soldiers fielded by 15:00, which is more than 8 extractors pay for. Reclaim is unaccounted.
- bluegecko-3v1-comet-catcher-6: When did his tier 2 start? Reapers were first seen at 17:00, and there is no truth file.
- bluegecko-3v1-comet-catcher-6: Would an immediate strike at 6:45, or three separate raids on his outer extractors (which need no coordination between seats), have won under the defect that each seat priced its own group alone?
- bluegecko-3v1-comet-catcher-6: The ledger row records the opponent as Armada (the lobby's side), but he played Cortex. Should the row be corrected?
- bluegecko-3v1-comet-catcher-6: Did the reclaim instruction of 11:26 fail in the decompression (P11 gave follow_list) or in the constructors' queued lists? bot.log shows 0 at work all game.
- bluegecko-3v1-comet-catcher-7: Did the game end on the commander kill at 13:59 (commander ends) or on his disconnect at 14:11? The record has armcom destroyed at 13:59, and both plants died 14:01-14:08.
- bluegecko-3v1-comet-catcher-7: How many extractors did he hold at 5:00? thebluegecko said he out-expanded three seats ('3v1 is about fair'); with no truth file, only 5-8 were seen.
- bluegecko-3v1-comet-catcher-7: Could the opponent read our player's chat (our extractor count at 5:09, the corner raid at 5:42, 'Sending our 22-Incisor group south' at 8:53)? The record does not give the chat's audience.
- bluegecko-3v1-comet-catcher-7: Why did group_A_t1 stand at C1 from 6:19 to 7:13 with fight_to spot_9 and no party in the picture: terrain (the player's reading) or pathing into the north corner?
- bluegecko-3v1-comet-catcher-7: Would the 15 Incisors at C3 at 5:43 have done better at his base than the 24 that arrived at 7:50? His commander was home both times.
- bluegecko-3v1-comet-catcher-8: Was his commander killed by our fire or self-destructed? The records show its destruction at 9:13:00, with its blast killing 17 of ours in the same frame, but not the cause; the demo would settle it.
- bluegecko-3v1-comet-catcher-8: What exactly froze purple's group from 8:05: the march rule stopping the strung-out leaders while the body held at the yard (this row's reading of the cmd rows), or something at the yard? The march is deleted, so it bears only on the episod
- bluegecko-3v1-comet-catcher-8: Would three groups at B4 from 8:05 have ended the game sooner, with fewer raiders lost to the commander's weapon?
- bluegecko-3v1-comet-catcher-9: Where was his base? His commander was seen once, at B5 at 2:35, and no factory of his was ever seen. What were his extractor count and income at 13:00?
- bluegecko-3v1-comet-catcher-9: Was the 6:16 pick (21 Blitzes walked back from 4 Rovers and 10 Blitzes at C6) a misjudgement, or right given what stood unseen behind them?
- bluegecko-3v1-comet-catcher-9: How much of our 29k lost west of the middle, and of his own wrecks, did he reclaim?
- bluegecko-3v1-comet-catcher-9: t3's income reads 16.9 flat with 8 extractors from 11:00 to 18:00: a stalled seat, or a sample artifact?
- bluegecko-3v1-comet-catcher-9: Were the 35 'site was bad' extractor refusals his extractors on our middle spots, or our three seats' lists colliding?
- bluegecko-3v1-cape-violet: Would a crossing at 4:00-5:00 (about 30 Blitzes against two Pawns, an LLT, a Beamer being built and his commander) have killed his lab and constructors, or did the D-gun make it a trade?
- bluegecko-3v1-cape-violet: Was his income jump from 68 (8:00) to 273 (10:00) the reclaim of our 6.7k at B3? The card does not separate reclaim.
- bluegecko-3v1-cape-violet: Why did the pick send every Rover home at about 0.6 on sighting one constructor and one Pawn, 700 from his start? Was it the old scout state ending on any sighting? Review item F6 names the same at player-9.
- bluegecko-3v1-cape-violet: With the sea spots listed, would the Beavers have taken them before his construction ships arrived (8:12 on our side), and would the player have sent the amphibious commanders?
- bluegecko-3v1-cape-violet: spot_12 is an island (+73), not under water, yet seat 2's Beaver skipped it as 'not free' too. Was that the same defect, or reach by class?
- bluegecko-3v1-cape-violet: The backlog's episode index tags this game with group O. The 'without reach' half was already in this build (d265666 is H-HANDS-ODDS-REACH); what bit was the odds by metal and tier (afc08b6).
- bluegecko-3v1-comet-catcher-10: Where was his base? It was never seen. His extractors at A4 (seen 12:56) and B3 and the raid routes (Incisors at C6/D5 at 8:03) point to the west middle or south-west (A4-B5), but there is no truth file.
- bluegecko-3v1-comet-catcher-10: How big were his economy and army at 8:00-12:00, when we held 9-17k against at most 900 seen? Would the 9k ball sent west along rows 3-5 at 8:31 have found and broken his base before his Pounders and drones massed?
- bluegecko-3v1-comet-catcher-10: 3086b7c fixed the second frame. Was the dropped bare 'assist' step (17:39 'could not be played this second and was dropped') also fixed, by the hands rebuild or later?
- bluegecko-3v1-comet-catcher-10: The backlog's episode index tags this game with group C (the water words) and group X (the heuristic bot). Comet is a land map and the pianist played, so both read as over-tags. Group K's tag likely means the second frame, which 3086b7c add
- bluegecko-3v1-comet-catcher-10: thebluegecko's 25:37 'you are getting better': this game ran 26:42 against game 9's 20:16. Was that the opponent's reading of the scouting and mix, or of the length alone?
- bluegecko-3v1-comet-catcher-11 (the label is wrong: the map was SailAway 2): How big were his economy and army? His shipyard, seaplane platform and advanced shipyard were never seen; the first Riptide at 4:10 bounds the shipyard at about 3:30, and the first Cutlass at 10:11 the platform at about 9:00.
- bluegecko-3v1-comet-catcher-11 (the label is wrong: the map was SailAway 2): Did the water block's hover-only list (no ship or shipyard) steer the 0:00 'hovers cross water'? It is still so in briefing.rs. An offline replay of the 0:00 turn with ships listed would tell.
- bluegecko-3v1-comet-catcher-11 (the label is wrong: the map was SailAway 2): What would have held: Riptides of our own from 4:23 on two seats plus sonar towers and Urchins at each harbour? Game 12 (shipyards on every seat by 1:09) lost too, on reach per the brief, so 'a navy' alone is not the whole answer.
- bluegecko-3v1-comet-catcher-11 (the label is wrong: the map was SailAway 2): Did the fresh session at 6:48 (brief edited on disk) change the plan? The 7:24 push was the second session's, built on the carried notes; the transcript cannot separate the two.
- bluegecko-3v1-comet-catcher-11 (the label is wrong: the map was SailAway 2): Were the 27 engine refusals of cormex sites (G3, F3, H3, D3, E2 from 5:30) his extractors already standing on spots of his half, i.e. constructors sent to taken spots?
- bluegecko-3v1-comet-catcher-11 (the label is wrong: the map was SailAway 2): Is the 'spot_11 is not a place in the picture' refusal for an under-water spot before any hover constructor stands still open, as the ledger says?
- bluegecko-3v1-comet-catcher-12: His true economy and fleet at 7:22 are unknown (no truth file); the replay would say whether the side was ever ahead in anything but seen metal.
- bluegecko-3v1-comet-catcher-12: Our player's 14:47 question 'What should our navy have been?' had no answer in the chat; did thebluegecko answer it anywhere?
- bluegecko-3v1-comet-catcher-12: With the ships' field fixed and Ellysaws/Corsairs from 4:50, would three seats at about 35 income have held his Riptides, or did the opening economy cap the side regardless?
- bluegecko-3v1-comet-catcher-12: Was the 3:19 yard pause, which funded 670-metal hover platforms and 17 converters, net positive against keeping the yards on boats?
- bluegecko-3v1-comet-catcher-12: Does a yard under fire still restart frames it cannot finish (31 Corsair frames 13:13-13:36) in today's hands?
- human-10: His extractor count and when he started tier 2 are unknown (no truth file); the user's '6x on income' at 20:19 cannot be checked from our records.
- human-10: Was blue's boxed yard (13-27 Thuds stuck 11:00-17:00) the extractor-in-the-lab's-exit-lane bug fixed on 2026-10-13, or the solars constructor_29192 placed at 8:44-10:09? Has any later fix covered it?
- human-10: Has the detachment dicing of a raid answer (13 detachments in 30 s at 5:06-5:36, against an explicit 'no detachments') been fixed since, or does it still happen?
- human-10: Would going around the D4 outpost at 10:09 (u6bkep's advice) have found his base before the Tzar came at 14:05?
- human-10: How much would tidals (25) have lifted t1's energy against wind 2-16, given that its store was empty 39.6% of seconds?
- human-11: His economy and timings are unknown (no truth file): when did his Maces start, and how many extractors did he hold at 4:00 and 8:00? The replay or the user could answer.
- human-11: Was the D6 Sentry creep (9:54-10:24) meant to draw our army out before the 10:55 attack? (the reader's reading; the user played it)
- human-11: At 1:18 the places audit marked the first constructor's gate a fault (asked about spot_12 alone while spot_18/11/8 stood open at 0.58-0.64): the places layer or the openers layer? (the review's candidate)
- human-11: Why did the 12:30 world pick undo the commander's walk at 24% health with 'Under a third health it walks to spot_18' in the instructions: does the player's 'everything attacks party_7 now' outrank its own health clause for Jev?
- human-11: Would wind as the generator have been right on Avalanche's 3-19 wind had the extractor steps been kept? The finding is the count and the dropped extractor, not the choice.
- human-12: Would 23 Pawns and Crossbows going north from B4 at 8:24 (four Centurions at D3, his commander and turrets unseen) have razed his base 9 minutes earlier? No truth file, and his base was unseen until 17:18.
- human-12: Were the hands' aircraft hunts against 'never hunt or chase an armca or armkam' ever fixed (the impedance fixes of 2026-10-16/17?), and is the attack/shell/walk churn at the bar closed?
- human-12: briefing.rs:639 still appends '(a builder: tier 2 is up)' to the tier-1 armca's first sighting: a fix candidate for the main session.
- human-12: Why was spot_11 (924 from home, second on the 0:10 constructor list) still missing at 12:03 (the player's note)? Not looked into.
- human-12: Did the energy-0 minutes (11, 13, 15) stop the extractors' metal (income 9.1 with 12 extractors at 15:00), as facts/economy.md's all-or-nothing upkeep implies? Not checked second by second.
- human-13: Why did group_N1's 28 Thugs stand idle at F6 from 12:20 to 13:13 while the hands' task read fight_to spot_21? The march's form-up (fixed in 9ba2889 later that day) is a candidate; the Jev log's group_N1 frames would say.
- human-13: What were his real army and economy? With no truth file, only the engine's demo (not read) can say whether the 5.7-20.5x and 15.7-19.1x windows were real.
- human-13: Would Grunts sent across at 2:00-3:00 have paid against a person's Pawns, given that a Pawn beats a Grunt one against one?
- human-13: What killed the four Thugs at E3 at 14:52-14:55 'to something unseen' before his commander died? (The 14:57 deaths fit the commander's death blast, 350 radius, 5,000 damage.)
- human-13: Human-13 has no ledger row, so the main session never read this game. Did the user say anything about it outside the game chat?
- human-14: Why did six Pawns walk back from the Armada lab at 2:40-2:42 against 3 Pawns? It could be the packet's outweigh clause, fire from the BARb commander, or the lab's defence; with no truth file it cannot be told.
- human-14: The user's first instruction was 'defend me', then 'Your goal is to win the 1v2'. No unit of ours was ever stationed toward his base at A7, and enemies stood at B7 at 11:04. Did he mean defence to stay part of the task?
- human-14: Did the user close the game at 11:04 because he judged it lost? No chat line says so, and no result row exists.
- human-14: Was the wind-only energy plan a map-specific error (the wind range on Altair is 12 to 27), or would the same three labs and nanos have stalled on solars too?
- human-15: What would thebluegecko have done differently against the three seats? Our player asked at 10:29 and the chat holds no answer; did he answer the user elsewhere?
- human-15: The player's note says groups' 'march' held the push back. hands_window shows group_G_t1 walking back to spot_37 at 8:27 after meeting four units at C3, then shelling from 8:41. Was that the march rule or the hands' retreat rule?
- human-15: Were the exit-lane blockers (constructors and the commander, 8 wakes) building in the lane (the 2026-10-13 fix) or stuck?
- human-15: Why did no constructor take apart the wreck fields after the 6:02 and 7:30 orders: were their lists never empty, or was the reclaim option never offered?
- human-16: Why did both hover platforms stay idle for 11 minutes with a Cayman-only list (the pick kept 'nothing' at Cayman ratings of 0.15-0.35), and does a lab with a player's list still idle this way today?
- human-16: The brief in force said to build a shipyard by 4:00 and never to walk into the B3 camp. Why did the player defer the shipyard ('decision at ~2:30') and then drop it? Did the brief's water lines reach the turn, or were they outweighed by the
- human-16: How many sea spots did he hold, and when? We never saw one of his extractors, and there is no truth file.
- human-16: What killed the two shipyard frames (12:11 at (7280, 4800) and 21:48 at (5824, 1840)) out of sight: torpedo launchers, the Despot, or submarines?
- human-16: Did the player's belief that he was Armada (0:05-2:45) shape the opening (Incisors chosen to 'beat Grunts', the D2/D4 shallows route)?
- human-17: How many extractors and how much navy did he have at 8:00 and 10:00? Without a truth file it cannot be told whether we were behind on economy before the fleet fights or level and lost on reach.
- human-17: Was the bare 'spot_N is not free' skip (the ships' sea spots at 2:00-2:46, commander_t2's land spot_12 at 15:19) ever fixed, and what excluded those spots?
- human-17: Did the hands hold the Crocodiles about 600 from the Corsairs at 10:20-10:37 by a reach-keeping rule, or does the group centre hide a spread that did close?
- human-17: Would the Cortex commanders taking the six near sea spots from 0:05 have given the yards an army before his Corsairs came at 10:00?
- human-18: How large were his army and economy at 12:00-20:00? The replay (run/replay_match.py on the .sdfz) would give his truth and settle whether the 4-8x lead was real.
- human-18: Did thebluegecko or the spectators say anything about this game afterwards (relayed by the user)? None is in the chat or the inbox.
- human-18: Would a player a seat (or the commander seat) have answered magenta's 18:59-21:43 raid and gathered the bodies at 14:00, or is the guard's 37 s hold a hands matter?
- human-18: Was the Cortex seat's opening list resolving to Armada names fixed, and does the record header still take his faction from the start script?
- human-19: Did t1's 7:44 advance to spot_26 follow the packet's 7:45 clock, or was it chance? K-jev-a-clock-condition-in-the-packet-is-not-followed measured one moment of this game (5:03) as clock-blind.
- human-19: Why were both players' sessions replaced (seat 0 at 10:54, seat 1 at 13:59), with rate limits 'allowed'?
- human-19: How many units did plant_28350 lose to the advanced solar in its exit lane (wake reasons at 9:03 and 12:00)?
- human-19: What would thebluegecko have answered to 'Any advice on what we should have done better?' (15:24)? He resigned first; worth asking him for the episode.
- human-19: Seat 1's hands at 10:38-10:57: group_C_t2 was under fire from 10:38, before the 10:41 'attack now' packet, and reached yard2 (home) at 10:57. Did the hands' fall-back or the packet decide that walk?
- human-19: His true army at 8:00-10:00 (no truth file): was 9:12 really 3:1, as t1's note says, apart from the D-gun?
- human-20: Were the two Rovers recorded as killed by armcv at B4 (2:57, 3:09) reclaimed by his constructors, killed by his commander, or given a wrong attacker?
- human-20: Did group_L hold 7:50-8:02 while party_17 killed the plant under the packet's 'outweighs' clause or because the hands posed no answer (party_17 answer read 0.83)?
- human-20: party_11 was 3 Blitzes at spot_3 (F1) at 5:20 and 6 at D3 at 6:07: the same party grown, or a name passed (H-HANDS-PARTY-NAME-STAYS was built after human-21)?
- human-20: With a Sentry at spot_3/spot_11 by 2:00 and the 2:28 words kept, would four Blitzes in his half by 3:00 have survived his Rovers and the B3/B5 Sentries? No truth file to price his economy.
- human-21: Would group_B, recalled correctly at 4:27 (seven Incisors at about (5300, 730), roughly 1,700-2,100 from the home spots at speed 85), have saved spot_6 (5:12) and spot_3 (5:25), or only traded with Blitzes it cannot catch? This decides how 
- human-21: Would a Pounder-led home body from 6:18, as the player chose, have held against a person's Blitz and then Stout and Janus raids? The simulator says yes head-on, but no game of ours with people measures it.
- human-21: How much of the home wreck metal (2,800 at 13:50) did the constructors actually reclaim between raids? Nothing run here measures it.
- human-22: His economy and extractor count: never seen and no truth file; the replay (.sdfz) would say how far ahead he was at 8:00 and 10:00.
- human-22: Would seat 0's 20 Maces, on fight_to spot_46 at 8:57, or a joint push at 9:15, have broken his Exploiters and Pounders and reached his base?
- human-22: Why seat 0's 7:49 packet (fight to spot_37, spot_46, spot_45), landed at 7:55, left the body on hold at home (5040, 1959) until the 8:12 packet moved it at 8:18.
- human-22: How much of the wreck field our constructors reclaimed (picks to reclaim at C5 at 7:50-7:52) and how much his Graverobbers raised.
- human-22: The ledger's trade line (18,318 for 14,905) appears to set both seats' kills against one seat's losses; worth a correction in docs/experiments.md if confirmed.
- human-23 (first game): When did he start his advanced aircraft plant, and where was his T1 air plant (never seen)? Would a Rascal at A8 at about 4:30 have seen it? (no truth file)
- human-23 (first game): Is the 'join group_B there' -> 'follow group_B' play, which pulled seven units out of his half, possible in today's hands?
- human-23 (first game): Is 'armdrag' credited with kills a disguised Dragon's Claw reported under its disguise, and does the report still name it so?
- human-23 (first game): Why did the engine refuse eight cormex orders as bad sites (F2, F3, G4, F4, E2, E1)? A placement-geometry fault of that build?
- human-23 (first game): Are the commander seat's wakes (first tier-2 aircraft, large losses) and the wake lines' loss counts fixed since 2026-10-04?
- human-23 (first game): What did his "holy crap eco balance" (4:08) and "crap" (5:19) refer to? Nothing in our record explains them.
- human-23 (second game): How many extractors did he hold at 5:00, when the kills were about level (his 6 extractors, a constructor and 8 Rovers against our 5 extractors, a constructor and 2 turret frames)? No truth file says.
- human-23 (second game): Was there a real army lead at 6:00-9:00? minutes.py's 2.2-9.3x counts only what was in sight at each sample, while the commander read the armies even at 7:47 and the fights then traded evenly.
- human-23 (second game): Would the post-2026-10-17 hands have sent the commander's D-gun at the lone Rover at 2:26-2:34, and the whole body rather than hunts of one or two at 6:30-9:00?
- human-23 (second game): Is a re-sent produce list still restarted from its head?
- human-23 (second game): Which faction did we play in the first human-23 game? The ledger says Cortex for it and 'as the first' for this one, but this record is Armada.
- human-25: Which person held A3/B4 and which B8/C8/D8? The records carry no team on enemy units; the engine replay would say.
- human-25: Did the D8 plant stand before 9:37 and make the Stouts that killed t3's commander, or did they come from B8? No truth file.
- human-25: Would the 3,500 at D5 sent to D8 at 8:44 instead of B4 have saved t3? Reader's counterfactual, untested.
- human-25: Is the transfer tool still refused whole for one stale name, and did the 2026-10-17 chase fix change how the hands walk a commander into a party that outranges it on a packet step?
- human-26: Where was the second person's base (never seen; the commander guessed A1-A5)? Did each person raid one of our seats or both?
- human-26: Would the commander's two-Blitz picket at G1, or a Sentry by 2:00, have held t2's home against 11 Rovers at 3:33?
- human-26: Does the side commander's report still double-count kills across seats (shared.rs fight() adds each seat's line to the side's map)?
- human-26: Did the game end on t3's commander death (deathmode=com) or did the host stop it at 10:07?
- human-26: What was the 9:00 income spike (t2 +49.7, t3 +72.6): reclaim of the commander's wreck, or something else?
- human-27: What was u6bkep's economy? With no truth file, the replay is the only source; our Rascal alone killed 13 of his extractors in the A and B columns.
- human-27: Why did the roving Rascal kill nothing from 2:51 to 8:49 and then 13 extractors in under four minutes: were his corner spots taken late, or did his army leave them when the waves went to our half?
- human-27: Why did the engine refuse the extractor at spot_6 (G1) at 9:43 ('the site was bad'): a heap or wreck on the dead extractor's place?
- human-27: Would spots on the east and south strip (G3-H8), each with a Guard, have held? Lone Blitzes reached spot_25 (H3) twice.
- human-27: The commander's 5:50 note says 'the brief's "go now" does not apply while behind'. Did the brief of the time steer both the player and the commander to 'home first'?
- human-28: Would one body stationed forward at F2-E3 from 4:16 (instead of spot_11/spot_6) have held the middle spots against his packs of 5-7 Blitzes, given the hands' detachments?
- human-28: How far ahead was he on extractors at 8:00 and 12:00? No truth file; the engine replay, if it is in the user's BAR install, could be read with run/replay_match.py.
- human-28: Was his south bare already at 8:00 (would the 7:59 push to spot_61/67/62 have paid), or only by 9:30?
- human-28: Would the commander's D-gun at 12:06-12:12 have saved the plant against about 11 Stouts?
- human-28: Why did the 0:11 spot order leave out the never-raided south-east (spot_42, 48, 51, 57, 60, 65, 70, 72), and would the map sheet have offered it?
- human-29: Did the early raids and the body keep him at 4-7 extractors, or were more of his unseen? There is no truth file.
- human-29: What blocked the spot_6 home extractor (no_site from 10:59 to the end): the bombed extractor's wreck, our Nettle at (6248, 632), or plant_20622's footprint?
- human-29: Would the 10:56 assault on B4 have ended the game with the home picket (about 1.5k) and full energy behind it? This is the reader's root; no person has looked at the game.
- human-29: How close was his tier 2 to mattering? The advanced vehicle plant and an armacv were first seen at 17:29-17:30, and when they were started is unknown.
- human-30 (no ledger row; the label is shared with the later batch 1791748300-human-30, a different game): When did he start the C4 forward plant and the Stout switch: before or after our 6:31 pullback? Both were first seen at 7:30, and there is no truth file.
- human-30 (no ledger row; the label is shared with the later batch 1791748300-human-30, a different game): What was his real economy? We saw at most 22 of his extractors, yet at least 27.5k of soldiers by unique id, so was 'three to one' ever true?
- human-30 (no ledger row; the label is shared with the later batch 1791748300-human-30, a different game): Would a gathering place in his half (C2/C3) from 6:31 have beaten his Stouts before the Janus came, or was holding right after the even Blitz trade at C1?
- human-30 (no ledger row; the label is shared with the later batch 1791748300-human-30, a different game): Where was his advanced vehicle plant and when was it started? The advanced constructor was first seen at 16:25 at C4, and the plant itself never.
- human-30 (no ledger row; the label is shared with the later batch 1791748300-human-30, a different game): Did the player session replacement at 11:12 cost anything in the D3 grind?
- human-30 (the second game of that label, on Eclipsed; no ledger row, the impedance study calls it 'human-30 Eclipsed'): His economy and tier-2 clocks: the engine replay of this game appears to be in the BAR install's demos (2026-10-11_19-52-13-420_Eclipsed 0.9.8); run/replay_match.py would give their side, as it did for Center Command 2v4.
- human-30 (the second game of that label, on Eclipsed; no ledger row, the impedance study calls it 'human-30 Eclipsed'): How the game ended at 21:45: t2's commander was at 100% at H1 in the last sample. A Liche on it, or the host leaving?
- human-30 (the second game of that label, on Eclipsed; no ledger row, the impedance study calls it 'human-30 Eclipsed'): Which person played the bots (Pawns, Welders) and which the vehicles (Stouts, Janus, Rattlesnake)? u6bkep played Armada vehicles on Center Command.
- human-30 (the second game of that label, on Eclipsed; no ledger row, the impedance study calls it 'human-30 Eclipsed'): What was the unseen fire on t3's body at G6/F5 (18:40-19:25)? Cloaked snipers as t3 suspected, or the Pit Bull and Rattlesnake under radar?
- human-30 (the second game of that label, on Eclipsed; no ledger row, the impedance study calls it 'human-30 Eclipsed'): Would a 5:15 transfer to one seat, with a gathering place out of the D4 Beamer's reach, have converted the 3.6k-to-0.3k lead against the C8 lab or the E6/E7 commander?
- human-31: When did thebluegecko start his advanced vehicle plant and air plant? Neither was ever seen (coracv first seen 17:20). The replay or a truth file would date his tier 2 against our 10:50 choice.
- human-31: How many extractors did the two people hold at 10:00 against our 31? This decides whether the side was behind on economy from about 5:00 or only on tech.
- human-31: Would one body of all three seats at B5 at 9:20 have broken thebluegecko before his tier 2? This is unmeasured; the side's seen-army lead was 3.7-5.9x.
- human-31: Did the 3:20 direction 'They do not go at his commander or his Guards at C4' give up the chance to deny his forward base, which later capped t2 at 9 extractors?
- human-31: Our player asked at 18:39 what the people would have done at about 13:00. No answer is in the chat; did either person answer outside the game?
- human-32: When did thebluegecko start his advanced plant and gantry, and how many extractors did the two people hold at 12:00-20:00? A replay dump through run/replay_match.py would give their truth.
- human-32: Would one body sent at 6:30-8:00 have won in his half, given that green's block was heavier than seen?
- human-32: Has the report's wording for an unidentified radar contact ('we outweigh it heavily') been fixed since?
- human-32: Why did W1 stand at F4 from 20:38 (when the strike packet landed) to 21:21: the pick's hold, or the player's 20:43 rewrite putting 'shell the Twin Guards first'?
- human-33: When did thebluegecko place his advanced plant, and what was the people's income at 8-13 min? The engine replay through run/replay_match.py would answer it, as it did for Center Command.
- human-33: How much of the 12-14k middle wreck field went to his Graverobbers and Lazarus against our reclaim?
- human-33: Could the side have funded a sprint at 8:00? The side bank was 1,680 and t2's income 31. Feeding by transfer from three seats was never tried.
- human-33: Did the report's floors of his extractors ('48 to his 23') delay the tech decision, or did the commander's 'spend the lead first' doctrine?
- human-8v8-1: The ledger's row says our side started in the north box, but bot.log puts it at '(0, 6554) to (8192, 8192) (ours)', the south strip. Correct the ledger?
- human-8v8-1: backlog.md §c files human-8v8-1 under defect X (the old heuristic bot, 2026-09-20) and D (the enemy-start guess). This build ran the pianist on Jev 1.14.0, so X looks misfiled.
- human-8v8-1: Was the slowdown the people reported (drakcore 52:15) our order storm (10,220 commands in minute 49 from t9)? It is unmeasured, and FOLLOW_STEP is unchanged.
- human-8v8-1: Did the user's 'shields' (46:49) mean the Soteria deflectors? The ledger has them seen at E1/F1/F2 from 42:00; the main session's memory note has them first seen at 50:05-51:00.
- human-8v8-1: How many people were still playing at each stage? Under deathmode=com a dead commander does not end a player, and Legion commanders died at H2 at 13:40 and about 45:17. The record holds no departures; the ledger dates them from 44:26.
- human (Azurite Shores 3v2; no ledger row): Where were his factories? No unit saw one in 39:42 although units stood 437-650 from A1/A2 at 15:00 and 24:00-26:00; the Finch the commander asked for at 10:50 is not traced here.
- human (Azurite Shores 3v2; no ledger row): Was the root the land door (12:55-15:47, then 24:28) or the sea? The land army was rebuilt to 44.6k by 33:30 after the A2 failures; the Epochs were the loss the side never recovered from.
- human (Azurite Shores 3v2; no ledger row): Was the 15:47 step back the hands' odds reading of seven Sumos against 51 soldiers under the packet's 'outweighs it' clause, and would a commit then have won?
- human (Azurite Shores 3v2; no ledger row): Did the Apocalypse missiles on A2 (34:43) and B1 (37:40) land or meet an anti-nuke? Nothing of ours saw.
- human (Azurite Shores 3v2; no ledger row): Could t3's idle advanced lab (7:00-12:00) or a shipyard have contested the sea while u6bkep had only tier-1 floating turrets?
- 1792369310-human (no ledger row; the second Azurite Shores game of 2026-10-18 evening, after 1792366754-human): What would thebluegecko have answered to our 18:35 question ('what would you have done differently on our side?')? Nothing is recorded after it.
- 1792369310-human (no ledger row; the second Azurite Shores game of 2026-10-18 evening, after 1792366754-human): At 17:18 the armacv's armmoho order at spot_3 removed our cormex and no moho frame appeared (t1's note): the cross-faction fault, or the extractor-grid snap fixed on 2026-10-20/21 (52e985b)?
- 1792369310-human (no ledger row; the second Azurite Shores game of 2026-10-18 evening, after 1792366754-human): Did the wrong enemy_start intent before 9:20 shape any hands words ('toward our side', 'in their half') the players or Jev read?
- 1792369310-human (no ledger row; the second Azurite Shores game of 2026-10-18 evening, after 1792366754-human): Would the 9:30 strike have ended the game near 11:00 if it had waited for t1's Caymans and Garpikes and the Goons (about 3,500 west of centre at 10:00)? Reader's reading, unverified.
- 1792370678-human: Why did thebluegecko stop at 3:15? Did the user tell him out of game, or did he see the side idle? No chat line says.
- 1792370678-human: Should run/human_game.sh refuse to start, or pick the other account, when the five-hour window reads near its limit? The Azurite transcripts' allowed_warning 0.99 would have caught this one.
- 1792370678-human: Should the side say in chat when its sessions stop? The person sat through three minutes of a side playing its default text with no word.
- 1792370678-human: Is the hands' default text worth tuning as a fallback (two winds then the lab, energy at zero by 1:12; seat 2's constructors assisting instead of expanding), or is a game without players always void?
- 1792372195-human: Would the 14 east-side water spots (about +13 extractors by 10:00) have held the economy against +100% income? We have no truth file for his extractors; the commander agent's predicts ranged from 13 to 22.
- 1792372195-human: At 27:46-28:20, did the Titans' anti-air fire while our ground units were in their reach, or only after the body pulled back? The scenes for #19-#22 would say.
- 1792372195-human: Does a nuclear strike reach the player's report as a missile? The wake called it shelling by 'armsilo (a turret)', and the commander agent misattributed the 31:02 losses.
- 1792372195-human: The brief's Cape Violet plan named Armada's Beaver (armbeaver) while we drew Cortex. The player translated it to a Muskrat, but the sheet's plan is tied to one faction.
- 1792459018-human (no ledger row; a 4v3 lobby game on Gecko Isle, 26 minutes before the Center Command 2v4 on the same build): When did the people start tier 2 and how many extractors did they hold? Is this game's engine replay still in the BAR install (the Center Command game 26 minutes later got one)?
- 1792459018-human (no ledger row; a 4v3 lobby game on Gecko Isle, 26 minutes before the Center Command 2v4 on the same build): Why could t3 build four armmoho with the Armada advanced constructor it received while t5's queue refused armmoho for another one? Different paths (Jev's pick against the queue tool)?
- 1792459018-human (no ledger row; a 4v3 lobby game on Gecko Isle, 26 minutes before the Center Command 2v4 on the same build): Why did the first moho come three minutes after the first advanced constructor (10:08 to 13:05) on t4: the empty store at 14.5 a second, or the constructor's list?
- 1792459018-human (no ledger row; a 4v3 lobby game on Gecko Isle, 26 minutes before the Center Command 2v4 on the same build): What held t5's builders (122 move failures, constructors stuck at avp_yard, spot_78 and solar_field with no blocker named)?
- 1792459018-human (no ledger row; a 4v3 lobby game on Gecko Isle, 26 minutes before the Center Command 2v4 on the same build): Is the advance/walk thrash under a long-range shooter (human-10, here at C4) still open in today's hands?
- human (Center Command 2v4): In a 4v2 with about 6 spots a seat, was matching the tier-2 sprint right, or would one combined body on his E2 post and B3 lab during 7:08-10:00 (side army 2.2-2.3x) have ended it first? The review skill's F7 says the scenario decides; this
- human (Center Command 2v4): Would t5 sending its first advanced constructor out at 8:50 as ordered have helped, given that the first recipient in the commander's order was the Armada seat that could not use it?
- human (Center Command 2v4): How much of the wreck metal named in the wake reasons (up to 12,064 at t2) was ever reclaimed?
- human (Center Command 2v4): Did the 2026-10-20 fixes (standing transfer, start on a list step, cross-faction constructors) change the moho clock in a later team game? (unmeasured as of the memory)
- human (Gasbag Grabens 3v2; no ledger row of its own: the SD-129 row names it as 'lost about 16:40, not reviewed'): Was the side ever ahead in army on the spot? Our side summed 2,839 at 8:00 and 3,369 at 9:00 against 964 and 1,126 seen, but by unique id he fielded 54 Pawns, 49 Grunts and 45 Maces over the game. Without a truth file the lead is unknown.
- human (Gasbag Grabens 3v2; no ledger row of its own: the SD-129 row names it as 'lost about 16:40, not reviewed'): Would a held line first (Sentries at every spot, labs on soldiers) and a sprint at the side's 24 spots have survived two people on this map, or does a 24-spot half against two people call for no early sprint at all? The procedure's 'first b
- human (Gasbag Grabens 3v2; no ledger row of its own: the SD-129 row names it as 'lost about 16:40, not reviewed'): Which person was Cortex? The start script says Armada for both; our models assumed thebluegecko.
- human (Gasbag Grabens 3v2; no ledger row of its own: the SD-129 row names it as 'lost about 16:40, not reviewed'): The direction of 7:58 reached t3's order at 8:22: how much of the one-at-a-time defence is the commander's latency (21 s turn wall), not the players' choices?
- human (SD-129 2v3): Was the sprint wrong for this scenario, or only its cover? The brief says 'long, covered, none: the sprint starts now'; this game was long and 3v2, and the user's reading is pending.
- human (SD-129 2v3): Would t2's eleven Incisors with t4's six soldiers, commander and two Sentries have held t4's home against 8 Blitzes and 3 Stouts at 8:20? The hands walked group_L_t2 to spot_77 at 9:11 instead, apparently judging the party the heavier.
- human (SD-129 2v3): Should a seat's plea, or a seat's plant under fire, wake the commander? At this build only its timer or a person's chat did.
- human (SD-129 2v3): What was his economy? No truth file, and his extractors went unseen from about 9:30.
- human (SD-129 2v3): Has the allies line's thousandfold income figure misled a commander or player in another team game since H-REPORT-ALLIES-LINE was built?
- human (SD-129 2v3): Twenty minutes earlier the same two people beat us on Gasbag Grabens (run/matches/1792546058-human, not reviewed): same pattern or not?
