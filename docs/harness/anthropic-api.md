# The Anthropic API directly (the Haiku 5.5 sandbox, 2026-10-07)

A sandbox for Claude Haiku 5.5 in Jev's seat, over `POST https://api.anthropic.com/v1/messages` with no SDK and no
`claude -p` (the pianist's calls are a few thousand tokens in, a few dozen out, hundreds a game minute: a process
launch a call is out of the question, and the subscription does not meter the API).

The key goes in `~/.config/within-reason/anthropic.env` (`chmod 600`), one line:

    ANTHROPIC_API_KEY=sk-ant-...

The environment variable of the same name overrides the file. Nothing in the repository prints or writes it: the
repository is public.

**Rate probe.** `run/haiku_rate_probe.py JEV_LOG` replays recorded Jev calls (`jev-<n>.jsonl` beside a record) as
streamed requests, the standing text as a cached system block, the picture and questions as the turn, one JSON object
of answers back, and reports input tokens, cache reads, time to first output token, output tokens a second, wall time
and cost, per call and as medians; `--sizes median,p90,max`, `--parallel N` for aggregate rate, `--effort`,
`--no-thinking`, `--model` for the same asks on another model.

What the model is (docs read 2026-10-07): `claude-haiku-5-5`, 1M context, 128K output, $0.10 in / $0.50 out per
million tokens for prompts up to 100K tokens ($0.50 / $2.50 past that), cache reads a tenth of the input rate. Thinking
is on by default (adaptive) and `output_config.effort` is the lever (`low` to `max`, default `medium`); `thinking:
disabled` is accepted at `high` or below. `temperature`, prefill and `budget_tokens` are refused. Safety classifiers can
answer `stop_reason: refusal` with no server-side fallback: handle it in the client. Haiku 5.5's tokenizer counts about
30% more tokens than Haiku 4.5's on the same text.
