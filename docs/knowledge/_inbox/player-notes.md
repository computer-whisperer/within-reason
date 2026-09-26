# Notes from experienced players (unreviewed claims, to be tested before anything rests on them)

## 2026-09-20, a more experienced player watching cmp-opus game 00 (relayed by the user)
- **Mass tier 1 with tier 2 up is a mistake.** Once the production exists to turn resources into tier-2 units, people
  stop making tier-1 soldiers and often reclaim ("eat") the Maces they have to pay for tier 2. Ours: H-T2-PRODUCTION keeps
  the tier-1 labs making soldiers while the advanced lab runs; nothing reclaims our own units. Untested.
- **Maces as the first units of the game are wrong.** Ours: H-PROD-MIX and the opening plan both put a line unit (Mace,
  Thug) first. Untested in games.
- **Well-microed Grunts kill Maces without taking damage early; the numbers shift a few minutes later.** First look in
  `combatsim` (2026-09-20, no dodging micro, 16 seeds): 10 Grunts (430 metal) beat 4 Maces (520) every time, margin
  +0.15, +0.57 when spread at 120; 20 Grunts lose to 8 Maces every time (-0.29). 10 Pawns against 4 Thugs: +0.16. The
  direction agrees: small early numbers favour the raider, bigger blobs favour the plasma unit. "Damageless" needs
  dodging the simulator's policies do not have.

## 2026-09-20, more from experienced players (relayed by the user)
- **Too many construction bots, made and kept out on the map.** Measured the same day: constructors alive at minutes
  10 / 20 / 30: cmp-opus 7.8 / 8.2 / 9.0 (up to 13), cmp-sonnet 8.4 / 8.6 / 10.8, heuristic (now-quicksilver) 7.0 / 8.8 /
  8.4 (up to 12); Opus has 6.4 at minute 5 against 3.5. What number is right, and what the surplus should turn into
  (assisting a lab, being reclaimed), is not known.
- **The tier-1 bot labs are themselves reclaimed once tier 2 is active**, as are the Maces. A timing point, not a
  rule for the rest of the game: **later, tier-1 raiders and general unit spam become useful again.** Untested; ours
  never reclaims a building or a unit of its own.

## 2026-09-27, the user reviewing onepass-player-8 in the viewer (his own words; "measured" lines by the main session the same night)
- **2:32 "a very predictable tick returned and resumed attacking a mex it was chased off of just a bit ago. Object
  permanence for jev is a recurring problem that we don't have a good solution for."** Measured: party_1 (a Tick) left
  sight at 2:24 at (2652,4948); party_3, the same Tick, appeared at 2:30 at (2237,4509) and was hunted by one of group_A
  at 2:31; the hands hold no link between the two. The same shape at 4:34-5:01: the Pawn that killed spot_63 at 5:05
  was seen at 4:34 far east, lost at 4:41, seen again at 5:01 and answered at 5:05 (`run/raid_ledger.py`, `hands_window`).
- **5:06 "we loose a mex to a single raiding pawn that it seems we could have defended with some blitzes nearby if they
  were sent soon after the pawn appeared on radar."** Measured: two soldiers within 900, no turret; first sight in the
  episode 7 s before the death, the hunt of one Blitz ordered at 5:05, the same second the extractor died.
- **6:41 "we start loosing structures to a pawn near our starting location while a rover sits nearby."** Measured:
  party_13 (one Pawn) killed spot_36 6:32, spot_45 6:50, spot_50 6:57 and both construction turrets 7:07-7:11; the
  answer from 6:29 to 7:11 was `commander: attack party_13; its list step waits` (the player's instruction; the rule
  that a slower chaser still drives a raider off); the Rover was group_E under the player's "runs from anything that
  shoots" and fell back home from the Pawn at 6:42. The Rover was never offered as an answer: the threat pass
  excludes a unit the party outweighs (31 metal against 54). **Retracted by the user the same night:** "I am told by
  an experienced player that the rover cannot repel the pawn, so that call was correct. I retract that issue and the
  correct response was better army stationing." The Rover was also stuck at home for 1:30 (the entry said so).
- **7:07 "our army is in a poor position. A 5-pawn detachment is visible in radar towards the north while our army
  either chases the one pawn near our starting point or sits idle in the middle."** Measured: party_17 (6 Pawns) at
  (3308,2226) at 7:00 with group_G told "attack party_17 with the whole group" [rule]; party_22 (6 Pawns) appeared
  7:13; group_G was told "walk to home" [rule] at 7:13 and "1 of group_G hunt party_13" [plan] at 7:19.
- **7:28 "we start responding, but some of the tanks hesitate and turn back, leaving the one tank that actually
  completed the initial attack movement to die alone."** Measured: group_A's order alternated between "attack party_22
  with the whole group" [rule, 7:13] and "4 of group_A hunt party_22" [plan, 7:21], then both offered at 7:35 for
  party_23; the two worlds win on different seconds (`hands_window 7:00 7:40`).
- **10:30 "I notice we apparently never built construction turrets? We are starting to float metal."** Measured: two
  built (4:48, 5:50), both killed by party_13's Pawn at 7:07 and 7:11, none again until 10:48; the store rose from 813
  at 10:00 to 2,736 of 3,050 at 12:00. escalate-7 had five by 12:00, player-6 two, none lost.
- **12:33 "unanswered enemy pawns burning bases in the south while our main army fights in the north. A detachment is
  sent eventually that kills the pawns but we already loose some structures."** Measured: spot_56 12:26, spot_44
  12:36, spot_47 12:39 to a party of 8 Pawns at D5/D6; the answer "group_A: attack party_59 with the whole group" at
  12:35, 19 s after first sight; seven raiders paid.
- **14:40 "the detachment in the south-east is ordered across our base to engage units in the north-west that are
  already covered, leaving the south-east undefended."** Measured in part: at 14:35 group_H (two Blitzes at B4) was sent
  after one Pawn at A1 (860,862) and called off at 14:45; at 14:43 four of group_A after a Pawn at B1. Which group the
  user watched is not pinned; the shape (a near group sent far while parties stand near) is in the log.
- **16:45 "we walk our north-east army back and forth under fire from a gauntlet."** Measured: group_G "walking to
  where it last held, passage_2" at 16:53 under losses to unseen fire (the fall-back rule), "advancing to far_north"
  again at 17:03 after the player's `fall_back_to far_north`; the rule stands over any packet written before the last
  loss, and under steady shelling every packet is.
- The user ended the review at 16:45: "other issues past that point ... may be more of the same artifact."
