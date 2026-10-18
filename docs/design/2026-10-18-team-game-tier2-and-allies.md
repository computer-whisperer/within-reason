# Team games: tier 2 as a sprint, and a commander that sees its allies

Written 2026-10-18 from the human-8v8-1 review (a lobby 5v5 with people, our two seats at +50 % beside three Apex
seats; `docs/experiments.md`, the ledger row; the memory of the review in the session notes). The user's rulings and
the change list agreed the same day; built in the commits that follow this file.

## 1. What the game showed

- Our two seats matched or led the Apex allies in extractors through 12:00 (9 and 15 against 7 to 10), then stopped
  growing: the south half holds 43 spots and our five seats held 50 at 12:00. From there income grows only by
  upgrading, and Apex's Legion seat had placed its advanced lab at 5:20 on eight spots and an income of 27, its
  first moho at 9:00, six by 15:00 and seventeen by 30:00; its income ran 104 at 15:00 and 206 at 30:00 against our
  42 and 57, 122 and 96. Our labs came at 12:39 (on seeing his) and 18:28 (on a bank). Build power: ours flat at 8 to
  10 constructors and 3 to 4 nanos from 5:00 to 50:00; Apex 20 to 28 constructors by 8 to 10 minutes.
- The commander never called the sprint, never used `transfer` (46 directions, 0 transfers), sent both bodies at
  his advanced lab at 12:36 in the words of the brief it reads ("out-range and out-splash our tier-1 army"), and
  could not see the allies' labs at 5:20, 8:00 and 8:30: the report renders no allied structure.
- Apex's tier-2 constructors upgraded our seats' extractors 22 times (the engine gives the moho to the spot's
  owner); both readers took the frames for a harness fault.
- The lobby ran `deathmode=com`: `game_team_com_ends.lua` kills a side's seats only when the side's live-commander
  count reaches zero; the brief's "lose when ours dies" is the `own_com` rule, not this one.

## 2. Rulings (the user, 2026-10-18)

- **Drop "we are weak past tier 1"**: "that is one of the things we are trying to fix." This reverses the
  2026-10-03 ruling in `2026-10-03-brief-rewrite-and-commander-pack.md` §6 ("Tier 2 is a decision, not a plan ...
  no timing rule"). Chose-because: the brief's argument made the commander deny his tech instead of running ours,
  in a game whose shape (five people a side, 84 spots, +50 %, allies at tier 2 by 5:20) called for the sprint.
  Rejected-because (the old ruling's ground): the duel pool rarely reaches tier 2 and the bot's transition under
  BARb's pressure lost at either gate; that evidence stays, scoped to duels on tight maps with nobody covering.
- **The reasoning to encode**: going to tier 2 is a sprint with the economy you have, from the old units to the
  new; the gap is usually lethal in a 1v1 on a tight map, but on a larger map with allies it can be covered; the
  units are far better and it unlocks the real economy growth any game past 20 minutes needs; team games almost
  always escalate to tier 2 and beyond, and that should be visible from the shape of the early game.
- **Team games**: one seat transitions early, eating its tier-1 lab or even its commander for the metal, then builds
  tier-2 constructors to share; the abridged transition is the commander's call.
- **Death mode**: a side loses only when it controls no commander; a seat plays on without one; commanders are
  resurrected in team games.
- **The commander should see the allies' structures and units.**
- **`transfer` may send resources to an allied seat that is not ours.**
- **Simplify**: the overlapping tier-2 bullets and cases are distilled into one argument.

## 3. The changes

Briefs: delete the weak paragraph; a scenario paragraph at the head of the cases; one "Tier 2 is a sprint" passage
with the team-game procedure and six cases in pairs, replacing the window bullet and "Tier 2: his and ours"; the
sharing procedure once; the five scattered clocks pointed at it; the experience brief's late-game duplicates cut.
Role texts: the commander's side-level economy calls named (the sprint above all), `transfer` as the economy tool;
the player's game-end rule corrected; the transfer text no longer points per-seat players at a `seats:` line.
Report: an allies block for the commander and an allies line for the player; given events said; the other seat's
bank for a per-seat player; spots in our half against the seats sharing them; the lobby's death mode (needs the
shim reinstalled). Tool: `transfer` of metal and energy to allied seats. Records: this file, the §6 reversal, the
knowledge claims, the heuristics rows.

Not in this round: the pre-game template's extractor-first shape; constructor and nano counts framed by income;
allied structures in the hands' picture.
