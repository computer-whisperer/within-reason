# The API backend (`--player-model fw:<model>` / `api:<model>`)

Design: `docs/design/2026-09-27-api-backend.md`. Code: `crates/bot/src/strategist/api.rs`.

Put the key in `~/.config/within-reason/fireworks.env` (`chmod 600`):

    FIREWORKS_API_KEY=fw_...
    # the model's serverless prices, USD per million tokens (required: the cap below is kept from them)
    FIREWORKS_PRICE_INPUT=0.30
    FIREWORKS_PRICE_OUTPUT=1.20
    # optional
    FIREWORKS_PRICE_CACHED=0.006
    FIREWORKS_COST_CAP=1
    # Fireworks' service tier: priority (stronger admission under congestion, about 1.5x the price, applied to the cap's arithmetic)
    FIREWORKS_SERVICE_TIER=priority
    FIREWORKS_BASE_URL=https://api.fireworks.ai/inference/v1
    FIREWORKS_MAX_TOKENS=8192

The prices are per model, so change them with the model. **The cost cap** (`FIREWORKS_COST_CAP` / `API_COST_CAP`,
USD a game, default 1): every call's usage is priced and summed across the game's sessions; the call that reaches
the cap still lands its tool calls, then the player is silent for the rest of the game and the hands carry on under
its last packet (the bot log: "cost cap reached: $X of $Y spent"; the transcript: a `cost_cap` line, and the turn's
`result` line carries `total_cost_usd`, `spent_usd` and `cap_usd`). Without both prices the session does not start.

Then, for example:

    target/release/arena --pianist --player --profile medium --map "Comet Catcher Remake 1.8" --corner nw \
      --mirror --place --side armada --think-penalty 1 --matches 1 --speed 50 --base-port 9300 \
      --label fw-deepseek-1 --player-model fw:accounts/fireworks/models/deepseek-v3p1 --player-effort low

A quick check of the key and a model id before a game, one call, no game:

    python3 -c 'import sys; sys.path.insert(0,"run"); from model_cli import ask; print(ask("You are a test.","Reply OK.","fw:accounts/fireworks/models/deepseek-v3p1"))'

Any OpenAI-compatible endpoint: `api:<model>` with `~/.config/within-reason/api.env` holding `API_KEY` and
`API_BASE_URL` (and `API_MAX_TOKENS`). The environment variable of the same name overrides the file. The key is never
printed or written by the bot or the checker: the repository is public.

**Reasoning.** `--player-effort` is sent as `reasoning_effort`; Fireworks takes `low`, `medium`, `high`, `xhigh`, `max` and
`none`. DeepSeek V4.1 Flash at `low` reasoned for the whole 8,192-token output budget on a real report (255 s, no
tool call; fw-deepseek-v41-flash-1's turns overran the 120 s cap) and at `none` answered the same report in 4.5 s
with tool calls, so its games run with `--player-effort none`. A session replaced after a turn passed the cap drops the
answer that comes later (a `late` transcript line) instead of playing it in the next session's turn.

What differs from the CLI backends: the session keeps its own message list (trimmed to `API_CONTEXT_CHARS`, default
320,000 characters, oldest turns first); the turn ends at the `orders` call's `wait` with no closing request; the
turn cap is 120 s; `reasoning_effort` is sent from `--player-effort` and dropped for the session if the endpoint refuses it.
The transcript lines are the same, so `run/floor.py` and the readers apply.
