# The API backend (`--commander-model fw:<model>` / `api:<model>`)

Design: `docs/design/2026-09-27-api-backend.md`. Code: `crates/bot/src/strategist/api.rs`.

Put the key in `~/.config/within-reason/fireworks.env` (`chmod 600`):

    FIREWORKS_API_KEY=fw_...
    # optional
    FIREWORKS_BASE_URL=https://api.fireworks.ai/inference/v1
    FIREWORKS_MAX_TOKENS=8192

Then, for example:

    target/release/arena --pianist --player --no-rules --profile medium --map "Comet Catcher Remake 1.8" --corner nw \
      --mirror --place --side armada --think-penalty 1 --matches 1 --speed 50 --base-port 9300 \
      --label fw-deepseek-1 --commander-model fw:accounts/fireworks/models/deepseek-v3p1 --effort low

A quick check of the key and a model id before a game, one call, no game:

    python3 -c 'import sys; sys.path.insert(0,"run"); from model_cli import ask; print(ask("You are a test.","Reply OK.","fw:accounts/fireworks/models/deepseek-v3p1"))'

Any OpenAI-compatible endpoint: `api:<model>` with `~/.config/within-reason/api.env` holding `API_KEY` and
`API_BASE_URL` (and `API_MAX_TOKENS`). The environment variable of the same name overrides the file. The key is never
printed or written by the bot or the checker: the repository is public.

What differs from the CLI backends: the session keeps its own message list (trimmed to `API_CONTEXT_CHARS`, default
320,000 characters, oldest turns first); the turn ends at the `orders` call's `wait` with no closing request; the
turn cap is 120 s; `reasoning_effort` is sent from `--effort` and dropped for the session if the endpoint refuses it.
The transcript lines are the same, so `run/floor.py` and the readers apply.
