# The commander seat

Status: draft 1, 2026-10-03 night. Written at the user's word after human-17 and human-18; nothing is built. It takes
over §2 and §5 step 4 of `2026-10-03-brief-rewrite-and-commander-pack.md` (that document keeps the brief and the
pack).

A word on names. "Commander" here is the new seat above the players. The game's own commander unit stays
`commander_t1` in the picture, and the arena's `--commander-model` today names the *player's* model, a leftover of an
older arc (§7 renames it).

## 1. What is measured

- **human-18** (3 seats of ours against thebluegecko, All That Glitters Extended, one Opus 5.5 medium player for all
  three seats, read to minute 28 while it ran). The player's turn went from 15.7 s at the median in minutes 0-5 to
  30.3 s in minutes 20-25, its report from 10,000 to 48,000 characters (56,000 by minute 28; 71 actor entries and 27
  groups at 17:54), one packet of 1,900 to 3,400 characters for every seat's hands. Its own notes: 12:14 "95
  extractors flat for 3 min with ~48 constructors mostly helping plants/standing in lanes"; 16:13 "Army 38.5k vs ~5k
  seen: the lead is unspent because groups never gather"; 17:54 "Energy STALLING with 6.2k metal banked and ~20
  constructors idling"; the magenta seat banked 3,200 to 3,900 metal from 10:49 to 16:13. Eighteen production calls
  in 28 minutes over up to seven plants; the switch off Blitzes at 14:01. He reached tier 2 by 22:02; at 27:24 "eco
  collapsing (63 ext)" from 112. The reading of the game in the notes is right each time and minutes late.
- **human-17** (3 seats, Coast To Coast, lost about 16:00): one packet for all seats rewritten whole 30 times; after
  3:26 the red seat's factory had six production changes and the two shipyards three between them; a commander in
  its factory's exit lane reported on 47 of 69 turns; the unit choice (Crocodiles of 370 reach against destroyers of
  700) corrected after the fight at 10:45 was lost. Two seats stood at 2 and 1 extractors all game on what looks
  like a harness fault (§9), which no seat arrangement would have mended.
- **Sonnet 5.5 in the player's seat** (sonnet-low-1, sonnet-medium-1, sonnet-medium-2, a duel on Comet): turns of
  3-4 s at the median against Opus's 10.7 s, and losses in opposite directions on the balance of economy and army.
  Fast enough for a seat; not steady enough alone on the game-scale calls.

## 2. The seats

| Seat | Count | Cadence | Holds the game? | Reads | Writes |
|---|---|---|---|---|---|
| commander | one a side, or none | a turn every minute or two, and on events | never | the commander's report (§3) | directions in prose, chat with people, transfers between seats |
| player | one for all seats, or one a seat | seconds | as today (lockstep) | its seat's report with the direction at the top | packets, lists, production: as today |
| hands | Jev, one a seat | every second | as today | the packet of its seat's player, the menus | picks: as today |

The commander orders no unit, writes no packet and no list. It decides what a person at the top of a team decides:
what the side is trying to do in the next minutes, what each seat makes and where its army goes, where and on what
condition the bodies of several seats meet and go together, which seat feeds which, when to go to tier 2 and with
what, and it answers the people in the game.

## 3. What the commander reads

A report built for it, small whatever the game's size (the measure: under 10,000 characters at human-18's minute 18,
where the player's was 38,000):

1. **The game**: the score, trade and curves lines as the player has them, over the whole side; both sides' curves
   against the reference where there is one; his spending seen; `to win`.
2. **One line a seat**: faction, home, bank and income of metal and energy, extractors and free spots in its
   reach, constructors and how many are idle or helping a factory, factories and what each is making, soldiers and
   their worth.
3. **The bodies**: one line a group of ours over the whole side (seat, name, what it is made of, worth, where,
   what it is doing, fighting or not), and the enemy parties in sight as the player has them.
4. **The scouting block** as the player has it.
5. **What the players said**: each player's notes since the commander's last turn and the plan paragraph of its
   standing packet. This is the way up; there is no tool for it.
6. **Chat** from people since its last turn, with who said it.
7. **Its own last direction**, how old it is, and what has changed on the lines it named since.

Tools to look closer, as the player has them: `situation`, `units`, `map`, `overview`.

Its start context: the experience brief whole (the ruling of 2026-10-03: everything in it always) and the pack (§4
of the brief document: the economy as arithmetic, units as value, timings as distributions), with a prompt of its own
for the tools below.

## 4. What the commander writes

- **`direct`**: prose, in two parts. `plan`: a few lines every seat is shown, the same words to all. `seats`: a
  paragraph for each seat it wants to speak to. A direction stands until the commander replaces it; a seat not
  named keeps its last paragraph. Places are named by the names in the report (spots, cells, marks).
- **`mark`**: a named place, seen by every seat's player and hands under the same name. The meeting place of a
  push is a mark, so that "gather at `meet_east`" means one place to three players.
- **`say`**: chat. With a commander in the game the players have no `say`; chat from people wakes the commander
  and is shown to it alone. (Chat goes out under the host's name as today.)
- **`transfer`**: metal, energy or units between seats, as the player's tool today. The players keep theirs for
  their own seat's giving.
- **`note`**, **`wait`** (wake on: seconds passed, chat, a seat's commander unit under attack or lost, a fight above
  a worth it sets, his tier 2 first seen, a seat's bank above a figure it sets).

What the direction is, by example (human-18 at 12:14, written as the commander would have):

> plan: He has one base in E2 and raiders in our middle rows. We hold three times his army and it stands at home.
> All three seats gather at `meet_east` (spot_259) and go north together at 14:00 or when 60 soldiers stand there,
> whichever is first; raiders in our half are for turrets and one small group a seat, never the body.
> red: Stouts and Whistlers from every plant, no more Blitzes. Your constructors are helping plants: 300 spots are
> free, send them out. green: as red. You are the seat nearest `meet_east`; your body arrives first and waits.
> magenta: you bank 3,200. A second plant and builders on it now; your body goes up the west column to D5 at the
> same clock, as the second place he has to answer.

## 5. What changes for the player

- Its report opens with the direction: the plan, its own seat's paragraph, how long ago each was written. In a game
  with one player a seat it is shown its own seat only, with the allied seats as it is shown any ally today.
- The prompt says what the direction is: the side's plan from the seat that sees all of it and talks to the
  people, to be carried out with the seat's own judgement of how; a player that leaves it for something the
  commander cannot have seen says so in a note, which the commander reads.
- One player a seat means one packet a seat. The packet for all seats goes (`hands_merged`'s "the lead's standing
  text") in that configuration; each seat's hands read their own player's words.
- No `say` when a commander is in the game.

## 6. Time

The commander never holds the game. In a game with people that is the whole of it: its direction lands when it is
written.

In the arena's lockstep the game is held for the player's turn and runs many times faster than the wall between
them, so a commander that took 20 s of wall would see a minute and a half of game go by (commander-smoke: the
direction from the report of 0:25 was first shown to the player at 2:00). So while a commander's turn is in hand the
lockstep game runs no faster than the wall, counted from the frame of the turn's report
(`Shared::pace_with_the_commander`): a turn of 20 s costs 20 s of game, as it does against a person. The direction
is dated the same way (`due`), and a player is shown it from that frame.

## 7. Configurations

The user: "It's worth allowing multiple configurations for these games." Three settings, free of each other, for the
arena and `run/human_game.sh` alike:

| Setting | Values | Today |
|---|---|---|
| `--players` | `one` (one player for all our seats), `seat` (one a seat) | `one` |
| `--player-model`, `--player-effort` | a model, an effort | `--commander-model`, `--effort` (renamed; the ledger's old rows keep the old word) |
| `--commander` | `none`, or a model; `--commander-effort` | `none` |

The ones worth playing first:

| Name | Players | Commander | For |
|---|---|---|---|
| solo | one Opus medium | none | today's games; the comparison's base |
| duel-split | one Sonnet | Opus high | the duel with the commander (the user: "the duel should also be able to run with the commander") |
| seats | one Opus or Sonnet a seat | none | what one player a seat buys alone |
| team-split | one Sonnet a seat | Opus high | the team game as designed |

Sessions are one `claude` process a player and one for the commander: four in a 3-seat game. They share the
subscription account given by `--claude-config-dir`.

## 8. What is built, in steps

Each step is played before the next is started.

1. **One player a seat** (`--players seat`): each seat's bot runs its own session over its own briefing and field;
   the merge in `strategist/seats.rs` stays for `one` and becomes the source of the commander's report. Rename
   `--commander-model`. Test: a realtime 3-against-1 rehearsal against BARb beside the same game with `one`
   (turn time, report size, idle constructors and banks by seat, groups gathered).
2. **The commander's report and session** with `direct`, `note`, `wait`, the looking tools; the direction at the
   top of the player's report; the lockstep charge of §6. Test: duel-split on the standing seed against solo and
   against Sonnet alone (the three Sonnet games in the ledger).
3. **`mark` across seats, `say` and chat moved up, `transfer`.** Test: team-split in a rehearsal, then with people.
4. **The pack** as the commander's start context, as it is written (the brief document's §4).

## 9. Found on the way, not part of this

- human-17: Cortex construction ships skipped every under-water spot as "not free" (2:00-2:46), and after being
  given to the Armada seat "this builder cannot build it" for the same spots; two seats never passed 2 extractors.
  Not traced.
- human-17: `remove` with destruct on `commander_t2` answered "no longer stands" twice while it walked.
- human-18: Jev on one seat read 9.2 million tokens in one minute (32,144 questions) at minute 28. The cost of a
  large team game is not looked at.

## 10. Open

- **Whose army is it.** As drafted each seat's groups stay its player's and a joint push is three players told one
  place and one clock. The other way is the commander handing a seat's groups to one player for the push
  (`transfer` of units does it today). Not ruled.
- **How often.** A turn every 60 to 120 s and on the wake events is a guess; high effort on a 10,000-character
  report is unmeasured.
- **How much the player may leave the direction.** Drafted as judgement with a note; the first games will show
  whether Sonnet follows it to the letter into a loss or drops it.
- **The word for the seat in the prompts**, since the picture's `commander_t1` is a unit. Candidates: keep
  "commander" and call the unit "the commander unit"; or name the seat otherwise in the prompts only.
- **Two people's advice at once**: chat goes to the commander alone; whether a player should see chat that names its
  seat.
