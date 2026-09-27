# Economy

### K-eco-judge-energy-by-storage
**Claim.** Energy shortage must be judged by stored energy, not by income against usage: metal converters absorb any
surplus, so usage always rises to meet income.
**Status.** supported (2026-09-19)
**Evidence.** v4-easy/01 and /05: energy income +2000 with extractors stuck at 6-8 for 60 minutes — constructors built
generators forever because `income < usage + 20` never turned false. After the change (v5-easy) extractors reached 18 by
minute 17 (v5-probe/00) and the win rate went from 4-8 to 7-4-1, confirmed 14-10.
**Would be wrong if.** With the storage test, energy still sat near zero for minutes while builders did something else.
(Partly seen: v5-probe/01 energy 0-6 of 2150 from minute 9 with 4 labs — fixed by removing the two-generator cap.)
**Used by.** H-ECO-ENERGY-BY-STORAGE, H-ECO-CONVERT-SURPLUS.

### K-eco-expansion-before-conversion
**Claim.** Extractors on free spots beat converters as a use of constructor time; converters are for energy that would
otherwise overflow storage.
**Status.** conjectured (2026-09-19)
**Evidence.** Indirect: same batches as above; never tested in isolation.
**Would be wrong if.** A converter-first ordering reached higher metal income at minute 15 over 24+ matches.
**Used by.** H-ECO-EXPAND, H-ECO-CONVERT-SURPLUS.

### K-eco-outposts-get-raided
**Claim.** Extractors far from the base are picked off by early raids; BARb easy raids before minute 10.
**Status.** conjectured (2026-09-19)
**Evidence.** v2-easy-0704/00: extractors 12 at minute 10 falling to 4-6 by minute 22. v3-easy/04: extractors 3-5 throughout,
commander retreat triggered 15 times. The turret-per-outpost response has not been evaluated on its own.
**Would be wrong if.** Extractor losses did not drop in matches where outposts had turrets.
**Used by.** H-ECO-OUTPOST-TURRET, H-ECO-OWN-HALF.

### K-eco-t1-ceiling
**Claim.** A tier-1-only economy tops out around +40 metal/s at minute 17 on Quicksilver, which is not enough against BARb medium.
**Status.** conjectured (2026-09-19)
**Evidence.** v5-probe/00 (+38 at minute 17, 18 extractors); v5-medium 0-12, all lost between minute 16 and 20. BARb's own
income was not measured, so "not enough" is inference.
**Reopened 2026-09-19.** The medium losses are decided by extractor raids at minute 8-10 (see K-opp-medium-wins-by-20), long
before a tier-1 ceiling could matter; the ceiling may still be real but is not why we lose today.
**Would be wrong if.** A tier-1 brain with better army handling beat medium, or a tier-2 economy did not change the result.
**Used by.** (motivates the planned tier-2 work)

### K-eco-nano-turrets-and-reclaim-are-normal-play
**Claim.** Two staples of ordinary play are missing from the bot: construction turrets (nano turrets) beside the
factory, which multiply its build power for far less metal than a second lab, and reclaiming wrecks, which returns a
large share of every dead unit's metal to whoever holds the field after a fight.
**Status.** reported (2026-09-19) by the user, watching the replay of the first Sonnet commander game; he judged them
important but possibly not why that game was lost.
**Evidence.** Consistent with the census: BARb medium plays one lab plus one nano turret all game where we build 3-4
labs (K-barb-medium-observed-build), and its resurrection bots feed on our wrecks (K-army-dead-waves-are-resurrected).
Until 2026-09-19 we built no nano turrets and never issued a reclaim order.
**Would be wrong if.** A version with nano turrets and wreck reclaim near home showed no gain in army built by minute 10.
**Used by.** H-ECO-NANO, H-ECO-RECLAIM.

### K-eco-production-is-the-bottleneck
**Claim.** Once early expansion works, one lab cannot spend the income: metal piles up while the opponent, on half the
extractors, builds the bigger army. Build power (construction turrets, labs, later tier 2) has to follow income.
**Status.** observed once (2026-09-19), first noticed by the user watching the game.
**Evidence.** commander-3-play-to-win: extractors 12 v 6 at minute 5, 13 v 6 at minute 8; income 25-30 and spending
11-19 with one lab; 1393-1749 metal banked from minute 6 to 9; army value 1938 v 2286 at minute 9. One construction
turret and 7 converters by minute 10. The rules that add build power sat behind "expand" in every focus order, and with
free spots left the order never got past it; under the expand focus the turret step was not in the list at all. No
build-site refusals: room was not the limit. There is no tier-2 logic in the bot at all.
**Would be wrong if.** With H-ECO-SPEND the bank still sat above 500 for minutes, or army value did not follow income.
**Used by.** H-ECO-SPEND.


### K-eco-raided-ground-is-raided-again
**Claim.** Against BARb medium the unit that killed an extractor leaves within seconds, but the ground does not become
safe: an armed enemy is near the spot again inside a minute about half the time. What we forgo by not expanding is far
larger than what rebuild delays cost: most of it is spots we never walk to, which the opponent then holds.
**Status.** measured in hindsight (2026-09-20), `run/spot_regret.py` over 96 heuristic games with ground truth (v23-v26).
**Evidence.** 1147 extractor losses. Armed enemy within 600 of the spot after the loss: median 16 s, 90th percentile 47 s.
Quiet after that: median 42 s (NW 30 s, SE 65 s); armed enemy back or still there within 60 s in 48 %, within 120 s in
65 %, within 240 s in 76 % (NW 85 %, SE 69 %). We rebuilt 70 %, the spot standing empty a median 135 s, so cover reopens
most hot spots before H-ECO-HOT-SPOTS' four minutes run out. Per game we held a median 134 extractor-minutes (NW 94,
SE 251) and left 466 quiet, usable extractor-minutes untaken (quiet runs over 60 s, less 60 s to get there). From NW the
8 spots 2-3k out: ours 1 minute a game, theirs 70, quiet and usable 78.
**Limits.** Hindsight: an extractor standing there might have drawn the visit that never came, and "quiet" on the
opponent's side is quiet because it is the opponent's side. Straight-line distance, not walking distance. "Armed enemy
within 600" counts an army passing through and ignores whether we had cover there.

### K-eco-reach-stops-at-the-first-gap
**Claim.** An expansion reach measured from home (H-ECO-REACH, 1500 + 50 per soldier on foot) stops at the first gap in
the map's metal and never crosses it, because the soldier count it depends on falls every time a wave leaves.
**Status.** inferred from the map and the probe (2026-09-20); the fix (H-ECO-FRONTIER) is unmeasured.
**Evidence.** Quicksilver from the north-west start: spots at 211, 428, 1097, 1376, 1929, 1961 and 2108 on foot, the
next eight at 2697-3400, which needs 24-38 soldiers alive at once. `run/spot_regret.py` over 48 NW games: those eight
held by us 1 minute a game, by the opponent 70, quiet and usable 78. NW games plateau at 5-7 extractors.

### K-eco-expansion-comes-last
**Claim.** In the default step order expansion comes after reclaiming and outpost turrets, and once raids begin both of
those always have work, so constructors stop taking metal spots exactly when the opponent is taking them fastest.
**Status.** observed once in detail (2026-09-20, Opus analyst on commander game 9 north-west; step order and rule counts
confirmed from the code and bot.log); consistent with every north-west batch.
**Evidence.** Minutes 0-15: constructors placed 26 light turrets and 9 extractors; 1105 metal stood in turrets against 400
in extractors; rule firings in those minutes H-ECO-RECLAIM 96, H-ECO-OUTPOST-TURRET 66, H-ECO-EXPAND 25 +
H-ECO-EARLY-EXPAND 22; no extractor started 7:19-12:02 nor 12:02-15:36. Constructor time 59 % walking, 13 s idle. 18
walkable free spots unattempted at minute 15, 13 of them with no armed enemy within 600 for 13 of the 15 minutes. The
opponent starting from the same corner in the sister game also sat at 5-6 extractors from minute 8 to 16: the corner
is poor, and we made it poorer. Full account: `run/matches/1789886040-commander-9-nw-low/00/postmortem.md`.
**Would be wrong if.** With expansion first below 9 extractors the north-west count at minutes 10-15 did not rise, or
rose and the constructors died for it.
**Used by.** H-ECO-EXPAND-FIRST.

### K-eco-straight-line-fallback-claims-islets
**Claim.** `walk_from_home` answers with the straight-line distance for ground we cannot walk to; any rule that reads it
as nearness without asking `reachable_on_foot` will send builders to islets and ledges.
**Status.** observed (2026-09-20), fixed for `claim_spot` and the outpost-turret step.
**Evidence.** Commander game 9 north-west: the commander built an extractor on the islet D1 (3224, 520) at 4:48 (engine
log: `wanted=(3224,520) placed=(3224,520)`; the map lists that spot with `walk_from_home: null`); the analyst counts six
turret orders for it and one constructor walking 321 of its first 422 seconds round the shore. The engine returns a long
path, not `move_failed`, so `note_unreachable_sites` never learns.

### K-eco-battlefields-hold-thousands
**Claim.** From about minute 10 the wrecks we can see hold 1200-4200 metal at a time (25-55 wrecks in 2-6 fields), against
a metal income of 15-30 a second: minutes of income. Most of it lies where the fighting is, on ground we do not hold; in
the games looked at 0-1000 of it was safe to work at any moment.
**Status.** observed (2026-09-20), rec-1 (4 games), the per-minute `wrecks:` lines of bot.log.
**Used by.** H-REC-CREW.

### K-eco-their-extractors-are-the-spots-we-do-not-hold
**Claim.** On a map with a fixed set of metal spots, our own extractor count is a better estimate of the opponent's
extractor count than anything we see of theirs. On Quicksilver (44 spots) the correlation between the two, within a
game minute, is −0.39 at minute 5, −0.54 at 10, −0.72 at 20 and −0.78 at 25. Used alone beside the clock it removes
28 % of the error in estimating their extractor count, against 7 % for the extractors of theirs we have actually seen.
The spots we do not hold, they hold.
**Status.** measured (2026-09-20) on one map, 430 games.
**Evidence.** `docs/studies/tempo-model.md`, section "What evidence actually moves the estimate"; correlations from
`run/tempo_model.py`'s dataset (`f_own_mex` against `y_t1mex + y_t2mex`, per minute).
**Would be wrong if.** The same correlation on the four v33 maps (53-92 spots) came out near zero, or a game where
both sides expanded freely showed no complementarity.
**Used by.** (none yet) — it is why the tempo model's economy estimate works at all, and an argument that our own
expansion count is worth reporting to the commander as information about the opponent.

### K-eco-lab-anchored-on-a-spot
**Claim.** A building anchored beside its builder can land over a metal spot: the engine's site search does not keep
buildings off spots (K-rules-site-search-ignores-metal-spots), and a lab 102 from a spot's centre leaves no site for
the extractor, so the spot is never taken.
**Status.** demonstrated (2026-09-22, human-1; the user's report "it builds the lab right on top of it")
**Evidence.** human-1: the lab created at (3936, 2160), 102 from spot_10 (3864, 2088); "no site for armmex" for the
commander's orders at 1:27 and 3:36 while the picture read the spot as free. In the arena's placed games the start
is 70 elmos elsewhere and the lab lands 173 from the spot (pianist-player-14: spot_10 taken at 1:51).
**Would be wrong if.** The engine's site search stepped off spots on its own (it did not: two orders, no site).
**Used by.** H-ECO-BASE-LAYOUT (`beside_builder` keeps 150 from a spot).

### K-eco-lab-before-generators-empties-the-store
**Claim.** A commander building the lab draws about 80 energy a second; with only the commander's 30 coming in the
store is empty within ten seconds and stays empty while wind generators (which cost energy to build) go up one by
one, and everything, the lab's first units included, builds at a crawl for a minute and more.
**Status.** demonstrated (2026-09-22, human-3; the people in the game said the same: "you are about to run out of
e", "solar is better when you are out of e, because you do not e stall making solar", "this map has 12 wind")
**Evidence.** human-3: lab created 0:37, energy 351 at 0:34 falling 56 a second, zero from 0:46 to 1:54 through
four windmills; the first constructor took 43 s (1:10 to 1:53). The same shape in human-1 and human-2 (energy 0 at
1:00 both).
**Would be wrong if.** The lab's draw were the windmills' (usage was 80 to 86 with the lab alone under construction,
30 between builds).
**Used by.** the brief's map paragraph (one solar, then wind); the lab option's draw words (H-HANDS-MENU); the flow
words' seconds to empty.

### K-energy-plant-draw-outruns-solars
**Claim.** A factory draws `build_power x energy_cost / build_time` energy a second at full speed: a Vehicle Plant on
Blitzes 68, an Aircraft Plant on Stormbringers 132, an Advanced Vehicle Plant on Bulls 339 (seventeen Solar
Collectors), a Construction Turret helping with a Bull 113, the commander building the Advanced Vehicle Plant 155.
The player, shown the store, the income and a stall word, queues solars in threes and sixes after the stall and never
catches a tier-2 plant: the energy store was empty 42 % of evidence-3-bulldogs ("the armavp has been starved of
energy for six minutes"), 46 % of roster-2, 25-43 % of the Opus A/B games.
**Status.** demonstrated for the numbers (2026-09-22, `crates/bot/data/units.json`: `energy`, `build_time`,
`build_power`); the remedy (H-HANDS-ENERGY-DRAW's budget line) untested.
**Evidence.** the ledger rows named; `run/floor.py` `e0%`.
**Would be wrong if.** The engine's build rate is not build power over build time (it is, for a lone builder at
full resources), or the player ignores a stated shortfall as it ignored the stall word.
**Used by.** H-HANDS-ENERGY-DRAW.

### K-sim-player-games-army-underestimated
**Claim.** Against the player games of 2026-09-22 (thirteen Armada vehicle openings on Comet), the build-order
simulator runs 15-25 % high on metal income from minute 4 and 30-50 % low on army metal built, and puts the first
Vehicle Plant 11 s late (71 s against 60): its replay carries what was built, not who assisted, and its walk-and-build
constants were fitted on bot labs. Its army curve is a floor for a fed plant and its income a ceiling.
**Status.** demonstrated (2026-09-22, `buildorder calibrate`; the table in `docs/studies/build-order.md`).
**Evidence.** `run/matches/1790104460-evidence-3-bulldogs`, `1790106884-ab55-jev-medium/0*`, `1790106884-ab5-jev-medium/0*`.
**Would be wrong if.** The assist and the factory overhead, once modelled from these records, close the army gap.
**Used by.** H-PLAYER-PLAN-SEARCH (the tools' note to the player).

## Comet Catcher Remake 1.8: the replay survey (2026-09-23)

Claims from the game cards of 29 public 1v1 duels (both players OS 25 and above, games of 2026-09-20 to 2026-09-23 on BAR test-31357 to 31383), filed by the synthesis agent against `docs/knowledge/questions.md` and checked with `run/replays/check.py`; the map file `docs/knowledge/maps/comet_catcher_remake_1.8.md` and the index `docs/knowledge/replays.md` point here. Spot numbers are the cards' (the game's own 80-spot list), not the bot's (the engine's 75); they map to the bot's by position (nearest within 130), and every spot carries its grid cell (8x8 over 8192x6144). Unit names: armfav Rover, corfav Rascal, armflash Blitz, corgator Incisor, corak Grunt, armpw Pawn, armflea Tick, armstump Stout, corraid Brute, armllt/corllt the light laser turret. "W"/"L" are the winners' and the losers' sides. Remaining card limits: `fights` is the eight costliest cell-minutes per game; the cards carry no unit positions over time (held back or chasing cannot be read); the killer of the first extractor is "?" on 10 sides; one manifest start (bcaeb16a, Chronopolize) disagrees with the card's B5 and the card is used. Fixed after the survey (cards re-cut 2026-09-23 evening): the faction field, the spots' cells, the first-enemy-seen field (removed: a replay sees everything), the factories' order, the last 15 s of kills counted apart, cells on every building, the first tier-2 unit.

### K-eco-winner-pulls-away-8-to-12
**Claim.** On Comet Catcher the income gap is small to 8:00 (winner minus loser, median: 0 at 2:00, 2.3 at 4:00 and 6:00,
6.9 at 8:00) and opens after (13.6 at 10:00 over 22 games, 21.2 at 12:00 over 18). The winners spend in the window and
bank after it: at 10:00 7 of 22 winners and 15 of 22 losers had 500 or more metal stored; at 12:00 11 of 18 winners
and 9 of 18 losers, at 14:00 9 of 14 and 4 of 14. Energy is tight on both sides (energy stored under 50 at some minute
from 3:00 to 8:00 in 40 of 58 sides) and the winners buy more of it: solars by 8:00 median 16 against 13, energy income
at 8:00 400 against 330.
**Status.** observed (2026-09-23, replay survey)
**Evidence.** all 29.
**Would be wrong if.** The losers' bank at 10:00 came from a factory killed rather than unspent income (the card cannot
tell). The spend-then-bank shape contradicts K-open-comet-vehicle-list's "2,100 metal banked from minute 9 on one
plant" as a habit to keep: the people's winners had under 500 banked at 10:00 in 15 of 22.
**Used by.** (candidate: the bot's spend rule from 8:00; the plateau at 22-25 extractors)

### K-eco-comet-pros-rez-bots
**Claim.** On Comet Catcher, players at OS 25 and above collect wrecks with resurrection bots more than with
constructors: 24 of 58 sides built them (11 winners, 13 losers), a median of ten a side, the first between minutes 4
and 10 (winners: 3, 4, 4, 4, 5, 6, 8, 8, 10, 13, 15); the bots worked wrecks on their own ground, not the field a
fight had just left (a bot within 400 of a death in the next 90 s after 22 of 2,730 deaths on the winners' side, 17 of
3,403 on the losers'); a constructor stood within 250 of a fresh death within 60 s after 15 % of deaths (median side);
income above the extractors' yield (2.1 a spot, 8.5 a moho, plus 8) in 12 % of the winners' minutes against 7 % of the
losers'.
**Status.** observed (2026-09-23, the replay survey's records, not its cards: the cards carry no wrecks or reclaim).
**Evidence.** the 29 matches' `record-<team>.jsonl` (created events for armrectr/cornecro; own-unit positions in the
samples against death events; resource samples against extractor counts), tallied in this session; to be a card
field (rez bots built, first at, income above extractors).
**Would be wrong if.** The replay records' income excluded reclaim (then the spike share says nothing), or a wider
radius showed constructors on the battlefield after most fights.
**Used by.** H-WAKE-WRECKS; H-REC-CREW under the pianist; the brief's economy section.

### K-eco-a-factory-lane-on-a-cliff-chokes-it
**Claim.** Every building of ours faces south and the site search never looked at the ground in front, so a factory
placed with its exit lane on unwalkable ground chokes: its units leave into the cliff and stand. In
bluegecko-3v1-comet-catcher-4 (2026-09-27) three of five factories had it (the air plant at G1 (6856, 944): 134 of
294 lane cells unwalkable for bots; t3's labs at (6652, 2638) and (5036, 2526): 70 and 61 of 273), the other three
directions clear at each, and seven of the ten "exit lane is blocked" wakes were theirs; the user saw the units
choking in one. The engine's site search takes keep-out strips, so the ban is expressed as the mirror of every
unwalkable cell near the anchor (up to about 1,100 strips for a vehicle plant on Comet Catcher's east strip).
**Evidence.** `run/matches/1790473186-bluegecko-3v1-comet-catcher-4/00` (terrain-0.bin, terrain-2.bin against the
`finished` events' positions and facing 0); the user's note.
**Status.** Fixed 2026-09-27, unmeasured in a game. Exploited by [[H-ECO-YARD-LANE]].

### K-eco-comet-no-converters-store-the-energy-and-two-hundred-a-second-with-nanos
**Claim.** On Comet Catcher (a metal-heavy map) energy converters are wasted metal; spare energy goes into an energy
store, which the tank phase spends on bombers ("if you have extra e make an e store, you will need the e later
anyway; you can use it for quick bombers": thebluegecko), and the economy aims at 200 energy a second with
construction turrets beside the plants by 5:30 ("get to 200+ E per second and add nanos"; "orange and red both need
nanos"; game 3: "you dont have enough build power", "orange needs nanos"). Our seats built converters (game 3, 4)
and no nanos until told, and ran the energy store to 1,736 falling 434 a second at 7:18 of game 5 on solars alone
behind three vehicle plants.
**Evidence.** `docs/knowledge/_inbox/players-chat-2026-09-27.md`; game 5's turn at 7:18.
**Status.** supported (the players' words, 2026-09-27), unmeasured. Exploited by [[H-PLAYER-PLAYERS-ADVICE]].

### K-eco-a-rising-store-buys-build-power-and-tier-2-pauses-the-seats-other-plants
**Claim.** When the metal store rises the answer is build power (constructors first, construction turrets where
energy allows: "con turrets take a lot of e"), constructors expanding with every one of them and standing at home
only when metal is spare; and when a seat's advanced plant comes online its other plants pause so the seat's income
goes into tier-2 units ("when your t2 lab first comes online, it's often worth it to stop production at other labs
owned by that hand to focus m income"; "one constructor and then tanks"). thebluegecko, game 7: "it is very
important to keep taking mexes, the human player is taking mexes so much faster that 3v1 is about fair".
**Evidence.** `docs/knowledge/_inbox/players-chat-2026-09-27.md` (game 7).
**Status.** supported (the players' words, 2026-09-27), unmeasured. Exploited by [[H-PLAYER-PLAYERS-ADVICE]].

