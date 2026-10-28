# Nuclear silos in the territory bot's seat (2026-10-27)

**Status.** Built (the user, 2026-10-27: "lets implement the nuke build up and attack for the territory bot as
well") and smoked: terr356 against terr357 (`nuke=off`), below.

## Why

WRDistilled's silos (`docs/design/2026-10-27-distilled-nukes.md`) are "one later-game behavior" for the games with
people, "where a seat may be left to its own devices for a long time and needs to be able to do something offensive
with the resources" (the user, 2026-10-27). The territory bot leaves more metal unspent than the distilled bot: its
store sits at its cap from the middle game (30,000-43,000 banked at 20:00 against NullAI, the late bank in the
territory README), and the program's objective counts none of it.

## One strike engine for both bots

The strikes are the same in both seats, so the rules that fire, judge and remember move out of
`bots/distilled/src/nuke.rs` into `crates/nukes`, a library both bots link (the flagship does not): D-NUKE-FIRE (the
place drawn by worth, the exclusions), D-NUKE-VERDICT (the ground probed at the aim before the launch and after),
D-NUKE-COVER (an interception marks his cover for ten minutes), D-NUKE-RUINS (a landing's kills by the model handed back
for the bot to forget), D-NUKE-HOLD (no place worth striking for a minute pauses every silo), and D-NUKE-LOOK's place and
sweep, run only for a bot that offers scouts. The engine takes a view each second: our silos, his buildings known, the
places no missile may land near, his units in sight, the sensors and the scouts. Its log lines keep their wording, with
each bot's prefix. The distilled bot's behaviour is unchanged by the move.

What stays with each bot is how it builds a silo and what it knows: the distilled bot's D-NUKE (a project of an
advanced constructor, D-ENERGY's draw) and its memory of his buildings (its army's sightings); the territory bot's own,
below.

Rejected: a copy of `nuke.rs` in the territory bot (two copies of the same strike rules to keep in step as they are
iterated).

## The territory bot's part

- **T-SEEN (his buildings).** His buildings seen with their type are remembered where they stood, and forgotten when
  seen destroyed or when their place is in our team's sight without them. The bot had no such memory: T-GUARD and
  T-AIR remember his armed mobile units only. It sees what its allies see (the sensors are the ally team's), which is
  where its targets come from in a team game: the bot has no army that goes to him.
- **T-NUKE (start).** A builder that builds the silo, at the base, starts one when: the arm is on; the bank holds the
  silo's metal (or 90 % of a smaller store); there is a place worth striking (the engine's); no silo of ours is
  framed or ordered; the silos standing do not all sit on two missiles or more; and the converters standing burn at
  least the silos' stockpile draw with this one's (about 1,040 energy a second a silo). The last is the territory
  bot's form of D-ENERGY: the silos take the energy the converters would turn into metal for a full store, and the
  program's plan, which has no stockpile column, keeps its energy for the rest. The silo is the faction's type by its
  data (a building's stockpiled command-fire weapon, not a paralyzer, the largest blast), not by name. It goes on the
  farm like the program's big frames (the slot with the most construction turrets in reach), and the turrets help the
  dearest frame in their reach. Before the plan's flows, after T-COVER's turrets and radars. No cap: the next comes when
  the bank holds the bill again.
- **The silos are not guns.** T-COVER, T-GUARD and T-RACE count our turrets as force; a silo (and an anti-nuke) is a
  turret to the unit table, with a reach of the whole map. They leave out every type that stockpiles; T-COVER and
  T-AIR guard the silos as buildings.
- **No look.** The territory bot's line has no aircraft plant (terr66: unlocked, never committed), so D-NUKE-LOOK does
  not run; the cover memory works without it (the distilled smokes: later strikes landed 27 of 41).
- Arm `nuke=on|off`, on by default.

## The bench

The team shape of the games with people: WRTerritory with WRDistilled as its ally against two BARb hard_aggressive on
All That Glitters, `--our-bonus 15`, cap 55 minutes (`run/nukes.py` reads either bot's strikes against the truth).
The economy bench against NullAI must not move: NullAI builds nothing, so no place is worth striking and no silo is
begun.

## The smoke (terr356, terr357)

Four games a side, the shape above. With silos (terr356): every territory seat began one, the first at 18-23 min, and
stood 3-7; 46 strikes, by the truth file 30 landed (207,000 of his worth killed) and 16 stopped, each of those with an
anti-nuke of his within 2,000 of the aim. The bot's verdict off the ground agreed on 44 of 46; the judge called the
other two landed from damage alone with no crater and his anti-nuke covering the aim. Two games lost to his tier-3
armies at 39.8 and 43.8 min, two to the cap. Without (terr357, the ally's silos on): all four lost, at 33.8-48.3 min.
On both sides the territory bot's engine thread rose from 0.3-0.5 to 2-4 s a game minute while his armies were in our
bases: the hands' work in a fight, not T-NUKE's (the control ordered no silo; it still ran the place search, which the arm now stops), not looked into.

After a review, the build as committed (terr364, the same shape): silos in every territory seat from 22.5 min (a
median), 6-8 a seat; 65 strikes, 46 landed by the truth file (260,000 of his worth), 18 stopped under his cover, the
bot's verdict agreeing on 62 of 64 (the other two the judge's limit, as above); all four lost, at 36.3-52.1 min. No
tick over 50 ms. Against NullAI (terr358-363) no silo was begun and no building of his ever known, and every start's
curve lies in the spread of the last three runs.
