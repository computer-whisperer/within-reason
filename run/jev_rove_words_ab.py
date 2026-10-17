#!/usr/bin/env python3
"""Offline: the gates of a game where the group menu's `rove` move was offered, asked again with the move's words
reworded (docs/design/2026-10-07-rove-approval.md; rove-move-1: offered at 1,905 gates, never picked, 0.10 under a
paragraph whose only move was rove). Each gate is sent whole, as recorded, with only the rove question's
"The move: ..." text replaced; the rove, scout and change nouls are read back. Gates are classed by what the packet
in force said: `named` (a line with "rove" naming this group), `generic` (a line with "rove" naming no group),
`none` (no such line: a wording that lifts rove here is a bad wording). `recorded` is the noise floor. The key is
read by run/jev_ab.py's api_key() and never printed or written.

    run/jev_rove_words_ab.py run/matches/<batch>/00 [--gates N] [--variants a,b,c] [--out file.jsonl] [--list]
    run/jev_rove_words_ab.py --read file.jsonl
"""
import argparse, collections, concurrent.futures, json, os, random, re, statistics, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match

ROVE = re.compile(r"^(.*\? The move: )(group_\S+) roves whole: .*$", re.S)

VARIANTS = {
    "recorded": None,
    # The scout move's words, for the whole group.
    "scoutlike": "{name} roves: every soldier looks at what we know least, his start box first, kills what it finds unguarded and keeps out of the reach of anything that can shoot it, on its own in code; the rest of your hands' moves for it end",
    # Plain, with the way back said as the player's.
    "plain": "{name} roves: its soldiers scout his side on their own, looking at what we know least, killing unguarded constructors and extractors, and keeping out of every reach; the player's lane setting brings it back",
    # Short.
    "short": "{name} roves: its soldiers scout on their own, his start box first, kill what they find unguarded and stay out of reach",
    # The recorded words without the two phrases suspected of reading as a loss of control.
    "nofear": "{name} roves whole: every soldier runs on its own in code, ten times a second: it looks at what we know least, his start box first, kills what it finds unguarded (a constructor, an extractor, a radar) and keeps out of the reach of anything that can shoot it",
}


def clock(f):
    return f"{f // 1800}:{f // 30 % 60:02d}"


def packet_class(packet, name):
    lines = [l for l in packet.split("\n") if re.search(r"\brov(e|es|ing)\b", l, re.I)]
    if any(name in l for l in lines):
        return "named"
    if any(not re.search(r"group_[A-Z]", l) for l in lines):
        return "generic"
    return "none"


def gates_of(m):
    for c, state, packet in requests_of(m):
        q = c["questions"]
        roves = [(k, v) for k, v in q.items() if k.endswith(".rove") and ROVE.match(v.get("instructions", ""))]
        if not roves:
            continue
        for k, v in roves:
            name = k[: -len(".rove")]
            cls = packet_class(packet, name)
            # A generic rove line ("Rover groups: they rove") is about fast units: a Stout army under it is `none`.
            entry = json.dumps(state.get("actors", {}).get(name, ""))
            if cls == "generic" and not re.search(r"armfav|corfav|Rover|Rascal|Tick|Flea", entry):
                cls = "none"
            yield c, state, name, cls


def reword(questions, name, text):
    out = dict(questions)
    q = dict(out[f"{name}.rove"])
    mm = ROVE.match(q["instructions"])
    q["instructions"] = mm.group(1) + text.format(name=name) + "."
    out[f"{name}.rove"] = q
    return out


def one(key, c, state, name, cls, variant):
    text = VARIANTS[variant]
    questions = c["questions"] if text is None else reword(c["questions"], name, text)
    r = ask(key, state, questions, "jev-1.14.0")
    answers = {k: a.get("noul") for k, a in (r.get("answers") or {}).items() if isinstance(a, dict)}
    logged = {k: a.get("noul") for k, a in c["answers"].items() if isinstance(a, dict)}
    others = sorted(((v, k) for k, v in answers.items() if k.startswith(name + ".") and not k.endswith((".rove", ".change", ".scout")) and not k.endswith(".forbidden")), reverse=True)
    return {"f": c["f"], "clock": clock(c["f"]), "actor": name, "class": cls, "variant": variant,
            "rove": answers.get(f"{name}.rove"), "scout": answers.get(f"{name}.scout"), "change": answers.get(f"{name}.change"),
            "best_other": others[0] if others else None, "logged_rove": logged.get(f"{name}.rove"), "logged_change": logged.get(f"{name}.change"),
            "tokens": (r.get("usage") or {}).get("input_tokens", 0)}


def summarize(rows):
    by = collections.defaultdict(list)
    for r in rows:
        by[(r["class"], r["variant"])].append(r)
    print(f"{'class':8s} {'variant':10s} {'n':>3s} {'median':>7s} {'>=0.5':>6s} {'>=0.3':>6s}  {'scout med':>9s} {'change med':>10s}")
    for (cls, v), rs in sorted(by.items()):
        roves = [r["rove"] for r in rs if r["rove"] is not None]
        if not roves:
            continue
        sc = [r["scout"] for r in rs if r["scout"] is not None]
        ch = [r["change"] for r in rs if r["change"] is not None]
        print(f"{cls:8s} {v:10s} {len(roves):3d} {statistics.median(roves):7.2f} {sum(x >= 0.5 for x in roves):6d} {sum(x >= 0.3 for x in roves):6d}  {statistics.median(sc) if sc else float('nan'):9.2f} {statistics.median(ch) if ch else float('nan'):10.2f}")
    logged = [r["logged_rove"] for r in rows if r["variant"] == "recorded" and r["logged_rove"] is not None]
    rec = [r["rove"] for r in rows if r["variant"] == "recorded" and r["rove"] is not None]
    if logged and rec:
        print(f"noise floor: recorded asked again vs logged, median |diff| {statistics.median(abs(a - b) for a, b in zip(rec, logged)):.2f}")
    print("tokens", sum(r["tokens"] for r in rows), "≈ $", round(sum(r["tokens"] for r in rows) / 1e6 * 0.042, 2))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("match", nargs="?")
    ap.add_argument("--gates", type=int, default=24, help="per class, at most")
    ap.add_argument("--variants", default=",".join(VARIANTS))
    ap.add_argument("--out")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--read")
    ap.add_argument("--workers", type=int, default=4)
    a = ap.parse_args()
    if a.read:
        summarize([json.loads(l) for l in open(a.read)])
        return
    m = Match(a.match)
    gates = list(gates_of(m))
    by = collections.defaultdict(list)
    for g in gates:
        by[g[3]].append(g)
    random.seed(7)
    chosen = []
    for cls, gs in by.items():
        # Spread over the game: sort by frame, take every k-th.
        gs.sort(key=lambda g: g[0]["f"])
        step = max(1, len(gs) // a.gates)
        chosen.extend(gs[::step][: a.gates])
    print(f"gates with a rove question: {len(gates)} ({', '.join(f'{k} {len(v)}' for k, v in by.items())}); replaying {len(chosen)}")
    if a.list:
        for c, state, name, cls in chosen:
            print(f"  {clock(c['f'])} {name:10s} {cls:8s} logged rove {c['answers'].get(name + '.rove', {}).get('noul')}")
        return
    key = api_key()
    variants = a.variants.split(",")
    out = a.out or f"docs/studies/data/jev-rove-words-{os.path.basename(os.path.dirname(a.match.rstrip('/')))}.jsonl"
    rows = []
    with concurrent.futures.ThreadPoolExecutor(a.workers) as ex, open(out, "w") as f:
        futures = [ex.submit(one, key, c, state, name, cls, v) for c, state, name, cls in chosen for v in variants]
        for fut in concurrent.futures.as_completed(futures):
            try:
                r = fut.result()
            except Exception as e:
                print("failed:", e, file=sys.stderr)
                continue
            rows.append(r)
            f.write(json.dumps(r) + "\n")
    print("wrote", out)
    summarize(rows)


if __name__ == "__main__":
    main()
