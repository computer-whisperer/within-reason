# The distilled bot's army, rebuilt (2026-10-20)

A bot of `bots/` (code decides; `bots/README.md`). The user's standing ask: "making the distilled bot better ...
run a few games against BARb and apex to get an idea where it falls short, watch the replays carefully for
mistakes, and see what you can do." Nothing here touches the flagship.

## What the batteries and the film showed

Three eight-game batteries on Comet Catcher at +0 with the bot of commit aa5966a (`docs/experiments.md`
dist6-barb-hardagg-comet-1, dist6-barb-hard-comet-1, dist6-apex-v032-comet-cold-1): BARb hard_aggressive 1-6-1, BARb
hard 4-3-1, Apex v0.3.2 from the cold exchange record 1-7. The economy is not the problem: against hard_aggressive
the bot holds twice his extractors from 12:00 (36 against 14 at the median) and banks 1,700-3,500 metal. The army
is: by 10:00 it loses 15-29 soldiers for his 4-15 against hard_aggressive and 16-26 for his 7-19 against Apex, and
it goes on losing them at that rate for the rest of the game (560 units made by 30:00 in one game, 64 alive).

Two instruments were added to the bot to see why, since a bot of `bots/` writes no match record: every soldier's
death with where it stood, which body it was in, what killed it and what stood within 600 (`unit_died`,
`run/bot_deaths.py`), and the body's position, spread, stance and order every ten seconds (`run/bot_body.py`); the
engine demo of one loss was played back through `run/replay_match.py` and filmed with the new `run/match_film.py`.

What they showed, in the order it happens in a game:

1. **The raiding body dies in his half, one unit at a time.** Of the deaths by 20:00 in the Apex battery, 71, 25,
   38, 47, 32, 22, 42 and 28 a game were the strike body in raid mode, in his half, to his mobile army (Thugs,
   Grunts, Centurions) and his sentries. The raid walks five Blitzes to his known extractor, which stands beside
   his base; at the first turret the local odds flip and D-RAID "steps back toward the rally": the next second the
   odds read fine again and it advances; the order toggles between his base and our rally every second while the
   body strings out across the map and each unit dies alone ("ours 1 w110 near" on death after death).
2. **His commander weighs 2,700.** Every weight is the unit's metal, so one commander in sight outweighs any body
   under 3,500 and makes it step back; the raiders then die walking away from it.
3. **The push gathers instead of pushing.** D-GATHER closes the body on its mean position whenever under 60 % of
   it is within 600; reinforcements sent out at 400 metal a time pull the mean back toward home, so a body of
   45 at 18:00 reads "gather" at a third of the way across the map. Against hard_aggressive the census shows 107
   Stouts and 44 Shellshockers wandering mid-map at 25:00 against eighteen turrets at his base, never reaching them.
4. **The push walks into turrets.** In the one Apex game where the body pushed (seed 3), 44 of 108 deaths were to
   turrets: the target is "the nearest known building of his", and the step-back rule reads the turrets only once
   they are within 900 of the body's mean.

## The rules now (D-BODY to D-TARGET replace D-GUARD, D-LAUNCH, D-RAID, D-GATHER, D-STEP-BACK, D-ADVANCE)

- **D-BODY**: one guard and one strike body. A body's place is its *core*: the mean of the 60 % of its members
  nearest to the plain mean. Members farther than 600 from the core are stragglers and are told to close on the
  core; the core's members carry the body's order. Fight odds are read within 900 of the core, ours as the core
  members' weight. (Replaces D-GATHER: the body never turns back for its stragglers.)
- **D-WEIGHT**: a soldier weighs its metal; a turret three times its metal, or its metal once the body carries a
  fifth of its weight in artillery (the Shellshocker's 710 outranges every tier-1 turret); his commander 600.
- **D-HOLD**: the guard stands at the hold point: our finished extractor nearest to his start, pulled back 300
  toward home, never nearer home than 500. It answers an armed mobile enemy within 900 of a building of ours when
  the raid weighs at least 80 metal and the guard weighs at least as much, fighting to the raid's place; a heavier
  raid it meets at the base point (300 out from home), by the plants. With no raid for 20 s it returns to the hold
  point.
- **D-LAUNCH**: the guard becomes the strike body when it weighs at least 1,200 (times the lobby's income bonus)
  and 1.5 times what he has shown of his army in the last two minutes, and a feasible target exists (D-TARGET).
  No early handful: the five raiders of 4:00 were the first deaths of every game. With the body out, the guard
  joins it only when it weighs a third of the body (and at least 600), walking to the body's core as stragglers.
- **D-TARGET**: a target is a known building of his. Its defence is the weight (D-WEIGHT) of his known turrets
  within 700 of it plus what he has shown of his army; it is feasible for a body of 1.5 times that. The body
  fights its way to the feasible target nearest to its core; with none feasible, it goes back to the hold point and
  merges into the guard. With nothing of his known, the nearest spot on his half nobody has looked at, else his start.
- **D-RETREAT**: a body outweighed 1.3 to 1 within 900 of its core walks to the hold point for twenty seconds
  before it reads the odds again, remembers the weight that drove it back (halving every three minutes, the place
  avoided for two) and then re-targets. (Replaces D-STEP-BACK's walk home and the same-second toggle.)
- **D-SCOUTS** as before. No raiding body: his expansion goes unpunished for now, which the batteries will price
  (the Opus player's raids held Apex to 13 extractors; the bot's raids cost it the game).

Not built, next if the batteries say so: artillery standing off at a turret's range (the shelling of nests),
raids by a small fast group at extractors with no known turret, the guard splitting for raids on two flanks.

## The iterations (the same seeds 1-8 on Comet +0; ledger rows dist7-* to dist11-*)

| step | built | hard_aggressive | Apex v0.3.2 cold |
|---|---|---|---|
| before (dist6) | the old army | 1-6-1 | 1-7 |
| 1 (dist7) | the rules above | 3-5 | 1-7 (deaths 0-51; the advanced plant at 6:10-8:51 on a field "won" at launch) |
| 2 (dist8) | his commander 1,800; a five-minute seen window, the dead forgotten; a clock floor on the launch; KEEP_ODDS hysteresis; the field held two minutes beyond 60 % of the way; sentries trimmed | 4-4 | 1-7 (his bombers over 105 Brutes at home; the launch held by raids) |
| 3 (dist9) | the launch never held; the body home for a raid the guard cannot match; aircraft out of the odds; D-ANTI-AIR; the death instrument on builders and buildings | 4-4 | 2-6 (15-72 extractors and 6-29 builders lost to Grunt packs; the whole guard launched) |
| 4 (dist10) | a 450 reserve at launch; the body home only for a raid worth 30 % of itself; D-FLEE; sentries back, one per 450 | 3-5 | 1-7 (builders fled 107-150 times; his 29-40 extractors at 15:00) |
| 5 (dist11) | D-RAIDERS (`raid=on`): four raiders at his unguarded extractors, running from anything; D-FLEE from 100 metal | | |

What the four rows of the rebuilt army say against hard_aggressive: 3-5, 4-4, 4-4, 3-5, one bot inside an
eight-game row's scatter, against 1-6-1 and 0-7-1 before it. The soldier deaths fell from 114-348 a game to 0-60 in
the games that end by 20:00; the long losses (his tier-2 Tigers standing at 15-20 min) still feed 300+. Every loss
of the first three rows was as Armada against BARb's Cortex, whose Incisor raids come at 2:26-3:04 (its Armada's at
4:00 or never). Against Apex v0.3.2 the army no longer loses the game; the economy does (his 2-3x expansion, our
builders and outposts to his Grunt packs), which is where the Opus player's raids made the difference.
