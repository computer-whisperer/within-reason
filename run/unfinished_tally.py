#!/usr/bin/env python3
"""Sums the `unfinished.json` files the unfinished-structures reviewers wrote into match directories (the brief:
docs/studies/data/unfinished-2026-10-17/BRIEF.md): one row a game (the denominators and the causes), the totals, the
causes by count and metal, the fates, the types, what the player knew, the items by cost, the never-started orders by
cause, and every mechanism the reviewers named.

usage: run/unfinished_tally.py MATCH... [--items] [--cost game|large|small|none] [--cause U1] [--mechanisms] [--json OUT]
  MATCH is a match directory (run/matches/<game> or its 00); a directory holding copies of the files works too.
"""
import argparse
import glob
import json
import os
import sys

CAUSES = ["U1", "U2", "U3", "U4", "U5", "U6", "U7", "U8", "U9", "U0"]
WORDS = {"U1": "the pick sent the builder elsewhere", "U2": "the builder's own list moved on", "U3": "the player re-listed or moved it",
         "U4": "the builder died or fled, or the frame was killed", "U5": "economy", "U6": "the site", "U7": "helpers withdrawn",
         "U8": "game end", "U9": "harness fault", "U0": "unknown"}
COST_RANK = {"game": 0, "large": 1, "small": 2, "none": 3}


def load(path):
    for p in (os.path.join(path, "unfinished.json"), os.path.join(path, "00", "unfinished.json"), path):
        if os.path.isfile(p):
            try:
                return json.load(open(p)), p
            except ValueError as e:
                print(f"{p}: {e}", file=sys.stderr)
                return None, p
    return None, path


def add(d, key, n=1, metal=0):
    e = d.setdefault(key, {"n": 0, "metal": 0})
    e["n"] += n
    e["metal"] += metal


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("matches", nargs="+")
    ap.add_argument("--items", action="store_true")
    ap.add_argument("--cost")
    ap.add_argument("--cause")
    ap.add_argument("--mechanisms", action="store_true")
    ap.add_argument("--json")
    a = ap.parse_args()

    games = []
    missing = []
    for m in a.matches:
        data, p = load(m)
        if data is None:
            missing.append(p)
            continue
        games.append(data)

    rows = []
    tot = {"frames": 0, "finished": 0, "decayed": 0, "killed": 0, "at_end": 0, "metal": 0, "never_started": 0, "seats": 0}
    causes = {}
    fates = {}
    types = {}
    knew = {"before": 0, "after": 0, "never": 0, "dropped_itself": 0}
    items = []
    never = []
    never_causes = {}
    mechanisms = []
    for g in games:
        den = g.get("denominators") or {}
        r = {"game": g.get("game"), "result": g.get("result"), "kind": g.get("kind"), "seats": len(den)}
        for k in ("frames", "finished", "decayed", "killed", "at_end", "metal_in_unfinished", "never_started"):
            r[k] = sum((d.get(k) or 0) for d in den.values())
        tot["frames"] += r["frames"]
        tot["finished"] += r["finished"]
        tot["decayed"] += r["decayed"]
        tot["killed"] += r["killed"]
        tot["at_end"] += r["at_end"]
        tot["metal"] += r["metal_in_unfinished"]
        tot["never_started"] += r["never_started"]
        tot["seats"] += len(den)
        t = g.get("tally") or {}
        for c in CAUSES:
            e = t.get(c) or {}
            r[c] = e.get("n", 0)
            add(causes, c, e.get("n", 0), e.get("metal", 0))
        for f, e in (g.get("by_fate") or {}).items():
            add(fates, f, e.get("n", 0), e.get("metal", 0))
        for ty, e in (g.get("by_type") or {}).items():
            add(types, ty, e.get("n", 0), e.get("metal", 0))
        for k in knew:
            knew[k] += (g.get("player_knew") or {}).get(k, 0)
        for it in g.get("items") or []:
            it = dict(it)
            it["game"] = g.get("game")
            items.append(it)
        for it in g.get("never_started") or []:
            it = dict(it)
            it["game"] = g.get("game")
            never.append(it)
            add(never_causes, it.get("cause", "U0"), 1, 0)
        for mech in g.get("mechanisms") or []:
            mechanisms.append((g.get("game"), mech))
        rows.append(r)

    if a.json:
        json.dump({"games": rows, "totals": tot, "causes": causes, "fates": fates, "types": types, "player_knew": knew, "items": items, "never_started": never, "never_causes": never_causes, "mechanisms": mechanisms, "missing": missing}, open(a.json, "w"), indent=1)

    if a.items:
        sel = [it for it in items if (not a.cost or it.get("cost_class") == a.cost) and (not a.cause or it.get("cause") == a.cause)]
        sel.sort(key=lambda it: (COST_RANK.get(it.get("cost_class"), 9), -(it.get("metal_in") or 0)))
        for it in sel:
            print(f"{it['game']} s{it.get('seat')} {it.get('started')}-{it.get('fate_clock')} {it.get('frame')} {it.get('cell')} {it.get('fate')} {int((it.get('share') or 0) * 100)}% {it.get('metal_in')}m {it.get('cause')}{'+' + it['also'] if it.get('also') else ''} [{it.get('cost_class')}, {it.get('confidence')}, fix {it.get('fix_class')}] {it.get('builder')}: {it.get('what_happened')}")
            print(f"    knew: {it.get('player_knew')}")
        return

    if a.mechanisms:
        for g, mech in mechanisms:
            print(f"{g}: {mech}")
        return

    print(f"{'game':42} {'result':16} {'st':>2} {'fr':>4} {'fin':>4} {'dec':>4} {'kil':>4} {'end':>4} {'metal':>6} {'nev':>4} " + " ".join(f"{c:>3}" for c in CAUSES))
    for r in rows:
        print(f"{(r['game'] or '?')[:42]:42} {(r['result'] or '?')[:16]:16} {r['seats']:>2} {r['frames']:>4} {r['finished']:>4} {r['decayed']:>4} {r['killed']:>4} {r['at_end']:>4} {r['metal_in_unfinished']:>6} {r['never_started']:>4} " + " ".join(f"{r[c]:>3}" for c in CAUSES))
    print(f"{'TOTAL':42} {len(rows):>2} games {tot['seats']:>8} {tot['frames']:>4} {tot['finished']:>4} {tot['decayed']:>4} {tot['killed']:>4} {tot['at_end']:>4} {tot['metal']:>6} {tot['never_started']:>4} " + " ".join(f"{causes.get(c, {}).get('n', 0):>3}" for c in CAUSES))
    print("\ncauses (frames):")
    for c in CAUSES:
        e = causes.get(c, {"n": 0, "metal": 0})
        print(f"  {c} {WORDS[c]:48} n {e['n']:>4}  metal {e['metal']:>6}")
    print("\nnever-started orders by cause:")
    for c, e in sorted(never_causes.items()):
        print(f"  {c} n {e['n']}")
    print("\nfates:")
    for f, e in sorted(fates.items()):
        print(f"  {f:12} n {e['n']:>4}  metal {e['metal']:>6}")
    print("\ntypes (by metal):")
    for ty, e in sorted(types.items(), key=lambda x: -x[1]["metal"])[:20]:
        print(f"  {ty:14} n {e['n']:>4}  metal {e['metal']:>6}")
    print(f"\nthe player knew: before {knew['before']}, only after {knew['after']}, never {knew['never']}, dropped itself {knew['dropped_itself']}")
    by_cost = {}
    for it in items:
        by_cost[it.get("cost_class")] = by_cost.get(it.get("cost_class"), 0) + 1
    print(f"items {len(items)} by cost: " + ", ".join(f"{k} {v}" for k, v in sorted(by_cost.items(), key=lambda x: COST_RANK.get(x[0], 9))))
    by_fix = {}
    for it in items:
        add(by_fix, it.get("fix_class"), 1, it.get("metal_in") or 0)
    print("fix classes: " + ", ".join(f"{k} {e['n']} ({e['metal']}m)" for k, e in sorted(by_fix.items(), key=lambda x: -x[1]["metal"])))
    if missing:
        print(f"\nmissing: {len(missing)}: " + ", ".join(missing))


if __name__ == "__main__":
    main()
