# Haiku 5.5 in Jev's seat: rate and a one-exchange frame, offline (2026-10-07)

The user's question: what token rate the Anthropic API gives Claude Haiku 5.5 directly (no `claude -p`), and whether
one Haiku exchange a frame, with the stable context cached, can stand in for the pianist's sequence of Jev calls
(gate, split, pick). Replayed from `run/matches/1791335540-human-26/00/jev-0.jsonl` (262 frames with calls, 399 calls,
Jev median 5,112 input tokens and 192 ms a call); no game played. Scripts: `run/haiku_rate_probe.py` (the recorded
request as is, every noul answered as a probability), `run/haiku_one_pass.py` (one exchange a frame).

## 1. The rate, the request as Jev got it

| ask | fresh input | first token | output | out tok/s | wall | cost |
|---|---|---|---|---|---|---|
| median (11 questions) | 5,069 | 516 ms | 201 | 355 | 1,085 ms | $0.0006 |
| p90 (65 questions) | 18,128 | 614 ms | 1,180 | 370 | 3,824 ms | $0.0024 |

Effort low, thinking adaptive, the standing text (1,286 tokens) cached from the second call. Output streams at
355-370 tokens a second; the first token comes at about 450-700 ms whatever the prompt size (prefill is about 12 ms
per 1,000 tokens on top of a floor near 450 ms). A probability per option is the wrong output shape: the p90 ask spends
3.2 s of its 3.8 s emitting 1,180 tokens of numbers.

## 2. One exchange a frame

System block (cached): the standing text and the player's packet (2,000-2,200 tokens; the packet changed on 18 of
262 frames, each change a cache write). Turn: the picture as compact JSON, then per actor its `keep` words and its
options as the "The move: ..." words of the gate's nouls (the preamble that repeats the keep words per option
dropped). Answer: `{"<actor>": "keep" | "<option id>", "<actor>.why": "..."}`.

| arm, 8 frames at 6:42-6:50 | fresh input | cache read | first token | output | wall | cost a frame |
|---|---|---|---|---|---|---|
| effort low, thinking adaptive | 7,058 | 2,197 | 6,298 ms | 1,333 | 6,573 ms | $0.0015 |
| effort low, thinking off, with `why` | 5,955 | 2,228 | 564 ms | 124 | 990 ms | $0.0007 |
| the same, p90 frame (7 actors, 44 options) | 10,222 | 0 | 567 ms | 229 | 1,421 ms | $0.0014 |
| the same, largest frame (6 actors, 103 options) | 21,701 | 0 | 703 ms | 314 | 2,052 ms | $0.0025 |
| Jev, the same 8 frames (two calls each, in sequence) | 15,112 | | | | 402 ms | |

1. **Thinking is the cost, not the picture.** At effort low with thinking on, Haiku thinks for 1,200-1,500 tokens before
   a 60-token answer: 6.5 s a frame. With thinking off the same frame answers in about 1 s, the first token at 0.5 s.
2. **One exchange replaces the sequence.** One call a frame answers every actor at once in 1.0 s median (1.4 s at the
   p90 frame, 2.1 s at the largest), against Jev's 0.4 s for its two calls in sequence (0.53 s at the large frames).
   In lockstep that is a game minute of 60 asked seconds in about 60 s of wall against Jev's 24.
3. **The cache buys little here.** The picture changes every frame (`actors` on 261 of 261 consecutive frames, `enemy`
   257, `economy` 239), so only the standing text and the packet cache: 2,200 of about 8,000 tokens. At $0.10 per
   million the fresh 6,000 tokens are $0.0006; moving the unit descriptions and the spots into the cached prefix would
   save about a quarter of that and a few tens of milliseconds. The levers that matter are the output (thinking off)
   and the floor of about 450 ms a call.
4. **Cost a game is about Jev's.** At $0.0007 a frame, human-26's 262 frames come to about $0.18 for the seat; Jev's
   40 calls a minute at 15,000 tokens and $0.042 per million are about the same.
5. **The decisions read sensibly and are not Jev's.** Haiku's `why` strings quote code's facts ("we outweigh it",
   "within 600", "party_42 is 244 away, not on it") and the packet ("instructions say raid his constructors"). Of the
   8 frames it agreed with Jev's change on 2 (group_D at party_36 at 6:42; the commander at party_37 at 6:46 in one
   run), chose a different actor or target on 3, and kept everything on 2 where Jev D-gunned or hunted. Two runs of
   the same frame differ (6:46: the commander at party_37 in one, group_D to spot_26 in the other). Unmeasured against
   the game; a judgement would need a game or the review packets.

## What this does not say

No game was played; the options were the ones code enumerated for Jev, and Haiku was not asked the parties' and
places' nouls (the threat pre-pass) at all: it chose among the actors' options directly. Whether its picks hold up
over a game, and whether a `split` (the worlds' consequence words) is still needed, are the next questions.
