# What the heuristic bot measured (2026-09-19 to 2026-09-25)

The heuristic bot's deciders (`army.rs`, `raid.rs`, `contact.rs`, `squads.rs`, `tier2.rs`, the economy's rule ladder,
the rolling planner, the route-making scout) were deleted on 2026-09-25 (`docs/design/2026-09-25-one-decider.md`): the
pianist plays every unit from the player's packet. The rules' arena measurements are facts about the game, not about
the rules, so they stay here as claims for the player's brief and the standing rules to draw on. The rows in
`docs/heuristics.md` say `retired 2026-09-25` and point here. Scope: BARb medium on Comet Catcher and Quicksilver
unless said, 2026-09-19 to 2026-09-23, arms of 12 to 48 games.

### K-bot-a-group-that-arrives-together-trades-better
**Claim.** A group sent somewhere strings out by speed and meets the enemy a few at a time; holding the ones in front
until the body is within 175 (nobody held with an enemy within 900) raises army metal killed per metal lost.
**Status.** supported (v32, 24-game arms: 0.98 against 0.87 north-west, 0.91 against 0.75 south-east). The march
still runs for the pianist's groups (`march.rs`, H-ARMY-MARCH).
**Would be wrong if.** A batch with the march off on the same seeds traded as well.

### K-bot-the-wave-gate-should-count-every-soldier-seen
**Claim.** Gating a wave on the opponent's army as every soldier of its seen and not seen die, with no time window,
beats gating on the biggest force in sight in the last two minutes.
**Status.** supported (gate-1-ab, 48 games: metal killed per lost 1.01 against 0.83; results 12-12 against 11-13;
waves 2.4 against 3.3 a game south-east). Was H-ARMY-GATE-ALL-SEEN.
**Would be wrong if.** The player's brief, told the same, launched worse.

### K-bot-territory-by-answer-time-takes-more-spots
**Claim.** Sending constructors where we can answer sooner and harder than the opponent can arrive (a 256-elmo grid
of what each side brings to a cell within 20 s) holds more extractors than a radius from home.
**Status.** supported (terr-2: north-west extractors 7.0/7.4/7.8 at minutes 5/7/11 against 5.0/6.2/6.9 on the same
seeds; Mithril 12-15 from minute 11 against 8-9). The constructors' rule went with the heuristic bot (2026-09-25) and
the grid itself on 2026-09-27 (H-MAP-TERRITORY retired: its words in the player's report measured reach of force, not
ownership, and read as an extractor count they misled). The claim is knowledge for the next spot-choosing rule.
**Would be wrong if.** A player told the ground words expanded no better than one told a radius.

### K-bot-the-base-layout-halves-nothing-but-move-failures
**Claim.** Labs in a yard 350 ahead of the start with 8-square gaps, generators 150 behind with 5-square gaps, base
turrets 650 ahead, nothing within 100 of a spot: soldiers' move failures in the base fell from 17,000 to 149 per 8
games. The placement stands (`place_planned`, H-ECO-BASE-LAYOUT).
**Status.** supported (2026-09-19).

### K-bot-a-staged-assault-wins-more
**Claim.** A wave that gathers 1,500 short of the target and starts at 70% gathered or after 150 s wins more than one
that walks straight in.
**Status.** weakly supported (v12-stage 17-5-2 against 13-7-4, one batch each). Was H-ARMY-STAGE.

### K-bot-regrouping-the-contact-beats-retreating-the-wave
**Claim.** When the attackers in contact are outmatched but the whole wave would win, falling back only those in
contact onto the wave's centre wins more than sending everyone home.
**Status.** weakly supported (v28, 12-game arms south-east: 5 wins 1 timeout with it, 2 wins 4 timeouts without).
Was H-ARMY-REGROUP.

### K-bot-tier-2-at-income-22-is-neutral-against-medium
**Claim.** Starting the advanced bot lab at metal income 22 and energy income 450 dips income at minute 20 and repays
it by 25, in games that end at 26: neutral against BARb medium.
**Status.** supported (v31). Was H-T2-GATE. See `tier2.md`.

### K-bot-constructors-first-and-early-extractors-pay
**Claim.** The lab's first two units as constructors, and below 5 extractors every builder taking a free spot before
generators or turrets, won the v21 A/B.
**Status.** supported (v21 A/B). Were H-PROD-BUILDERS-FIRST and H-ECO-EARLY-EXPAND; the player's default opening
list (`pianist/default.md`) says the same in words.

### K-bot-the-commander-trip-cap-changed-nothing-after-the-plan
**Claim.** Capping the commander's extractor trips at 12 s of its own walking once a constructor exists moved nothing
after the opening (com-trip-ab2: 8,693 elmos walked against 8,735) and cost two extractors by 3:00 when applied to the
plan's own steps (com-trip-ab).
**Status.** supported (2026-09-20 to 21). Was H-COM-TRIP; the leash is the player's to write.

### K-bot-a-search-of-the-opening-matches-a-hand-written-one
**Claim.** The annealed build-order search (`crates/buildorder`) plays an opening as faithful as the rules playing
the same opening by hand (open-exec-1-ab), and its army term at 6 times a soldier's metal when it can stand at the
opponent's base by 2:30 gives 10 soldiers by minute 3 beside 5 extractors on Quicksilver's north start, near the
experienced player's 7 beside 4.
**Status.** supported (2026-09-20 to 21). Were H-OPEN-PLAN, H-OPEN-SEARCH, H-OPEN-CONTACT; the search lives on as
the player's `plan` and `search` tools (H-PLAYER-PLAN-SEARCH).

### K-bot-resurrection-crews-did-not-pay
**Claim.** Building resurrection bots ahead of the mix, one per 600 metal of safe wrecks, left the bot behind on
income, extractors and soldiers to minute 20 (rec-3-ab, 8 games an arm).
**Status.** supported (2026-09-22). Was H-REC-CREW; under the player the crew is its to `produce`.
