#!/usr/bin/env python3
"""Which products a game's Jev bill is made of: the gate's questions by family (who is asked about what), the
questions per pair of asker and target, how each family's share grows through the game, and the costliest gates.

    run/jev_products.py run/matches/<batch> [...]
"""
import collections, glob, json, re, sys


def family(k):
    actor, _, move = k.partition(".")
    if move == "place": return "a place (one a gate)"
    if actor.startswith("party_"): return "a party (one a gate)"
    who = "group" if actor.startswith("group_") else "builder" if actor.startswith(("constructor", "commander")) else "plant"
    if move in ("change", "hold", "gather", "scout") or move.startswith("change"): return f"{who}: itself"
    if "party_" in move: return f"{who} x party"
    if who == "group" and re.match(r"(join|follow)_", move): return "group x group or builder (join, follow)"
    if who == "builder" and re.match(r"(help|repair|follow)_", move): return "builder x plant, builder or building (help, repair)"
    if who == "builder" and move.startswith("build_"):
        return "builder x type x spot or place" if re.search(r"_(spot_\d+|[a-z0-9]+_[a-z0-9_]+)$", move[6:]) and "_" in move[6:] else "builder x type"
    if who == "plant": return "plant x unit type"
    return f"{who} x place"


def main():
    for D in sys.argv[1:]:
        fam = collections.Counter(); famn = collections.Counter(); bymin = collections.defaultdict(collections.Counter); pic = collections.Counter(); total = 0
        pairs = collections.defaultdict(list); gates = []; dims = []
        for fn in sorted(glob.glob(D.rstrip("/") + "/00/jev-*.jsonl")):
            seat = fn[-7]
            for l in open(fn):
                try: r = json.loads(l)
                except ValueError: continue
                t = r.get("t"); T = (r.get("usage") or {}).get("input_tokens", 0) or 0
                if t == "decode": total += T; fam["the decode (actor x place, type, group)"] += T; bymin[r["f"] // 9000]["decode"] += T; continue
                if t != "call" or "questions" not in r: continue
                total += T; q = r["questions"]; b = r.get("batches", 1) or 1
                st = (sum(len(json.dumps(v)) for v in (r.get("state") or {}).values()) + 9000) * b
                tot = st + sum(len(json.dumps(v)) for v in q.values())
                actors = len((r.get("state") or {}).get("actors") or {})
                if any(k.startswith("worlds") for k in q):
                    fam["the pick (its picture and questions)"] += T; bymin[r["f"] // 9000]["pick"] += T; continue
                fam["the picture in the gate (actors x requests)"] += T * st / tot; bymin[r["f"] // 9000]["picture"] += T * st / tot
                here = collections.Counter(); askers = collections.defaultdict(set); targets = collections.defaultdict(set)
                for k, v in q.items():
                    f = family(k); c = T * len(json.dumps(v)) / tot
                    fam[f] += c; famn[f] += 1; here[f] += 1; bymin[r["f"] // 9000][f] += c
                    a, _, move = k.partition("."); askers[f].add(a)
                    m = re.search(r"(party_\d+\w*|spot_\d+|group_\w+|constructor_\d+|plant_\d+|lab_\d+)", move)
                    if m: targets[f].add(m.group(1))
                for f in here:
                    if askers[f] and targets[f]: pairs[f].append((len(askers[f]), len(targets[f]), here[f]))
                gates.append((T, r["f"], seat, len(q), actors, b, here.most_common(3)))
        print(f"== {D.rstrip('/').split('/')[-1][11:]}: {total/1e6:.1f}M tokens, ${total*0.042/1e6:.2f}")
        mins = sorted(bymin)
        print(f"   {'':52s} share   " + " ".join(f"{m*5:>3d}-" for m in mins) + "  (M tokens by five game minutes)")
        for f, v in fam.most_common():
            key = "decode" if f.startswith("the decode") else "pick" if f.startswith("the pick") else "picture" if f.startswith("the picture") else f
            row = " ".join(f"{bymin[m][key]/1e6:4.1f}" for m in mins)
            p = pairs.get(f)
            mult = f"  [{sum(x[2] for x in p)/sum(x[0]*x[1] for x in p):.2f} questions a pair; askers {sum(x[0] for x in p)/len(p):.1f} x targets {sum(x[1] for x in p)/len(p):.1f} a gate, max {max(x[0] for x in p)} x {max(x[1] for x in p)}]" if p else ""
            print(f"   {f[:52]:52s} {100*v/total:4.1f}%   {row}{mult}")
        print("   the costliest gates:")
        for T, f, seat, n, actors, b, top in sorted(gates, reverse=True)[:4]:
            print(f"     {f//1800}:{f%1800//30:02d} seat {seat}: {T/1000:.0f}k tokens, {n} questions in {b} requests, {actors} actors in the picture; " + ", ".join(f"{k} {c}" for k, c in top))


if __name__ == "__main__":
    main()
