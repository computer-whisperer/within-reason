# Opponents

### K-opp-barb-tiers
**Claim.** BARb has profiles `easy`, `medium`, `hard`, `hard_aggressive` (plus `dev`), selected with `[OPTIONS]{profile=...}`;
the default is `hard`. It loads its AngelScript and config from the GAME archive, not the engine's AI directory.
**Status.** supported (2026-09-19)
**Evidence.** `AI/Skirmish/BARb/stable/AIOptions.lua`; engine log "Load script: LuaRules/Configs/BARb/stable/script/hard/init.as".
**Used by.** arena `--profile`.

### K-opp-faction-asymmetry
**Claim.** Against BARb easy our results follow the OPPONENT's faction: easy BARb is much harder as Cortex (it raids) than as
Armada (it does not). It is not our Armada roster.
**Status.** supported (2026-09-19)
**Evidence.** v5-easy-confirm: Cortex 10-2, Armada 4-8 (also SE 9-3, NW 5-7). The arena always gives BARb the other faction,
so "our Armada roster is weak" and "BARb plays Cortex better" cannot be told apart.
**Likely cause (2026-09-19, see K-barb entries in opponents-barb.md).** In `config/easy/behaviour.json` armpw is relabelled
`skirmish` (line 411) but corak is still `raider` + `scout` (line 750): easy BARb raids only when it plays Cortex, which is
exactly when we play Armada. Config fact verified; that it explains the split is inference.
**Mirror test (v5-easy-mirror, 24 matches).** vs Armada BARb: 7-2-3 as Armada (and 10-2 as Cortex in v5-easy-confirm); vs Cortex
BARb: 5-4-3 as Cortex (and 4-8 as Armada). The split tracks BARb's faction. In that batch losses averaged 66 base-defence
firings and 10 commander retreats per match against 5 and 0 in wins.
**Would be wrong if.** A batch with BARb's `disabledunits=corak` did not close the gap.
**Used by.** (none; needs a mirror-matchup option in the arena)

### K-opp-medium-wins-by-20
**Claim.** BARb medium beats the v5 brain every time, between minute 16 and 20.
**Status.** supported (2026-09-19)
**Evidence.** v5-medium 0-12. Diagnosis (2026-09-19): the loss is decided around minute 8-10, not 16-20 — our extractor count
peaks at 4-9 near minute 8 and collapses to 1-3 by minute 10-12 (checked in v5-medium/00, /03, /07, /10). Medium makes its
basic bots raiders, and every BARb fighter carries ANTI_STAT (targets buildings, skips mobile units), so raids walk past our
army to the outermost extractors. In v5-easy-confirm, extractors at minute 10: wins 8-15, losses 1-12 (overlapping, so
extractor survival is a strong factor on easy, not the whole story).
**Used by.** (none)

### K-opp-thebluegecko-one-ball-behind-turrets-then-tier-2
**Claim.** [gecko]thebluegecko (lobby skill 34, OS 48 in the duel list) plays the same game whatever the odds: the
whole side's spots by 5:00-9:00 with three or four constructors and no outposts, two turrets at home and
turrets at whatever narrows the way in, one unit type massed into one ball that sits on its own turrets and never
splits, tier 2 at 10:00-11:00, and one push at 15:00-18:00 with the tier-2 army that ends the game. Against
several opponents the same, and the raids come first: scout cars and Blitzes from 2:30 into every seat's outer
extractors while the ball grows.
- Great Divide V1 alone against six BARb hard_aggressive (2026-09-26, 19:45, won): Armada north; 2 spots by 0:30,
  the lab at 1:11, three Beamers and two rocket AA at the pass's north mouth by 5:10 and nothing else forward, 2 to
  7 Pawns and 2 Warriors the whole first ten minutes, 11 of the 11 northern spots between 6:39 and 9:02 with about
  25 wind generators, three nanos, the advanced bot lab at 10:09 (first tier-2 unit 11:01), then Fidos, Zeus,
  Sharpshooters and Archangels: 5.9k of army at 16:00, 9.6k at 18:00, and the sweep through all six seats from
  15:00 (the fighting cells: D5 at 5 and 7 with 26 of theirs dead and none of his, then F7, G7, C7, D7, F8 at 15-18).
- Altair Crossing alone against two BARb hard_aggressive (2026-09-26, 20:42, won): Cortex; 16 Grunts at 4:00 to
  take and hold the west strip's spots, then Thugs only: 13 at 8:00, 20 at 10:00, 32 at 16:00, 36 at 18:00, with
  radars and a light turret at each outer spot; 10 spots at 12:00, 15 at 20:00; one push at 18:00 into both
  bases (H2, H4, H5: 3/21, 4/15, 0/16) that ended it. No tier 2 at all.
- Great Divide V1 against two seats of ours (bluegecko-2v1-great-divide, 15:34, won): all 11 northern spots by
  5:25 with three constructors; two light turrets at home by 0:58, two rocket AA by 6:26, radars, a nano at 7:13;
  Warriors only (8 at 8:00, 12 at 12:00) that met our Hammers and Thugs at the pass's south mouth (D7, E7) from
  5:00 as they arrived one group at a time: 42 Thugs, 37 Hammers and 16 Storms of ours for 16 Warriors of his; the
  advanced bot lab at 11:16, Zeus and a Fatboy by 14:00.
- Comet Catcher against three seats of ours (three games, 2026-09-26/27, all won): scout cars and Blitzes at every
  seat's outer extractors from 2:30 (17, 45 and about 30 extractors of ours lost per game), Stouts and Janus in
  blocks of ten from 10:00 into one seat at a time, Banshee gunships in a wave of twenty at 14:00 in the third game,
  a Mauser shelling from out of sight at 16:00, Bulls from 18:30, thirty Falcons by 21:00, and each of our
  commanders killed at its plant by the block. He never met our ball: it fought his raids and blocks in pieces.
  His side of the second game (`run/matches/1790471894-replay-human-comet-3v1-2/card.md`, team 0): three
  extractors and three solars, the vehicle plant at 0:58, Rovers from 1:28 (nine at 4:00) and a constructor every
  fourth unit; extractors 18 at 8:00, 24 at 10:00, 34 at 14:00, 49 at 20:00 (the strong duellists' 23 at 12:00, ours
  32 over three seats), income 50 at 10:00, 132 at 16:00, 176 at 20:00; a second vehicle plant at 9:12, the advanced
  one at 16:08, Bulls from 18:00 (twelve at 20:00). He lost 63 Stouts, 56 Rovers and 28 Janus that game and could,
  on twice our income. The raids and blocks took our outer spots and his constructors took them over (spot_23 E3 at
  4:56 and 15:15, spot_37 E4 at 14:07, spot_1 D1 at 14:22).
**What it means for us.** He wins the trade by never fighting our whole army: our seats meet him one at a time.
What beats his way is the same way, held together: the whole side's spots early, one army on turrets between the
seats, tier 2 by 12:00 (he brings Bulls, Mausers and gunships at 14:00-18:00 and we have nothing that reaches
a Mauser or outguns a Bull), flak at each home by 14:00, and a ball that goes at his plants only as one body when
it outweighs what stands there.
**Evidence.** Cards: `run/matches/1790471170-replay-gecko-gd-1v6-barb/card.md`,
`run/matches/1790471255-replay-gecko-altair-1v2-barb/card.md`, `run/matches/1790471126-replay-human-gd-2/card.md`
(his side is team 0 in each); the three Comet games' records and reviews (`run/matches/1790467496-*`,
`1790469236-*`, `1790471259-*`); his 17 public duels in replays.md (13 on Comet Catcher, 12 of 17 won).
**Status.** supported (2026-09-27, six games). Exploited by [[H-PLAYER-SEATS-VS-A-PERSON]].
