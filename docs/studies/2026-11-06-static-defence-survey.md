# Static defence: the survey (2026-11-06)

The user's observation that opened this: we use stationary attacking structures far less than our opponents, and the
people who hold off and then roll over far more BARb seats than they have do it with structures, by damage efficiency,
while their economy and tier catch up. The user's framing of the cases (2026-11-06, on a first skim of this study):
a duel is won by projecting force across the map and means defending a large conflicted perimeter efficiently; the
8v8 is a grind over the same small patches of ground, each seat defending and pushing a small cross-section of the
map, and our weakness there is that the early harassment cannot be leveraged because there is no way round the
contended area; and the asymmetric game (one seat against two BARbs or two weak people) is where the one seat builds
up under light static defences on the two opponents' inexperience. So the pools are read with the team game and the
asymmetric game first and the duel as background. Before any brain change the question is what people actually do
and what the units are. This is the survey: five pools scanned by script, the unit table, and what the repo already
says. No code decides anything new here; the proposals are at the end.

The scripts: `run/replays/defence_duels.py` (the friend's processed duels), `run/defence_pool.py` (our records and the
heuristic bots' census lines). Every number below is theirs; rerun them before quoting a number somewhere else.

## 1. The pools

| pool | what | sides | source |
|---|---|---|---|
| A. high-OS duels | public 1v1s, both players above OS 40, June to 2026-10-17, every map, processed by the friend's bar-replays tool (events with killers, a per-team `defenseValue` every half minute) | 1,054 sides, 527 games | `run/data/bar-replays/{events,matches}/` (the newest 500 of selection 3 plus the 29 Legion duels) |
| B. our carded replays | humans' games replayed through our pipeline (Comet, Quicksilver, thebluegecko's, the lobby games), both sides' records | 2,313 sides of people in 355 replayed games (254 at OS 35+, 659 at 20-35, 1,400 under 20; 745 sides without a card are not labelled) | `/ceph/.../matches-archive/*-replay-*`, `run/matches/*-replay-*` |
| C. the flagship's games | our seat's fair view: what we finished, what killed ours (`destroyed`+`by_d`), what of his died to our turrets (`enemy_destroyed`+`by_d`) | 609 flagship sides (`player` mode), 62 commander-mode, 81 pianist-only, 2,165 of the deleted heuristic bot; the opponent seen from our side in each | records under both match directories |
| D. the heuristic bots' games | WRDistilled and WRTerritory against BARb, Apex and NullAI: both sides' census once a minute (no killers) | 1,541 games: WRDistilled 595 against BARb hard_aggressive, 442 against Apex v0.5.04, the older Apex and BARb rungs, WRTerritory 36 against Apex | `engine.log` census lines |
| E. humans against many BARbs | public team games with bots (BAR's API, `hasBots=true`): the last 1,200 | listed only; one replayed as a pilot | `docs/studies/data/2026-11-06-public-bot-team-games.json` (the 1,200 listed: id, map, minutes, seats a side, winner; `-sample.json` the 40 read in full with bonus and OS), `run/matches/*-replay-defence-pilot-*` |

Pool E is the case the user described and it is nowhere in our data: the friend's catalogue holds no game with an AI
seat (0 of 14,212 duels or 2,134 8v8s), and our own lobby games have us in them. Of the 1,200 newest public bot games,
1,083 are pure humans against pure AIs, but the shape "a few humans against many more BARbs" is rare: 28 games over
15 minutes had the AIs at twice the humans or more (humans won 11). The common shape is eight humans against eight
bonused BARbs: 330 of the 1,200 over 15 minutes, humans winning 215. A 40-game sample of the games with four or more a
side over 20 minutes shows the bonus at +40 to +100 (SMRTBARb at +57 to +65), the humans' mean OS 12 to 23 with the best
seat at 17 to 40, humans winning 30 of 40. So the public record of "humans outscale bonused BARbs" is mostly even seat
counts and big bonuses, not few against many.

## 2. The team game and the asymmetric game (pools B, C, E)

**People in our replayed team games against our seats in ours** (pool B and C split by game size: a duel is two
teams, a team game more; quartiles lower / median / upper; "killed by defence" is the metal of enemy units whose
last hit was one of the seat's defences, for our own records from 2026-10-18 only):

| seat | sides | ground defence built | soldiers built | defence as % of soldiers | killed by defence | killed / defence | defence lost / defence | first defence |
|---|---|---|---|---|---|---|---|---|
| person OS 35+, team game | 132 | 720 / 2,325 / 8,050 | 6,980 / 20,072 / 54,157 | 15 | 415 / 2,537 / 9,989 | 1.1 | 1.1 | 4.0 min |
| person OS 20-35, team game | 631 | 340 / 1,970 / 5,780 | 7,336 / 21,800 / 64,596 | 9 | 86 / 1,820 / 7,686 | 1.1 | 1.1 | 3.9 |
| person under 20, team game | 1,246 | 350 / 1,890 / 5,535 | 7,058 / 19,637 / 53,455 | 10 | 52 / 1,674 / 6,609 | 1.0 | 1.1 | 3.8 |
| the flagship, team game | 244 | 630 / 1,445 / 2,690 | 7,350 / 13,913 / 27,221 | 9 | 686 / 1,565 / 2,350 | 0.7 | 1.1 | 3.7 |
| the flagship, duel | 296 | 1,190 / 2,635 / 3,770 | 7,069 / 23,894 / 46,091 | 10 | 267 / 779 / 1,365 | 0.3 | 0.5 | 3.7 |
| BARb as seen from our seat, team game | 96 | 1,185 / 11,170 / 16,475 | | | 140 / 3,893 / 6,405 | 0.1 | 0.6 | |
| SMRTBARb, the Glitters 8v8 replay | 32 | 4,970 / 19,820 / 31,970 | 43,136 / 66,711 / 88,299 | 26 | 0 / 916 / 3,025 | 0.1 | 0.0 | 3.8 |

In the team game the OS 35+ people hold 15 % of their soldiers' metal in defence and it kills 1.1 times its metal;
the flagship holds 9 % and gets 0.7. In the duel the flagship's yield is 0.3, the people's 0.8 (OS 35+ duels, 122
sides, 16 %). The bonused BARbs of the lobby games stand five to eight times our defence metal and get a tenth of
it back by last hit; it is the people's and our units they kill, slowly, and that is the grind the user describes.

**What the team-game pools cannot say** is the thing asked: how the people hold a cross-section against a trickle
and when they break out. Our carded team games are the lobby games we played in (us on one side) and three replays;
the friend's catalogue holds 2,134 all-human 8v8s above OS 30 and none ingested, and no game with an AI seat. The
public record of people against bonused BARbs (pool E) is the pool for it; the pilot below is the first game of it
read two-sidedly.

**The asymmetric game.** In the newest 1,200 public bot team games the shapes with the humans outnumbered are few:
one against two (3 games, 1 won), one against three (13, 2 won, both over 15 minutes), one against four (4, 1 won),
two against three (18, 12 won), two against four (8, 4 won), three against four (11, 5 won). They exist, they are
on every map, and the winners' games are listed in `docs/studies/data/2026-11-06-public-bot-team-games.json` by id. We can also make this case
ourselves: the arena seats one of ours against N opponents (`batch.json` `enemies`), and the flagship against two
BARbs is the natural bench for the user's hypothesis (light static defences and a build-up on the two opponents'
piecemeal arrival). No such batch has been run.

**The pilot (pool E).** One game of pool E replayed through our pipeline with the audit dump, read
with `run/defence_pool.py` (two-sided: every seat's own record):
`run/matches/1794011179-replay-defence-pilot-fn-8v8-barb100`, Failed Negotiations 1.0, 2026-11-06, 48:55, eight
people (OS 10 to 37.8) against eight BARb at +100 bonus, the people won. The lobby ran non-standard options
(build power x1.5, build distance x1.5, evolving commanders, experimental shields), so its clocks are faster than a
stock game's.

| side | ground defence built | soldiers built | defence as % | metal killed by defence (last hit) | yield | defence lost |
|---|---|---|---|---|---|---|
| the eight people | 593,700 | 3,785,349 | 16 | 1,657,614 | 2.8 | 159,785 |
| the eight BARb at +100 | 380,880 | 3,943,302 | 10 | 255,085 | 0.7 | 508,340 |

The people's share is the 15-16 % of every other pool; the yield is not: 2.8 against the 1.1 of our lobby games,
and it comes from four seats and one unit. Sukael (OS 31, the second-best seat): 28 Pulsars (3,500 metal, 1,400
reach, 1,091 dps) from 15:00, standing a median 2,050 from his own start and 3,300 from the nearest BARb start, on
the lane in front of the base; 172,000 metal of ground defence, 53 % of his soldiers, which killed 570,000 of BARb
(3.3). IsItInYet (OS 10): 16 Pulsars from 17:00 at 2,800 out, 400,000 killed on 59,000 (6.8). PainTrain (OS 14): 22
Beamers from 2:00 at 840 from his start, then 17 Pulsars from 24:00; 281,000 killed on 72,000 (3.9), his defence 87
% of his soldiers. ArchEnemy (OS 18): 11 Beamers from 2:00, 22 Pulsars from 14:00 at home (520 from his start), 13
Rattlesnakes from 29:00, six Basilicas from 42:00; 300,000 killed on 215,000 (1.4). The best seat, Gabiiiiiii7 (OS
37.8), built 4,240 of defence and 729,000 of soldiers: the top player plays the army while the lane-holders hold.
Two seats (the Legion player and Highskky) built nothing that mattered.

The BARbs at +100 built Bulwarks (15 in one seat, 8 in another), Bastions, Twin Guards and Wardens at home, 380,000
in all, and lost 508,000 of it: the people's Pulsars out-reach everything BARb walks in with, and the lane is the
same ground for 49 minutes (the human starts are 6,000 to 7,000 elmos north of the BARb starts and nobody went
round). Of the people's 594,000 of defence they lost 160,000.

What this one game says and the batch has to confirm: the hold is a 1,400-reach tower bought at tier 2 from about
14:00, placed 2,000-2,800 forward of the base on the lane where the trickle comes, after a Beamer line at 2:00-4:00
at the base's edge; the top seat spends on army instead; the BARbs' own towers are all at home and die to the
Pulsars' reach. The audit's FRONT and ARMY tables read one seat against everyone else, so for an 8v8 they need an
ally-aware mode before the batch (proposal 1).


**The 5v5 against people (the user's second check: human-8v8-1, Altored Divide, 2026-10-18, 57:38).** Two flagship
seats (t5 Legion, t9 Cortex, +50 % income) beside three Apex v0.3.0 seats against five people (OS 8 to 36); the
people left one by one from 44:26. From our two seats' records (fair view; the two-sided replay is
`defence-5v5-altored-humans`, read two-sidedly after the replay (`run/matches/1794014650-replay-defence-5v5-altored-humans`, 57 minutes of
engine time; every seat's own record):

| seat | ground defence built | soldiers built | defence as % | killed by defence | yield | defence lost | first defence |
|---|---|---|---|---|---|---|---|
| SSFA (OS 17, Legion, the north-east turtle) | 78,640 (+20,420 anti-air) | 193,765 | 41 | 180,090 | 2.3 | 114,910 | 1:24 |
| drakcore (OS 11, Legion) | 11,360 (+14,300 anti-air) | 63,965 | 18 | 53,261 | 4.7 | 38,770 | 2:06 |
| apexearth (OS 31) | 17,340 | 25,091 | 69 | 23,085 | 1.3 | 34,365 | 1:42 |
| Gabiiiiiii7 (OS 36) | 12,500 | 47,123 | 27 | 15,092 | 1.2 | 16,430 | 3:24 |
| TimmyT (OS 8) | 7,695 | 11,427 | 67 | 1,605 | 0.2 | 13,535 | 2:18 |
| our t5 (Legion) | 2,030 | 345,300 | 1 | 616 | 0.3 | 570 | 2:18 |
| our t9 (Cortex) | 6,210 | 252,583 | 2 | 525 | 0.1 | 2,610 | 3:54 |
| the three Apex seats | 24,590 to 42,035 each | 256,325 to 700,500 | 4 to 10 | 150 to 9,499 | 0.0 to 0.2 | 25,150 to 33,430 | 6:00 to 8:06 |

The people's five seats killed 273,000 of our side's metal with towers; our five killed 12,500 of theirs. SSFA's
hold: 16 Bastions from 27:22 (median 46:39) a median 915 from his start, 9 Xystons from 42:28, 7 Soteria shields from
36:50, 4 Eviscerators from 34:31, with four anti-nukes (22:42, then three at 50:10 to 51:26); drakcore 7 Xystons from
28:52 and a Supernova at 44:31 (killed 48:50 by a Tyrannus). Every person had an anti-nuke by 24:52. Apex's silos:
eight at the Legion seat t7 from 33:24, eight at t6 from 42:26, five at t8 from 52:47; 21 finished, 23 standing in
our records at 57:00. The people's anti-nukes died at 36:47 (a Marauder), 40:37 (Thanatos), 51:03 and 51:13
(Tyrannus), and the last four at 56:40 and 56:41, one to an Armageddon's warhead and three unattributed in the same
second: the stock ran out and the missiles took the anti-nukes themselves, twenty-three minutes after the first
silo stood. Apex's own towers, 98,000 metal at home, killed 11,000 and never saw the fight.):

- *Their escalation, as seen:* Cacophonies 5:59, Beamers from 8:33 (20 seen, median 12:25), Overwatches 12:25 (10),
  Twin Guards 15:17, a Pit Bull 19:13, Scorpions 22:38 (5), a Pulsar 23:48, Rattlesnakes 24:23 (3), Chimeras 26:56,
  **Bastions from 26:57 (23 seen, median 48:03)**, anti-nukes from 27:23 (two Prevailers, two Citadels by 36:03, four
  Aegis at 51:00), an Overseer shield 37:18, Bulwarks 37:19, **Xystons from 44:55 (12)**, a Supernova silo 45:08,
  Eviscerators 49:35. The line stood at E1-H2, the north-east.
- *Ours:* t5 29 Pharos and 4 Brambles (2,030 metal, 0.6 % of 347,550 of soldiers), t9 69 Guards (6,210, 2.4 % of
  254,383), one anti-nuke each; our allies (Apex) stood at 57:00 eight Bastions, a Pulsar, three Basilicas, three
  Olympus, seven anti-nukes and **23 nuclear silos** (14 Supernovas, 9 Armageddons).
- *How we died:* t5 lost 16,300 to towers in sight, 57,300 to units in sight, 62,350 to attackers it never saw (the
  Bastions' 1,100 reach on their own 1,200 radar, the Astraeus at 3,100) and 136,200 to no attacker at all (missile
  strikes and blasts; 93,900 of it in 45:00-50:00, the Supernova's minutes); t9 9,045, 40,900, 4,230 and 108,600.
  The towers in sight were a tenth of it.
- *What the player knew:* its turns name the Bastion's 1,100 and the Pulsar's 1,400, hold the bodies outside them,
  and send Thanatos and Arbiters (1,200) to shell "with radar sight"; the loss at E2 (45:35-47:38, 205 soldiers to
  59) came from a strike a minute early into an unscouted line (the ledger row's break 2). The shield at C3 it
  could not read (the user, 46:50); a shield stops cannon shells and nothing else, so the Thugs' plasma died on it and
  the Thanatos' and Arbiters' rockets would have passed (K-def-shields-stop-cannon-shells-and-nothing-else).
- *How it ended:* the people's eight anti-nukes against 23 allied silos; the turtle's tier-2 and tier-3 line was
  never broken by units, it was erased (K-def-anti-nukes-are-one-interceptor-per-90-s-and-silos-outnumber-them).

The derived claims from this game are in `docs/knowledge/defence.md`.

**The article's first tests (2026-11-06 night, D-defence-hold in the manifest).** defence-barb-1, a duel against
BARb hard_aggressive on Comet: WON 15:44 with 26 Sentries beside extractors, 10 % of soldier metal, yield 0.24, no
Beamer, as the article allows in a duel. defence-barb-8v8-1, eight seats against eight BARb hard_aggressive on All
That Glitters, the line the four earlier 8v8s lost on at 28 to 30 minutes: called a win by the user at 30:26 with
our army 198,807 against his 29,133 (6.8 to 1), extractors 68 against 67 and seven of his commanders still alive.
The commander directed the seats with the article's words (a sixth, towers where he comes, four hold points on row
4 with radar by 3:00, a cap at 10:00 when 38 stood), the seats built 104 Sentries, 56 Beamers, 38 Nettles, 9
Ferrets, 3 Pit Bulls and 2 Chainsaws (21,960 metal, 6 % of soldier metal built; the people's 15 %), yield 0.51 (the
flagship's team games before: 0.7 median over 244 sides, so no gain there), his towers 32,195 seen and 22,165
killed. The army ratio held above one through minutes 10 to 16, where the earlier games turned, and then ran away.
Faults for the article's wording: the first Sentry posts went 3,000 to 5,100 forward and died to Thugs and Grunts by
11:00 (4,365 of the first 6,280 lost), "where he comes" having been read as deep in the lane rather than at the hold
with the body; the standing share ran to half the standing army at 8:00 before the commander capped it; two seats
built almost nothing. One game, seed 3; the win may be the article's or the commander's hold points' or the
cap's; the ledger rows have the curves.

**defence-barb-8v8-1 read tower by tower** (scratch script over the eight records: every ground defence finished, where it
stood from its seat's start, when, what it killed by last hit, whether and to what it died):

| towers built in minutes | n | metal | killed | yield | lost |
|---|---|---|---|---|---|
| 0-3 | 2 | 170 | 503 | 2.96 | 0 |
| 3-6 | 29 | 2,465 | 2,281 | 0.93 | 1,785 |
| 6-9 | 31 | 3,265 | 4,287 | 1.31 | 2,440 |
| 9-12 | 3 | 465 | 1,103 | 2.37 | 0 |
| 12-15 | 4 | 550 | 273 | 0.50 | 0 |
| 15-30 | 95 | 14,705 | 2,180 | 0.15 | 1,780 |

| distance from the seat's start | n | metal | killed | yield | lost | median build minute |
|---|---|---|---|---|---|---|
| under 1,000 (home) | 55 | 9,405 | 1,770 | 0.19 | 190 | 18 |
| 1,000-2,000 (the row-4 holds) | 47 | 5,955 | 4,513 | 0.76 | 2,015 | 7 |
| 2,000-3,000 | 37 | 4,265 | 2,730 | 0.64 | 1,910 | 20 |
| 3,000-4,000 | 20 | 1,910 | 1,571 | 0.82 | 1,465 | 6 |
| 4,000 and more | 5 | 425 | 43 | 0.10 | 425 | 7 |

The towers bought while the hold was contested (3:00 to 12:00, 6,400 metal, at the holds and beyond) returned their price
and more, the people's number, though the ones past 3,000 lost three quarters of themselves (Thuds 16, Grunts 10, Incisors
8, unseen 12 of the 56 tower deaths); the five past 4,000 returned nothing. Two thirds of the game's tower metal (14,705)
was bought from 15:00 at home, at a lead of 1.3 to 5 to 1, and returned a sixth of its price. It was bought against a real
problem: from 15:00 to 25:00 Grunts, Thuds, Stumps and Incisors walking past the armies killed 20,000 of our structures
and 76 extractors (41 in 15-20 alone), and the commander lifted its own "no towers" for "two Beamers and a Nettle at each
home's spots" at 15:30 and "every spot behind the armies with a Sentry" at 28:15. That is the raid article's item 1 again,
beside the extractor, after the raider has shown; the towers caught 2,180 of the raiders while 20,000 of structures died
around them. (2,965 + 950 of the losses after 15:00 were our own constructors reclaiming our structures, a layout matter
for its own ledger, not this one.) The early overshoot has a stated mechanism: at 7:42 the commander read 73 towers for
6,500 metal and said why, "a plant alone spends only 10 metal a second while seats earn 25 to 30, so constructors with
nothing else build towers and banks climb"; the sixth was understood and the overflow went to towers anyway. Beamers
returned 0.47 to the Sentries' 0.64 only because most were the late home ones; the two best towers of the game were t7's
Beamers 1,800 forward at 7:39 and 10:03, 1,060 each, standing at the end. The three Pit Bulls and the Overwatch, late and
at home, killed nothing.

**defence-barb3-8v8-1 (2026-11-07, the revised article, Barb3 experimental_balanced, WON 20:56 by elimination).** The
zero-token baseline (barb3-distilled-8v8-1) had lost to Barb3 at 32:17 on the flagship's old curve. With the revision
the commander put towers and bodies together at five hold points by 3:00, bought nothing at home after the raiders, sent
the bodies past the holds at 5:00, and the ratio rose every minute from 6:00; his tier 2 never came. Towers 10,040
(104 Sentries, 4 Beamers, an Overwatch), 9 % of standing army at the end, 935 lost; by build minute 3-6 0.34, 6-12
1.4-2.4, 15-21 0.05-0.11 (Sentries at taken spots in his half). The ledger row has the curve.

## 3. Background: what high-OS duellists do with defence (pool A)

Quartiles are lower / median / upper over sides. "Ground defence" is every armed structure that shoots the ground
(laser, plasma and pop-up towers, torpedo launchers, the Exploiter, long-range cannons); anti-air towers are counted
apart; construction turrets are not defence.

| | all sides | winners | losers |
|---|---|---|---|
| ground defence metal built | 805 / 2,025 / 4,320 | 870 / 2,130 / 4,740 | 765 / 1,955 / 4,005 |
| anti-air tower metal built | 0 / 0 / 320 | same | same |
| soldier metal built | 4,670 / 13,120 / 26,561 | 5,589 / 14,951 / 29,423 | 3,902 / 11,529 / 23,769 |
| ground defence as % of soldiers | 9.6 / 15.6 / 23.5 | 9.2 / 14.7 / 21.9 | 10.6 / 16.6 / 25.9 |
| metal his units lost to my defences (last hit) | 246 / 1,231 / 3,336 | 184 / 992 / 3,036 | 301 / 1,466 / 3,707 |
| that, over my ground defence metal | 0.3 / 0.6 / 1.0 | 0.2 / 0.5 / 0.8 | 0.4 / 0.8 / 1.1 |
| defence metal lost | 270 / 1,080 / 2,890 | 85 / 550 / 1,500 | 780 / 2,100 / 4,360 |
| first defence started | 2.8 / 3.3 / 3.9 min | same | same |
| ground defence metal by 5:00 | 165 / 255 / 340 | | |
| ground defence metal by 10:00 | 900 / 1,295 / 1,830 | | |

The engine's own `defenseValue` (what stands, not what was built) as a share of everything standing: 3 % at 5:00, 6 %
at 8:00, 7 % at 10:00, 8 % from 12:00 to 30:00, winners and losers within a point of each other at every mark. So in a
high-OS duel defence is a steady twelfth of the estate, started around the third minute, and it does not separate
winners from losers; what separates them is that losers lose their defence (2,100 against 550) and winners lose
soldiers to it. Kills credited to defences run from 5:00 to 20:00 (pool total 0.38 M metal in minutes 5-10, 0.93 M in
10-15, 0.72 M in 15-20), the same minutes the defence is built in.

Damage dealt per metal, over units that died (one damage row per unit at its death, so survivors are left out):
ground defence 7.4 health points a metal, raiders and scouts 6.2, other soldiers 6.5. Dead turrets out-dealt dead
soldiers by about a seventh; not the "far more efficient" of folklore, in duels, by this measure.

**What they build.** Armada per side: 14.9 Sentries, 3.3 Beamers, 0.4 Pit Bulls, almost nothing else. Cortex: 14.1
Guards, 5.1 Exploiters (349 of 573 Cortex sides build the armed extractor, median 5 where they do), 3.7 Twin Guards,
0.5 Scorpions. Legion: 28 Pharos (the 70-metal tower; Legion sides are 36). Tier-2 defence stands in 110 of 1,054
sides; the tier-1 plasma tower (Gauntlet, Agitator) in 16. Anti-air towers in under a quarter of sides. The 1,220-reach
plasma nest the brief warns of is a team-game and BARb thing, not a duel thing.

**Where they stand.** Of 23,700 ground defences placed: 3,840 within 1,500 of the own start (home), 15,800 in the own
half beyond that (at the extractors and the lanes), 4,085 nearer his start than ours (forward). Two of three turrets are
outpost turrets; one in six is forward.

**What they kill.** By metal: Incisors and Blitzes first (0.42 M each), then Brutes (0.29 M), Grunts (0.18 M), Stouts,
Pounders, Pawns, Reapers, and fighters (Vengeances and Falcons, 0.13 M together, from the towers' small anti-air
weapons). Raiders are the turrets' food; the line tanks walking into them come next.

**What kills them.** Where the engine names a killer: Incisors (1,581 turret deaths), Blitzes (1,328), Brutes (1,243),
Grunts, Stouts, Pounders, Pawns, Thugs, Reapers, Shivas, Banshees. Raiders kill turrets too, by walking onto an
unfinished frame or a lone Sentry in numbers; 6,995 of 15,000 turret deaths have no attacker in the event (the engine
names none for artillery hits, death blasts and some beams).

**By map** (sides, median ground defence, median first defence, median kills over defence): Isidis crack 244, 1,485,
3.5 min, 0.4; Feast of Hades 200, 4,392, 3.2, 0.7; Aurelia 96, 2,005, 3.6, 0.5; Mithril Mountain 80, 1,260, 3.2, 0.6;
Theta Crystals 42, 3,242, 3.6, 0.6; Canis River 38, 1,935, 2.9, 0.5; Hades Ponds 36, 1,995, 3.1, 0.8; Comet Catcher 34,
765, 2.5, 0.7; Flats and Forests 22, 2,932, 3.4, 1.0. Comet is the least-turreted map of the fourteen and the earliest
(the 0:27 Sentry of the first survey is a Comet habit, not a general one); Feast of Hades carries three times the
median.

## 4. Our own seats, all game sizes (pools B, C, D)

Quartiles lower / median / upper again. "Killed by defence" is the metal of his units whose last hit was one of
our defences; for our own records it is known only from 2026-10-18 (when the killer's type was added to
`enemy_destroyed`), so that column is over the recent records. For the opponent in our own records "ground defence"
is what we saw of his, not what he built.

**People in our replayed games (pool B)**, every game type (the Comet and Quicksilver duels, thebluegecko's games,
the lobby team games):

| seat | sides | ground defence built | soldiers built | defence as % of soldiers | killed by defence | killed / defence | defence lost / defence | first defence |
|---|---|---|---|---|---|---|---|---|
| person, OS 35+ | 254 | 720 / 1,785 / 4,710 | 5,058 / 15,526 / 32,709 | 15 | 412 / 1,703 / 5,109 | 0.9 | 1.0 | 3.5 min |
| person, OS 20-35 | 659 | 360 / 1,875 / 5,560 | 6,981 / 20,494 / 62,592 | 10 | 110 / 1,664 / 7,106 | 1.1 | 1.1 | 3.9 |
| person, OS under 20 | 1,400 | 425 / 1,835 / 5,070 | 6,534 / 17,822 / 48,093 | 12 | 120 / 1,531 / 5,903 | 0.9 | 1.1 | 3.6 |
| BARb (in those replays) | 50 | 1,705 / 2,530 / 5,640 | 6,755 / 11,729 / 21,488 | 28 | 0 / 255 / 1,600 | 0.1 | 1.1 | 2.6 |
| SMRTBARb, 8v8 Glitters | 32 | 4,970 / 19,820 / 31,970 | 43,136 / 66,711 / 88,299 | 26 | 0 / 916 / 3,025 | 0.1 | 0.0 | 3.8 |

The OS 35+ people build Sentries and Guards (2,548), Beamers and Twin Guards (1,042: one heavy laser per 2.4 light
ones), Nettles (206), and in the team games Pit Bulls (145) and Pulsars (115). Their defence kills about its own
metal before it dies. BARb at 28 % of its soldiers kills a tenth of its defence metal back.

**Our seats (pools C and D):**

| seat | sides | ground defence built | soldiers built | % | killed by defence (recent records) | killed / defence | defence lost / defence | first defence |
|---|---|---|---|---|---|---|---|---|
| the flagship (`player`) | 609 | 595 / 1,785 / 3,165 | 6,883 / 16,811 / 35,731 | 9 | 360 / 903 / 1,945 | 0.4 | 0.8 | 3.7 min |
| commander mode (retired) | 62 | 935 / 1,260 / 1,955 | 4,144 / 8,771 / 24,058 | 14 | | | 0.8 | 3.9 |
| pianist alone | 81 | 0 / 85 / 255 | 330 / 1,992 / 4,188 | 4 | | | | 4.4 |
| the deleted heuristic bot | 2,165 | 595 / 1,080 / 1,710 | 5,557 / 11,772 / 30,428 | 7 | | | 0.9 | 3.8 |
| BARb as seen from our seat | 426 | 85 / 2,290 / 8,245 | | | 31 / 1,660 / 5,207 | 0.4 | 0.6 | |
| Apex as seen from our seat | 44 | 1,950 / 5,320 / 18,495 | | | 1,520 / 2,575 / 5,441 | 0.4 | 0.9 | |
| people as seen from our seat (the lobby) | 127 | 695 / 2,225 / 6,080 | | | 0 / 530 / 1,850 | 0.1 | 0.5 | |

The flagship's 13,000 defences over 609 sides are Sentries (11,550) and Guards (1,348), 835 Nettles, 167 Beamers, 122
Twin Guards, no tier-2 defence at all. Its defence is 9 % of its soldiers against the OS 35+ people's 15 % and the
duellists' 16 %, and kills 0.4 of its metal against their 0.9: half the share, half the yield. The yield is the
larger gap: a Sentry beside an extractor nobody raids earns nothing, and that is where almost all of ours stand
(H-PLAYER-TURRET-WITH-THE-SPOT, D-SENTRY).

The heuristic bots from the census (both sides standing at 10:00 and 15:00, medians over games):

| us | opponent | games | won | our ground defence at 10 / 15 | his at 10 / 15 | our towers at 15 | his | our defence as % of standing army at 15 | his |
|---|---|---|---|---|---|---|---|---|---|
| WRDistilled | BARb hard_aggressive | 595 | 251 | 1,170 / 1,530 | 1,505 / 4,765 | 17 | 21 | 19 | 135 |
| WRDistilled | Apex v0.5.04 | 442 | 216 | 1,170 / 1,530 | 2,550 / 3,940 | 18 | 16 | 18 | 159 |
| WRDistilled | BARb medium | 41 | 22 | 850 / 1,020 | 735 / 1,575 | 12 | 12 | 33 | 36 |
| WRTerritory | Apex v0.5.04 | 12 | 0 | 955 / 2,060 | 675 / 2,475 | 12 | 8 | 104 | 52 |

WRDistilled builds the 85-metal tower only (9,343 Sentries and Guards over the two big rows, 2 Exploiters, 60
Nettles); at 15:00 it stands a third of BARb's defence metal and under half of Apex's, and both opponents' stock is
Twin Guards, Beamers, Wardens and Overwatches beside the light towers, Apex with Gauntlets (661 in 442 games). His
"defence as % of standing army" above 100 at 15:00 is his army dead and his towers standing: the games we win, we
win through the towers, not round them, at the price the FIGHTS tables of the audits show (dist140 game 2 against Apex: in the 18:13 fight at his
base 13 of our 18 deaths were to turrets, in the 25:41 fight 13 of 51).

**The user's observation holds in every pool.** Our seats put 7 to 9 % of their soldiers' metal into defence where
people put 15 to 16 %, build the cheapest tower only where people build one heavy laser per two or three light ones
and pop-ups at tier 2, and get back 0.4 of the metal where people get 0.9 to 1.1.

## 5. The units

From `crates/bot/data/units.json` (game source 54199a0d). Damage a second per hundred metal and health per metal are
the two efficiency numbers; range is the third thing that matters and it does not divide.

Tier 1, Armada and Cortex, the ground defences:

| unit | metal | energy | health | range | dps | dps per 100 m | hp per m | notes |
|---|---|---|---|---|---|---|---|---|
| Sentry `armllt` / Guard `corllt` | 85 / 90 | 680 / 700 | 620 / 650 | 430 / 435 | 161 | 189 / 179 | 7.3 / 7.2 | 20 energy a shot, stops in a stall |
| Beamer `armbeamer` / Twin Guard `corhllt` | 190 / 195 | 1,500 / 1,750 | 1,430 / 1,670 | 490 | 266 / 321 | 140 / 165 | 7.5 / 8.6 | energy to fire |
| Dragon's Claw `armclaw` / Dragon's Maw `cormaw` | 340 / 290 | 1,600 / 1,550 | 1,330 / 1,610 | 430 / 410 | 159 / 320 | 47 / 110 | 3.9 / 5.6 | pop-up, stealth (the Maw's flame hits everything in a cone) |
| Overwatch `armhlt` / Warden `corhlt` | 440 / 480 | 4,700 | 2,600 / 2,750 | 620 | 215 / 218 | 49 / 45 | 5.9 / 5.7 | out-reaches every tier-1 raider and line unit |
| Exploiter `corexp` (Cortex only) | 240 | 1,900 | 1,440 | 435 | 161 | 67 | 6.0 | an extractor with a Guard's gun; the pros' outpost turret |
| Gauntlet `armguard` / Agitator `corpun` | 1,250 / 1,300 | 12,500 / 13,500 | 3,050 / 3,250 | 1,220 / 1,245 | 106 / 110 | 8 | 2.4 / 2.5 | plasma, area; the "nest" the brief fears |
| Nettle `armrl` / Thistle `corrl` | 80 | 900 / 850 | 330 / 335 | 765 | 68 vs air | | 4.1 | the tier-1 anti-air |
| Ferret `armferret` / SAM `cormadsam` | 330 / 350 | 5,700 / 5,500 | 1,600 / 2,500 | 950 | 208 / 225 vs air | | 4.8 / 7.1 | |
| Chainsaw `armcir` / Eradicator `corerad` | 750 / 800 | 7,300 / 7,900 | 4,450 | 1,200 | 625 vs air | | 5.9 / 5.6 | long-range anti-air |
| Dragon's Teeth `armdrag` / `cordrag` | 8 | 0 | 2,800 | | | | 350 | the wall block: funnels tier 1, stops low shots |
| mines `armmine1-3` / `cormine1-3` | 7 / 25 / 50 | 270 / 1,050 / 2,800 | | | | | | laid by the Groundhog / Trapper, cloaked |

Tier 2 (both factions): Pit Bull `armpb` 680 m, 730 reach, 422 dps (62 per 100 m), pop-up; Scorpion `corvipe` 730 m,
730, 412 dps; Pulsar `armanni` 3,500 m, 1,400 reach, 1,091 dps (31); Bulwark `cordoom` 3,000 m, 950, 1,513 dps (50);
Rattlesnake `armamb` / Persecutor `cortoast` 2,500 m, 1,380 / 1,390 reach, plasma; Arbalest / Birdshot flak 820 / 850 m,
500 dps vs air; Mercury / Screamer 1,600 m, 2,400 reach anti-air; Fortification Wall 40 m, 8,900 health; Keeper /
Overseer plasma shields 3,000 m, radius 550. Legion: Pharos `leglht` 70 m, 182 dps (260 per 100 m, the best in the
game), Cacophony `legmg` 380 m (hits ground and air), Chimera `legapopupdef` 750 m pop-up, Bastion 4,200 m.

The tier-1 soldiers for the same two numbers: Pawn 167 dps per 100 m and 6.9 hp per m at 180 reach; Grunt 172 and
6.5 at 215; Blitz 82 and 6.6 at 180; Incisor 78 and 6.8 at 230; Stout 36 and 8.0 at 350; Brute 34 and 8.5 at 350;
Thug 43 and 7.9 at 380; Centurion 68 and 5.9 at 325; Rocketeer 34 and 6.0 at 475.

What the table says, from first principles:

- A Sentry is a Pawn's damage per metal with a Stout's health per metal at 2.4 times the raider's reach, for one
  shot of energy each half second. Against raiders it is strictly better metal than the raiders; against anything
  that reaches 435 or more (Rocketeer 475, Aggravator 475, Whistler 550, Lasher 575, every artillery) it is dead
  metal unless something of ours covers it. That is the whole "static defence" trade in one line: the turret is a
  bet that the opponent brings the units it out-reaches.
- The Overwatch and Warden (620) out-reach every tier-1 ground unit but the artillery; the Beamer and Twin Guard (490)
  out-reach every tier-1 unit but the skirmishers and artillery. The pros build the cheap tower fifteen to one
  over the big one: the cheap tower's job is raiders, and raiders are what comes in a duel.
- Energy. A Sentry's 20 a shot is 40 a second firing; a Beamer's 6 a shot at ten shots a second is 60; an Overwatch
  75 a shot, a Pulsar 1,500, a Bastion 500 every 3 s, a Xyston 1,800. A line of them firing is a solar farm, and
  in a stall it is silent. The simulator charges this (`crates/combatsim` `energy_per_shot`, a weapon holds its shot
  until the store can pay); the brief says "turrets stop firing in a stall" and no more.
- The pop-ups (Claw, Maw, Pit Bull, Scorpion) take no damage closed and are cloaked; they are the ambush towers.
  Pros place 0.4 Pit Bulls and 0.5 Scorpions a side: the tier-2 tower that gets built is the pop-up, not the Pulsar.
- Walls at 8 metal for 2,800 health are the cheapest health in the game by forty times; what they do is geometry
  (a tier-1 unit cannot shoot over or through them), which no efficiency number captures.

## 6. What the repo already says

- The brief (`docs/briefs/player.md`): "Static defence is out-ranged or bypassed, never walked into" with six cases of
  our ball dying to nests; a Sentry beside each outer extractor pair and at the plant by 2:00; pickets plus turrets as
  the raid answer; turrets counted at about three times their metal in a fight's odds (H-HANDS-TURRETS-WITH-A-PARTY).
  Nothing on turrets as the way to hold a front while out-scaling, nothing on tier-2 defence, pop-ups, walls, mines,
  nor on building defence where the fight will be rather than where the extractor is.
- Lessons: Infern8 (L-other-infern8-defences, 2026-09-27): "defensive structures are generally stronger per metal than
  mobile units; attacking them effectively requires either long-range units or overwhelming firepower". thebluegecko:
  one ball behind turrets at whatever narrows the way in, then tier 2 (K-opp-thebluegecko-one-ball-behind-turrets-then-tier-2).
- Knowledge: K-army-waves-die-to-static-defence, K-eco-outposts-get-raided (turret per outpost), K-map-comet-...-turret-at-0-27,
  -raids-met-by-turrets (winners 9 by 8:00); K-hands-a-defence-at-a-place-faced-nowhere (a turret's site is the engine's
  nearest free one, on whichever side). tier2.md: "light towers 12-14 at minute 20-25; heavy towers 1.3-2.1" from BARb
  census.
- The distilled bot: D-SENTRY (a sentry on the enemy side of an extractor farther than 900 from home, trimmed after 19
  sentries by 10:00 cost 12,900 energy), D-ANTI-AIR (a Nettle by a plant while fighters are short), D-WEIGHT (a turret
  three times its metal in a target's defence), D-REACH (off). Nothing places a defence where the opponent will come
  rather than where our extractor is.

## 7. What the survey says, in short

1. **Defence is a steady share, not a phase.** High-OS duellists hold it at about 8 % of everything standing from
   12:00 on, 16 % of their soldiers' metal built, started at 3:20; winners and losers alike. It is not a lever that
   wins duels; what differs between winners and losers is who loses theirs.
2. **The yield is the thing we lack, not the count.** People's defences kill their own metal back (0.9-1.1 by last
   hit); ours kill 0.4. Theirs stand where the raiders come, two thirds at the outposts and one in six forward of
   the midline; ours stand beside extractors by rule.
3. **The cheap tower is the tower.** Fifteen Sentries to three Beamers to half a Pit Bull per duel side; the Exploiter
   is Cortex's outpost turret (five a side in 61 % of Cortex sides); tier-1 plasma almost never; the tier-2 tower that
   gets built is the pop-up. We build the cheap tower only and never the next size up, and no pop-up, wall or mine.
4. **The folklore is "far stronger per metal"; the files say "a Pawn's gun on a Stout's body at 2.4 times the reach,
   paid in energy".** Damage dealt per metal by dead units: defence 7.4, soldiers 6.2-6.5. The structure's edge is
   the reach and the health, cashed only against units that come inside it. Everything that out-reaches 435 makes a
   Sentry dead metal: the brief's "out-ranged or bypassed, never walked into" is right about our pushes and is
   exactly what the opponent does to our towers.
5. **The team game is where the gap is widest and the data thinnest.** People at OS 35+ hold 15 % and get 1.1 in
   team games; the flagship 9 % and 0.7. How they hold a cross-section against the trickle, and when they break
   out, is in no pool we hold; the public 8v8s against bonused BARbs (pool E, the pilot) are the pool for it, and
   the asymmetric one-against-two case can be made in our own arena.
6. **What the files cannot tell us** and a replay batch can: where exactly the towers stand relative to the lanes
   the BARbs walk, whether the people wall and mine, what the energy bill of a firing line is, when the people step
   from light lasers to Beamers to pop-ups to plasma, and how much of the BARb trickle dies to towers against units
   standing on them.

## 8. Survey steps still open (not run)

1. **Replay batch, pool E**: twenty public 8-humans-against-8-BARb games at +50 or more with a 30+ OS seat, and the
   dozen asymmetric wins (one or two people against three or four BARbs) from the same list, through
   `run/replay_match.py --audit`; one engine at a time, outside arena batches. First `run/audit.py` needs an
   ally-aware mode (it reads one seat against everyone else). Then an Opus reading of the twenty for placement,
   walls and the step-ups in tower class, as claims in `docs/knowledge/defence.md`.
2. **An asymmetric bench**: the flagship (and the distilled bot) one against two BARbs on a lane map, judged by the
   yield column of `run/defence_pool.py --summary` and the clock of the break-out.
3. **The friend's tool**: 2,134 all-human 8v8s above OS 30 are in his catalogue and none ingested (100 CPU-s a game,
   on the user's word): the all-human lane game, for how people hold a cross-section against people.

## 9. What is established, what we may be missing, what to change

The claims are filed in `docs/knowledge/defence.md` with their evidence; this is the reading of them.

**Established (supported by the pools or the files):**

1. People hold ground defence at a sixth of their soldier metal from the third minute on, at every level and in
   every game size; our seats hold a tenth and WRDistilled a third of BARb's. The share is not what wins; losing
   the defence is what loses.
2. The yield is placement: the people's towers stand where the raiders come and earn their metal back; ours stand
   beside extractors and earn half.
3. The cheap tower is the tower, one heavy laser per two or three light ones, the Exploiter for Cortex, the pop-up
   at tier 2; in the lane game against bonused BARb the hold is a Pulsar line at 14-24 minutes, 2,000-2,800 forward.
4. A tower is a raider's gun on a line unit's body at twice the reach, paid in energy: its edge is what comes inside
   the reach, and everything that out-reaches it makes it dead metal. "Three times its metal" is the number for a
   fight on the spot, not an efficiency.
5. Towers fire on radar, and the tier-2 towers carry radar past their reach; a Bastion line kills what nothing of
   his sees. Shields stop cannon shells and nothing else. Anti-nukes stock one interceptor per
   90 s; silos in numbers beat them, and that is how the 5v5's turtle ended.

**Mechanics we may be missing (the ones the games turned up, each with where it would live):**

- *A tower's own radar and radar-fire.* The report's "turrets covering it reach N" carries the reach, not that the
  reach needs no sight of his; the simulator's towers have sight only. Both read a Bastion line as safe to shell
  from 1,200 with radar, which it is, and as blind beyond 780, which it is not.
- *What a shield stops.* The report names a shield seen (`scout.rs`); the fact layer says a shield stops cannon
  shells and nothing else (F-fight-big-weapons), and that is right for stock lobby options; what is missing is the
  consequence written for the player: a shielded nest is shelled with rockets, lasers and bombs, not with Thugs,
  Stouts, Fatboys or Mausers, and the generator (3,550 health, 273 of sight) is the target. Some lobbies run
  `experimentalshields` (the public 8v8 pilot did: everything bounced), and the start script says which.
- *The pop-up's closed state* (cloaked, no damage taken until it fires; `armpb.lua` `cloakcost` 16, `mincloakdistance`
  55): the simulator and the odds lines count it as a standing tower.
- *Walls and mines* (`armdrag` 8 metal, 2,800 health; `armmine1-3`): not on any menu, not in the odds, not in the
  layout engine's vocabulary. Their value is geometry (a funnel that puts the trickle inside the towers' reach),
  which no metal ledger shows and the replay batch has to read.
- *The energy bill of a line.* Charged by the simulator, unsaid by the brief: a seat that stands ten Beamers has
  bought 600 energy a second of firing.
- *Silo counts against interceptor stock.* The brief says one anti-nuke under every home; it does not say that two
  silos firing together pass one anti-nuke, nor that the way out of a turtle on tier-2 defence is silos in numbers,
  which Apex knew and we did not (23 against our two).

**How to adjust play (the user's decision; none built):**

1. *The brief's defence paragraph* (the flagship): the share people hold and when; a tower pays where the raiders
   come (the hold point, the lane, the outer pair raided last game), not beside every extractor; one Beamer or Twin
   Guard per two or three light towers once his line units have shown; the pop-up at tier 2; a line is an energy
   bill; against bonused BARb in a lane game the hold is Pulsars forward from the first tier-2 bank; a tier-2 tower
   fires by radar past its sight; a shield stops cannon shells only; a turtle on tier-2 defence is ended by silos in
   numbers, not by the army. The economy page's "metal in the bank is army we do not have" gets its twin: a tower
   nothing walks into is metal in the ground.
2. *The hands' menu* (`roster.rs` `USUAL_*`): add the Beamer and Twin Guard (the people's second tower), the Exploiter
   for Cortex, the Dragon's Teeth and the shield generators, each with the gloss it has; the place words
   (`armbeamer spot_35`) already serve the siting.
3. *The report*: the covering-turret line carries the tower's radar reach and "fires by radar"; the shield line
   says what it stops; the losses line already separates unseen attackers and no attacker (K-mech-missile-and-blast-deaths-unattributed).
4. *WRDistilled*: D-SENTRY sites the Sentry at the raid's source (the hold point and the threat grid know it), a
   Twin Guard or Beamer at the hold point once his line units have shown, and the yield column as the kill test
   against today's 0.4; the silo rule (D-NUKE) counts his anti-nukes' interceptor stock before firing one at a time.
5. *The simulator*: towers with their radar; the pop-up's closed state.
