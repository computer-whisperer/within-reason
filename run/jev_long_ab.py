#!/usr/bin/env python3
"""Replay a recorded game's Jev calls under the long-context header (`TypeSafe-Danger-Zone-Long-Context: true`,
262k tokens a request instead of 64k) and through the 1.14 preview, against the recorded answers (jev-1.13.0,
sent in batches of at most `REQUEST_CHARS` by `crates/jev`): agreement, the drift baseline, latency by size.

    run/jev_long_ab.py run/matches/<match>/00                  # seat 0: every 10th ordinary call, every batched call
    run/jev_long_ab.py LOG --every 20 --big 4 --variants latest,latest+long,preview,preview+long
    run/jev_long_ab.py LOG --out results.jsonl --parallel 4

A variant is a model alias (`latest` = jev-latest, `preview` = jev-preview, or a versioned id) with `+long` for the
header. Without the header a request past the client's budget goes as the client sends it: batches of whole
questions over the same state, at once, the wall time of the slowest counted (`crates/jev/src/lib.rs` `batches`).
With the header every request goes whole. The `latest` variant is sent twice: the second send is the drift
baseline (the vendor's answers move between calls), and every other variant is read against it as well as against
the recorded answers. Key: `~/.config/within-reason/jev.env` or TYPESAFE_API_KEY. Nothing prints or writes it.
"""
import argparse, collections, concurrent.futures, json, os, statistics, sys, time, urllib.error, urllib.request
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import jevlog
from clef_replay import load_calls, compare

URL = "https://api.typesafe.ai/v1/systemone"
PRICE = 0.042  # USD per million input tokens
REQUEST_CHARS = 150_000  # crates/jev REQUEST_CHARS
ALIASES = {"latest": "jev-latest", "preview": "jev-preview"}


def jev_key():
    key = os.environ.get("TYPESAFE_API_KEY", "")
    path = os.path.expanduser("~/.config/within-reason/jev.env")
    if not key and os.path.exists(path):
        for line in open(path):
            if line.startswith("TYPESAFE_API_KEY="):
                key = line.split("=", 1)[1].strip().strip("\"'")
    if not key:
        sys.exit("no TYPESAFE_API_KEY")
    return key


def compact(v):
    return json.dumps(v, separators=(",", ":"))


def client_batches(state, questions, budget=REQUEST_CHARS):
    """The client's split: batches of whole questions in the map's order, each with the whole state."""
    base = len(compact(state)) + 64
    if base + len(compact(questions)) <= budget:
        return [questions]
    out, current, used = [], {}, base
    for qid, q in questions.items():
        cost = len(qid) + 4 + len(compact(q))
        if current and used + cost > budget:
            out.append(current)
            current, used = {}, base
        current[qid] = q
        used += cost
    if current:
        out.append(current)
    return out


def send(key, model, state, questions, long, timeout=120, retries=3):
    """One HTTP request. Returns (answers, usage, model, seconds); raises RuntimeError with the status and body."""
    body = compact({"model": model, "state": state, "questions": questions}).encode()
    headers = {"Authorization": f"Bearer {key}", "Content-Type": "application/json", "User-Agent": "within-reason-jev-long-ab"}
    if long:
        headers["TypeSafe-Danger-Zone-Long-Context"] = "true"
    for attempt in range(retries + 1):
        req = urllib.request.Request(URL, data=body, method="POST", headers=headers)
        t = time.monotonic()
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                out = json.load(r)
            return out["answers"], out.get("usage", {}), out.get("model", ""), time.monotonic() - t
        except urllib.error.HTTPError as e:
            text = e.read().decode(errors="replace")
            if e.code in (429, 500, 502, 503, 529) and attempt < retries:
                time.sleep(min(10, 2 ** attempt))
                continue
            raise RuntimeError(f"HTTP {e.code}: {text[:300]}")
        except (urllib.error.URLError, TimeoutError) as e:
            if attempt < retries:
                time.sleep(2)
                continue
            raise RuntimeError(str(e))


def ask(key, model, state, questions, long):
    """A whole request as the variant sends it: one request under the header, else the client's batches at once."""
    parts = [questions] if long else client_batches(state, questions)
    t = time.monotonic()
    if len(parts) == 1:
        answers, usage, served, _ = send(key, model, state, parts[0], long)
        return answers, usage, served, time.monotonic() - t, 1
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(parts)) as pool:
        results = list(pool.map(lambda qs: send(key, model, state, qs, long), parts))
    answers, usage, served = {}, collections.Counter(), ""
    for a, u, m, _ in results:
        answers.update(a)
        usage.update({k: v for k, v in u.items() if isinstance(v, (int, float))})
        served = m
    return answers, dict(usage), served, time.monotonic() - t, len(parts)


def variant_of(name):
    alias, _, flag = name.partition("+")
    return ALIASES.get(alias, alias), flag == "long"


def bucket(n):
    return "<8k" if n < 8000 else "8-16k" if n < 16000 else "16-32k" if n < 32000 else "32-64k" if n < 64000 else ">=64k"


def p90(xs):
    xs = sorted(xs)
    return xs[int(0.9 * (len(xs) - 1))] if xs else 0


def agreement(rows, against):
    """rows: result rows with 'answers'; against: a function row -> the answers to compare with (or None)."""
    nn = na = cn = cs = 0
    nd, cp = [], []
    for r in rows:
        other = against(r)
        if not other:
            continue
        c = compare(other, r["answers"])
        nn += c["noul_n"]; na += c["noul_agree"]; nd += c["noul_diff"]; cn += c["choice_n"]; cs += c["choice_same"]; cp += c["choice_p_jev"]
    return {"nouls": nn, "same_side": na / nn if nn else 0.0, "noul_diff_median": statistics.median(nd) if nd else 0.0,
            "noul_diff_mean": statistics.mean(nd) if nd else 0.0, "choices": cn, "same_pick": cs / cn if cn else 0.0,
            "p_of_pick_median": statistics.median(cp) if cp else 0.0}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("log", help="a match's seat directory or a jev-N.jsonl[.zst]")
    ap.add_argument("--seat", type=int, default=0)
    ap.add_argument("--every", type=int, default=10, help="every Nth ordinary call (0: none)")
    ap.add_argument("--big", type=int, default=12, help="at most this many of the calls the client sent in batches (the largest first)")
    ap.add_argument("--variants", default="latest,latest+long,preview,preview+long")
    ap.add_argument("--parallel", type=int, default=3)
    ap.add_argument("--out")
    args = ap.parse_args()
    key = jev_key()
    path = jevlog.seat(args.log, args.seat) if os.path.isdir(args.log) else args.log
    calls = load_calls(path)
    ordinary = [c for c in calls if c["batches"] <= 1]
    big = sorted((c for c in calls if c["batches"] > 1), key=lambda c: -c["jev_tokens"])[:args.big]
    sample = ([c for i, c in enumerate(ordinary) if args.every and i % args.every == 0]) + big
    print(f"{path}: {len(calls)} recorded calls ({len(calls) - len(ordinary)} in batches), recorded model {calls[0]['model'] if calls else '?'}; "
          f"replaying {len(sample)} ({len(sample) - len(big)} ordinary, {len(big)} batched: {', '.join(f'{c['jev_tokens'] // 1000}k/{c['batches']}' for c in big)})")
    variants = args.variants.split(",")
    jobs = []
    for ci, c in enumerate(sample):
        for v in variants:
            jobs.append((ci, v, 0))
        if variants and variants[0] == "latest":
            jobs.append((ci, "latest", 1))

    def run(job):
        ci, v, rep = job
        c = sample[ci]
        model, long = variant_of(v)
        row = {"ci": ci, "f": c["f"], "variant": v, "repeat": rep, "recorded_tokens": c["jev_tokens"], "recorded_ms": c["ms"], "recorded_batches": c["batches"]}
        try:
            answers, usage, served, seconds, parts = ask(key, model, c["state"], c["questions"], long)
            row.update(answers=answers, usage=usage, served=served, seconds=seconds, parts=parts)
        except RuntimeError as e:
            row["error"] = str(e)
        return row

    t0 = time.monotonic()
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.parallel) as pool:
        results = list(pool.map(run, jobs))
    wall = time.monotonic() - t0
    if args.out:
        with open(args.out, "w") as f:
            for r in results:
                f.write(json.dumps(r) + "\n")
    tokens = sum(r.get("usage", {}).get("input_tokens", 0) for r in results)
    print(f"{len(results)} requests in {wall:.0f} s at {args.parallel} at once, {tokens / 1e6:.2f}M tokens (${tokens / 1e6 * PRICE:.2f})\n")
    baseline = {r["ci"]: r for r in results if r["variant"] == "latest" and r["repeat"] == 1 and "answers" in r}
    for v in variants:
        rows = [r for r in results if r["variant"] == v and r["repeat"] == 0]
        ok = [r for r in rows if "answers" in r]
        errors = [r for r in rows if "error" in r]
        served = collections.Counter(r["served"] for r in ok)
        print(f"== {v}: {len(ok)} ok, {len(errors)} failed; served by {dict(served)}")
        for e in errors[:3]:
            print(f"   error (f={e['f']}, {e['recorded_tokens'] // 1000}k recorded): {e['error'][:160]}")
        by = collections.defaultdict(list)
        for r in ok:
            by[bucket(r["recorded_tokens"])].append(r)
        print("   latency ms (median / p90, this run) against the recorded ms (median), by the recorded size:")
        for b in ("<8k", "8-16k", "16-32k", "32-64k", ">=64k"):
            if by[b]:
                print(f"     {b:6s} n={len(by[b]):3d}  {statistics.median(r['seconds'] * 1000 for r in by[b]):6.0f} / {p90([r['seconds'] * 1000 for r in by[b]]):5.0f}   "
                      f"recorded {statistics.median(r['recorded_ms'] for r in by[b]):5.0f}   parts {statistics.median(r['parts'] for r in by[b]):.0f}")
        for label, group in (("ordinary", [r for r in ok if r["recorded_batches"] <= 1]), ("batched", [r for r in ok if r["recorded_batches"] > 1])):
            if not group:
                continue
            a = agreement(group, lambda r: sample[r["ci"]]["jev"])
            print(f"   {label:8s} against the recorded answers: nouls {a['nouls']} same side {a['same_side']:.3f} |diff| median {a['noul_diff_median']:.3f} mean {a['noul_diff_mean']:.3f}; "
                  f"choices {a['choices']} same pick {a['same_pick']:.3f} (p of the recorded pick, median {a['p_of_pick_median']:.2f})")
            if baseline:
                a = agreement(group, lambda r: baseline.get(r["ci"], {}).get("answers"))
                print(f"   {'':8s} against jev-latest re-sent now:   nouls {a['nouls']} same side {a['same_side']:.3f} |diff| median {a['noul_diff_median']:.3f} mean {a['noul_diff_mean']:.3f}; "
                      f"choices {a['choices']} same pick {a['same_pick']:.3f} (p of its pick, median {a['p_of_pick_median']:.2f})")
        print()


if __name__ == "__main__":
    main()
