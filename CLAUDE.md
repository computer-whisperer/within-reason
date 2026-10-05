# Within Reason (repo formerly `bar_bots`; the local checkout directory may still carry that name)

AI players for Beyond All Reason (Recoil engine): a Rust AI shim loaded by the engine, a Rust bot process that decides,
and an arena that runs headless batches against the stock BARb AI.

- `DESIGN.md` — architecture and protocol. Decisions, not findings.
- `docs/README.md` — how knowledge is organised and the observe -> claim -> exploit -> verify -> retire loop. **Follow it:**
  every brain change registers its heuristic in `docs/heuristics.md` and the claim it rests on in `docs/knowledge/`;
  every arena batch gets a line in `docs/experiments.md`.
- `docs/harness/record-format.md` — the per-match record the bot writes; `run/view_match.py` (the matches directory, or one match) replays it in `viewer/`.
- `docs/harness/pitfalls.md` — read before running engines or installing the AI.
- `upstream/`, `run/engines/`, `run/data/`, `run/matches/` are git-ignored working data.

Two regimes, by directory. Know which one you are in before adding a decision:
- `crates/bot` (with `crates/ai-shim`, the brief, the hands) is the **flagship**: Jev and the Claude player decide. No code
  decides a live game there except on the user's explicit word, as an audited layer entered in `docs/code-decisions.md`.
- `bots/*` are **heuristic bots**: code decides, no model, no tokens, each a library the engine loads whole
  (`bots/README.md`). The flagship's rule does not govern them, and nothing written for them goes into `crates/bot`.
Shared crates (`ai-host`, `bot-protocol`, `micro`, `combatsim`, `terrain`) serve both; a change there is a change to both.

Build: `cargo build --release`. Evaluate: `target/release/arena --matches 24 --parallel 8 --speed 50 --label <what-you-test>`.
Never touch `~/.local/share/BeyondAllReason` (the user's BAR install) except through `run/install_to_bar.sh`, and only when the user asks.
