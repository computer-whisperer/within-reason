# Heuristic bots

Bots that decide in code and cost no tokens: no Jev, no Claude player, no bot process. Each directory here is a
`cdylib` the engine loads as `libSkirmishAI.so`: the host (`crates/ai-host`) over `ai_host::InProcess<B>`, where `B`
implements `bot_protocol::Bot` (`start` from the `Hello`, then `tick` to `Commands` on the engine thread).

**This is a different regime from the flagship.** In `crates/bot` no code decides a live game without Jev or the
player (`docs/code-decisions.md`, `docs/design/2026-09-28-no-live-heuristics.md`). That rule stands there and does not
govern this directory: here heuristics are the product. Keep the two apart: a rule written for a bot of `bots/` never
goes into `crates/bot`, and each bot here registers its own rules in its own directory (`D-` ids for `distilled`), not
in `docs/heuristics.md`.

Why they exist (the user, 2026-10-05): now that the flagship's games show what decent play looks like, fit a bot to
that shape which anyone in the BAR community can run for nothing.

## The bots

| Directory | Lobby name | What it is |
|---|---|---|
| `template` | `WRTemplate` | The skeleton to copy. One rule: an idle builder builds an extractor on the nearest metal spot it has not been sent to. |
| `distilled` | `WRDistilled` | The flagship's opening, expansion and production as rules, read from seven games it won, and a simple army over them. Its rules, their sources and its results: `distilled/README.md`. |
| `territory` | `WRTerritory` | Plans its economy by a rolling-horizon search over the build-order simulator (`crates/buildorder`): no rule names a unit. Stage 1 is metal against an opponent that does nothing; the territory model comes after. Its design, arms and results: `territory/README.md`. |

## Running one

`target/release/arena --our-ai WRTemplate:0.1 --map "<map>" ...` seats it in place of the flagship; the arena then
starts no bot process. `run/install_ai.sh` installs the bots into the arena's data directory only, never into a BAR
install.

## Limits of the in-library host, as built

- `tick` runs on the engine thread: the game waits for it. Keep it well inside a frame (33 ms at speed 1).
- A panic in the bot is caught at the library's edge and the bot's state is whatever the panic left.
- No match record is written (`record-<ai>.jsonl` is the flagship's recorder, in `crates/bot`), so the viewer and the
  review tools have nothing to read for these games yet; the engine log's census, balance and truth lines are there.
- Linux only so far; nothing builds the library for Windows.
- Linking the host makes the library GPL-2.0-or-later (the engine's headers).
