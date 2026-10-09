# The late-game economy: what it takes, from the simulator and from games

Status: draft 1, 2026-10-09, after two evenings of discussion with the user (2026-10-08 and 09). Written before
anything is built, as the ground truth for the arc. Built so far: the knowledge claims it cites and the builders
table in the brief (`docs/briefs/experience/units.md`). Nothing in the bot, the simulator or `bots/` has changed.

## 1. What is measured

The briefs know the economy to about minute 15 from people's games and know tier 2 as arithmetic plus our own
failures. The reference tables of the experience brief end at minute 16 (Armada) and 18 (Cortex) on 7 to 13 sides;
4 of 80 pool sides ever started an advanced factory. Our own late game, from the ledger:

| Shape | Games |
|---|---|
| Our advanced plant up at 21 to 27 min, first moho 25 to 33 min; his at 10:41 and 15:12 (medians) | player-16, 30, 31, 45 |
| Energy at zero through the whole plant build | player-10, 11, 32 |
| Income per extractor 2.0 against the pool's 4.7 | player-16 |
| Banks of thousands with constructors idle and energy stalling ("Energy STALLING with 6.2k metal banked and ~20 constructors idling", human-18 17:54) | human-18, player-50, the apex games |
| Advanced plant at 0% for three and a half minutes under one constructor | bluegecko-3v1-10 |
| Income peaks at minute 12 and falls (player-31: 64.8 at 12:00, 20.7 at 26:00); extractors peak at 25 to 39 by 18:00 and halve by 26:00 | player-30, 31, 32 |

The player's report shows stocks (store, income, usage, energy, a stall word, curves 3 and 6 minutes back) and not
flow: no build power, no idle-factory or store-full seconds, no constructors by what they are doing. The hands'
picture carries the energy-draw budget line (H-HANDS-ENERGY-DRAW) and the commander's report a build-power line
(`command.rs` `build_power_line`); the player's report has neither. The arena calls a game at a 3x or 5x lead
(`Settled`) and caps it at 30 to 40 minutes, so a game is almost never about the economy.

## 2. The user's rulings (2026-10-08 and 09)

1. **Lean less on the human replays for this.** Strong duels end before tier 2 and team games carry too much
   context. The experience comes from running games, real and in the build-order simulator: what it takes to beat
   a basic enemy while reaching a level of economy.
2. **Scaling into the late game off a small set of spots is a key skill**, above all for team games, where our side
   of the map saturates in minutes.
3. **Nuke rushes are to be learned in theory and in practice.**
4. **The late game is heavy build-power concentration**: massed constructors and construction turrets pushing power
   into the critical economy structures. Base geometry is an arc of its own, so this arc may need explicit logic for
   a mass of turrets near the structures we want, or a preference for massed bot, vehicle or air constructors. Air
   constructors are interesting (no collision, they huddle densely in the air round a structure); anti-air is the
   obvious downside.
5. **The tier-2 construction turret is not in normal play**: it is the "Extra Units Pack" lobby option, rarely on
   in serious play (K-rules-extra-units-pack-is-off-in-normal-play).
6. **Minelayer ships assist** (they are the naval construction-assist units); **the land minelayers do not**. The
   unit files say nothing either way (K-t2-mobile-build-power-by-cost).
7. Agreed as next steps (2026-10-09): the build-power units into knowledge and the brief's unit tables; a check that
   the simulator's palette takes them (it does: any mobile unit with a build list is a builder to it,
   `anneal.rs` `Palette::roster`); a cheap `bots/` row, a Butler swarm against a turret ring on one fusion.

## 3. What the arithmetic settles (claims written today)

- **Build power by cost** (K-t2-mobile-build-power-by-cost): the construction turret is the cheap power at 1.15
  metal a point with a 400 reach and no mobility; the tier-2 fast assist units (Butler `armfark` 1.50, Consul
  `armconsul` 1.67, Twitcher `corfast` 1.68) are the mobile ground power at near the turret's price, from the
  advanced labs; advanced constructors 1.9 to 2.1; air constructors 2.1 to 2.8 with a seventh of a turret's health.
  On water the Voyager and Pathfinder (`armmls`, `cormls`) are the cheapest mobile power in the game at 0.95 to
  1.00 with a ship's health, and the construction seaplanes beat the land air constructors on price and health.
- **A project's power follows the spend** (K-t2-build-power-for-a-project-follows-the-spend): a frame builds at the
  sum of what touches it until the store empties, then everything on it slows in proportion, so power beyond
  `build time x spend / metal` only stalls. An advanced fusion fed at 100 metal a second wants about 3,200 power
  and stands in 97 s; at 50, half that in twice the time. The players' "stop the other factories during the
  transition" is this rule.
- **The ring is single-site.** Without the tier-2 turret, 6,000 power is thirty turrets within 400 of the frame.
  A ring pays only when the projects it serves all stand inside its reach: the eco core as a cluster round one
  site, built once. Mobile power earns its price for everything outside it.
- **The nuke race** (the unit files): Armada's silo is 8,100 metal, 90,000 energy, build time 178,500 (fourteen
  minutes for one advanced constructor, three for a thousand power); a missile 1,000 metal, 125,000 energy, a
  two-minute stockpile; Cortex 7,700 / 82,000, a missile 1,500 / 187,500 / three minutes. The counter is an
  anti-nuke at 1,500 metal with a 2,000 coverage and a 90-second, 150-metal first interceptor. BARb hard_aggressive had one in
  27 of the 33 long player games, standing at minute 21 to 25 (median 22); medium in one long game in six
  (K-barb-anti-nuke-by-minute). A nuke must therefore be stockpiled before about 21:00 to land on hard_aggressive, which
  the silo's build time puts out of a Comet duel's reach: in the arena the nuke is not a rush weapon, and the
  practice half of ruling 3 is for games with people (the seat with no anti-nuke) and for team games.

## 4. Decisions proposed (mine, for ratification where marked)

1. **The simulator is the instrument for the frontier.** `crates/buildorder`: the palette takes the silo and the
   anti-nuke (they neither make, store, convert nor extract, so today's rule leaves them out); a spend-floor
   objective (metal a second that must go to soldiers, the tax of a basic enemy); a recalibration against our
   tier-2 records (player-30, 31, 32 carry advanced plants and mohos; the calibration today is thirteen openings to
   minute 6, 15 to 25% high on income). Output: frontier tables per map and spot count (11, 22, 40 spots: the
   fastest order to income 50 / 100 / 200, the first moho, fusion, advanced fusion, the first missile), each with
   the build power it wants, with and without the spend floor, by a generator under `run/` whose output
   `run/build_brief.py` assembles like the reference set. The gap between the simulator's frontier and the engine's
   result against BARb is the cost of pressure and is itself tabled.
2. **The player's report shows the economy as flow, observations only** (the tempo memo holds): the commander's
   build-power line; the picture's draw-against-income line; factory-idle and store-full seconds since the last
   turn; constructors by idle / helping / expanding; an upgrade line (extractors of ours on ground no enemy has
   neared for some minutes, and what mohos there would add in metal, cost and energy upkeep); payback seconds per
   income source at this map's spot value.
3. **The brief gains a late-game section** in its own form: the frontier tables; the identities as arithmetic
   (power per income, a project's power follows the spend, the moho's 90 s payback, converter rates, a tier-2 army's
   energy, reclaim at 60%); our transition cases with their mechanisms; the builders table (done); the players'
   words. The argument section gets one paragraph: a lead is spent or teched, never banked.
4. **Measurement: economy-objective games.** The arena's `--objective` with `--play-out` (no calls), on Great
   Divide (22 spots, both sides at 11 from minute 4 in the seats games) and Avalanche (19), against BARb medium
   then hard, never hard_aggressive for this: scored from the record (income curve, energy-empty share, store-full
   share, idle builders, time to tier 2, fusion and the first missile, power in reach of the project over its
   build, stuck seconds, frame-stall seconds). The commander-over-Sonnet configuration for the tier-2 and nuke
   calls (the third-seat ruling: decide from arithmetic, not build plans).
5. **`bots/` is the mechanics lab.** Tier 2 into the distilled bot (advanced lab, advanced constructor, mohos,
   fusion, Butlers, help on a project), then eight-game rows: tier 2 at an income against never; mohos before fusion
   against after; converters on a wind map; a Butler swarm against a turret ring on one fusion; the 85% spend rule.
   Findings go to `docs/knowledge/` as game facts labelled by map and opponent, never into `crates/bot`.
6. **Placement, a section-6 rule (the user's word needed).** A core mark with a reserved ring; projects named by
   list-step id sited inside it; a turret listed against a project sited at a free site within its reach, off every
   lane and spot. The engine's site search with keep-out strips does it.
7. **A mechanic, widened (the user's word needed).** An idle construction turret helps the dearest frame in its
   reach, else its factory (today: the nearest finished factory only, `nanos.rs`).
8. **A crew actor in the hands.** Massed constructors as one actor with one menu (help this project, take apart that
   field, build this): Jev's bill follows the actor count (player-40: five times its opening rate at thirty
   actors), so a swarm cannot be a swarm of actors.
9. **The nuke arc**, after the economy steps: protocol `Stockpile` and an attack at a position (the shim has a D-gun
   order and nothing else of the kind), the silo as an actor, anti-air sited with the project, and the firing pick:
   the player's tool or a silo actor on Jev's menu aimed at a place with his buildings known (the user's).
10. **Roles for build power** (the brief says the numbers; the player decides): turrets for the core cluster;
    Butlers, Twitchers and Consuls as the mobile ground swarm (the Consul and Twitcher carry a tier-1 constructor's
    build list and also expand); advanced ground constructors for mohos and expansion; air constructors only where
    ground cannot stand; on water the Voyager and Pathfinder first, then the construction seaplanes.

## 5. Order of work

1. The simulator's frontier (offline): palette, spend floor, recalibration, the generator and its tables. Judged by
   the calibration error on the tier-2 records and by a reviewer reading the tables against the unit files.
2. The report's flow lines. Judged by a replayed report of player-32 and human-18 at the minutes named in §1.
3. The brief's late-game section from 1 and the claims. Judged by a blind review of the section against the claims.
4. `bots/`: tier 2 in the distilled bot and the rows of §4.5 (can run beside 1 to 3). Judged by `run/bot_curves.py`.
5. The economy-objective games (§4.4), first in `bots/`, then the flagship. Judged by the record's scoring lines.
6. The nuke arc (§4.9).

## 6. Open for the user

- §4.6 (the ring as a placement rule) and §4.7 (the widened turret job): both need his word, being code that acts.
- The order of §4.8 (the crew actor) against §4.9 (the nuke harness): the silo is the first project that needs the
  crew.
- The firing pick (§4.9).
- Which basic enemy for each step: medium for the economy objective, hard for the tax, is the proposal.
- Whether tier 2 in the distilled bot (§4.5) starts now, beside the simulator work.

## 7. Not in this arc

The base-geometry arc beyond the core ring; team-game roles (the eco seat) beyond the feed-one-seat pattern
(K-team-two-seats-feed-one-seats-tier-2); the human replays for the late game (set aside by ruling 1; to revisit if
the simulator and the games disagree with what people do); the T3 experimentals.
