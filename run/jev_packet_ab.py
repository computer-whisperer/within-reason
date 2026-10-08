#!/usr/bin/env python3
"""Offline: the gates of one actor in a window of a game, asked again under variants of the packet in force and of
the actor's move words (docs/studies/2026-10-07-raid-packets.md: human-26 seat 0, 3:33-4:05, six Blitzes at home
and a Rover swarm on the base). Each gate is sent whole, as recorded, with the variant's edits applied: `packet`
is a list of [old, new] replacements on the instructions (old must occur once; "" old appends), `words` a list of
[regex, replacement] applied to the actor's own questions. The nouls named by --keys are read back; `recorded` is
the noise floor. The key is read by run/jev_ab.py's api_key() and never printed or written.

    run/jev_packet_ab.py <match>/<seat dir> --from m:ss --to m:ss --actor group_B_t2 --variants variants.json
                         [--keys attack_party_1,change,fight_to_home] [--out file.jsonl] [--list] [--workers 4]
"""
import argparse, collections, concurrent.futures, json, os, re, statistics, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match


def frames(clock):
    m, s = clock.split(":")
    return (int(m) * 60 + int(s)) * 30


def clock(f):
    return f"{f // 1800}:{f // 30 % 60:02d}"


def apply(state, questions, actor, variant):
    st = dict(state)
    packet = st.get("instructions", "")
    for old, new in variant.get("packet", []):
        if old == "":
            packet = packet.rstrip("\n") + "\n\n" + new
        else:
            assert packet.count(old) == 1, f"packet edit: {old[:60]!r} occurs {packet.count(old)} times"
            packet = packet.replace(old, new)
    st["instructions"] = packet
    q = {}
    for k, v in questions.items():
        if k.startswith(actor + ".") and "instructions" in v:
            text = v["instructions"]
            for pattern, repl in variant.get("words", []):
                text = re.sub(pattern, repl, text)
            v = {**v, "instructions": text}
        q[k] = v
    return st, q


def one(key, c, state, actor, name, variant, keys):
    st, q = apply(state, c["questions"], actor, variant)
    r = ask(key, st, q, "jev-latest")
    answers = {k: a.get("noul") for k, a in (r.get("answers") or {}).items() if isinstance(a, dict)}
    logged = {k: a.get("noul") for k, a in c["answers"].items() if isinstance(a, dict)}
    row = {"f": c["f"], "clock": clock(c["f"]), "actor": actor, "variant": name, "tokens": (r.get("usage") or {}).get("input_tokens", 0)}
    for k in keys:
        full = k if "." in k else f"{actor}.{k}"
        row[k] = answers.get(full)
        row[f"logged_{k}"] = logged.get(full)
    # The best-rated of the actor's own moves, for the eye.
    own = sorted(((v, k) for k, v in answers.items() if k.startswith(actor + ".") and not k.endswith((".change", ".forbidden")) and v is not None), reverse=True)
    row["best"] = own[0] if own else None
    return row


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("match")
    ap.add_argument("--from", dest="start", required=True)
    ap.add_argument("--to", dest="end", required=True)
    ap.add_argument("--actor", required=True)
    ap.add_argument("--variants", required=True)
    ap.add_argument("--keys", default="attack_party_1,change")
    ap.add_argument("--out")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--workers", type=int, default=4)
    a = ap.parse_args()
    variants = json.load(open(a.variants))
    keys = a.keys.split(",")
    m = Match(a.match)
    lo, hi = frames(a.start), frames(a.end)
    gates = [(c, state) for c, state, _ in requests_of(m) if lo <= c["f"] <= hi and any(k.startswith(a.actor + ".") for k in c["questions"]) and not any(k.startswith("worlds.") for k in c["questions"])]
    print(f"{len(gates)} gates of {a.actor} in {a.start}-{a.end}; variants {list(variants)}")
    if a.list:
        for c, _ in gates:
            print(f"  {clock(c['f'])} {[k.split('.', 1)[1] for k in c['questions'] if k.startswith(a.actor + '.')][:12]}")
        return
    key = api_key()
    rows = []
    out = a.out or f"docs/studies/data/jev-packet-{a.actor}-{a.start.replace(':', '')}-{a.end.replace(':', '')}.jsonl"
    with concurrent.futures.ThreadPoolExecutor(a.workers) as ex, open(out, "w") as f:
        futures = [ex.submit(one, key, c, state, a.actor, name, v, keys) for c, state in gates for name, v in variants.items()]
        for fut in concurrent.futures.as_completed(futures):
            try:
                r = fut.result()
            except Exception as e:
                print("failed:", e, file=sys.stderr)
                continue
            rows.append(r)
            f.write(json.dumps(r) + "\n")
    print("wrote", out)
    rows.sort(key=lambda r: (r["f"], list(variants).index(r["variant"])))
    print(f"{'gate':5s} {'variant':14s} " + " ".join(f"{k:>16s}" for k in keys) + "   best own move")
    for r in rows:
        print(f"{r['clock']:5s} {r['variant']:14s} " + " ".join(f"{(r[k] if r[k] is not None else float('nan')):16.2f}" for k in keys) + f"   {r['best']}")
    print()
    for name in variants:
        for k in keys:
            vals = [r[k] for r in rows if r["variant"] == name and r[k] is not None]
            if vals:
                print(f"{name:14s} {k:16s} median {statistics.median(vals):.2f}  at/over 0.5: {sum(v >= 0.5 for v in vals)}/{len(vals)}")
    print("tokens", sum(r["tokens"] for r in rows), "≈ $", round(sum(r["tokens"] for r in rows) / 1e6 * 0.042, 2))


if __name__ == "__main__":
    main()
