# Nuclear silos in the distilled bot's seat (2026-10-27)

**Status.** The basic design built (`bots/distilled/src/nuke.rs`, arm `nuke=on|off`, on by default) and smoked
(dist52, below); the crater's verdict and what follows from it built (below), the crater measured in the duel
harness. Since 2026-10-27 the strikes (D-NUKE-FIRE to D-NUKE-LOOK) are `crates/nukes`', shared with the territory
bot (`docs/design/2026-10-27-territory-nukes.md`); D-NUKE stays in `nuke.rs`.

## Why

The user, 2026-10-26: "let's teach wrdistilled to do something with its money once it has it. For now, let's try a
nuke rush. When practical, the bot should build 5+ nuclear silos and just start bombarding the enemy with them,
scaling silo count as possible." And 2026-10-27: "This is primarily for the human games, where a seat may be left to
its own devices for a long time and needs to be able to do something offensive with the resources. In that sense, one
bot applying constant pressure with random missile strikes is far better than letting the resources go to waste.
Consider this as one later-game behavior that should enable under certain circumstances and may disable otherwise."
On interception: "Let's run with the expectation of not knowing -- but I expect that is something we can fix
eventually. Let's proceed with the basic design."

What the money is: on the 2v2 bench against BARb with `gate=tech` (dist44, medians a seat) the bank stood at 450 at
25:00, 8,500 at 30:00 and 19,100 (the store full) at 35:00, on 127 / 192 / 238 metal a second; the energy income at
30:00 was 2,900 a second with about 600 spare. The user's eight-seat game: stores full at 7,500-12,300 from 24:00.

## What the game gives (unit files, engine and BAR source)

- Armada's silo 8,100 metal, 90,000 energy; a missile 1,000 metal and 125,000 energy every 120 s, a blast of radius
  640 and 12,500 damage falling to nothing at the edge. Cortex's and Legion's 7,700; a missile 1,500 and 187,500
  every 180 s, radius 960, 15,000. Either way a silo stockpiling draws about 1,040 energy a second. The advanced
  constructor of each faction builds it. The game keeps every silo stockpiling to ten on its own
  (K-rules-a-stockpile-fills-itself-to-its-limit); a missile goes by an attack at the ground
  (K-rules-the-silo-fires-on-an-attack-at-the-ground), about 9 s from the order to the launch and 12 s of flight; a
  silo reloads 30 s.
- An anti-nuke takes a missile only when the missile's aim point is within its 2,000 coverage (BAR's gadget "Don't
  target flyover nukes", K-rules-interception-needs-the-aim-point-in-coverage). BARb's anti-nukes stand beside its
  fusions from 21-25 min (K-barb-anti-nuke-beside-fusions).
- Nothing tells a bot that a missile was intercepted. What it can see: its silo firing (`WeaponFired`), and the
  damage and kills the warhead does where the circle is in our team's sight or radar (`EnemyDamaged`,
  `EnemyDestroyed` naming the silo). Out of sight, a strike's outcome is unknown, and the design runs on that.

## The rules

- **D-NUKE (start).** An advanced constructor starts a silo when the bank holds a silo's metal (or 90 % of a smaller
  store: "once it has it"), no project is under way, and the energy covers one more silo's draw (D-ENERGY with that
  draw counted is not short). While the bank holds the bill and the energy does not cover it, D-ENERGY counts the
  draw anyway, so the energy comes first (fusions and advanced fusions by the advanced constructors' ladder, solars by
  the rest). After the advanced constructor's tier-2 extractors, before helping. The silo is a project (D-PROJECT):
  helpers and turrets in reach mass on its frame. It stands in the back field with the fusions. No cap: the next one
  comes when the money and the energy are there again, so the count follows the economy.
- **D-NUKE-FIRE (fire).** Once a second, a silo with a missile ready and no order of ours in the last 40 s fires at a
  place chosen at random, weighted by worth, among the places worth striking:
  - a place is one of his known buildings (`his_buildings`), the blast centred on it;
  - its worth is the metal plus energy over 70 (the bombers' rate) of his known buildings the warhead would kill
    (each building's health against the damage at its distance), and of those a killed building's own death blast
    would kill in turn (one step of the chain the user named for energy fields);
  - not worth striking: under `STRIKE_LEAST`; within his known anti-nuke coverage plus a margin (a building of his
    with coverage seen, or a mobile one in sight); with any unit of ours or our allies within the blast plus a margin,
    or our strike body's order there; within the blast radius of a place struck in the last five minutes.
  No target, no order: the missiles wait in the silo, which stops drawing at ten.
- **D-ENERGY (amended).** The draw the energy rule weighs adds each silo's stockpile draw, frames counted, and one more
  while D-NUKE's money is there.
- **The instrument.** `nuke:` lines: each order (silo, place, predicted worth and buildings, the nearest known
  anti-nuke, whether the place was in our sight or radar), each launch, and 45 s after the launch what the events
  showed (kills and damage naming the silo, and their worth); a minute line (silos, frames, missiles ready and
  stockpiling, the draw, strikes, places worth striking and those excluded by cover). Offline, the truth files tell
  what happened at every strike (`run/nukes.py`): his units inside the circle before and after.

Not in it, for later: our own anti-nuke; pausing a silo's stockpile when the energy runs short (the protocol's
`Stockpile` step, unmeasured); turning the programme off when the base is under attack; inferring his anti-nuke cover
from strikes that were seen to do nothing; radar or a scout to watch the places struck; salvos at covered places.

## The bench

The 2v2 against two BARb hard_aggressive on All That Glitters (`run/distilled_team_bench.sh`), the cap raised to 55
minutes since the bank fills from about 30:00: `nuke=off` against `nuke=on`, sixteen games each on the same seeds.

## The smoke (dist52, four 2v2 games, cap 55:00)

The mechanics work: the first silo begun at 31.4 min once the bank held its metal, standing at 35.2, its first missile
two minutes later; each order launched 9.1 s after it, one missile an order. But on this bench the seats lose at 35-43
min, before the programme runs: silos begun in 3 of 8 seats, two strikes in all. The first went at a place 1,120 and
1,293 from two Cortex anti-nukes the bot had not seen, and every building there stood (intercepted, by the truth
file); the second launched 30 s before its game ended. The two games lost before 40:00 never had a silo's money, so
the rules did nothing there.

## The crater (the user, 2026-10-27)

"Checking in a human game: while the impact does not grant any vision automatically, the terrain deformation from a
succesful nuke is vary obvious. That and the fact that the full nuke flight and explosion (in the air if interecepted,
or on the ground if not) is visible tells me that the previously suggested ground deformation test should be all above
board. Let's figure out what we would do with that information if we built the filter." So K-rules-a-bot-sees-no-
interception's crater is information a player has, and the filter is allowed.

A correction from the user the same day: "In the game gui there is a very obvious glowing crater that appears in the
fog-of-war terrain content. It is not exactly the real deformation (getting real vision reveals the real terrain
damage), but the deliberate success signal is still there." So the bot reads the real ground for the one bit a player
is given, landed or not, and nothing more.

## What the verdict is used for (the user, 2026-10-27: "Build 1-3 plus the scout, skip chat for now")

- **The verdict (D-NUKE-VERDICT).** `Command::Probe` (the shim answers it with `Event::Ground`, the synced ground's
  height at each point) at the aim and two rings of eight round it, with the order and 25 s after the launch; the mean
  fall over the points decides, 2 or more a landing. Measured in the duel harness's October scenarios with a `probe`
  script command (2026-10-27): an Armada missile that landed took the ground down 21 on the mean (52 at the aim, 37 on
  the inner ring, 7 on the outer), one taken by a Cortex anti-nuke 0.00 at every point (K-rules-a-landed-nuke-digs-a-
  crater). The engine digs the crater over
  the frames after the blast and moves a building's ground with it as a block (`BasicMapDamage.cpp`), so points on
  buildings count too. A lobby with deformation off gives no verdict.
- **His cover (D-NUKE-COVER).** An interception proves an anti-nuke stood within 2,000 of the aim: places within that of
  it are left alone for ten minutes, unless a later landing lies nearer to them. A landing favours the places near it.
- **The ruins (D-NUKE-RUINS).** A landing takes what the warhead killed by the model out of the memory of his
  buildings, so neither the silos nor the army go back for it until it is seen again.
- **The switch (D-NUKE-HOLD).** No place worth striking for a minute: the silos' kept count goes to nothing (no energy
  drawn) and no silo is started; a place found again brings them back. D-NUKE also wants no new silo while those
  standing sit on missiles.
- **The look (D-NUKE-LOOK).** The anti-nuke behind interceptions no known one explains may stand at the grid points
  within 2,000 of every such aim near it and not within 2,000 of a later landing; the aircraft plant's scout plane
  sweeps them, each time to the nearest not yet in sight, until a known anti-nuke covers the aims, every point has
  been seen, or three scouts are lost. His mobile anti-nukes are known where last seen for ten minutes. Chat to allies
  left out (the user).

## The loop smoke (dist54, four 2v2 games at +15 %, cap 55:00)

Thirteen strikes. By the truth file eight were stopped, every one with an anti-nuke of his within 2,000 of the aim, and
three landed (2,900 of his worth killed a landing, 10,100 in all), none covered; one more landed on empty ground. The
bot's verdict agreed every time: intercepted at 0.0 on the mean, landed at 4.0 to 15.0 (Glitters digs shallower than
Comet's 21). The hold paused the silos when the cover memory left nothing to strike and brought them back when new
buildings were seen; seats stood up to four silos. The look as first built went wrong: it looked only at the middle of
the place and ended as soon as that was in sight, so it found nothing, and a BARb seat's mobile anti-nuke (`cormabm`),
roaming its base and coming in and out of sight, made it flip between places. Rebuilt as above: one look kept to its
end, a sweep of the whole place, mobile anti-nukes remembered. The truth file's own judge counted an extractor upgraded
in place as a kill; it no longer does.

## The look's second form and the last smokes (dist55-57, 2v2 at +15 %)

The sweep (dist55) lost nineteen scout planes over the middle, many in our half, to the mobile anti-air with his
armies, none having seen a point; so from our half the scout flies to the map's side edge nearer the point, along it
and in (`flank_route`). In dist57 (eight games) that found an anti-nuke once in seven looks; four were given up with
three scouts lost. A cheap scout that sometimes works; the cover memory does the work without it: over the three smokes
every seat's first strike was intercepted (0 of 10), later strikes landed 27 of 41, the strike after an interception
11 of 20. Every verdict the bot gave agreed with the truth file (24 intercepted at 0.0 on the mean, 24 landed at 5.5 to
17.6, three landings on empty ground), and every interception had an anti-nuke within 2,000 of the aim, no landing one.
The landings killed 218,000 of his worth (metal and energy over 70) for 53 missiles launched.
