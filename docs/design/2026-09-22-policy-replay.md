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

## Other models

`--model` picks the backend (`run/model_cli.py`: `claude -p` for Claude models, `codex exec` for OpenAI models) and
`--out NAME` keeps each model's run in its own directory. Turns 0-12 of game five in amendment mode for sonnet,
haiku, gpt-5.5, gpt-5.6-sol, gpt-5.6-luna, gpt-5.6-terra and gpt-6-astra, against the user's 5-second turn
target: `docs/studies/policy-models.md`.
