# Jev (TypeSafe AI) — notes for when we slot it in

Access granted 2026-09-19 (waitlist). Integrated 2026-09-21 as the pianist (`docs/design/2026-09-21-pianist.md`): `crates/jev` is
the client (`jev SCENARIO.json` reproduces the probe below), `crates/bot/src/brain/pianist/` the player of the keyboard.
Credentials come from the environment only — this repository is public.

What it is: a hosted "System One" model. A request carries a **state** (text or JSON) plus typed **questions**; every question is
answered in parallel, in isolation, in one pass, with probabilities. No text generation. Vendor figures: 70-500 ms end to end,
$0.042 per million input tokens, output free, options per Choice capped at 255, no image input.

Primitives: **Choice** (one of a defined set; distribution + confidence), **Noul** (probability that a condition holds; one per
label when several may apply; 0.5 means unsure, not "medium"), **Score** (position on described ordered levels; use comparable
per-item Scores for graded ranking).

Where the real documentation is: index `https://docs.typesafe.ai/llms.txt`; any page as Markdown by appending `.md`
(e.g. `/api.md`, `/primitives/choice.md`, `/confidence.md`, `/concepts/state.md`, `/patterns/fan-out.md`). Python and
JavaScript SDKs exist; from Rust we would use the HTTP API. A copy of TypeSafe's agent skill file (MIT) sits untracked at the
repo root as `JEV_SKILL.md`; it says to read the live docs before writing an integration, which has not been done yet.

Guidance from that file that shapes our design:
- Code owns the workflow; the model supplies judgments. Heuristics generate the legal options, Jev picks or scores them.
- Ask independent questions over the same state together, including speculative ones ("if we attack, where?") and consume
  only the branch that applies — one request per decision point, not a chain.
- Put named JSON fields in the state and reference them by path in the question; question IDs are not sent to the model.
- Always include a no-match option; the model cannot choose a candidate that was not offered.
- Confidence on Choice/Score is distribution concentration, not permission to act; thresholds must be tuned on our own data
  — the Opus game transcripts are the intended source of labelled decisions for that.
- Keep raw judgments reusable: score dimensions once, let code re-weight without re-asking.

## First contact (2026-09-19, `experiments/jev/`, model jev-1.13.0)
API facts confirmed by use: `POST https://api.typesafe.ai/v1/systemone`, `Authorization: Bearer $TYPESAFE_API_KEY`, body
`{model, state, questions:{id:{type: noul|choice|score, instructions, criteria}}}`; answers carry probabilities (and
`confidence` for choice/score). Limits from their docs: 64k context, 1,200 requests/min, 250k tokens/s, text only.
The key lives in `~/.config/within-reason/jev.env` (mode 600), never in this repository.
- **Latency from this machine: median 318-350 ms** for three questions over a ~650-730 token request (5 calls each,
  min 289, max 386). Cost per call about $0.00003.
- Three real decision points from the first Opus game, state rewritten qualitatively:
  | Scenario | Opus | Jev |
  |---|---|---|
  | 01 raiders kill extractors (4:55) | kept `defend`, wished for an escort order | `escort` 0.72 (defend 0.27); raid-is-main-threat 0.78 |
  | 02 launch or hold with 8 idle bots, enemy unscouted (9:39) | attacked mid-map; a 10-unit push hit the base 27 s later | `defend` 0.51, escort 0.37, **attack 0.03**, low confidence 0.34 |
  | 03 push arrives just after the army left (10:06) | recall and defend | `return_and_defend` 0.93; commander-in-danger 0.82; threat level "critical" 0.93 |
  Jev matched Opus where Opus was right, chose the order Opus had wished for in 01, and did not make Opus's mistake in 02.
  Three scenarios are an anecdote, not an evaluation.
- Sensitivity (scenario 02, only the scouting sentence changed): large-attack-likely-soon went 0.09 (enemy scouted tiny) /
  0.23 (unscouted) / 0.69 (scouted massing); `attack` rose to 0.30 and confidence fell to 0.19 when the enemy was tiny;
  `defend` firmed to 0.67 when it was massing. The derived question "would the base stay safe if the army left" barely moved
  (0.65 / 0.60 / 0.53) — consistent with their warning about multi-hop questions: ask about the facts, combine in code.
- Repeated identical requests differ slightly (escort 0.37 vs 0.31), so answers are not bit-stable; thresholds need margin.
- Their documented weaknesses (arithmetic, counting, comparing quantities, dates, multi-hop, distraction by unrelated state)
  mean the heuristics must hand Jev digested judgments ("metal income is very low", "a large group"), not raw numbers.

## The client (2026-09-21, `crates/jev`)
`jev::Client::from_env()` (the key file or `TYPESAFE_API_KEY`; the model `jev-latest` unless `TYPESAFE_DEFAULT_MODEL`),
`ask(&Request { state, questions })` blocking over `ureq` with rustls, 20 s timeout, two retries on 429 or 5xx waiting
`retry-after` (5 s at most), the versioned model and the usage in every `Response`. The key is never written by the crate.
- The three probe scenarios again (`cargo run -p jev -- experiments/jev/0*.json --repeat 3`, model jev-1.13.0): latency
  **median 145-174 ms** (min 127, max 339) against 318-350 two days earlier, on the same request sizes (630-730 tokens).
- **The same state and model do not give the same answers two days apart.** Scenario 01: `escort` 0.72 on 2026-09-19,
  `defend` 0.58 / `escort` 0.39 on 2026-09-21 (the top choice flipped). Scenario 02: `attack` 0.03 then 0.05,
  `sending_army_out_is_safe` 0.71 (was not asked then). Scenario 03: `return_and_defend` 0.93 then 0.96. Between calls
  minutes apart the wobble was a few hundredths (2026-09-19). Whatever the cause (the vendor's serving, or a wider
  spread than the first day showed), a decision layer needs hysteresis: the pianist holds a busy actor's course unless
  the winner beats "continue" by 0.15 (H-HANDS-SWITCH).

## The live docs, read again (2026-09-23, before the prompt rework after wake-1)
`docs.typesafe.ai` (`llms.txt` index; `concepts/state.md`, `how-to-build-with-system-one.md`, `primitives/choice.md`,
`primitives/advanced.md`, `confidence.md`, `patterns/fan-out.md`, `model-jaggedness/jev-1.13.md`, `models.md`). The model
is still jev-1.13.0 (`jev-latest` = `jev-preview`). What applies to the hands, and what was done with it:
- **Context: 64k per request, of which 32k for the state plus the longest question.** Stricter than the 64k we had
  noted; wake-1's largest call was 28k tokens. The question budget stays at 60k characters (lean); the state is what the
  cuts below shrink.
- **"Accuracy falls as the state grows with content unrelated to the decision"; "include only the context relevant to
  the current questions"; "omit historical information".** The five cuts: a group's `where` options carry names only
  (the place entry is the description), unasked actors are one line each, the never-looked and long-unseen spot lists
  are three names (the scout's target is chosen in code anyway, and every name kept its place entry in the block),
  building completions are no longer in `recent`, and the engage option no longer repeats the parties (they are under
  `enemy.in_sight`, and the odds against the group are now on its own `enemies_near` line).
- **Literal reading: "answers the question you wrote, not the one you meant ... when wrong answers require explanation,
  that explanation belongs in the prompt".** The fall-back precedence is said in the group's `do` question itself and
  in the rules, with its boundary (a few losses against a party we outweigh are the cost of fighting). The first
  wording without the boundary flipped 18 of 21 replayed moments to fall back, fights we were winning included
  (K-hands-precedence-wording).
- **Numbers: "pass computed numbers or semantic categories instead of raw numeric values".** The `losses` line carries
  the share in words (a few / a noticeable share / a large share: it is losing this fight / most of it: it is being
  wiped out) beside the count and metal, as the odds lines already did.
- **Contradictory instructions and criteria degrade answers; treat criteria as an extension of the instruction.** The
  engage option's words no longer restate the parties with their odds while the instruction says the odds are on the
  group's line. Structured criteria (`what` / `not_for` objects) for the confused group options (continue, engage,
  fall_back, hold) are the documented next step when options are still confused; not done.
- **Question IDs are not sent to the model**: every question names its actor in the text (they do).
- **Reference nested state by backtick path** (`actors.group_C`, `enemy.in_sight`, `player`): done in the group
  questions; the `where` question now says the places are described under `places`.
- **Confidence thresholds are to be tuned on our own data**, and answers wobble by a few hundredths between calls: the
  A/B script repeats each request three times (`run/jev_inflight_ab.py`).

**2026-09-27, the two calls as fast as they can be made (the user).** Measured in onepass-player-8: the gate 278 ms
median (p90 316) over ~25 KB of state and 50 questions, the pick 238 ms (p90 274) over the same state resent with one
Choice; `places` was 10 KB of it. Now the pick's state is cut to what the worlds name (`plan::pick_state`: actors no
world sends are one line, places nothing names are dropped), and in realtime the worker composes the worlds and makes
the pick the instant the gate answers instead of waiting for the next think, and answers are polled every tick (3
frames) rather than every think (15). In lockstep the calls take no game time; the saving there is wall time.
