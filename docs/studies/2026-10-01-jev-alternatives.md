# What could replace or complement Jev (market check, 2026-10-01)

Desk research only: web search and reading documentation pages on 2026-10-01. No call was made to any service, nothing
was installed or signed up for, no credential was read. Source kinds are marked **[vendor]** (the vendor's own docs,
posts or model pages), **[3rd-party measured]** (someone else ran it), **[commentary]** (articles, aggregators, blogs:
often SEO pages that paraphrase each other), **[inferred]** (my own arithmetic or reasoning). Prices and latencies are as
read on 2026-10-01. Several fetches were refused (openai.com 403, x.com 402, openrouter.ai model pages 404 to the
fetcher); where a fact rests only on a search-result snippet of such a page, it says so.

The space changed under our feet: TypeSafe released Jev on 2026-09-15 and opened the API on 09-21; within two weeks
there are at least six hosted "System One" decision endpoints, several of them speaking Jev's own `/v1/systemone`
wire format, plus a crowd of open-weight reproductions and two community benchmarks. The prior "the properties we rely
on will be hard to find elsewhere" was true a fortnight ago and is no longer true *on paper*. What nobody has shown
yet is property 4 (sub-second at our 25k-65k-token calls) and our property 5 (reading our prose).

**Checked by the main session, 2026-10-01** (the report below is a subagent's, on Opus; these lines are what was read
again and what it got wrong):
- Read again at the source and as written: TypeSafe's price, limits and rate limit (docs.typesafe.ai/models.md); Liquid's
  endpoint and `d1:free` with noul/choice/score and `output_tokens` 0, and no price, context, rate limit, latency or
  calibration on Liquid's own page; $0.04 and 65,536 on Vercel's page; Solar Decide at $0.05 (half of $0.10) with a
  524,288-token context on OpenRouter's listing (through search, as for the subagent); OpenAI's developer docs index has
  no Decisions entry; `K-jev-answers-drift` and the `score` type in the repo. Not read again: everything in sections 3
  and 4, the benchmarks, the smaller vendors, and whether the OpenAI endpoint returns a number per option.
- **The game costs below are computed on player-29-hard (120.7M tokens), a game from before the gate asked a busy actor
  only on its own news (f9070f2).** The games since are 47M to 74M tokens: Jev $1.98 to $3.11, so d1 about $1.9 to $3.0
  and Solar Decide about $4.7 to $7.4 at list ($2.4 to $3.7 at the promotion), tokenizers taken as equal.
- **"A late realtime game sits at or past the new token limit" is too strong on today's load.** player-32-hard per game
  second: median 35k tokens, p90 69k, 40 of its 1,890 gate seconds over 100k (the largest 134k), at most 5 requests in a
  second. In lockstep the arena waits for Jev and is under the limit by construction; in a realtime game about 2% of
  seconds would pass it.

## 0. What we rely on, checked against the repo (corrections to the brief)

Read: `crates/jev/src/lib.rs`, `run/jev_load.py`, `run/jev_ab.py`, `docs/harness/jev.md`,
`docs/studies/2026-09-30-jev-load.md`, the `K-jev-*` headings of `docs/knowledge/jev.md`.

1. **Typed answers with numbers.** Right, and there is a third type: `score` (a position on 2-10 described levels, with
   a distribution). A `choice` carries `probabilities` over every option plus `confidence`; a `noul` is one number.
   Two corrections:
   - *Stable on a replay* is not what we have. `K-jev-answers-drift`: minutes apart the answers wobble by a few
     hundredths; two days apart on the same `jev-1.13.0` the top choice of scenario 01 flipped (escort 0.72 → defend
     0.58). H-HANDS-SWITCH's 0.15 margin exists because of it.
   - *Calibrated* is our hope, not TypeSafe's claim. Their `confidence` page defines confidence as a statistic of the
     distribution's shape and makes no calibration claim (https://docs.typesafe.ai/confidence.md, [vendor]). Marketing
     around Jev says "calibrated"; the docs do not. The one outside measurement: JevBench v1.5.4 scores Jev 1.13.0's
     calibration 88.0 of 100 (https://benchmarkheaven.com/jev-models, [3rd-party measured]).
2. **Many questions over one state, billed once for the state.** Right per request. When a gate passes
   `REQUEST_CHARS` (150,000 characters) it goes as batches each carrying the whole state, so the state is billed once
   per batch. The limit is "64k tokens per request; 32k tokens for `state` plus the longest question"
   (https://docs.typesafe.ai/models.md, [vendor]).
3. **Price.** $0.042 per million input tokens, output free (https://docs.typesafe.ai/models.md, [vendor]).
   player-29-hard: 120.7M tokens, $5.07, of which gates 88.4M (questions 79% of their characters, the state 19%) and
   picks 32.3M (load study §1).
4. **Latency.** The 0.6 s for 65,000 tokens is a single observation in the client's comment. The fitted one-batch gate
   is 151 ms + 2.47 ms per 1,000 characters (n = 799; ≈ 0.5 s at 150,000 characters); a three-batch late gate sent at
   once took 683 ms median (load study §1, §4). JevBench measured Jev's median at 0.62 s on its own set
   ([3rd-party measured]).
5. **Zero-shot reading of prose instructions against the picture.** Right, with the documented limits (no counting
   against a threshold in prose, `K-jev-words-not-numbers`; literal reading).

One thing the brief did not list and the trial should know: **Jev's published rate limit has changed** to
"100K tokens per second / 40 requests per second ... adjusting dynamically" (https://docs.typesafe.ai/models.md,
[vendor]); our notes of 2026-09-19 say 250k tokens/s and 1,200 requests/min. A late realtime game sends a
~110k-token gate each second plus the picks ([inferred] from load study §2), so it sits at or past the new token limit.
A second provider is worth having for that reason alone.

## 1. OpenAI "Decisions API" (the user's name is the real name)

- **What it is.** "Decisions API", announced at OpenAI DevDay on 2026-09-29, limited preview. OpenAI's words:
  "Give your app real-time decision-making with Decisions API, powered by GPT-6 Luna. Define questions and possible
  answers to classify content, route requests, or choose an agent's next action. Available in limited preview."
  (@OpenAIDevs, https://x.com/OpenAIDevs/status/2105003318917697873, [vendor]; text from the search index, the page
  itself refused the fetch). The DevDay recap (https://openai.com/index/devday-2026-recap/, [vendor]; 403 to my fetch,
  quoted in search snippets and in https://modelsystem.one/runtimes/openai-decisions-api/): "focusing Luna's
  intelligence on a specific set of user-defined questions with finite pre-defined answers. Developers supply context
  using text or images, and get back answers in about 150ms." OpenAI's community announcement thread: "Decisions API
  is in limited preview. It uses Luna to classify inputs, route requests, or choose an action from predefined
  answers." (https://community.openai.com/t/devday-2026-announcements-and-developer-resources/1402006, [vendor]).
- **No public documentation found.** The OpenAI developer docs index (https://developers.openai.com/api/docs/llms.txt)
  has no Decisions page; `/api/docs/guides/decisions.md` is 404; the GPT-6 Luna model page lists Chat Completions,
  Responses and Batch only. Every third-party write-up I read says the same: no public schema, endpoint, price or rate
  limits (e.g. https://alphasignal.ai/news/openai-s-decisions-api-gives-developers-a-constrained-gpt-6-luna-router,
  https://smartscope.blog/en/blog/openai-decisions-api-competitor-jev-similarities-constrained-choice-2026/,
  [commentary]).
- **Against the properties.**
  1. Probabilities: **unconfirmed.** Some pages say "a probability for every option"; I could not trace that to
     OpenAI. The alphasignal piece reads OpenAI as returning "one of those answers"; Latent Space's roundup says the
     implementation "lacks calibration or RLCD" (https://www.latent.space/p/ainews-openai-devday-2026-dots-61,
     [commentary]); explainx quotes OpenAI's framing as "a confidence signal"
     (https://www.explainx.ai/blog/openai-decisions-api-gpt-6-luna-devday-2026, [commentary]). Nothing confirms a
     yes/no probability (our `noul`) as a primitive.
  2. Many questions over one context: **unconfirmed.** The OpenAI post says "questions" in the plural; no source
     shows a request shape with a map of questions over a shared context, or how the context is billed.
  3. Price: **unpublished** (all sources agree). GPT-6 Luna's ordinary price is $0.10 input / $0.01 cached input /
     $0.50 output per million (https://developers.openai.com/api/docs/models/gpt-6-luna.md, [vendor]); commentators
     stress that this is not the Decisions rate card.
  4. Latency: OpenAI says about 150 ms against 1.6 s for a standard Luna call (https://the-decoder.com/openai-expands-codex-and-its-api-at-devday-with-security-scans-a-decisions-api-and-ultrafast/,
     [commentary] reporting the keynote); Thibault Sottiaux (OpenAI): "less than a few hundreds of milliseconds end to
     end" (quoted by modelsystem.one, [commentary]). Request size behind those figures unknown. Context limit unknown
     (Luna itself: 1,050,000 tokens).
  5. Luna is a general LLM, so prose reading is plausible; untested.
- **Access.** Limited preview "for selected API customers", broader availability promised "in the coming days"
  (as above). It is an **API product**: nothing anywhere ties it to ChatGPT plans. OpenAI's Codex docs: signing in
  with ChatGPT bills Codex usage to the plan; "OpenAI bills API key usage through your OpenAI Platform account at
  standard API rates" (https://learn.chatgpt.com/docs/auth.md, [vendor]). A ChatGPT subscription and Codex CLI login do
  not give Platform API access or credits (OpenAI help article
  https://help.openai.com/en/articles/9039756-managing-billing-for-chatgpt-and-the-api-platform, 403 to my fetch;
  the same statement in every third-party summary, [commentary]). [inferred] So the user needs a Platform account with
  billing and must be admitted to the preview; the Codex login does not help.
- **A minimal test call.** Cannot be written honestly: there is no published endpoint path or body. Any example on the
  web is invented (several SEO sites, including look-alike domains such as `decisionsapi.pro`, `jevtypesafeai.com`,
  `thejevai.com`, present code; none cites OpenAI docs; I did not use them). The test, once admitted: take the
  schema from OpenAI's preview docs, send one recorded gate (`jev-0.jsonl`) with ~10 nouls and 2 choices, check whether
  the answer carries a number per option and per yes/no, and the `usage` billing of the context.
- **Disqualifiers today.** No access, no docs, no price. Not disqualified in principle.

## 2. Hosted services that speak Jev's own format

These accept `{model, state, questions:{id:{type: noul|choice|score, instructions, criteria}}}` and answer
`noul` / `choice` + `probabilities` + `confidence` / `score`: the change on our side would be the URL, the key and
the model name.

### Liquid AI `d1` (closest drop-in)
- Liquid AI, released 2026-09-29. Endpoint `https://api.liquid.ai/decisions/v1/systemone`, model `d1:free`, "TypeSafe
  compatible", noul/choice/score, `usage.output_tokens` always 0
  (https://docs.liquid.ai/lfm/models/decision-models, https://docs.liquid.ai/guides/decision-model-guide, [vendor]).
- Price $0.04 per million input, context 65,536 tokens (Vercel AI Gateway model page,
  https://vercel.com/ai-gateway/models/d1, [commentary/reseller]; Liquid's own docs give neither, and MarkTechPost says
  paid rates and the context are unpublished:
  https://www.marktechpost.com/2026/09/29/liquid-ai-releases-d1-a-decision-model-that-returns-calibrated-probabilities-with-zero-output-tokens/,
  [commentary]). No open weights.
- Quality: Liquid reports d1 58.9 against Jev 1.13 57.9 on the community "Decision Index 0.2.1" (relayed in
  https://github.com/baobab-tech/decision-models-experiments, [vendor claim relayed]); not on JevBench yet.
  Calibration: claimed "calibrated", no number published.
- Latency, rate limits, free-tier terms (`:free` naming, data use): **not published.**
- Our game [inferred]: same tokenizer assumption, 120.7M × $0.04 = **$4.83** (Jev $5.07). Same 64k-class context, so
  our batching stays as is.

### Upstage `Solar Decide`
- Upstage, beta on its API since 2026-09-22, also on OpenRouter's Decisions endpoint. "Same /v1/systemone schema as
  Jev", built on Solar Mini 4, **512K-token context**, output free; $0.10 per million input list, $0.05 during a
  "50% off for a limited time" promotion; a cache-read rate is listed at the same $0.05
  (https://openrouter.ai/upstage/solar-decide, [vendor listing], read through search snippets: the page 404s to the
  fetcher; https://systemonemodels.org/, [commentary]).
- One outside measurement (2026-09-28/29, small IE benchmark, 48 questions): 95.8% against cloud Jev's 93.8%,
  0.308 s per question on its "mixed pool", ~8,700 input tokens per batched request from a verbose tokenizer
  (https://github.com/umstek/zero-shot-ie-bench/pull/17, [3rd-party measured]). Tiny sample; not our task.
- Our game [inferred]: $0.10 × 120.7M = **$12.07** list, **$6.04** at the promotion, before the tokenizer penalty
  (if Solar's tokenizer counts ~20-30% more tokens on our English JSON, $14-16 list). The 512K context removes our
  batching: one request per gate, the state billed once instead of once per batch (saves a few percent of gate
  tokens; the load study's late gates send 2-3 batches).
- Latency at 50k-110k tokens: **unknown.** The only figure is per question on short inputs.

### Inception `Mercury Decide`
- Inception (diffusion LLMs), on OpenRouter's Decisions endpoint, released 2026-09-30, **free**, **32,768-token
  context** (https://openrouter.ai/inception/mercury-decide:free, [vendor listing via search snippet];
  https://alphasignal.ai/news/inception-releases-mercury-decide-to-make-ai-routing-14x-faster, [commentary]).
- Disqualifier for us: 32k total is half of Jev's; our late gates would go as 4-6 batches, and a free endpoint will
  be rate limited (terms unread). A complement for small calls (picks) at most.

### meraGPT `Decider 1`, Respan `Span-01`, Together `Tev1` (out)
- Decider 1: $0.03 per million, **4,096-token request** (https://systemonemodels.org/, https://meragpt.com/,
  [commentary/vendor]). Disqualified: our smallest states are 5k tokens.
- Span-01: $0.02 per million, free Lite, released 2026-09-24; answers *present / absent / not observable* per
  behaviour definition over AI traces (https://www.respan.ai/blog/introducing-span-1, [vendor]). No choice
  primitive, built for trace monitoring: does not fit our menus.
- Tev1 (Together AI): a fine-tune of Qwen3.5-0.8B / 4B that returns **one option letter**, not a distribution
  (https://huggingface.co/togethercomputer/Tev1-0.8B-experimental, [vendor]; Together's hosted price listed at $0.042).
  Disqualified on property 1.

### Jev itself through OpenRouter (a complement, not a competitor)
- OpenRouter serves `typesafe/jev-1.13` at the same $0.042 through `POST https://openrouter.ai/api/v1/systemone`
  (TypeSafe-compatible) or `POST /api/alpha/decisions`
  (https://openrouter.ai/docs/api/api-reference/alphadecisions/submit-a-decisions-questions-and-answers-request,
  https://openrouter.ai/blog/insights/what-is-jev/, [vendor]). The blog gives Jev a 32,000-token context there
  (vs TypeSafe's 64k) and the endpoint is alpha; one source says ~15% of calls hang on a read timeout (search snippet,
  source page not identified: treat as unverified). Useful as a fallback route when TypeSafe rate-limits, and the
  same alpha endpoint also routes Solar Decide and Mercury Decide, so one key reaches three of the candidates.

## 3. Open weights, self-hosted

- An open "Jev-class" ecosystem appeared in two weeks: JevBench v1.5.4 ranks 106 systems
  (https://benchmarkheaven.com/jev-models, [3rd-party measured]). Top rows (score / intelligence / calibration /
  median latency on their set): Jev 1.13.0 80.0 / 72.0 / 88.0 / 0.62 s; Winnow-12B Q8 (open, Gemma-4-12B based)
  79.3 / 74.4 / 84.1 / 0.34 s; Cygnet (open) 79.0 / 71.1 / 87.0 / 0.23 s; the many 4B fine-tunes 53-56 intelligence.
  The Decision Index (https://huggingface.co/spaces/multimodalart/jev-decision-index, [3rd-party measured]) puts
  decider-35b-a3b best calibrated (ECE 3.1). Both benchmarks use short states; none tests 50k-token pictures.
- Laya (Convai, Apache-2.0, ModernBERT encoder): 512-token context (1,024 multilingual, extendable to 8,192)
  (https://huggingface.co/convaiinnovations/laya, [vendor]). Disqualified by context.
- Ollama 0.35 serves a local `/v1/systemone` endpoint with Nimble 9B and Tev1 (2026-09-29;
  https://ollama.com/blog/ollama-now-supports-jev-style-decision-models, [vendor]; its 91 ms is per decision on a
  MacBook M5 Max, on an 841-token input).
- On this machine [inferred]: the GPUs are an AMD Navi 21 (RX 6800/6900 class, 16 GB) and a Radeon VII (16 GB); no
  CUDA. Our cost is prefill: a gate is 25k-110k tokens, 79% of them question text that a shared-prefix cache cannot
  skip. A 4-12B model prefills at a few thousand tokens a second on these cards, so a late gate takes several seconds
  to tens of seconds. Fails property 4 locally. On a rented datacentre GPU (H100-class, ~$2-3 an hour) a 12B model
  could plausibly prefill a 55k-token gate in about a second and a 40-minute game would cost ~$2 of GPU time, but that
  is an estimate, not a measurement, and it buys the operations work.

## 4. Do-it-yourself on a general LLM (logprobs of a constrained answer token)

The route: one request per question, the state as a cached prefix, the answer forced to one token, read the token's
logprob as the probability.

- **Availability of logprobs on today's fast models is poor.** GPT-6 Luna: the docs say to remove `top_logprobs` (and
  `logprobs` on Chat Completions) "when reasoning effort is not `none`"; Luna supports `none`
  (https://developers.openai.com/api/docs/guides/latest-model.md, [vendor]), so [inferred] logprobs may work at effort
  `none`, unconfirmed. Gemini 3.x: logprobs reported deprecated/disabled
  (https://discuss.ai.google.dev/t/were-logprobs-disabled-for-gemini-3-3-1-in-vertex-api/132426, [commentary]).
  Groq: no logprobs ([commentary] via its compatibility notes). `top_logprobs` caps at 20, below our 22-41 group options.
- **The billing shape is the disqualifier.** An LLM bills the state per question (at the cache rate at best) or
  generates text. Our gates had ~470,700 option questions in player-29 (load study §3) over states of roughly 7k
  tokens per batch [inferred from §1-2]. On Luna: 470,700 × 7k = 3.3B cached tokens × $0.01 = $33, the question text
  69.8M × $0.10 = $7, picks 32.3M × $0.10 = $3, first writes and one output token each ~$1-2: **≈ $45 a game, about 9×
  Jev** [inferred]. Answering all questions in one call as JSON instead means ~2,500 output tokens a gate, seconds of
  decoding.
- **Rate and latency.** ~475k requests in a 40-minute game is ~200 a second sustained, 12,000 RPM (Luna's tier 5 is
  30,000 RPM; tier 1 is 500). A standard Luna call is ~1.6 s per OpenAI's own comparison; the cache must be written
  before the fan-out hits it, so two round trips a gate. Fails property 4 [inferred].
- **Calibration**: raw token logprobs of instruction-tuned models are not calibrated by any vendor's claim.
- **Rerankers / cross-encoders** (e.g. Cohere Rerank 4: 32,768 tokens per query+document, $0.002-0.0025 per search of
  100 documents, documents chunked at 500 tokens for billing; https://openrouter.ai/cohere/rerank-4-pro, [vendor
  listing via snippet]) return relevance scores, not probabilities of a stated condition, and have no prose-question
  semantics. Not a fit.
- **Anthropic / Google**: I found no decision-style endpoint from either as of 2026-10-01. Whether the Claude API
  exposes logprobs I could not confirm from a primary source.

## 5. Comparison

Per-game cost uses player-29-hard's 120.7M input tokens and assumes the same token count (tokenizers differ).

| service | typed probs (1) | many Qs, state once, output free (2) | price /M in (3) | game cost | latency at our size (4) | context | access | verdict |
|---|---|---|---|---|---|---|---|---|
| **Jev 1.13** (TypeSafe) | yes; not bit-stable; no vendor calibration claim; JevBench cal. 88 | yes (state per batch) | $0.042 | $5.07 | 0.15-0.7 s measured by us | 64k (32k state + longest Q) | API key | baseline |
| **Liquid d1** | yes, same format; "calibrated" unquantified | yes; output 0 | $0.04 (reseller page) | ~$4.83 | unpublished | 65,536 (reseller page) | API key (`d1:free`) | **trial** |
| **Upstage Solar Decide** | yes, same format; "calibrated" unquantified | yes; output free | $0.10 ($0.05 promo) | ~$12 ($6 promo) | unpublished at size; 0.31 s/question small | 512K | Upstage key or OpenRouter | **trial** |
| **OpenAI Decisions API** | unconfirmed (pick + "confidence"?) | unconfirmed | unpublished (Luna list $0.10/$0.50) | unknown | ~150 ms claimed, size unknown | unknown | Platform API key + preview admission; not ChatGPT | wait for docs |
| Mercury Decide | yes (claimed) | yes | free | $0 | unpublished | 32,768 | OpenRouter | fails (2)-context; picks only |
| Decider 1 | yes (claimed) | yes (20 Qs/pass claimed) | $0.03 | — | — | 4,096 | meraGPT | fails context |
| Span-01 | tri-state per behaviour | yes | $0.02 | — | — | ? | Respan | fails (1): no choice |
| Tev1 | one letter only | ? | $0.042 | — | — | ? | Together / Ollama | fails (1) |
| Open Jev-class (Winnow-12B, Cygnet, ...) | yes; JevBench cal. 84-87 | yes via local `/systemone` | GPU time | ~$2 rented [inferred] | seconds-tens of s on our AMD cards [inferred] | base-model dependent | self-host | fails (4) here |
| GPT-6 Luna + logprobs | token logprobs, if at effort `none`; ≤20 options | no: state billed per question | $0.10 / $0.01 cached | ~$45 [inferred] | ≥1.6 s a round, 2 rounds | 1.05M | API key | fails (2)(3)(4) |
| Rerankers | relevance scores | no | per search | — | — | 32k | API key | fails (1)(5) |

## 6. Verdict on the prior

Partly wrong. As of 2026-10-01 two hosted services **claim** properties 1-3 together: Liquid's d1 (same format,
same price, same context as Jev) and Upstage's Solar Decide (same format, 2.4× the list price, eight times the
context). Nobody has shown property 4 at our request size for either, and nobody has shown property 5 on anything
like our pictures; their "calibrated" is a word, not a measurement. So: nothing *demonstrably* matches Jev on 1-4
together today, but the reason is missing evidence, not missing products. The failures by candidate: OpenAI Decisions
fails on evidence (1, 2, 3 unpublished); Mercury and Decider 1 on context; Span-01 and Tev1 on (1); self-hosting on
(4) on this machine; the DIY LLM route on (2)/(3)/(4), structurally; rerankers on (1)/(5). The OpenAI offering exists
and is the user's name for it, but is a preview without public docs; a ChatGPT subscription does not reach it.

## 7. Trials worth running

Both speak `/v1/systemone`, so `run/jev_ab.py`'s `ask()` with a different URL, key variable and model is the whole
client change (that is a code change; not done here).

1. **Liquid d1** (drop-in, same price): replay ~200 recorded gate calls from `jev-0.jsonl` spread over minutes 4-30
   (including late two- and three-batch gates) and ~200 picks.
2. **Upstage Solar Decide** (big context, small premium), the same replay, sending each late gate as one request.

Measure, per service against the recorded Jev answers:
- latency median / p90 by request size (one-batch gates against the 151 ms + 2.47 ms/1,000 chars fit), errors,
  429s, timeouts;
- `usage.input_tokens` per call against Jev's for the same body (tokenizer cost), so the game cost is exact;
- noul agreement: share crossing 0.5 and 0.7 on the same side as Jev, mean absolute difference; choice: top-pick
  agreement and total-variation distance of the distributions;
- repeat stability: each request three times (as `run/jev_inflight_ab.py` does for Jev), wobble against Jev's few
  hundredths;
- the moments with a known right answer from the user's reviews (the jev_ab moments: shelling sentence, fight-in-sight,
  the hunt forbids): which service gets them right. Agreement with Jev is not correctness; these are.

Do the OpenAI Decisions API once its docs are public and the user is admitted: first check that it returns a number
per option and per yes/no, then the same replay.
