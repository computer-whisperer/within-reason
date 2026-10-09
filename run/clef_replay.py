#!/usr/bin/env python3
"""Replay a recorded game's Jev calls (`jev-<n>.jsonl`) through Cloudflare's Clef models and compare with Jev's
recorded answers: agreement, stability, latency by size, cost.

    run/clef_replay.py run/matches/<match>/00/jev-0.jsonl                    # every call, clef-flash and clef
    run/clef_replay.py JEV_LOG --models clef-flash --limit 50 --repeat 5      # the first 50 calls, every 5th sent twice
    run/clef_replay.py JEV_LOG --out results.jsonl

The request is what the hands sent Jev: the logged `state` with the packet (`instructions`) and the standing text
(`rules`) put back, and the questions; a call past 64 questions goes as batches of 64 (the hosted limit), their
latencies summed as the pianist would wait for them. Nouls are compared on the flag (0.5) and in value; choices on
the pick and on Clef's probability for Jev's pick. `--repeat K` sends every Kth call a second time for stability.

Key: `~/.config/within-reason/cloudflare.env` with CLOUDFLARE_ACCOUNT_ID and CLOUDFLARE_API_TOKEN (mode 600), or the
environment variables of the same names. Nothing prints or writes them.
"""
import jevlog
import argparse, collections, concurrent.futures, json, os, statistics, sys, time, urllib.error, urllib.request

KEY_FILE = os.path.expanduser("~/.config/within-reason/cloudflare.env")
# USD per million input tokens (developers.cloudflare.com/workers-ai/platform/pricing, 2026-10-07); output is free
PRICES = {"clef-flash": 0.09, "clef": 0.24}
MAX_QUESTIONS = 64


def credentials():
    env = {k: os.environ.get(k, "") for k in ("CLOUDFLARE_ACCOUNT_ID", "CLOUDFLARE_API_TOKEN")}
    if not all(env.values()) and os.path.exists(KEY_FILE):
        for line in open(KEY_FILE):
            if "=" in line and not line.startswith("#"):
                k, v = line.strip().split("=", 1)
                env.setdefault(k, "")
                if not env[k]:
                    env[k] = v.strip().strip("\"'")
    if not all(env.get(k) for k in ("CLOUDFLARE_ACCOUNT_ID", "CLOUDFLARE_API_TOKEN")):
        sys.exit(f"no Cloudflare credentials: put CLOUDFLARE_ACCOUNT_ID and CLOUDFLARE_API_TOKEN in {KEY_FILE}")
    return env["CLOUDFLARE_ACCOUNT_ID"], env["CLOUDFLARE_API_TOKEN"]


def clef_call(creds, model, state, questions, images=None, timeout=180, retries=3):
    """One request to @cf/cloudflare/<model>; returns (answers, usage, seconds, retries) or raises RuntimeError."""
    acct, tok = creds
    body = {"model": model, "state": state, "questions": questions}
    if images:
        body["images"] = images
    url = f"https://api.cloudflare.com/client/v4/accounts/{acct}/ai/run/@cf/cloudflare/{model}"
    data = json.dumps(body).encode()
    waited = 0
    for attempt in range(retries + 1):
        req = urllib.request.Request(url, data=data, method="POST", headers={"Authorization": f"Bearer {tok}", "Content-Type": "application/json", "User-Agent": "within-reason-clef"})
        t = time.monotonic()
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                out = json.load(r)
            res = out["result"]
            return res["answers"], res.get("usage", {}), time.monotonic() - t, waited
        except urllib.error.HTTPError as e:
            text = e.read().decode(errors="replace")
            if e.code in (429, 500, 502, 503, 504) and attempt < retries:
                time.sleep(min(10, 2 ** attempt))
                waited += 1
                continue
            raise RuntimeError(f"HTTP {e.code}: {text[:300]}")
        except (urllib.error.URLError, TimeoutError) as e:
            if attempt < retries:
                time.sleep(2)
                waited += 1
                continue
            raise RuntimeError(str(e))


def clef_ask(creds, model, state, questions, images=None):
    """A whole request, batched by MAX_QUESTIONS; latencies summed, usage summed, answers merged."""
    keys = list(questions)
    batches = [keys[i:i + MAX_QUESTIONS] for i in range(0, len(keys), MAX_QUESTIONS)] or [[]]
    answers, usage, seconds, retried = {}, collections.Counter(), 0.0, 0
    for batch in batches:
        a, u, s, w = clef_call(creds, model, state, {k: questions[k] for k in batch}, images)
        answers.update(a)
        usage.update({k: v for k, v in u.items() if isinstance(v, (int, float))})
        seconds += s
        retried += w
    return answers, dict(usage), seconds, len(batches), retried


def load_calls(path):
    calls, rules, instructions = [], "", ""
    for line in jevlog.lines(path):
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            break
        if d.get("t") == "header":
            rules = d.get("rules", "")
        elif d.get("t") == "call":
            rules = d.get("rules", rules)
            instructions = d.get("instructions", instructions)
            if d.get("usage") and d.get("answers"):
                state = dict(d["state"])
                state["instructions"] = instructions
                state["rules"] = rules
                calls.append({"f": d["f"], "ms": d["ms"], "jev_tokens": d["usage"]["input_tokens"], "state": state, "questions": d["questions"], "jev": d["answers"]})
    return calls


def compare(jev, clef):
    """Per-question comparisons: nouls (flag agreement, abs diff), choices (same pick, p of Jev's pick)."""
    out = {"noul_n": 0, "noul_agree": 0, "noul_diff": [], "choice_n": 0, "choice_same": 0, "choice_p_jev": []}
    for k, a in jev.items():
        b = clef.get(k)
        if not b:
            continue
        if a.get("type") == "noul" and "noul" in b:
            out["noul_n"] += 1
            out["noul_agree"] += int((a["noul"] >= 0.5) == (b["noul"] >= 0.5))
            out["noul_diff"].append(abs(a["noul"] - b["noul"]))
        elif a.get("type") == "choice" and "choice" in b:
            out["choice_n"] += 1
            out["choice_same"] += int(a["choice"] == b["choice"])
            out["choice_p_jev"].append(b.get("probabilities", {}).get(a["choice"], 0.0))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("jev_log")
    ap.add_argument("--models", default="clef-flash,clef")
    ap.add_argument("--limit", type=int, help="only the first N calls")
    ap.add_argument("--repeat", type=int, default=10, help="send every Kth call twice (0: never)")
    ap.add_argument("--parallel", type=int, default=4)
    ap.add_argument("--out", help="every call's record as JSON lines")
    args = ap.parse_args()
    creds = credentials()
    calls = load_calls(args.jev_log)
    if args.limit:
        calls = calls[:args.limit]
    print(f"{len(calls)} recorded calls, {sum(c['jev_tokens'] for c in calls) / 1e6:.2f}M Jev tokens, Jev median {statistics.median(c['ms'] for c in calls):.0f} ms\n")
    out = open(args.out, "w") if args.out else None

    for model in args.models.split(","):
        def run(job):
            i, c, rep = job
            try:
                answers, usage, seconds, batches, retried = clef_ask(creds, model, c["state"], c["questions"])
                return {"i": i, "f": c["f"], "repeat": rep, "answers": answers, "usage": usage, "seconds": seconds, "batches": batches, "retried": retried, "cmp": compare(c["jev"], answers)}
            except RuntimeError as e:
                return {"i": i, "f": c["f"], "repeat": rep, "error": str(e)}
        jobs = [(i, c, 0) for i, c in enumerate(calls)] + ([(i, c, 1) for i, c in enumerate(calls) if args.repeat and i % args.repeat == 0])
        t0 = time.monotonic()
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.parallel) as pool:
            results = list(pool.map(run, jobs))
        wall = time.monotonic() - t0
        if out:
            for r in results:
                out.write(json.dumps({"model": model, **r}) + "\n")
        ok = [r for r in results if "error" not in r]
        errors = [r for r in results if "error" in r]
        firsts = [r for r in ok if r["repeat"] == 0]
        tokens = sum(r["usage"].get("input_tokens", 0) for r in ok)
        print(f"== {model}: {len(ok)} calls ok, {len(errors)} failed, {tokens / 1e6:.2f}M tokens (${tokens / 1e6 * PRICES[model]:.2f}), "
              f"{wall:.0f} s wall at {args.parallel} at once, {sum(r['retried'] for r in ok)} retries")
        for e in errors[:3]:
            print("   error:", e["error"][:200])
        # latency by size
        buckets = collections.defaultdict(list)
        for r in firsts:
            n = r["usage"].get("input_tokens", 0)
            b = "<8k" if n < 8000 else "8-16k" if n < 16000 else "16-32k" if n < 32000 else ">=32k"
            buckets[b].append(r["seconds"] * 1000)
        jev_buckets = collections.defaultdict(list)
        for r, c in ((r, calls[r["i"]]) for r in firsts):
            n = r["usage"].get("input_tokens", 0)
            b = "<8k" if n < 8000 else "8-16k" if n < 16000 else "16-32k" if n < 32000 else ">=32k"
            jev_buckets[b].append(c["ms"])
        print("   latency (ms, median, whole request incl. batches)  clef / jev:")
        for b in ("<8k", "8-16k", "16-32k", ">=32k"):
            if buckets[b]:
                print(f"     {b:6s} n={len(buckets[b]):3d}  {statistics.median(buckets[b]):6.0f} / {statistics.median(jev_buckets[b]):4.0f}   p90 {sorted(buckets[b])[int(.9 * (len(buckets[b]) - 1))]:.0f}")
        # agreement
        nn = sum(r["cmp"]["noul_n"] for r in firsts); na = sum(r["cmp"]["noul_agree"] for r in firsts)
        nd = [d for r in firsts for d in r["cmp"]["noul_diff"]]
        cn = sum(r["cmp"]["choice_n"] for r in firsts); cs = sum(r["cmp"]["choice_same"] for r in firsts)
        cp = [p for r in firsts for p in r["cmp"]["choice_p_jev"]]
        print(f"   nouls: {nn} compared, same side of 0.5 on {na / nn if nn else 0:.2f}, median |diff| {statistics.median(nd) if nd else 0:.2f}, mean {statistics.mean(nd) if nd else 0:.2f}")
        # flagged-side detail: of the nouls Jev put over 0.5, how many did Clef
        jev_hi = [(r, k) for r in firsts for k, a in calls[r["i"]]["jev"].items() if a.get("type") == "noul" and a["noul"] >= 0.5 and k in r["answers"]]
        clef_hi_too = sum(1 for r, k in jev_hi if r["answers"][k].get("noul", 0) >= 0.5)
        clef_hi = [(r, k) for r in firsts for k, b in r["answers"].items() if "noul" in b and b["noul"] >= 0.5 and calls[r["i"]]["jev"].get(k, {}).get("type") == "noul"]
        jev_hi_too = sum(1 for r, k in clef_hi if calls[r["i"]]["jev"][k]["noul"] >= 0.5)
        print(f"   of Jev's {len(jev_hi)} nouls over 0.5, Clef over 0.5 on {clef_hi_too} ({clef_hi_too / len(jev_hi) if jev_hi else 0:.2f}); "
              f"of Clef's {len(clef_hi)} over 0.5, Jev over 0.5 on {jev_hi_too} ({jev_hi_too / len(clef_hi) if clef_hi else 0:.2f})")
        print(f"   choices: {cn} compared, same pick on {cs / cn if cn else 0:.2f}, Clef's median probability for Jev's pick {statistics.median(cp) if cp else 0:.2f}")
        # stability
        pairs = collections.defaultdict(dict)
        for r in ok:
            pairs[r["i"]][r["repeat"]] = r
        diffs, flips, picks, pick_flips = [], 0, 0, 0
        for i, pr in pairs.items():
            if 0 in pr and 1 in pr:
                for k, a in pr[0]["answers"].items():
                    b = pr[1]["answers"].get(k, {})
                    if "noul" in a and "noul" in b:
                        diffs.append(abs(a["noul"] - b["noul"])); flips += int((a["noul"] >= 0.5) != (b["noul"] >= 0.5))
                    elif "choice" in a and "choice" in b:
                        picks += 1; pick_flips += int(a["choice"] != b["choice"])
        if diffs or picks:
            print(f"   stability on {sum(1 for pr in pairs.values() if 1 in pr)} repeated calls: nouls median |diff| {statistics.median(diffs) if diffs else 0:.3f}, "
                  f"max {max(diffs) if diffs else 0:.2f}, {flips} of {len(diffs)} crossed 0.5; choices {pick_flips} of {picks} changed")
        print()


if __name__ == "__main__":
    main()
