# The API backend for the player (2026-09-27)

**Decision.** A third session kind beside Claude Code and the Codex CLI: `SessionKind::Api`, an OpenAI-compatible
chat-completions client with tools, for models billed per token on an API key (Fireworks.ai first, for the open
models: DeepSeek, Qwen, Kimi, GLM, gpt-oss). The user's lean, agreed: the CLIs earn their place only through the two
subscriptions; for API-billed models a CLI adds a translation proxy (Claude Code speaks the Messages API, Fireworks
OpenAI's) and hidden scaffolding around every turn, and a tool-call fault in that layer would be indistinguishable
from a model fault on the scorecard. Rejected: Claude Code with `ANTHROPIC_BASE_URL` through LiteLLM or the like.

**Model names.** `--commander-model fw:<model id>` runs on Fireworks (`accounts/fireworks/models/...`), `api:<model>`
on any OpenAI-compatible endpoint. Everything else keeps its backend: `gpt-*`/`o3`/`o4` the Codex CLI, the rest
Claude Code.

**Configuration, never in the tree.** `~/.config/within-reason/fireworks.env` (mode 600):
`FIREWORKS_API_KEY=...`, optional `FIREWORKS_BASE_URL` (default `https://api.fireworks.ai/inference/v1`),
`FIREWORKS_MAX_TOKENS` (default 8192). `~/.config/within-reason/api.env` for `api:`: `API_KEY`, `API_BASE_URL`
(required), `API_MAX_TOKENS`. The environment variable of the same name overrides the file. Read as the Jev key is
(`crates/jev`): never printed, never written.

**The turn.** The session owns its message list: the system prompt (`player.md` + the brief + the objective, exactly
as the other backends get it), then per turn the report as a user message, the model's reply, its tool calls and
their results as tool messages. The tools are the MCP tool list in OpenAI's shape (`function` with the same
`inputSchema` as `parameters`), dispatched in-process through the same `call_tool`/`orders` path as the MCP server,
recorded in the transcript by the same line (`kind: tool_call`). A turn ends when the `orders` call carried its
`wait` (`Shared::turn_over`, set by `end_turn_at_wait`): no closing call is made, which removes the second request
the CLI backends pay for every turn (3.3 s median, 19% of the wall in onepass-player-2). A reply with no tool call
ends the turn too (`ended_by: response`). Reasoning effort goes as `reasoning_effort` when the endpoint accepts it;
a 400 naming it makes the session drop the parameter for good.

**Context.** The list is trimmed before each call to a character budget (`API_CONTEXT_CHARS`, default 320,000, about
80k tokens): whole turns are dropped oldest first, the system prompt kept. The session is replaced at
`TURNS_PER_SESSION` and on a prompt edit as the others are, the notes handed over by the fresh session's report.

**Transcript.** The same lines as the CLI backends so `run/floor.py`, `player_read` and the bundle reader work
unchanged: `turn`, `assistant` (`message.message.content` with `text` and `tool_use` blocks), `tool_call`,
`turn_end` (`ended_by` wait/response/abandoned), `result` (`usage.input_tokens`, `usage.output_tokens`,
`usage.cache_read_input_tokens` when the endpoint reports it, `duration_api_ms` summed over the turn's calls,
`num_turns` = calls in the turn, `modelUsage`). The turn cap is the 120 s one (not Claude's 45 s).

**Status.** Built 2026-09-27; untested in a game until the first Fireworks key is dropped in.
