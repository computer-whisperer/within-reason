# How many of each unit the experienced players keep in one body

The user, 2026-09-28: "how many of each unit did they ever have in the same group? Many units in bar with no special
micro attention lose effectiveness at a certain scale, and on many occasions I have seen us throw an army of units
that don't have the range to engage at once splash against a few t2 units the opponent built instead."

`run/replays/composition.py --ours run/matches/1790550188-player-14-gecko-opening/00 run/matches/1790551892-player-15-gecko-opening/00`
over the 40 carded Comet Catcher games with both players at OS 40+ (the replay survey; every sample of both sides), a
body being soldiers chained at 200 elmos or less (the pro-fight-shapes study's core; commanders and aircraft left
out). Per type: the largest number that ever stood in one body over the pool, the median over games of each game's
largest body count (the typical ceiling), the largest owned at once; then ours in player-14 and player-15.

| unit | pro largest body | median of the games' largest | pro largest owned | ours largest body | ours largest owned | where the pros' largest stood |
|---|---|---|---|---|---|---|
| Instigator corgator | 53 | 11 | 82 | - | - | AgentOG 16:54, a body of 101 with 48 Grunts |
| Bulldog corraid | 49 | 13 | 74 | - | - | Hellontoast 18:04, 60 with 11 Instigators |
| Grunt corak | 48 | 13 | 76 | - | - | the same 16:54 body |
| Blitz armflash | 31 | 13 | 65 | 14 | 23 | Artur91 14:19, 35 with 4 Stouts |
| Stout armstump | 23 | **8** | 47 | **24** | 31 | Artur91 15:59, 26 with 2 Whistlers |
| Tumbleweed corfav | 17 | 5 | 22 | - | - | 3:50, 17 alone |
| Rover armfav | 16 | 4 | 28 | 15 | 18 | 5:10, 16 alone |
| Hammer armham | 13 | 4 | 15 | - | - | Artur91 12:46, 16 with 3 Rocketeers |
| Thud corthud | 12 | 9 | 19 | - | - | 18:02, 33 with 15 Instigators |
| Leveler corlevlr | 12 | 4 | 17 | - | - | modulo 16:57, 51 with 28 Bulldogs and 10 Slashers |
| Pawn armpw | 10 | 10 | 18 | - | - | 14:14, 35 with 14 Blitzes |
| Slasher cormist | 10 | 2 | 20 | - | - | modulo 16:32, 48 with 27 Bulldogs |
| Whistler armsam | 7 | 7 | 18 | - | - | Artur91 15:34, 7 alone |
| Mortar cormort | 7 | 7 | 7 | - | - | Baldric 16:49, 7 alone |
| Centurion armwar | 5 | 3 | 7 | - | - | |
| Janus armjanus | 4 | 2 | 7 | 3 | 3 | |
| Shellshocker armart | - | - | - | 4 | 4 | |
| Bull armbull | - | - | - | 3 | 3 | |

Ours: 24 Stouts in a body of 32 (with 5 Blitzes and 3 Shellshockers) at 17:13 of player-15; 15 Rovers in a body of
16 at 3:10; 14 Blitzes at 9:08 of player-14.

**What it says.**
- **The typical ceiling is a dozen of a raider or line tank and eight Stouts.** The median game's largest body holds
  13 Blitzes, 13 Bulldogs, 11 Instigators, 13 Grunts, 10 Pawns, 9 Thuds, and 8 Stouts; 7 Whistlers or Mortars; 4
  Hammers, Rovers or Levelers; 2 Slashers. Our 24 Stouts in one body is three times the pros' typical largest, and
  our 31 owned at once is two thirds of the pool's record.
- **The big bodies are late blobs, and mixed.** Every pro body over 30 stood after 14:00 in a long game, and every one
  of them is two or three types together (Instigators with Grunts, Bulldogs with Levelers and Slashers, Blitzes with
  Pawns): the raiders and the line and the long-range in one place, never 30 of one thing. A Stout body over 15 is
  Artur91's alone (26 at 15:59, with Whistlers).
- **Rovers are the one type the pros mass by themselves**, 16 to 17 in a body at 3:50 to 5:10 (thebluegecko's line,
  K-open-comet-thebluegecko-rover-mass), and our 15 at 3:10 matches it.
- **Long-range units come in sevens at most**: 7 Whistlers, 7 Mortars, 4 Hammers in a body typically, beside the
  line. We had 4 Shellshockers behind 16 Stouts at 21:25; the pros' record for artillery in one body is 7 Mortars.

**The count at 400 elmos** (`--link 400`, the "would be wrong if" of the claim): the typical ceiling barely moves,
13 Blitzes, 14 Bulldogs, 13 Instigators, 15 Grunts, 14 Thuds and 11 Stouts; the pool's largest single bodies grow to
65 Instigators, 58 Bulldogs, 41 Blitzes and 25 Stouts (the late mixed blobs joined). Our largest Blitz body reads 20
at 400 (14 at 200) and the Stouts stay 24.

## What a body over the ceiling costs, in the duel harness (2026-09-28)

`target/release/duel` on Quicksilver's flat sites, speed 50, the director's attack-move both sides, ranks of eight
at 56 elmos; batches `run/matches/1790555341-duel-scale-stout-6` .. `-36` (six reps each) and
`1790555408-duel-ball-vs-armbull-5400` .. `1790555479-duel-ball-vs-armsnipe-1800` (eight reps each; equal metal by
`--budget`, so 21 Stouts against 5 Bulls, 25 against 4 Fatboys, 24 against 8 Sharpshooters, and 8, 6 and 9 Stouts
against 2, 1 and 3 at 1,800). Margin = our value left minus theirs, per duel, meaned; "in reach" = the share of a
side's engaged unit-seconds (an enemy within 700) in which the unit had an enemy within its reach.

| pairing | a side | margin (sd) | in reach while engaged | shots per reach-second | seconds |
|---|---|---|---|---|---|
| Stouts mirror, 6 a side | 6 | 0.00 (0.13) | 0.91 | 0.82 | 36 |
| Stouts mirror, 12 | 12 | 0.00 (0.17) | 0.92 | 0.83 | 73 |
| Stouts mirror, 24 | 24 | 0.00 (0.16) | 0.90 | 0.82 | 77 |
| Stouts mirror, 36 | 36 | 0.00 (0.11) | 0.90 | 0.81 | 93 |
| 8 Stouts against 2 Bulls | 1,800 | **-0.18** (0.09) | 0.81 / 0.92 | 0.80 / 0.87 | 29 |
| 21 Stouts against 5 Bulls | 5,400 | **-0.31** (0.06) | 0.79 / 0.91 | 0.80 / 0.85 | 29 |
| 6 Stouts against 1 Fatboy | 1,800 | **+0.12** (0.08) | 0.82 / 1.00 | 0.81 | 31 |
| 25 Stouts against 4 Fatboys | 5,400 | **-0.54** (0.14) | 0.63 / 1.00 | 0.79 | 28 |
| 9 Stouts against 3 Sharpshooters | 1,800 | +0.83 (0.06) | 0.59 | 0.72 | 14 |
| 24 Stouts against 8 Sharpshooters | 5,400 | +0.94 (0.03) | 0.63 | 0.70 | 17 |

**What it says.**
- **A Stout ball does not lose to itself.** In the mirror the in-reach share and the rate of fire are the same at 6
  and at 36 (0.90-0.92, 0.82 a second): on flat ground against a line of the same reach, the whole front fires at
  every size (the muzzle study's phase 1 again).
- **It loses to a few long-range units, and worse the bigger it is.** At the same metal ratio, six Stouts beat one
  Fatboy (+0.12) and twenty-five lose to four (-0.54); eight Stouts lose to two Bulls by 0.18 and twenty-one to five
  by 0.31. Against the Fatboys only 63% of the ball's engaged seconds had a target in reach: the back of a 25-Stout
  body (spread 105 across the front, ranks behind) closes 350 under 700-range fire and never shoots. The ninth Stout
  is worth less than the eighth, and the sixteenth less again, exactly against the units that appear at 12:00.
- **Sharpshooters alone die** (+0.83 and +0.94: 580 health, one shot every 20 s): their value in the games is the
  line in front of them (player-15's one Gunslinger, 65 kills among our Stouts from behind his), which the duel has
  no arm for yet (a Sharpshooter pair behind eight Bulls against the same metal of Stouts is the next arm).
- **So the ceiling is about what the enemy fields, not the ball's own geometry**: past about a dozen, more of the
  same tank buys nothing against tier 2 and less against artillery; the metal is a second group elsewhere, or tier 2.

## Splash or range? The same arms at 160 spacing (2026-09-28, the user's guess: "the deciding factor there is the splash of the t2 units")

Mithril Mountain v2.0.1's flat sites (Quicksilver's hold no 25-unit line at 160: the two `ball-vs-*-5400-sp160`
batches there ran 0 duels), `--spacing 56` against `--spacing 160`, eight reps each:
`run/matches/1790556033-duel-mm-ball-vs-armfboy-5400-sp56`, `1790556045-...-sp160`, `1790556061-duel-mm-ball-vs-armbull-5400-sp56`,
`1790556073-...-sp160`, and `1790555999-duel-ball-vs-armfboy-1800-sp160` on Quicksilver.

| pairing | spacing 56 | spacing 160 | in reach 56 / 160 |
|---|---|---|---|
| 25 Stouts against 4 Fatboys | -0.55 (sd 0.13) | **-0.22** (0.04) | 0.64 / 0.74 |
| 21 Stouts against 5 Bulls | -0.34 (0.04) | **+0.01** (0.07) | 0.78 / 0.78 |
| 6 Stouts against 1 Fatboy | +0.12 (0.08) | **+0.52** (0.02) | 0.82 / 0.82 |

Splash it is: the Fatboy's shell has a 300 area (800 damage, 0.15 at the edge), the Bull's 130, the Stout's own 48
(`crates/combatsim/data/units.json`). Spread out, the Bulls' win vanishes and the Fatboys' halves; what remains of
the Fatboys' margin is the range (700 against 350, 0.74 in reach). The Sharpshooter (16 area) had none of it.

**Not measured here.** Whether a body's units could all fire at contact (the shapes study has the friend-on-the-line
share by body size: 22% for the pros' bodies of 6, 43% for ours of 13); what a body over the ceiling costs in a
fight (the duel harness can: 8 Stouts against 24 in `--formation` arms, the muzzled share and the margin). Both are
the next measurements if the ceiling is to become a rule for the hands or the brief.
