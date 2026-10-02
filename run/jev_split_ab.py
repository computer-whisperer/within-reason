#!/usr/bin/env python3
"""Offline: the pick split into the parts that do not touch each other. A recorded pick's worlds are every
combination of the opened actors' changes. Here the slots with a change on offer are grouped into components (two
slots are of one component when a change of one names the other's actor, the same group, builder or party), and
each component gets its own small pick: the best of its changes, and that change against "nothing changes". All of
it goes in ONE request, the second stage asked for every candidate at once, in place of the game's two calls.

Set beside the recorded pick: which change each component would make, against the share the joint pick's
probabilities give that component's changes; how many actors change that second; the tokens.

usage: run/jev_split_ab.py run/matches/<batch>/<NN> --out <file.jsonl> [--picks N] [--min-worlds N]
       run/jev_split_ab.py --read <file.jsonl>
The key is read by run/jev_ab.py's api_key() and never printed or written. Jev: $0.042 a million input tokens."""
import argparse, collections, concurrent.futures, json, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from jev_ab import api_key, ask, requests_of
from match_read import Match
from route_ab import STAGE_ONE, STAGE_TWO

NAME = re.compile(r"group_\w+|constructor_\d+|commander\w*|plant_\d+|lab_\d+|factory_\d+|party_\d+")


def components(slots, worlds):
    base = worlds[0]
    touched = sorted({k for w in worlds[1:] for k, (a, b) in enumerate(zip(w, base)) if a != b})
    names = {}
    for k in touched:
        tokens = {slots[k]["name"]}
        for w in worlds[1:]:
            if w[k] != base[k]:
                s = slots[k]["states"][w[k]]
                tokens.add(s.get("actor") or slots[k]["name"])
                tokens.update(x for x in NAME.findall(s.get("words", "")) if not x.startswith("party_") or slots[k]["kind"].startswith("threat"))
        names[k] = tokens - {""}
    parent = {k: k for k in touched}

    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x
    for a in touched:
        for b in touched:
            if a < b and names[a] & names[b]:
                parent[find(a)] = find(b)
    out = collections.defaultdict(list)
    for k in touched:
        out[find(k)].append(k)
    return list(out.values())


def run(args):
    key = api_key()
    m = Match(args.match)
    passes = {c["f"]: c for c in m.calls if c.get("t") == "pass" and c.get("slots")}
    gates = {c["f"]: c for c in m.calls if c.get("t") == "worlds_gate" and c.get("lines")}
    plans = {c["f"]: c for c in m.calls if c.get("t") == "plan"}
    picks = collections.defaultdict(dict)
    for c, state, _ in requests_of(m):
        if "worlds.pick" in c["questions"]:
            picks[c["f"]]["two" if "w1" in c["questions"]["worlds.pick"]["criteria"] else "one"] = (c, state)
    jobs = []
    for f in sorted(gates):
        if f not in passes or f not in plans or "one" not in picks[f] or "two" not in picks[f]:
            continue
        slots, worlds, lines = passes[f]["slots"], gates[f]["worlds"], gates[f]["lines"]
        if len(worlds) != len(lines) or len(worlds) < args.min_worlds or any(len(w) != len(slots) for w in worlds):
            continue
        jobs.append(f)
    jobs = jobs[:: max(1, len(jobs) // args.picks)][: args.picks]
    print(f"{len(jobs)} picks", file=sys.stderr)

    def one(f):
        slots, worlds, lines = passes[f]["slots"], gates[f]["worlds"], gates[f]["lines"]
        (c1, state), (c2, _) = picks[f]["one"], picks[f]["two"]
        base = worlds[0]
        comps = components(slots, worlds)
        dev = lambda w, comp: tuple((k, w[k]) for k in comp if w[k] != base[k])
        questions, cands = {}, []
        for ci, comp in enumerate(comps):
            inside = [i for i, w in enumerate(worlds) if i and dev(w, comp) and all(w[k] == base[k] for k in range(len(base)) if k not in comp)]
            cands.append(inside)
            if len(inside) >= 2:
                questions[f"c{ci}.one"] = {"type": "choice", "instructions": STAGE_ONE, "criteria": {f"w{i + 1}": lines[i] for i in inside}}
            for i in inside:
                questions[f"c{ci}.two.w{i + 1}"] = {"type": "choice", "instructions": STAGE_TWO, "criteria": {"w1": lines[0], f"w{i + 1}": lines[i]}}
        a = ask(key, state, questions, c1["model"])
        ans = a.get("answers") or {}
        recorded = c1["answers"]["worlds.pick"].get("probabilities") or {}
        final = worlds[plans[f]["pick"] - 1] if plans[f].get("pick") else base
        row = {"f": f, "worlds": len(worlds), "components": [], "tokens": (a.get("usage") or {}).get("input_tokens", 0),
               "recorded_tokens": (c1.get("usage") or {}).get("input_tokens", 0) + (c2.get("usage") or {}).get("input_tokens", 0),
               "joint_line_chars": sum(len(l) for l in lines[1:]), "split_line_chars": sum(len(lines[i]) for inside in cands for i in inside)}
        for ci, comp in enumerate(comps):
            inside = cands[ci]
            if not inside:
                continue
            if len(inside) >= 2:
                p = (ans.get(f"c{ci}.one") or {}).get("probabilities") or {}
                top = int(max(p, key=p.get)[1:]) - 1 if p else inside[0]
            else:
                top = inside[0]
            p2 = (ans.get(f"c{ci}.two.w{top + 1}") or {}).get("probabilities") or {}
            wins = p2.get(f"w{top + 1}", 0) > p2.get("w1", 0)
            # The joint pick's probabilities, summed by what each world does inside this component.
            mass = collections.Counter()
            for w, pr in recorded.items():
                i = int(w[1:]) - 1
                if i < len(worlds):
                    mass[dev(worlds[i], comp)] += pr
            changes = {d: v for d, v in mass.items() if d}
            joint_top = max(changes, key=changes.get) if changes else None
            row["components"].append({"slots": [slots[k]["name"] for k in comp], "candidates": len(inside), "top": [slots[k]["states"][v]["id"] for k, v in dev(worlds[top], comp)],
                                      "same_as_joint_top": joint_top == dev(worlds[top], comp), "joint_mass_on_top": mass.get(dev(worlds[top], comp), 0), "joint_mass_any_change": sum(changes.values()),
                                      "wins": wins, "p_change": p2.get(f"w{top + 1}", 0), "recorded_final": [slots[k]["states"][v]["id"] for k, v in dev(final, comp)]})
        return row

    with open(args.out, "w") as out, concurrent.futures.ThreadPoolExecutor(6) as pool:
        tokens = 0
        for row in pool.map(one, jobs):
            tokens += row["tokens"]
            out.write(json.dumps(row) + "\n")
    print(f"{tokens} input tokens, ${tokens * 0.042 / 1e6:.3f}", file=sys.stderr)


def read(args):
    rows = [json.loads(line) for line in open(args.read)]
    med = lambda xs: sorted(xs)[len(xs) // 2]
    comps = [c for r in rows for c in r["components"]]
    print(f"{len(rows)} picks, a median {med([r['worlds'] for r in rows])} worlds (max {max(r['worlds'] for r in rows)}); components a pick: median {med([len(r['components']) for r in rows])}, max {max(len(r['components']) for r in rows)}; candidates a component: median {med([c['candidates'] for c in comps])}, max {max(c['candidates'] for c in comps)}")
    multi = [c for c in comps if c["candidates"] >= 2]
    print(f"components with two or more candidates: {len(multi)}; the split's best change is the joint pick's most-weighted change for that component in {sum(c['same_as_joint_top'] for c in multi)}")
    print(f"the split's change beats 'nothing changes' in {sum(c['wins'] for c in comps)} of {len(comps)} components; the game's final world changed that component in {sum(bool(c['recorded_final']) for c in comps)}")
    both = sum(1 for c in comps if c["wins"] and c["recorded_final"])
    same = sum(1 for c in comps if c["wins"] and c["recorded_final"] == c["top"])
    print(f"  both changed it in {both}, to the same change in {same}; the split alone in {sum(1 for c in comps if c['wins'] and not c['recorded_final'])}; the game alone in {sum(1 for c in comps if not c['wins'] and c['recorded_final'])}")
    print(f"actors changed a pick: the split {sum(c['wins'] for c in comps) / len(rows):.2f}, the game {sum(bool(c['recorded_final']) for c in comps) / len(rows):.2f}")
    print(f"tokens: the split's one request {sum(r['tokens'] for r in rows) / len(rows):.0f} a pick against the game's two calls {sum(r['recorded_tokens'] for r in rows) / len(rows):.0f}; world-line characters {sum(r['split_line_chars'] for r in rows) / len(rows):.0f} against {sum(r['joint_line_chars'] for r in rows) / len(rows):.0f}")
    big = [r for r in rows if r["worlds"] >= 20]
    if big:
        print(f"picks of 20 worlds or more ({len(big)}): the split {sum(r['tokens'] for r in big) / len(big):.0f} tokens against {sum(r['recorded_tokens'] for r in big) / len(big):.0f}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("match", nargs="?")
    ap.add_argument("--out")
    ap.add_argument("--read")
    ap.add_argument("--picks", type=int, default=120)
    ap.add_argument("--min-worlds", type=int, default=3)
    args = ap.parse_args()
    if args.read:
        read(args)
    elif args.match and args.out:
        run(args)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
