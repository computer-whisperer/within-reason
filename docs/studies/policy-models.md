# Other models as the policy-writing player (2026-09-22)

The policy replay (`docs/design/2026-09-22-policy-replay.md`, `run/policy_replay.py`) run with models other than
Opus as the player: for turns 0-12 of game five (comet-5, medium, `run/matches/1790053348-comet-5-medium`, 0:00 to
6:40 of the game), the model gets the report Opus saw, its policy in force and the harness's feedback, and answers
with an amendment (`--mode amend`). The amendment is appended, the policy is run in Lua 5.4 over the hands' calls
recorded until the next turn, and its orders are scored against the menu, Jev's play and the packet's prohibitions.
The user's target: a player turn of about 5 s or less end to end. Recorded data only; nothing here was played.

**Short answer.** No model at any setting makes the 5-second turn as a median. The closest is claude-sonnet-5
(thinking off): 7 s median wall, 6 s of it the model's own time, 4 of 13 turns within 5 s, and those four are the
turns it answered `-- unchanged` or sent under 15 lines; a real amendment (20-30 lines, 600-800 output tokens)
costs it 6-15 s. Every Claude and Codex run wrote legal policies (0 illegal orders in 2,000 decisions), and the
faithful ones (labs ordered every second, raiders answered, agreement with Jev at or above Opus's) came from
sonnet without thinking, haiku without thinking, gpt-5.5 and gpt-5.6-sol at effort low, all of them at 7-56 s a
turn. The OpenAI models cannot reach 5 s through `codex exec` whatever they write: the CLI's floor on a one-word
answer is 3.3-3.8 s alone (2.7-3.1 s of it inside codex's own turn, 0.25 s to spawn), and they write 30-80 lines
per amendment. Haiku with its default thinking is unusable here (140 s a turn, 12,600 output tokens of which
11,000-14,000 are thinking); with thinking off it is faithful but slower than sonnet, because it writes twice as
much. Time here is almost all generation: the process floor is about 1.5 s for `claude -p` and 3.4 s for
`codex exec`, and the rest is output tokens.

## A. Turns 0-12 of game five, amendment mode

`policy-amend` is the Opus run the main session made on the same text at 10:16 (after the 4e49b61 fix); it is the
reference and I did not run it (its metas predate the model-time field). All other rows are this study's runs.
Decisions are the 200 unscripted menus the hands answered in the window (30 lab, 41 builder, 129 group); `legal`
counts orders (or silence, which is `continue`) naming an option on that menu; `agree` counts orders naming what
Jev played; `violate` counts detachments or walks to `shelling` under a packet forbidding them; `reacted` counts
group orders changed within a window in which a party appeared at one of our extractors (14 such episodes).

| run | model, setting | turns with a script | load err | runtime err turns | wall med / max s | turns within 5 s | model time med / max s | out tokens med | lines sent med (unchanged) | decisions | legal | agree | violate | reacted |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| policy-amend | claude-opus-5 (main session's run), effort low | 13/13 | 0 | 0 | 12 / 22 | 0/13 | not recorded | 787 | 30 (3) | 200 | 200 | 100 | 0 | 8/14 |
| policy-sonnet | claude-sonnet-5, effort low, thinking on | 13/13 | 0 | 0 | 8 / 14 | 3/13 | 7 / 13 | 637 | 14 (5) | 200 | 200 | 88 | 0 | 8/14 |
| policy-sonnet-nothink | claude-sonnet-5, effort low, thinking off | 13/13 | 0 | 0 | 7 / 15 | 4/13 | 6 / 13 | 632 | 23 (4) | 200 | 200 | 111 | 0 | 8/14 |
| policy-haiku | claude-haiku-4-5-20251001, effort low, thinking on | 13/13 | 0 | 0 | 140 / 223 | 0/13 | 139 / 222 | 12627 | 52 (0) | 200 | 200 | 101 | 0 | 8/14 |
| policy-haiku-nothink | claude-haiku-4-5-20251001, effort low, thinking off | 13/13 | 0 | 0 | 16 / 29 | 0/13 | 15 / 28 | 1344 | 66 (0) | 200 | 200 | 121 | 0 | 8/14 |
| policy-gpt-5.5 | gpt-5.5, effort low | 13/13 | 0 | 0 | 33 / 43 | 0/13 | 32 / 43 | 1415 | 59 (0) | 200 | 200 | 124 | 0 | 7/14 |
| policy-gpt-5.5-none | gpt-5.5, effort none | 13/13 | 0 | 0 | 22 / 44 | 0/13 | 21 / 43 | 721 | 45 (0) | 200 | 200 | 101 | 0 | 8/14 |
| policy-gpt-5.6-sol | gpt-5.6-sol, effort low | 13/13 | 0 | 0 | 56 / 100 | 0/13 | 55 / 99 | 1477 | 63 (2) | 200 | 200 | 122 | 0 | 8/14 |
| policy-gpt-5.6-sol-none | gpt-5.6-sol, effort none | 12/13 | 10 | 0 | 29 / 74 | 0/13 | 28 / 73 | 564 | 44 (0) | 182 | 182 | 80 | 0 | 0/14 |
| policy-gpt-6-astra | gpt-6-astra, effort low | 13/13 | 0 | 0 | 39 / 72 | 0/13 | 38 / 72 | 986 | 82 (1) | 200 | 200 | 40 | 0 | 4/14 |
| policy-gpt-5.6-luna-low | gpt-5.6-luna, effort low | 13/13 | 0 | 0 | 31 / 64 | 0/13 | 30 / 63 | 863 | 33 (0) | 200 | 200 | 85 | 0 | 1/14 |
| policy-gpt-5.6-terra-low | gpt-5.6-terra, effort low | 11/13 | 0 | 0 | 22 / 73 | 0/13 | 21 / 72 | 879 | 35 (0) | 165 | 165 | 73 | 0 | 5/14 |

Per turn: wall seconds (the model's own time in parentheses), output tokens, lines sent (turn 0: the whole first
script), `ERR` where the policy did not load or raised runtime errors.

| turn | policy-amend | policy-sonnet | policy-sonnet-nothink | policy-haiku | policy-haiku-nothink | policy-gpt-5.5 | policy-gpt-5.5-none | policy-gpt-5.6-sol | policy-gpt-5.6-sol-none | policy-gpt-6-astra | policy-gpt-5.6-luna-low | policy-gpt-5.6-terra-low |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 22 s, 1992 tok, 120 lines | 13 s (12), 1337 tok, 86 lines | 15 s (13), 1190 tok, 60 lines | 107 s (105), 11166 tok, 133 lines | 29 s (28), 2518 tok, 200 lines | 43 s (41), 1987 tok, 151 lines | 44 s (43), 1900 tok, 151 lines | 100 s (99), 2950 tok, 213 lines | 74 s (73), 2083 tok, 182 lines | 72 s (72), 2211 tok, 197 lines | 32 s (30), 1385 tok, 97 lines | 40 s (40), 1885 tok, 114 lines |
| 1 | 6 s, 153 tok, 0 lines | 3 s (1), 49 tok, 0 lines | 3 s (2), 74 tok, 0 lines | 137 s (135), 12444 tok, 48 lines | 20 s (19), 1520 tok, 85 lines | 16 s (15), 416 tok, 11 lines | 14 s (14), 477 tok, 33 lines | 22 s (22), 516 tok, 0 lines | 30 s, no script | 34 s (33), 778 tok, 67 lines | 29 s (29), 1289 tok, 64 lines | 27 s (26), 1020 tok, 56 lines |
| 2 | 6 s, 162 tok, 0 lines | 14 s (13), 1633 tok, 92 lines | 5 s (3), 247 tok, 11 lines | 160 s (159), 14678 tok, 45 lines | 11 s (9), 839 tok, 61 lines | 35 s (34), 1481 tok, 80 lines | 19 s (17), 466 tok, 33 lines | 49 s (48), 1177 tok, 54 lines | 45 s (44), 1088 tok, 83 lines | 41 s (40), 1031 tok, 97 lines | 27 s (26), 866 tok, 41 lines | 13 s, no script |
| 3 | 12 s, 756 tok, 19 lines | 6 s (4), 400 tok, 14 lines | 4 s (2), 111 tok, 0 lines | 223 s (222), 21416 tok, 43 lines | 6 s (4), 291 tok, 22 lines | 29 s (28), 1244 tok, 81 lines | 12 s (12), 316 tok, 22 lines | 15 s (15), 279 tok, 0 lines | 23 s (23), 533 tok, 39 lines, ERR | 50 s (49), 1308 tok, 116 lines | 23 s (22), 650 tok, 31 lines | 13 s, no script |
| 4 | 7 s, 265 tok, 0 lines | 8 s (7), 672 tok, 23 lines | 6 s (4), 504 tok, 21 lines | 214 s (213), 19448 tok, 83 lines | 21 s (20), 1397 tok, 42 lines | 25 s (24), 1071 tok, 36 lines | 19 s (18), 721 tok, 55 lines | 64 s (63), 1748 tok, 103 lines | 26 s (25), 539 tok, 39 lines, ERR | 11 s (10), 77 tok, 0 lines | 25 s (22), 924 tok, 31 lines | 18 s (17), 588 tok, 19 lines |
| 5 | 9 s, 529 tok, 19 lines | 4 s (3), 185 tok, 0 lines | 3 s (2), 74 tok, 0 lines | 147 s (146), 13869 tok, 33 lines | 18 s (16), 1335 tok, 60 lines | 23 s (22), 820 tok, 41 lines | 14 s (13), 348 tok, 22 lines | 54 s (52), 1448 tok, 103 lines | 25 s (24), 537 tok, 40 lines, ERR | 36 s (36), 986 tok, 75 lines | 31 s (30), 539 tok, 22 lines | 24 s (23), 967 tok, 41 lines |
| 6 | 9 s, 708 tok, 33 lines | 3 s (2), 58 tok, 0 lines | 5 s (4), 74 tok, 0 lines | 89 s (88), 8112 tok, 47 lines | 17 s (15), 637 tok, 31 lines | 32 s (31), 1314 tok, 47 lines | 15 s (14), 320 tok, 22 lines | 46 s (46), 1256 tok, 58 lines | 32 s (31), 552 tok, 41 lines, ERR | 35 s (32), 739 tok, 57 lines | 49 s (48), 483 tok, 35 lines | 16 s (15), 573 tok, 25 lines |
| 7 | 12 s, 787 tok, 27 lines | 6 s (5), 267 tok, 0 lines | 9 s (8), 632 tok, 25 lines | 152 s (150), 14134 tok, 62 lines | 14 s (13), 1344 tok, 98 lines | 43 s (43), 2027 tok, 108 lines | 25 s (24), 958 tok, 71 lines | 70 s (69), 1958 tok, 141 lines | 29 s (28), 552 tok, 44 lines, ERR | 39 s (38), 940 tok, 77 lines | 19 s (18), 488 tok, 17 lines | 73 s (72), 816 tok, 22 lines |
| 8 | 18 s, 1614 tok, 85 lines | 8 s (7), 483 tok, 0 lines | 7 s (6), 692 tok, 26 lines | 102 s (100), 10662 tok, 60 lines | 18 s (17), 1581 tok, 118 lines | 33 s (32), 1415 tok, 55 lines | 22 s (21), 842 tok, 59 lines | 56 s (55), 1477 tok, 53 lines | 24 s (24), 555 tok, 44 lines, ERR | 33 s (32), 849 tok, 83 lines | 46 s (45), 863 tok, 30 lines | 20 s (19), 621 tok, 17 lines |
| 9 | 16 s, 1306 tok, 66 lines | 12 s (11), 706 tok, 14 lines | 14 s (8), 826 tok, 28 lines | 166 s (165), 15525 tok, 66 lines | 14 s (12), 1346 tok, 70 lines | 42 s (41), 1655 tok, 102 lines | 22 s (21), 542 tok, 35 lines | 73 s (72), 1995 tok, 132 lines | 44 s (43), 1091 tok, 91 lines, ERR | 46 s (45), 1280 tok, 129 lines | 32 s (32), 950 tok, 53 lines | 22 s (21), 879 tok, 48 lines |
| 10 | 13 s, 951 tok, 40 lines | 8 s (7), 699 tok, 26 lines | 8 s (7), 810 tok, 28 lines | 140 s (139), 12627 tok, 41 lines | 7 s (5), 471 tok, 39 lines | 35 s (34), 1645 tok, 98 lines | 27 s (26), 1035 tok, 68 lines | 48 s (47), 1194 tok, 63 lines | 25 s (24), 574 tok, 46 lines, ERR | 36 s (36), 898 tok, 82 lines | 39 s (38), 521 tok, 48 lines | 20 s (19), 788 tok, 32 lines |
| 11 | 18 s, 1590 tok, 73 lines | 11 s (9), 661 tok, 14 lines | 13 s (12), 846 tok, 25 lines | 117 s (116), 12407 tok, 64 lines | 13 s (11), 1197 tok, 128 lines | 32 s (31), 1334 tok, 54 lines | 36 s (36), 1576 tok, 103 lines | 84 s (83), 1653 tok, 95 lines | 42 s (40), 1039 tok, 95 lines, ERR | 51 s (51), 1426 tok, 142 lines | 64 s (63), 1394 tok, 55 lines | 26 s (25), 1003 tok, 38 lines |
| 12 | 13 s, 1003 tok, 45 lines | 8 s (7), 637 tok, 20 lines | 7 s (6), 749 tok, 29 lines | 100 s (99), 9245 tok, 55 lines | 16 s (15), 1366 tok, 90 lines | 35 s (34), 1509 tok, 63 lines | 27 s (26), 1106 tok, 72 lines | 61 s (60), 1616 tok, 63 lines | 24 s (23), 572 tok, 49 lines, ERR | 55 s (54), 1524 tok, 160 lines | 23 s (23), 580 tok, 27 lines | 25 s (25), 1070 tok, 42 lines |

Per kind of actor: orders given out of decisions, and agreement with Jev.

| run | lab ordered / agree (of 30) | builder ordered / agree (of 41) | group ordered / agree (of 129) | holds |
|---|---|---|---|---|
| policy-amend (Opus) | 30 / 30 | 0 / 23 | 120 / 47 | 34 |
| sonnet, thinking on | 0 / 0 | 41 / 9 | 129 / 79 | 117 |
| sonnet, thinking off | 30 / 28 | 41 / 4 | 129 / 79 | 116 |
| haiku, thinking on | 1 / 1 | 1 / 22 | 129 / 78 | 119 |
| haiku, thinking off | 30 / 30 | 13 / 19 | 129 / 72 | 126 |
| gpt-5.5, low | 30 / 30 | 32 / 15 | 114 / 79 | 89 |
| gpt-5.5, none | 30 / 30 | 41 / 6 | 129 / 65 | 98 |
| gpt-5.6-sol, low | 30 / 30 | 41 / 6 | 123 / 86 | 105 |
| gpt-5.6-sol, none | 7 / 7 (of 25) | 7 / 19 (of 31) | 16 / 54 (of 126) | 16 |
| gpt-6-astra, low | 0 / 0 | 26 / 15 | 72 / 25 | 0 |
| gpt-5.6-luna, low | 30 / 28 | 19 / 17 | 4 / 40 | 0 |
| gpt-5.6-terra, low | 1 / 1 (of 23) | 10 / 14 (of 30) | 50 / 58 (of 112) | 32 |

(A builder that is not ordered keeps its list, so `0 / 23` for Opus means its script left the builders to the lists
and Jev did the same 23 times; the "of N" rows lost decisions to turns without a script.)

## B. The process floor: one trivial call, alone on the machine

Three calls of "answer with the single word pong" per setting, one at a time, after the batches had finished;
median wall, and the CLI's own time (claude: `duration_api_ms`; codex: `turn.started` to the final message).

| CLI, model, setting | wall | CLI's own time | of which spawn |
|---|---|---|---|
| claude -p, haiku, thinking on (99 output tokens: it thinks even for "pong") | 3.41 s | 1.85 s | |
| claude -p, haiku, thinking off | 2.12 s | 0.62 s | |
| claude -p, sonnet, thinking on | 2.56 s | 1.04 s | |
| claude -p, sonnet, thinking off | 2.70 s | 1.17 s | |
| codex exec, gpt-5.5, low | 3.41 s | 2.71 s | 0.27 s |
| codex exec, gpt-5.5, none | 3.32 s | 2.71 s | 0.23 s |
| codex exec, gpt-5.6-sol, low | 3.43 s | 2.85 s | 0.24 s |
| codex exec, gpt-5.6-sol, none | 3.54 s | 2.94 s | 0.23 s |
| codex exec, gpt-6-astra, low | 3.78 s | 3.10 s | 0.25 s |
| codex exec, gpt-5.6-luna, low | 3.41 s | 2.80 s | 0.24 s |
| codex exec, gpt-5.6-terra, low | 3.31 s | 2.71 s | 0.23 s |

So `claude -p` costs about 1.5 s of process around the request (wall minus the API time, the same on the real
turns: sonnet's 7 s median wall is 6 s of model time), and its model time on a one-word answer is 0.6-1.2 s.
`codex exec` spawns in 0.25 s but its turn takes 2.7-3.1 s before a five-token answer is back: every call carries
13-15k input tokens of codex's own prompt and tool definitions (the trivial call's `input_tokens`), and the turn
span is the only timing codex gives, so its generation time proper cannot be separated from that overhead. Under
load the floor moves: the same trivial codex call took 6-11 s while three batches ran, and the batch walls below
were measured with three or four runs side by side.

## Method

- Branch `worktree-agent-a2c4a666ff7208e27`, started from main at 4e49b61 (the role text fix: `enemies_near` is a
  string; the next turn's prompt carries the script's run count, errors and orders). Every run below used that text.
- `run/model_cli.py` is the backend switch: a `claude-*` model runs through `claude -p --tools "" --strict-mcp-config
  --output-format json --effort E --system-prompt ROLE` on `~/.claude2` (a usage snapshot before each batch,
  `--since` after); a `gpt-*` model through `codex exec -m MODEL -c model_reasoning_effort=E -s read-only
  --skip-git-repo-check --ephemeral --json -C <empty tmp dir> -o <file> -`, the prompt on stdin with a preamble that
  says to answer directly and use no tool, then the role text and the report (codex has no system prompt of its own
  to set). `--out NAME` puts each run in `<match>/00/NAME`, so runs never touch `policy/` or `policy-amend/`.
- Time: `wall` is the whole process from spawn to exit. The model's own time is what the CLI reports: claude's
  JSON result carries `duration_api_ms`; codex reports no timing, so the harness stamps its `--json` events and
  takes `turn.started` to the final `agent_message`.
- Effort: `claude --effort low` is that CLI's floor (low, medium, high, xhigh, max). Codex accepts
  `model_reasoning_effort=none` on gpt-5.5 and gpt-5.6-sol; gpt-6-astra rejects it ("'none' is not supported with
  the 'gpt-6-astra' model"); `minimal` is rejected by the API for gpt-5.5 ("not supported"), and with codex's web
  search on it is rejected earlier still ("cannot be used with reasoning.effort 'minimal': web_search"). Haiku at
  `--effort low` still thinks 10-14k tokens a turn; `MAX_THINKING_TOKENS=0` (the harness's `--thinking 0`) turns
  thinking off for Claude models, and sonnet and haiku were run both ways.
- Scoring is `run/policy_replay.py`'s, unchanged.

Commands (one per run; `--out` names are the directories under the match's `00/`):

```
run/policy_replay.py run/matches/1790053348-comet-5-medium --model claude-sonnet-5 --mode amend --turns 0-12 --out policy-sonnet
run/policy_replay.py ... --model claude-haiku-4-5-20251001 --mode amend --turns 0-12 --out policy-haiku
run/policy_replay.py ... --model claude-sonnet-5 --thinking 0 --mode amend --turns 0-12 --out policy-sonnet-nothink
run/policy_replay.py ... --model claude-haiku-4-5-20251001 --thinking 0 --mode amend --turns 0-12 --out policy-haiku-nothink
run/policy_replay.py ... --model gpt-5.5 --mode amend --turns 0-12 --out policy-gpt-5.5            # effort low (default)
run/policy_replay.py ... --model gpt-5.5 --effort none --mode amend --turns 0-12 --out policy-gpt-5.5-none
run/policy_replay.py ... --model gpt-5.6-sol --mode amend --turns 0-12 --out policy-gpt-5.6-sol
run/policy_replay.py ... --model gpt-5.6-sol --effort none --mode amend --turns 0-12 --out policy-gpt-5.6-sol-none
run/policy_replay.py ... --model gpt-6-astra --mode amend --turns 0-12 --out policy-gpt-6-astra
run/policy_replay.py ... --model gpt-5.6-luna --effort low --mode amend --turns 0-12 --out policy-gpt-5.6-luna-low
run/policy_replay.py ... --model gpt-5.6-terra --effort low --mode amend --turns 0-12 --out policy-gpt-5.6-terra-low
run/policy_replay.py ... --summarize --out <name>
```

Sample sizes and usage: 13 turns per run, 12 runs (4 Claude, 8 Codex), 200 decisions scored per complete run.
Claude: 52 policy calls plus 18 trivial ones on `claude2`; the 7-day window rose 3.0 points from the first snapshot
to the last reading (sonnet with thinking: +0.0; haiku with thinking: +2.0; the two no-thinking runs share the rest
with the main session's own Opus runs on the same account during the same hour, so 3.0 is an upper bound); extra
usage +0.00 and not enabled on that account; nominal cost as `claude -p` prints it: sonnet $0.87 and $0.82, haiku
$1.09 and $0.78. Codex: 104 policy calls plus 33 trivial ones on the ChatGPT subscription, no retries, no rate
limits; codex prints no cost. The four Claude runs were made one after another; the eight Codex runs three or four
at a time, side by side with the Claude runs.

## Reading

**Time.** Legal and faithful under 5 s: nobody, as a median. Sonnet without thinking gets a turn under 5 s only when
it has little to say (turns 1, 3, 5, 6: `-- unchanged` or 11 lines, 3-5 s, 74-250 output tokens); when it amends a
handler (20-30 lines, 500-850 tokens) the turn is 6-15 s. Its time is 1-2 s of process and the rest generation at
roughly 100 output tokens a second, so the lever is output length: an amendment held to 15 lines and no closing
sentence would sit at the target, at the cost of the harness feedback loop the closing sentence carries. Thinking
on or off changes sonnet's time little (median 8 vs 7 s; 100-300 thinking tokens at effort low) and its agreement
somewhat (88 vs 111, see below). Haiku is not the fast option: with thinking it spends 100-220 s a turn on 8,000-
21,000 thinking tokens, and with thinking off it still writes 1,344 output tokens a turn (a page of analysis prose
before the code, 66 lines sent) at about 90 tokens a second, so 16 s median. The Codex models are 22-56 s a turn:
a 3.4 s floor, 30-80 lines of code per amendment (700-1,500 output tokens), and whatever the model's own latency is
on a 30k-token prompt, which codex does not expose. Effort `none` cuts gpt-5.5 from 33 to 22 s and halves its
output tokens at no cost in legality; on gpt-5.6-sol it cut 56 to 29 s but produced the one broken policy of the
study.

**Legality.** 2,000 decisions across ten complete runs, 0 illegal orders; one order to a place not in the list
(gpt-5.5, both settings, the same line). The menu-as-options design (`a.options.X` must exist) makes illegal orders
hard to write, and every model used it. Load and runtime errors: none in eleven runs; gpt-5.6-sol at `none` left a
stray `")` in turn 3 (`not has(S.enemy.commander, "unknown")") then`), the policy stopped loading, and it never
recovered, because an amendment is appended and cannot remove a line, while the model believed it had ("I fixed the
stray quote that prevented the policy loading", turn 4) and went on amending a script that was dead for the
remaining ten turns. That is a harness finding as much as a model one: in amendment mode an amendment that leaves
the policy unloadable should be rejected and the previous policy kept, or the harness should replace the redefined
function rather than append. Three turns had no script at all (gpt-5.6-terra turns 2 and 3, gpt-5.6-sol `none`
turn 1): the model answered with bare Lua and no fence, which the harness scores as no script; the code itself was
plausible.

**Faithfulness.** Agreement with Jev over the 200 decisions: gpt-5.5 low 124, gpt-5.6-sol low 122, haiku off 121,
sonnet off 111, gpt-5.5 none 101, haiku on 101, Opus 100, sonnet on 88, luna 85, sol none 80, terra 73, astra 40.
Most of it is holds agreeing with holds (Jev holds most of the time; sonnet's 79 group agreements sit inside 117
holds), so the finer reading is by kind. Labs: Opus, gpt-5.5, sol, haiku off, sonnet off and luna order the plant
every second and agree with Jev's produce choice 28-30 of 30; sonnet with thinking, haiku with thinking, astra and
terra never or almost never order it (0-1 of 30), leaving production to whatever the plant was doing, which in
this window was Jev's earlier pick. Raiders: Opus, sonnet (both), haiku (both), gpt-5.5 `none` and sol answered 8 of
the 14 episodes, gpt-5.5 low 7, terra 5, astra 4, luna 1, sol `none` 0 (dead policy). Astra and luna wrote the most
code and ordered the least: astra's group handler keeps a `raid_sent` table and returns nil for anything already
travelling, so it ordered 72 of 129 group decisions and held on none; luna ordered 4 of 129. Length is not
faithfulness here: the two shortest Claude policies (sonnet, 60-300 lines) and the two longest Codex ones (astra
1,304 lines, sol 1,098 by turn 12) sit at opposite ends of the agreement column.

**What the columns do not say.** `violate` is 0 everywhere because the packets of turns 0-12 contain no "never
splits" or "never advances to shelling" clause (the first appears at 23:14 in this game); the column measured
nothing in this window. `agree` is agreement with Jev's play under Opus's packets, a proxy for faithfulness to the
same report, not a quality score: a better decision than Jev's counts as disagreement. The scripts' orders did not
change what came next, so nothing here says which policy would have won. The Opus row was run by the main session,
not by me; I read its files. Walls were measured with three or four runs side by side on one machine and the
Codex floor moved from 3.4 s alone to 6-11 s under that load, so the Codex batch walls are high by a few seconds
each.

**What would get a turn under 5 s.** Not a smaller model: haiku writes more than sonnet, not less, and the Codex
CLI's floor is most of the budget by itself. The levers that the numbers point at: output length (sonnet is at
100 tokens a second, so a 5 s turn is 300-400 tokens after the 1.5 s floor: an amendment of one handler and no
prose); a persistent process instead of a spawn per turn (1.5 s for claude, 3.4 s for codex, if the API behind
codex can be reached without its per-call prompt); and, for the Codex models, an API path rather than `codex exec`,
which was not tested here.
