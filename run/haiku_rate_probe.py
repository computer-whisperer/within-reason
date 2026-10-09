#!/usr/bin/env python3
"""What token rate Claude Haiku 5.5 gives on Jev-shaped asks, over the API directly (no SDK, no `claude -p`).

Replays recorded Jev calls (a `jev-<n>.jsonl` beside a match record) as one streamed Messages request each: the
standing text as a cached system block, the picture and the questions as the user turn, one JSON object back with an
answer per question. Reports, per call: input tokens (and what came from the cache), time to first output token,
output tokens, output tokens a second, wall time, cost; then the medians. `--parallel N` sends N calls at once and
reports the aggregate rate too, since the pianist asks for many actors in one game second.

    run/haiku_rate_probe.py run/matches/<match>/00/jev-0.jsonl            # 5 calls at the median size, effort low
    run/haiku_rate_probe.py JEV_LOG --sizes median,p90,max --calls 3
    run/haiku_rate_probe.py JEV_LOG --parallel 8 --effort low --no-thinking
    run/haiku_rate_probe.py JEV_LOG --model claude-sonnet-5-5             # the same asks on another model

The key: `ANTHROPIC_API_KEY` in the environment, else `~/.config/within-reason/anthropic.env` (mode 600). Never
printed, never written: the repository is public.
"""
import jevlog
import argparse, concurrent.futures, json, os, statistics, sys, time, urllib.error, urllib.request

URL = "https://api.anthropic.com/v1/messages"
KEY_FILE = os.path.expanduser("~/.config/within-reason/anthropic.env")
DEFAULT_MODEL = "claude-haiku-5-5"
# USD per million tokens, prompts up to 100K tokens (docs.anthropic.com pricing, read 2026-10-07): input, cache read, output.
PRICES = {"claude-haiku-5-5": (0.10, 0.01, 0.50), "claude-sonnet-5-5": (2.00, 0.20, 10.00), "claude-opus-5-5": (4.00, 0.20, 20.00)}


def api_key():
    key = os.environ.get("ANTHROPIC_API_KEY", "").strip()
    if not key and os.path.exists(KEY_FILE):
        for line in open(KEY_FILE):
            if line.startswith("ANTHROPIC_API_KEY="):
                key = line.split("=", 1)[1].strip().strip("\"'")
    if not key:
        sys.exit(f"no API key: put ANTHROPIC_API_KEY=sk-ant-... in {KEY_FILE} (or export it)")
    return key


def load_calls(path):
    """The log's standing text and every call, each with the packet it was asked under (carried forward)."""
    rules, instructions, calls = "", "", []
    for line in jevlog.lines(path):
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            break  # a torn last line
        if d.get("t") == "header":
            rules = d.get("rules", "")
        elif d.get("t") == "call":
            rules = d.get("rules", rules)
            instructions = d.get("instructions", instructions)
            if d.get("usage"):
                calls.append({**d, "rules": rules, "instructions": instructions})
    if not calls:
        sys.exit(f"no calls with usage in {path}")
    return calls


def pick(calls, size):
    toks = sorted(c["usage"]["input_tokens"] for c in calls)
    target = {"median": toks[len(toks) // 2], "p90": toks[int(0.9 * (len(toks) - 1))], "max": toks[-1]}[size]
    return min(calls, key=lambda c: abs(c["usage"]["input_tokens"] - target))


def render(call):
    """One Jev request as a Messages request body: the standing text cached, the picture and questions in the turn."""
    system = [{"type": "text", "text": "You answer typed questions over a picture of a game, as the hands of a player. "
                                       "Answer with one JSON object and nothing else: for each question id, a noul is a "
                                       "probability of yes from 0 to 1, a choice is the name of one of its options, a score "
                                       "is one of its level numbers.\n\n" + call["rules"], "cache_control": {"type": "ephemeral"}}]
    state = dict(call["state"])
    if call["instructions"]:
        state["instructions"] = call["instructions"]
    user = "State:\n" + json.dumps(state, separators=(",", ":")) + "\n\nQuestions:\n" + json.dumps(call["questions"], separators=(",", ":"))
    return system, user


def stream(key, model, system, user, effort, thinking, max_tokens, schema=None):
    """One streamed Messages call; `schema` (a JSON schema) makes the answer structured output: JSON only, no prose."""
    body = {"model": model, "max_tokens": max_tokens, "stream": True, "system": system, "output_config": {"effort": effort},
            "messages": [{"role": "user", "content": user}]}  # `user`: a string, or content blocks (text, image)
    if schema is not None:
        body["output_config"]["format"] = {"type": "json_schema", "schema": schema}
    if not thinking:
        body["thinking"] = {"type": "disabled"}
    req = urllib.request.Request(URL, data=json.dumps(body).encode(), method="POST",
                                 headers={"x-api-key": key, "anthropic-version": "2023-06-01", "content-type": "application/json",
                                          "user-agent": "within-reason-haiku-probe"})
    out = {"model": model, "text": "", "first_token_s": None, "stop_reason": None, "stop_details": None, "usage": {}}
    started = time.monotonic()
    try:
        with urllib.request.urlopen(req, timeout=120) as r:
            for raw in r:
                line = raw.decode().strip()
                if not line.startswith("data:"):
                    continue
                event = json.loads(line[5:])
                kind = event.get("type")
                if kind == "message_start":
                    out["usage"].update(event["message"].get("usage", {}))
                elif kind == "content_block_delta":
                    delta = event["delta"]
                    if delta.get("type") == "text_delta":
                        if out["first_token_s"] is None:
                            out["first_token_s"] = time.monotonic() - started
                        out["text"] += delta["text"]
                elif kind == "message_delta":
                    out["usage"].update(event.get("usage", {}))
                    out["stop_reason"] = event["delta"].get("stop_reason")
                    out["stop_details"] = event["delta"].get("stop_details")
                elif kind == "error":
                    out["error"] = event.get("error")
    except urllib.error.HTTPError as e:
        out["error"] = f"HTTP {e.code}: {e.read().decode(errors='replace')[:500]}"
    out["wall_s"] = time.monotonic() - started
    return out


def cost(model, usage):
    p_in, p_cache, p_out = PRICES.get(model, PRICES[DEFAULT_MODEL])
    fresh = usage.get("input_tokens", 0) + usage.get("cache_creation_input_tokens", 0)
    return (fresh * p_in + usage.get("cache_read_input_tokens", 0) * p_cache + usage.get("output_tokens", 0) * p_out) / 1e6


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("jev_log")
    ap.add_argument("--model", default=DEFAULT_MODEL)
    ap.add_argument("--sizes", default="median", help="comma-separated: median, p90, max")
    ap.add_argument("--calls", type=int, default=5, help="calls per size (sequential unless --parallel)")
    ap.add_argument("--parallel", type=int, default=1, help="send this many calls at once per round")
    ap.add_argument("--effort", default="low", choices=["low", "medium", "high"])
    ap.add_argument("--no-thinking", action="store_true", help="send thinking: disabled (accepted at effort high or below)")
    ap.add_argument("--max-tokens", type=int, default=4096)
    ap.add_argument("--show", action="store_true", help="print the first answer's text")
    args = ap.parse_args()

    key = api_key()
    calls = load_calls(args.jev_log)
    toks = [c["usage"]["input_tokens"] for c in calls]
    print(f"{len(calls)} recorded Jev calls; Jev input tokens median {statistics.median(toks):.0f}, "
          f"Jev latency median {statistics.median(c['ms'] for c in calls):.0f} ms")
    print(f"model {args.model}, effort {args.effort}, thinking {'off' if args.no_thinking else 'adaptive'}, parallel {args.parallel}\n")

    for size in args.sizes.split(","):
        call = pick(calls, size)
        system, user = render(call)
        print(f"== {size}: Jev {call['usage']['input_tokens']} input tokens, {len(call['questions'])} questions, "
              f"Jev answered in {call['ms']} ms (frame {call['f']})")
        results = []
        rounds = max(1, args.calls // args.parallel)
        for _ in range(rounds):
            round_start = time.monotonic()
            with concurrent.futures.ThreadPoolExecutor(max_workers=args.parallel) as pool:
                batch = list(pool.map(lambda _: stream(key, args.model, system, user, args.effort, not args.no_thinking, args.max_tokens),
                                      range(args.parallel)))
            round_wall = time.monotonic() - round_start
            for r in batch:
                if r.get("error"):
                    print(f"  error: {r['error']}")
                    continue
                if r["stop_reason"] == "refusal":
                    print(f"  refusal: {r['stop_details']}")
                    continue
                u = r["usage"]
                out_tokens = u.get("output_tokens", 0)
                gen_s = r["wall_s"] - (r["first_token_s"] or r["wall_s"])
                r["out_rate"] = out_tokens / gen_s if gen_s > 0 and out_tokens else 0.0
                r["cost"] = cost(args.model, u)
                results.append(r)
                print(f"  in {u.get('input_tokens', 0):6d} (+{u.get('cache_creation_input_tokens', 0)} cached write, "
                      f"{u.get('cache_read_input_tokens', 0)} cached read)  first token {1000 * (r['first_token_s'] or 0):6.0f} ms  "
                      f"out {out_tokens:4d} tok  {r['out_rate']:6.1f} tok/s  wall {1000 * r['wall_s']:6.0f} ms  "
                      f"${r['cost']:.5f}  {r['stop_reason']}")
            if args.parallel > 1:
                total_out = sum(r["usage"].get("output_tokens", 0) for r in batch if not r.get("error"))
                total_in = sum(r["usage"].get("input_tokens", 0) + r["usage"].get("cache_read_input_tokens", 0) for r in batch if not r.get("error"))
                print(f"  round of {args.parallel}: wall {1000 * round_wall:.0f} ms, aggregate {total_out / round_wall:.0f} out tok/s, "
                      f"{total_in / round_wall:.0f} in tok/s")
        if results:
            med = lambda k: statistics.median(r[k] for r in results)
            print(f"  median: first token {1000 * med('first_token_s'):.0f} ms, wall {1000 * med('wall_s'):.0f} ms, "
                  f"{med('out_rate'):.1f} out tok/s, {statistics.median(r['usage'].get('output_tokens', 0) for r in results):.0f} out tokens, "
                  f"${med('cost'):.5f} a call ({statistics.median(r['usage'].get('input_tokens', 0) for r in results):.0f} fresh input tokens)")
            if args.show:
                print("  answer:", results[0]["text"][:1500])
        print()


if __name__ == "__main__":
    main()
