# The coverage pass of 2026-10-22: the old brief against the new set, by substance

Eight Opus 5.5 readers, one file group each, judged every passage of the annotation study that `run/brief_coverage.py` counts as open (492 of 523; the 31 with a `covers:` entry left out) against the documents of `docs/briefs/rework.manifest` as they stood at a3693a6 (facts, meta, derived/metal.md, derived/lead.md, lessons.md). The coverage tool's tally is bookkeeping (a `covers:` list in a new document's meta block); this pass asks whether the substance is there. Each row's verdict: **covered** (the claim, number or reasoning is in the new set, in any layer), **partly** (a core is there, a named piece is not), **missing** (nothing carries it), **deferred** (an episode whose lesson is not carried either; the episode index and the worked examples are not built, by design), **out** (tool text, a harness ruling, or a discard that should go). `verdicts.jsonl` beside this file has the 492 rows with the reader's note, the place that carries a covered row, the layer and backlog subject of a gap, and any conflict between the old and the new text. **Status: model verdicts.** Twelve were spot-checked against the texts by the main session and held; three conflicts inside the new set were checked against the game's files and fixed (a3693a6); the rest are candidates until a person reads them.

## The tally

| verdict | rows |
|---|---|
| covered | 145 |
| partly | 259 |
| missing | 27 |
| deferred | 36 |
| out | 25 |

By the annotators' fate:

| fate | covered | partly | missing | deferred | out |
|---|---|---|---|---|---|
| fact | 50 | 29 | 4 | 0 | 0 |
| meta | 47 | 27 | 5 | 0 | 0 |
| rederive | 19 | 106 | 18 | 0 | 4 |
| episode | 27 | 97 | 0 | 36 | 1 |
| discard | 2 | 0 | 0 | 0 | 20 |

The 286 gap rows (missing and partly) by the layer the missing piece belongs to: derived 144, episode 89, fact 30, meta 22, lesson 1.

By the backlog's §a subject (the derivation backlog of `backlog.md`; "none" is a fact, a meta row or an episode the backlog lists only in §b or §c):

| subject | gap rows |
|---|---|
| none | 94 |
| Going and the lead | 29 |
| The tier-2 sprint | 23 |
| Reach, turret lines and nests, the odds model | 16 |
| The nuke and the anti-nuke | 16 |
| Unit matchups and mixes | 14 |
| The frontier | 14 |
| Expansion and income | 13 |
| The raid answer | 13 |
| Water maps | 10 |
| The bank and build power | 9 |
| Energy in the opening | 6 |
| The commander's conduct | 6 |
| Great Divide | 6 |
| Answers that take minutes start on earlier signs | 5 |
| Pressure from the first units | 3 |
| The Quicksilver packet from Matt's line | 3 |
| The scenario variables and scaling by the map | 2 |
| The opening numbers derived from the map sheet | 2 |
| Base layout | 1 |
| Reach, turret lines and nests | 1 |

By old file:

| old file | gap rows |
|---|---|
| docs/briefs/player.md | 95 |
| docs/briefs/experience/cases.md | 63 |
| docs/briefs/experience/late-game.md | 18 |
| crates/bot/src/strategist/player.md | 16 |
| docs/briefs/experience/units.md | 14 |
| docs/briefs/experience/winning.md | 13 |
| crates/bot/src/strategist/commander.md | 13 |
| docs/briefs/experience/other-maps.md | 10 |
| docs/briefs/experience/team-games.md | 8 |
| docs/briefs/reference/comet-catcher-remake-1.8-cor.md | 7 |
| docs/briefs/reference/comet-catcher-remake-1.8-arm.md | 5 |
| crates/bot/src/brain/pianist/rules.md | 4 |
| docs/briefs/reference/frontier-comet-catcher-remake-1-8.md | 4 |
| docs/briefs/reference/frontier-great-divide-v1.md | 4 |
| docs/briefs/reference/frontier-quicksilver-remake-1-24.md | 4 |
| docs/briefs/reference/frontier.md | 4 |
| docs/briefs/experience/players-words.md | 2 |
| crates/bot/src/strategist/packet-example.md | 1 |
| docs/briefs/player-opus.md | 1 |

Rows whose note records a conflict between the old text and the new set: 124. Most are regenerated numbers (the pool's sets grew, BARb's from 110 to 150 games) or arithmetic the new set gets right (a plant alone making a constructor draws 72 energy a second, not 48; the start holds 1,000 energy, not 1,240; a bot lab draws 57, not 80); a dozen are doctrine the new set reads the other way (a bank: the OS 40+ winners bank more; the commander: people walk theirs to the seams; the first Grunts as a guard against the first units crossing in every format). The reader's note holds both values.

## What the readers found missing, by theme

The readers' summaries, merged by the main session; the examples are theirs.

1. **The derivations the backlog names, not yet written** (143 gap rows are derived). The premises are nearly all in facts/ and meta/; only metal.md and lead.md draw conclusions. Largest: *going and the lead* beyond lead.md (29: when to gather and only before a defended position, arriving together, going on after a won fight, the route something of ours already travelled, a look before the last step; what stays home and what goes); *the tier-2 sprint* (23: the gap itself with the army flat while plant, constructor and mohos are paid; lethal in a tight duel, covered by allies or a settled front; the team procedure and its trigger; feeding as stopped production; the entry cost 3,150 metal; tier 3 and the Juggernaut's blast); *reach, turret lines, nests, the odds model* (16: the turret at three times its metal, unseen fire and its four answers, scouting before committing, bypassing a post); *the nuke and the anti-nuke* (16: the window by opponent, the target after his anti-nuke, the insurance cost of 2.4 mohos, the stockpile's energy draw, the 9 s launch and 13 s flight, a missile needs no sight); *unit matchups and mixes* (14: no head-on result at equal metal survives, the simulator's group outcomes such as 4 Blitzes on a Sentry keep 72% and 6 Rovers all die, splash figures, body size); *the frontier* (14: no simulator clock anywhere, the 16:25/16:50/15:30 first missile on three maps against BARb's anti-nuke window); *expansion and income* (13: the post-moho ladder's order, the midline turret, our own curve 5/12/16 at 4/8/12); *the raid answer* (13: pickets, catch-by-speed, the 600 rule, the scale rule "a lone Rover is the picket's, a swarm is everyone's"); *water maps* (10: our sea plan and the seats' division of sea work; a yard first with one or two extractors for eight minutes is no economy); *the bank and build power* (9) and *energy in the opening* (6); *the commander's conduct* (6: never toward what outreaches its D-gun, its fighting worth about 400 metal of tier-1 soldiers); *Great Divide* (6: the spot list and start positions, the plan for the north mouth, a camp broken with artillery behind a screen); *air* (5: the gunship clock after an air plant is seen, 2:16 to 3:54 in human-9; one Liche bomb kills a commander).
2. **Our own games** (89 gap rows are episodes, 36 more deferred): the opponents' side is carried in meta/people.md and the lessons survive, but no account of our seats does; the episode index would hold them. Three rows record measurements later derivations need: the +60 against +40/+50 Altored comparison of handing out advanced constructors, the human-8v8-1 game whose armies covered Apex's sprint, the distilled bot's 1 of 16 with tier 2 against 3 of 16 without.
3. **Facts** (30): Comet's per-strip spot lists; Great Divide's spots and starts; the Juggernaut's death blast (10,600 to radius 640); Cortex's missile (1,500 metal, 187,500 energy, 180 s); radar range; the Whistler's and Lasher's anti-air reach 700; jammer radii and the Juno's 1,400; the Butler, Twitcher and Consul build lists; Legion's energy per metal and that BARb and Apex play Legion; Azurite Shores' 16 of 76 spots under water; the pool's full per-minute income curves and opening orders by result (kept only as key minutes).
4. **Meta** (22): the pool's top quarter by extractors at 4:00 with its names (Artur91 five times, Baldric three); thebluegecko's army line on Quicksilver (Pawns gathered at home into one group, the commander home); BARb hard on Quicksilver (the 5:45 commander sortie, Shellshockers from out of sight) and medium's raid clock there; the one Legion player in the pool; what raiders meet at 2:00-4:00; the body ceiling for long-range types.
5. **One lesson**: thebluegecko's "Top should stop building other units while doing t2 transition" (game 10, 17:27, players-chat-2026-09-28.md) is not in lessons.md.

## Inside the new set

Checked and fixed (a3693a6): the unit tables named one builder a unit (`made_by[0]`), so the construction turret read as the Beaver's and the Pawn as the Consul's alone; the economy's build table had the commander building a construction turret, which armcom.lua's build list does not hold; the people catalogue's u6bkep block said four wind before thebluegecko's Quicksilver lab against the record's three; metal.md's ten-against-five extractors (Armada at OS 40+) read against pool.md's eleven-against-one (the OS 25+ survey) until it named its population.

Still to check: pool.md M-pool-opening's bank at 3:00 (72 against 107) against the Armada reference table's 45 against 96 (likely both factions against Armada alone; the sentence should say); pool.md's "winners bank more at every minute from 4:00 to 16:00" is Armada's, the Cortex table has the losers ahead at 4:00-7:00 and 12:00; M-pool-expansion's 80% in the nearest quarter is Armada's, Cortex's is 94%; Hellontoast's OS (41.15 in the Cortex sheet, 39.7-40.8 in pool.md and people.md, below the pool's floor); barb.md's state at tier 2 matches the 110-game sheet to the unit while the clocks are the 150-game set's; the maps catalogue's tide of 0 for SailAway 2 (the sheet's field, unverified against the map's file); the standing claim K-mech-stall-slows-everything cited in fights.md against F-eco-build's all-or-nothing stall.
