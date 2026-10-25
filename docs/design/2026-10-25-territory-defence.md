# The territory bot's defence: territory as a race of force (proposal, 2026-10-25)

Accepted 2026-10-25. The user's model, in his words: "the basic model I had in mind is a projection of defence force.
If you start from a basic model of valuing combat units by their metal (with a multiplier for static defences), then
we can model our claimed territory by how much of our material can reach (movement plus attack range) a given point
on the map before an enemy force of a certain size can do the same from fog of war. With that basic model we can
figure out how many units we need to build as we expand and how to keep reasonably efficient defences up for our
perimeter. Whether to have defence or construction lead our territory claims is an open question." On the
refinements and the order below: "Agreed, lets write this up and try it."

The economy stage is closed (`docs/design/2026-10-25-territory-walks.md`; the standing build benched as terr202-207
against NullAI). This is stage 2 of the regions design (`docs/design/2026-10-25-territory-regions.md`, part 3), which
already says how defence enters the planner: a region's income less its expected loss, the loss reduced by cover.
It never said what cover is. The race says it.

## Where the bot starts from

Nothing. The territory bot builds no soldiers and no turret for defence (the factory palette is constructors only
unless the `soldiers` arm is on, and the program values nothing but metal made). It knows his start and his armed
units in sight, which make the ground within 1,000 of them dead to its builders (T-SCENARIO). Its commander walks out to the
spots near home in the opening. Its deaths are logged (`died:` lines, with the attacker), and every match's
`truth-N.jsonl` has his units (id, name, position, health, frame or not) every 2 s.

## The model

**Currency.** A combat unit is worth its metal. A static defence is worth its metal times a multiplier: it cannot
move, but inside its reach it answers at once, and it is cheap for its gun. The multipliers are measured, not
guessed: the fight simulator (`crates/combatsim`) says what one light laser turret is worth in Pawns or Flashes,
once per pair of classes, not per decision.

**The race.** For a place on the map, two clocks:

- *his arrival*: the seconds his force needs to bring its weapons onto what stands there, starting from where we
  cannot see;
- *our response*: the seconds until enough of our armed metal is in reach of the place to meet that force.

What a place risks is what stands there times the share his force kills before our response arrives. Not who arrives
first: a Flash (101 elmos a second, reach 180) kills a 50-metal extractor in a few seconds, so help that comes 15 s
late has lost it. A turret's response is zero inside its reach (430 for the Armada's light laser turret), which is
most of the static multiplier falling out of the model by itself. Our territory is where the risk is small; the
contested ground is where the clocks are close; his is where he answers first.

**His force starts at the edge of our sight.** Not anywhere in the fog: our buildings see 270-490 elmos around them,
3 to 5 s of a Flash's run; a radar (60 metal) sees 2,100, some 20 s. So scouting and radar are defence in this model,
bought for the warning they add, which is how players use them. The edge is bounded by where his army could have got
since we last saw it; without that bound all fog counts and everything is contested. (The first form has no bound:
by minute 3 a raider can be anywhere on the bench's maps.)

**The threat is two things.**

- *Raids*: small, fast, early, at the edges. Turrets beat them on cost.
- *Pushes*: large, slow, aimed at the base. They need the mobile army.

Their sizes over time, S(t), start as a measured curve per opponent: the size of his armed groups that enter our
ground, read from the truth files.

**Mobile metal counts once; turrets count everywhere.** An army answers one raid at a time, and two raids on opposite
edges is how players beat a defence that is all army. So the army is a few stations, each answering for the places
within its response time. The perimeter's length sets how many stations; each region added at the edge adds its share
of one. That is the user's "how many units we need to build as we expand".

**Who leads.** Priced, not ruled. Cheap things can lead into contested ground: a basic extractor is 50 metal and pays
back quickly, so losing a few is cheap. Dear things go only where the ground is held: an advanced extractor (620),
the turret farm, a constructor walking out (135). Defence leads where the stakes are high, construction where they
are low. In the planner that is the loss term: a region's income is worth its income less its expected loss, and
soldiers and turrets are worth the loss they remove.

## What the game gives the model

- **Our sight**: the engine's own coverage, cut by the ground, which the host sends once a second (`Snapshot::sensors`:
  sight in 64-elmo squares, radar in 32). Radar counts: it shows his units as dots, which is warning enough.
- **The walks**: `terrain::Field` per movement class over the 16-elmo grid. One field from the edge of our sight for
  his approach; one per group of our armed mobile units for our response. Turrets need no field.
- **His speed and ground**: the first form walks his approach on the commanders' ground (a bot's, more permissive
  than his vehicles') at a fixed raid speed of 100 and reach of 180 (a Flash; an Incisor is alike). His seen units
  replace that when the instrument shows it matters.
- **Armed**: a unit with a weapon (`weapon_count` above zero, `reach` above zero) that is not a constructor; the
  commander counts, at its metal.
- **The regions**: the map's spots in regions of about 20 s of a constructor's walk (T-REGIONS). What stands of ours
  belongs to the region of its nearest spot, so home and the farm fall in home's region.

## The bench

NullAI cannot measure defence. The defence bench is the economy bench's six starts and four seeds against BARb
medium (`run/territory_bench.sh FIRST TAG --opponent BARb:stable --profile medium`), to 20:00, played out. BARb medium
raids, and our distilled bot beats it 8-0 (the `distilled-v0` rows), so a bot that survives it exists. What it reads:

- the outcome and when (our commander's death ends the game);
- income at 5, 10, 15 and 20 minutes while alive, as on the economy bench;
- metal lost to his weapons, by type and minute, and by region;
- his army's metal over time and the size of his groups in our ground (the truth files);
- the promise kept (T-PROMISE), now with losses in it.

The economy bench against NullAI stays: a defence change must not cost the economy where nobody attacks.

## Steps

Each step its bench rows and its register rows; what it replaces, deleted.

1. **The baseline.** Today's bot against BARb medium on the six starts (terr208-213). No change to the bot. Done:
   18 of 24 lost, 6 to the cap (below, "Measured").
2. **The race as an instrument.** Built (T-RACE, 6878eb7) and benched (terr214-219); the kill test as written does
   not separate held from open, and cannot yet (below). A `race:` line every 30 s: per region, the metal standing,
   his arrival (warning) in seconds, and our armed metal in reach by then and 15, 30 and 60 s later. A reader (`run/territory_race.py`)
   sets it beside the deaths and his groups from the truth files. Deciding nothing. The kill test: do the losses
   land where the race says the ground is open, and do they spare the regions it says are held? If losses are no
   likelier in the open regions than in the held ones, the model is wrong and nothing gets built on it. With no army
   the commander is the only response, so step 2 tests the threat side most (warning times, his group sizes, where
   he goes); the response side is tested again at step 3.
3. **The army, first form.** Soldiers to a size set by the measured S(t) and the number of stations; stations at the
   perimeter's regions; the hands answer a raid with the station whose response is soonest; turrets where the race
   says raids arrive before any station can. The planner sees the army as a draw on metal and factory time, so its
   promise stays honest. Its details wait for steps 1-2's numbers.
4. **Region status from the race.** Ours, contested, his. Construction respects it: cheap work may go into contested
   regions, dear work only into held ones. This is step 5 of the regions design, with the race as its source.
5. **Cover in the planner.** The loss term: the program trades soldiers and turrets against extractors itself.

## What could go wrong

- **The edge of sight is too pessimistic.** With no bound on where he could be, every building at the edge of our
  sight reads 3-5 s from danger, all game. If the losses do not follow the warning, the bound (his last seen army
  and his speed) comes in before step 3.
- **Metal is a poor currency for some pairs.** Light laser turrets against Pawns, Ticks against everything. The
  multipliers per pair of classes are combatsim's to measure; if one number per class misleads, the race uses the
  simulator's odds for the pairs that matter.
- **The fields cost time.** A field over Glitters' 246,000 cells every 30 s, one per group, is fine for an
  instrument; a decider that wants one per region per tick needs a coarser grid or a worker thread.
- **BARb medium may not be the right pressure.** If it kills the baseline before 5:00 on every start, the bench
  measures only the opening; if it never raids, it measures nothing. The baseline says which.

## Measured

**The baseline (terr208-213).** The standing build, no soldiers, against BARb medium: 18 of 24 games lost (Great
Divide all four at 9-10 minutes), 6 to the cap. Income at 5:00 mostly held (Comet from his start 34-41 against
NullAI's 41-53); 10:00 fell to a third or less (46-73 against 172-230). Losses before 5:00 are small (936-5,540 metal
over a start's four games); from 5:00 to 15:00 they are 10,000-50,000. His armed metal, his commander apart: 650-1,150
at 5:00, 2,800-4,000 at 10:00, 6,500-9,500 at 15:00 (`run/territory_race.py`, from the truth files; terr208-219
pooled). Great Divide's at 5:00 was sixteen Grunts, at 7:30 twenty Grunts and thirteen Thuds. The raids that kill are
small: his metal within 800 of each loss, median 92-159 before 5:00 (one or two raiders), 258-412 from 5:00 to 10:00,
438-1,210 from 10:00 to 15:00, the largest 480-758, 1,250-3,708 and 3,126-6,790.

**The race (terr214-219).** The same build with the `race:` line: 22 lost, 2 to the cap, the spread of one build.

- *The warning is 0-1 s at every region, every start, all game.* The bot builds no radar, and a building sees about a
  second's run past a raider's reach (an extractor's sight 273, a Flash's reach 180). With sight alone a mobile
  response cannot save the first building a raid reaches; it can only end the raid. A radar (2,100) would make it
  about 20 s.
- *The kill test, as written, does not separate held from open.* Held regions lost a building in 34-67 % of the
  windows with his metal near, open ones in 31-65 %; held lost one as often or more on five starts. But held meant
  home: the commander is the only armed unit, and held windows are 42-79 of each start's at home against 3-53 away.
  Home is where most stands (a median 900-6,200 metal against 86-150 for an open region away) and where his pushes
  come, so a building lost there is likelier whatever holds it.
- *As a share of what stands, held loses less on every start*: 3.5-18.6 % of the metal standing a window at home,
  against 19.4-58.1 % in open regions away; held regions away (15-53 windows on four starts) 5.6-9.0 % on three and
  35.3 % on Comet's boxes. That fits the model, and is confounded as well: a raid kills a few of a large cluster.
- *The response side is not tested.* "Held" counts our armed metal that could reach a region within 15 s of his
  arrival; nothing in the bot sends it. The commander fires at what comes into its reach while it builds and chases
  nothing. The test means something only once a guard answers raids, which is step 3.

What the numbers say for step 3: with a second's warning, defence that works is defence already standing where the
raid arrives (a turret, or a guard stationed there) or warning bought (radar), and the raids it must meet are 100-400
metal to 10:00 and about 1,000 after; his whole army, the push, is 3,000-4,000 at 10:00.
