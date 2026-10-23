<!--meta
id: D-tier2-sprint
layer: derived
premises: [D-metal-is-the-game, D-lead-convert-it, F-eco-store, F-eco-extractors, F-eco-energy, F-eco-build, F-eco-decay, F-eco-consequences, F-fight-range, F-fight-consequences, F-unit-armavp, F-unit-armalab, F-unit-coravp, F-unit-coralab, F-unit-armacv, F-unit-armack, F-unit-coracv, F-unit-corack, F-unit-armmoho, F-unit-cormoho, F-unit-armadvsol, F-unit-armbull, F-unit-armfboy, F-unit-armguard, M-pool-tier2-and-wrecks, M-pool-closing, M-barb-hard-aggressive-comet, M-barb-bonus, M-gecko-team, M-format-team-small, M-format-8v8]
lessons: [L-tier2-go, L-tier2-tier1-under-tier2, L-tier2-stop-other-units]
sources: {people: [the user, thebluegecko, Irishstud14], dates: [2026-09-28, 2026-10-18, 2026-10-23], relay: the lessons' chat lines; the user's framing of tier 2 as a sprint whose gap is covered or not (2026-10-18) and his ruling that in a close duel the opponent's tech is the moment to kill his base (2026-09-28); his steer of 2026-10-23 that the derivations stay lean, ideas with worked arithmetic rather than rules}
status: live, checked (the kill-test run 2026-10-23 with run/metal_killtest.py tier2 over 1,641 recorded 1v1s of fourteen minutes or more; the tests are correlations across our own games and say so)
-->
### The tier-2 sprint

**The heuristic.** The user: tier 2 is a sprint paid out of the income you have, and the question is whether the gap
is covered. thebluegecko, to our top seat at 17:27 of game 10, its advanced plant at zero for two minutes while its
lab kept producing: "Top should stop building other units while doing t2 transition". The plain line: the sprint is
a bill, about 3,150 metal and 620 a moho; what it costs the army is the seconds the bill takes, and those are set by
the build power on the frames and the store, not by the price; and the army's job does not change while it is paid.
A game past twenty minutes belongs to the side with mohos.

**The quantity.** Two numbers. The gap: how long the army does not grow, and what covers those seconds (a surplus
already in his half, an ally's front, his own gap). The payback: when the mohos have returned the bill, after which
the sprint is income. Not "his tech is seen": that is a fact about him, and against BARb it comes on a clock.

**The mechanism.**

- *The bill.* The advanced plant costs 2,600 metal, 14,000 to 16,000 energy and 27,000 of build; its first advanced
  constructor 550 (the bot 430); an advanced extractor 620 for four times the spot, 6.9 a second more on a Comet spot,
  repaid in 90 seconds. Plant and constructor together are 57 seconds of income at 55 a second, 45 at 70, 105 at 30.
  Our games placed theirs on 37 / 50 / 64 (quartiles).
- *Time is build power.* On the commander and four constructors (660) the plant stands in 41 seconds, drawing 64
  metal and 342 energy a second; a store of 1,000 with the plants paused covers that at any income over 40 (the pause
  frees about 20 metal and 190 energy a second). On one constructor it stands in five minutes and locks the 2,600 the
  whole time; a frame nobody stands on decays in seven. The constructor comes out of the plant's 600 in 27 seconds, 18
  with the commander assisting; a moho takes it 51 seconds alone and 22 beside two construction turrets. From the
  placement: plant 0:41, constructor 1:08, six mohos standing by 3:20 with the turrets or 6:14 without, the whole
  6,870 repaid 5.2 or 6.9 minutes after placement, income from 55 to 96. The army forgone is the bill over income,
  about two minutes of production at 55 and nothing from a bank of 2,600. The old brief's "three to five minutes" is
  the one-constructor case, or a sprint the income cannot pay.
- *Energy is the second bill.* Plant, constructor and six mohos take 67,000 energy; tier-2 units draw 13.7 to 14.5
  energy a metal against the Stout's 9.3, a plant alone on Bulls 339 a second; six mohos' upkeep is 120. One advanced
  solar (350 metal) per 80 a second of new draw, or the fusion.
- *His gap, when he is BARb.* hard_aggressive at +0 places its lab at 10:11 / 11:04 / 11:57 and its first moho 2.1
  minutes later; in our 97 games against it his army stood still for the four minutes after the placement, -525 /
  +190 / +1,379, while ours grew +2,318 / +3,965 / +5,249. With a bonus the lab comes at 8:12 / 10:06 / 11:24.
- *What tier 2 buys.* The economy: at 20:00 in our long games the side with a moho by 18:00 drew 103 to 106 a second
  from its spots and the side without 28 to 37 (both populations, medians); our own income is flat from 14:00 (37, 39,
  39 at 14:00, 16:00 and 20:00) while his doubles (39, 48, 78). The army: in the open at equal metal tier 1 still trades
  by the square law (4.2 Stouts kill a Bull in 13.6 seconds and lose two of themselves; 6.2 Stouts and a Fatboy kill
  each other in about 15.5), so "tier 2 beats any number" is false in the open. What it buys is reach and splash: a
  Fatboy 700, a Starlight 950, a Gauntlet 1,220 against the Stout's 350; a Stout walking into one Gauntlet's reach
  takes 1,228 damage, two thirds of its health, before its first shot. The cliff is a reach cliff: his base becomes
  unreachable, not his army unbeatable.

**Consider, and why it may work.**

1. *The plant as the economy's project, not the army's.* Beside the home turrets, the commander and every constructor
   within reach on it and the plants paused for a minute, the first product the advanced constructor, two construction
   turrets at the nearest spot cluster, mohos nearest home first. Why: it turns a five-minute hole with 2,600 locked
   into 41 seconds, and the first moho stands a minute and a half after placement. Our 93 plants stood after 1.4 / 2.2
   / 3.6 minutes, 20 died unfinished (18 of those games lost), and the first moho came 3.7 minutes after placement;
   today's five on this brief 1.2 and 2.2.
2. *The army's standing order does not change.* Why: the sprint costs production, not position. In 70 of our games his
   lab was placed while we led 1.5 times. Where we never placed a plant the game ended first (won 32 of 38, median 20
   minutes, half our army in his half). Where ours was placed before his, 12 of 13. Where ours followed his, 5 of 19:
   those games ran 31 minutes, our army was 12 percent in his half in the four minutes after his lab and 23 after ours
   (45 in the won games, 9 in the lost), the plant stood after 4.4 minutes and five were killed unfinished. The plant
   that came with the army standing lost both the lead and the plant; the one that came with the army in his half won.
3. *A bank is a plant paid.* 2,600 stored at 12:00 is the plant with nothing forgone, and metal above the store's cap
   is lost every second; the ladder's long losses banked 5,800 to 7,200 while the plant came six minutes late or never.
4. *Decide at his lab or at the window's end, whichever comes first.* Against BARb on Comet his lab is placed around
   11:00, and the first window of the lead ends about then (D-lead-convert-it). If the surplus is in his half and his
   extractor count is falling, the sprint waits for the kill: the pool ends 3.4 minutes after holding 2x and techs in
   4 of 80 sides. If it is not, the plant goes down now with the army's order unchanged, since his four flat minutes
   cover our 41 seconds. Level or behind in a duel the plant after his did not help (4 of 13 and 0 of 4 won); against a
   person nothing covers the gap but a lead, and the pool's answer is the kill.
5. *In a team, one seat sprints and is fed.* A seat spending as it earns has a store near zero, so feeding is the
   feeders' plants paused for the minute the frame needs: two seats at 30 a second stand the plant in 43 seconds. The
   first products are advanced constructors, one for each seat (the people's way, M-gecko-team; the won 8v8 sides hand
   eight a side, M-format-8v8).

**Where it bends.** With a bonus his lab comes at 8:12 to 11:24 and his income runs 1.35 to 1.5 times: matching it
on income that cannot pay stalls (the +50 game: plant at 8:12, the constructor 1.7 minutes to build, the first moho
at 13:06, lost); there the window is before his lab stands, and the sprint is from a bank or waits until it is. On
eleven spots a side (Great Divide) the moho is the only growth and the sprint is the plan from the first bank. The
first product a tier-2 soldier instead of the constructor (one game of today's five) puts the mohos a minute later
for one unit.

**When to depart from the line.** When his raiders are on our extractors with nothing answering: the raid answer
first, the plant after (D-metal-is-the-game). When the kill is in sight within the payback: no sprint, the surplus
goes in. When the plant would stand outside the turrets' reach: not there.

**The tests.** `run/metal_killtest.py tier2` (2026-10-23) read both sides' advanced-factory and first-moho clocks from
the record and the truth file of every recorded 1v1 of fourteen minutes or more with a result (236 played by the
model, 1,405 by the old code) and joined them to the per-minute cache. The model's games: his factory placed in 165
at 10:12 / 11:36 / 14:48, ours in 93 at 10:54 / 13:30 / 17:36; who placed one and who won: neither 21 of 58, his only
44 of 85, ours only 9 of 13, both 31 of 80; ours placed while ahead with his not yet placed won 18 of 21, ahead with
his placed 9 of 40 (item 2 has that table), level with his placed 4 of 13, behind 1 of 7. The old code's games point
the same way on a later clock (his lab at 19:24; ours while ahead 279 of 309, level 41 of 85, behind 2 of 10). All of
it is correlation across games we played: a player that keeps the army home also techs, so the 19 lost sprints are to
be read one by one when the episode index exists, and the plant's slow standing is unassigned between the player's
orders and the hands (no assist order is measured here).
