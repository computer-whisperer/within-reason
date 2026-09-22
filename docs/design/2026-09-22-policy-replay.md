# Opus writes the policy: an offline replay of game five (2026-09-22)

The user's idea (2026-09-22, after `docs/studies/jev-comet-series.md`): "having opus write lua scripts on the fly,
to be executed repeatedly realtime until opus's next turn. It's possible this should be added as an additional tool
alongside the current system, but we should test how reliably opus can emit instructions with enough nuance while
under fire." The roadmap (memory `roadmap`) had the Lua runtime as steps 3 and 4 with the rule that its API be
designed from what Opus tried to express in the games. This is the test, offline, before any runtime goes into the
bot. Ruling: "Go ahead, build the replay harness and run game five."

## What is built

`run/policy_replay.py <match>`: for each recorded player turn of a match, Opus is given the same report it saw in the
game (the `turn` prompt in `strategist-<ai>.jsonl`), a role text in which its lever is a Lua policy instead of the
packet (`run/policy/policy-player.md` + the same brief), and its policy in force from the previous turn. It answers
with a script. The script is then run, in stock Lua 5.4, once per recorded hands' call between that turn and the next
(the `state`, `groups`, `places` and per-actor menus of `jev-<ai>.jsonl`), and its orders are scored against what the
menu offered, what Jev played, and what the packet of that turn forbade.

The policy API is the hands' picture, unchanged, as a Lua table, and the hands' menu verbs, unchanged, as the orders
(`docs/design/2026-09-21-pianist.md`). Same information, same levers, so the comparison with Jev is fair and the gap
between the policy and the game stays the one the user asked to keep small.

```
function decide(S)            -- called once a game second
  -- S.clock "12:34", S.frame
  -- S.actors[name]: kind ("builder"|"lab"|"group"), at, doing, health, units, is, from_home, list, next, allowed,
  --   we_have, enemies_near, enemies_at_our_extractors (list), under_fire, footwork, lane,
  --   options[name] = the option's words (the menu this second; only these can be ordered)
  -- S.groups[name]: x, z, members, task           S.economy.metal / .energy (words)
  -- S.enemy: in_sight (list), army_known, base, commander      S.ours: soldiers, extractors, ... (words)
  -- S.places[name]: what, grid, from_home, ground, x, z, spot
  return { [actor] = { ["do"] = option, where = place, whom = party, how_many = "2"|"4"|"8"|"half", where_scout = place }, ... }
end
```
An actor absent from the table keeps its course (the hands' `continue`). Helpers: `has(s, words)` (plain substring),
`starts(s, words)`.

## What is measured, per turn's script

- Loads and runs: syntax errors, runtime errors per call, time Opus took to write it (the game's turn cap is 45 s).
- Legal: orders naming an option the actor was offered that second; illegal ones counted.
- Coverage: of the actors the hands asked that second, how many the script ordered; the share of scripts that order
  nothing but hold.
- Reaction: whether a group's order changed within the window when a party appeared at one of our extractors.
- Agreement with Jev's played choice, and the script's order at the moments the study flagged (3:53 the first
  detachment, 8:26 the fold-back, 17:07 the Janus, 23:14-23:29 detachments after "forbade detachments", 24:11).
- Prohibitions: the same "never" checks the study ran, applied to the script's orders under the packet Opus wrote at
  that turn in the real game (the script should embody it).

## What it cannot measure

The recorded states were produced under Jev's play; a script's orders do not change what came next, so this measures
whether Opus can write a policy that is legal, complete, reactive and faithful under the same pressure, not whether
the policy would have won. A played game is the next step if the scripts hold up.

## Cost

One Opus session per turn (claude-opus-5, effort low, the claude2 account, snapshot before and `--since` after);
Lua runs are local and free. Game five: 51 turns, 1,100 decisions.

## Results, game five (2026-09-22)

Three runs of `run/policy_replay.py`, Opus (claude-opus-5, effort low) on the claude2 account, no overage.

| run | turns | wall per turn | output tokens | script size | errors | legal | agree with Jev | reacted |
|---|---|---|---|---|---|---|---|---|
| full rewrite, process per turn (the old role text) | 51 | median 29 s, max 43 | median 2,870 | 99-216 lines | runtime errors on 36 turns, all one planted bug | 1,100 of 1,100 | 390 | 3 of 64 |
| amendments, process per turn, corrected text | 13 (0-12) | median 12 s; "unchanged" turns 6-7 s | median 787 | grows 120 to 545 lines | none | 200 of 200 | 100 | 8 of 14 |
| amendments, warm session, corrected text | 13 (0-12) | median 12 s; "unchanged" turns 2.4-3.0 s | median 855 | grows 134 to 488 lines | none | 200 of 200 | 75 | 3 of 14 |

An earlier amendment run with the old text (kept as `policy-amend-v1`) gave median 14 s, no errors, 81 agreements.

What the numbers say:

- **Opus writes legal policies under fire.** 1,500 scored decisions across the runs, no illegal option, no bad place,
  no load error; one packet violation. The first script of a game came in 22-43 s and 99-147 lines.
- **The one bug was the prompt's.** The role text's example indexed `enemies_near` as a list; the picture gives a
  string. Every full-rewrite turn from 3:53 to 21:00 raised "attempt to get length of a nil value" on every call and
  gave no orders, because each rewrite carried the previous script's line and the model never saw the error. The
  corrected text describes the fields as strings and the harness now feeds the script's run count, errors and orders
  into the next prompt; the corrected runs raised nothing.
- **Time is generation, not startup.** A warm session cuts the floor from 6-7 s to 2.5 s and no more: at about
  100 output tokens a second, an 850-token amendment is 8-13 s. Opus at low effort (the lowest the CLI offers)
  rewrites whole handlers of 20-75 lines. A five-second turn, the user's target, is an amendment of about 250 tokens.
- **Appending grows the policy.** Twelve amendments took the script from 120 to 545 lines, most of it dead copies of
  redefined handlers, and the prompt's new tokens per turn from 11k to 17k. A runtime must replace a handler in place.
- **The scripts re-issue orders.** On 29 of 61 asks where the actor was busy the corrected script ordered `fight_to`
  to the destination it was already walking to; the runtime must treat an order equal to the current task as continue.
- **Agreement with Jev is not a quality measure here** (the scripts disagree with Jev on groups by design: posts and
  detachments where Jev held); `reacted` and the packet checks are.

Levers toward five seconds, for the user's choice: amendments bounded to one short function with in-place
replacement; rules as data (a one-line rule per change); a faster model writing the amendments (a subagent is
measuring Sonnet 5, Haiku 4.5 and the OpenAI models through the Codex CLI on the same 13 turns).

The user's ruling on the targets (2026-09-22): Jev turns sub-second (they are, 200 ms); player turns about 5 s or
faster. The user's next interest: a real game, Opus against BARb easy with the policy and no Jev. Needed: the `mlua`
runtime in the bot calling `decide` once a second and feeding the orders to `play_one` as scripted answers, a
`policy` tool (set or amend, handler replaced in place, errors and orders in the turn report), a `--policy` switch.

## The runtime (2026-09-22, the user: "replace in place and allow state; build it so that lua can plausibly run alongside jev, with opus sending commands to both")

Target design, written before the build.

- **Where it sits.** `brain/pianist/policy.rs`: a `Policy` (an `mlua` Lua 5.4 state, vendored) owned by the
  `Pianist` beside the Jev client, which becomes optional. Each ask, after the picture and the menus are built, the
  policy is called once with the picture as a table (`S`, exactly the replay harness's shape: the actors with their
  `options` from the menus, `groups`, `places`, `economy`, `enemy`, `ours`). Its orders become synthetic answers
  (`<actor>.do` chosen at probability one, plus `where`, `whom`, `how_many`, `where_scout`, `where_extractor`) and
  the menus it ordered are played by `play_one` unchanged (the switch margin is passed by construction). The menus
  it did not order go to Jev when Jev is on, and continue when it is off. So both hands can play at once, the
  policy taking the actors it names, and the player commands both: `policy` for the script, `instruct` for Jev.
- **Replace in place, allow state.** The Lua state lives for the game; globals persist between seconds and across
  amendments. An amendment is executed in that state, so a redefined function or table replaces the old one at the
  Lua level; the policy's text is kept as top-level blocks keyed by the name they define (`function NAME`,
  `NAME = ...`; unnamed blocks keep their order), and an amendment's blocks replace the blocks of the same name, so
  the text shown to the player stays flat. `set` replaces the whole script and starts a fresh state.
- **An order equal to the actor's current task is continue**: a group already advancing to the place, a builder
  already helping that factory or walking there; builds are covered by H-HANDS-STARTED.
- **Feedback.** The pianist counts the policy's runs, errors (first text kept), orders by option and illegal orders
  (an option the actor was not offered, a place not in the picture); the player's report carries one line of it
  every turn and the policy in force at a session's first turn; the `policy` tool with no arguments returns the
  text in force. The Jev log's `played` entries and the record's decisions carry source `policy`.
- **The tool.** `policy { "set": script }` or `{ "amend": chunk }` (or `{}` to read); the chunk is syntax-checked
  in the tool call so the player hears of a parse error at once; runtime errors come in the report. Batchable in
  `orders`.
- **Switches.** `bot --policy [--pianist]` and `arena --policy`: the policy runtime on; with `--pianist` too, Jev
  beside it; `--player` needs one of them. The player's role text in policy mode is
  `crates/bot/src/strategist/policy.md` (the replay's text adapted to the live tools) + the same brief.
- **Not now:** realtime (the policy call is in the brain thread and takes milliseconds, so it is fine either way);
  the global Nouls without Jev (no hands' wake); a time budget per call (a runaway script; `mlua`'s instruction
  hook is the place if it is ever needed).

### policy-1-easy (2026-09-22): the first game, won

`run/matches/1790088367-policy-1-easy`, Comet Catcher, BARb easy, the policy alone (no Jev): **won in 13.9 min**.
Twelve turns, wall median 7.1 s: the 41-line first script took 12 s; the amendments (one or two handlers, 3 to 42
lines) 5 to 10 s, the 42-line rebuild of 8:01 20 s. The runtime ran 343 times at 0.09 ms median with no error. Three
illegal orders, all one cause: the policy named `spot_25`, which was not among the picture's places (a spot named in
the packet is put on the menu, H-HANDS-NAMED-PLACES; one named in code was not); the player marked five search
points and swept them. Opus wrote the policy as the role text asked (handlers per kind, globals for state: a search
index and a "made_plant" flag) and its turns read like a player's: scout east at 4:00, sweep the eastern spots at
6:00 when the base guess was empty, recall and mass at 8:01 when Blitzes died piecemeal to turrets, one attack at
34 units, advance onto the shelling at 12:01, engage beside the march at 12:15. Its two wishes: to see where unseen
fire comes from (the picture names `shelling`; the policy did not use it until told), and a standing rule for
reinforcements to join the ball where it is.

Follow-ups from the game: spots named in the policy's text join the picture's places as the packet's do; the
scripted (list) plays in the policy's log line carry source `jev` and should say `list`.

## Other models

`--model` picks the backend (`run/model_cli.py`: `claude -p` for Claude models, `codex exec` for OpenAI models) and
`--out NAME` keeps each model's run in its own directory. Turns 0-12 of game five in amendment mode for sonnet,
haiku, gpt-5.5, gpt-5.6-sol, gpt-5.6-luna, gpt-5.6-terra and gpt-6-astra, against the user's 5-second turn
target: `docs/studies/policy-models.md`.
