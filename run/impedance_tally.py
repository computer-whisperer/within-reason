"""The Opus-to-hands impedance tally over recorded games: sums the `impedance.json` files the reviewers wrote into the
match directories (the brief of 2026-10-16: categories A1-A6, the harness not doing what the player wanted; B1-B6,
the player lacking context that was available; C, followed cleanly) and prints one row a game, the totals by
category, the items by cost and category, and the "no lever" sentences (what the player asked for in words that no
move, step or tool can do).

    run/impedance_tally.py run/matches/<batch>... [--items] [--cost large,game] [--category A3,B3] [--no-lever] [--json OUT]

A batch directory means its `00`. `--items` prints every item (default: the large and game ones); `--cost` and
`--category` filter the items printed; `--no-lever` prints the no-lever list; `--json` writes the merged items
and totals for the study's data directory.
"""
import argparse
import collections
import json
import os
import sys

CATS = ["A1", "A2", "A3", "A4", "A5", "A6", "B1", "B2", "B3", "B4", "B5", "B6"]
COST_RANK = {"game": 3, "large": 2, "small": 1, "none": 0}


def load(paths):
    out = []
    for p in paths:
        d = p if os.path.exists(os.path.join(p, "impedance.json")) else os.path.join(p, "00")
        f = os.path.join(d, "impedance.json")
        if not os.path.exists(f):
            print(f"{p}: no impedance.json", file=sys.stderr)
            continue
        try:
            j = json.load(open(f))
        except json.JSONDecodeError as e:
            print(f"{f}: {e}", file=sys.stderr)
            continue
        j["_dir"] = d
        j.setdefault("game", os.path.basename(os.path.dirname(d)))
        out.append(j)
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("matches", nargs="+")
    ap.add_argument("--items", action="store_true")
    ap.add_argument("--cost", default="large,game")
    ap.add_argument("--category", default="")
    ap.add_argument("--no-lever", action="store_true")
    ap.add_argument("--json")
    a = ap.parse_args()
    games = load(a.matches)
    if not games:
        sys.exit("nothing to tally")
    total = collections.Counter()
    moments = clean = 0
    orders = refused = hands_lines = flagged = turns = 0
    print(f"{'game':42} {'result':16} {'A':>3} {'B':>3} {'C/mom':>6} {'ord':>5} {'ref':>4} {'flag':>5}  per category")
    for g in games:
        t = {k: int(g.get("tally", {}).get(k, 0) or 0) for k in CATS}
        total.update(t)
        m, c = int(g.get("moments_examined", 0) or 0), int(g.get("clean", 0) or 0)
        moments += m
        clean += c
        co = sum(int(s.get("orders", 0) or 0) for s in g.get("counts", {}).values())
        cr = sum(int(s.get("refused", 0) or 0) for s in g.get("counts", {}).values())
        ch = sum(int(s.get("hands_lines", 0) or 0) for s in g.get("counts", {}).values())
        cf = sum(sum(int(v) for v in (s.get("flagged") or {}).values()) for s in g.get("counts", {}).values())
        ct = sum(int(s.get("turns", 0) or 0) for s in g.get("counts", {}).values())
        orders, refused, hands_lines, flagged, turns = orders + co, refused + cr, hands_lines + ch, flagged + cf, turns + ct
        A, B = sum(t[k] for k in CATS[:6]), sum(t[k] for k in CATS[6:])
        per = " ".join(f"{k}{t[k]}" for k in CATS if t[k])
        print(f"{g['game'][:42]:42} {str(g.get('result', ''))[:16]:16} {A:3} {B:3} {c:2}/{m:<3} {co:5} {cr:4} {cf:5}  {per}")
    A, B = sum(total[k] for k in CATS[:6]), sum(total[k] for k in CATS[6:])
    print(f"\n{len(games)} games, {turns} turns, {orders} orders ({refused} refused at the tool), {hands_lines} hands' lines ({flagged} flagged by the sweep)")
    print(f"A (not followed) {A}: " + ", ".join(f"{k} {total[k]}" for k in CATS[:6]))
    print(f"B (context missing) {B}: " + ", ".join(f"{k} {total[k]}" for k in CATS[6:]))
    print(f"C (followed, context enough) {clean} of {moments} judgement moments ({100 * clean / moments if moments else 0:.0f}%)")
    items = []
    for g in games:
        for it in g.get("items", []):
            it = dict(it)
            it["game"] = g["game"]
            items.append(it)
    by_cost = collections.Counter(it.get("cost", "?") for it in items)
    by_cat = collections.Counter(it.get("category", "?") for it in items)
    by_fix = collections.Counter(it.get("fix_class", "?") for it in items)
    print(f"items {len(items)}: by cost {dict(by_cost)}; by category {dict(sorted(by_cat.items()))}; by fix class {dict(by_fix.most_common())}")
    costs = set(a.cost.split(",")) if a.cost else set()
    cats = set(a.category.split(",")) if a.category else set()
    shown = [it for it in items if (a.items or it.get("cost") in costs) and (not cats or it.get("category") in cats)]
    shown.sort(key=lambda it: (-COST_RANK.get(it.get("cost"), -1), it.get("category", ""), it["game"], it.get("clock", "")))
    if shown:
        print(f"\nitems shown {len(shown)}:")
        for it in shown:
            print(f"- [{it.get('category')}/{it.get('cost')}/{it.get('confidence', '?')}/{it.get('fix_class', '?')}] {it['game'][11:]} {it.get('seat', '')} {it.get('clock', '')}: "
                  f"wanted: {str(it.get('wanted', ''))[:200]} | happened: {str(it.get('happened', ''))[:200]}")
    if a.no_lever:
        print("\nno lever:")
        for g in games:
            for nl in g.get("no_lever", []):
                if isinstance(nl, dict):
                    print(f"- {g['game'][11:]} {nl.get('seat', '')} {nl.get('clock', '')}: {str(nl.get('wanted', ''))[:220]} | {str(nl.get('why_no_lever', ''))[:160]}")
                else:
                    print(f"- {g['game'][11:]}: {str(nl)[:300]}")
    unreadable = [(g["game"], u) for g in games for u in g.get("unreadable", []) if u]
    if unreadable:
        print("\nunreadable:")
        for game, u in unreadable:
            print(f"- {game[11:]}: {str(u)[:200]}")
    if a.json:
        json.dump({"games": [{k: v for k, v in g.items() if k != "_dir"} for g in games], "totals": dict(total), "moments": moments, "clean": clean, "items": items}, open(a.json, "w"), indent=1)
        print(f"\nwritten {a.json}")


if __name__ == "__main__":
    main()
