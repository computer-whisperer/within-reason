#!/usr/bin/env python3
"""How long a new soldier takes to be with a group (in some group's members at a hands' call), by the minute it
was made; and how its coming ended.

usage: run/slots_arrivals.py run/matches/<batch>/<NN> [...]
"""
import collections, json, os, statistics, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from match_read import first as first_file, is_army_def, rows as read_rows  # noqa: E402


def read(d):
    first = {}
    for r in read_rows(first_file(d, "jev-")):
        if r.get("t") != "call" or not r.get("groups"):
            continue
        for g in r["groups"]:
            for u in g["members"]:
                first.setdefault(u, r["f"])
    defs, soldiers, born, died, end = None, set(), {}, {}, 0
    for line in open(d + "/record-0.jsonl"):
        r = json.loads(line)
        if "unit_defs" in r:
            defs = r["unit_defs"]
        end = max(end, r.get("f", 0))
        if r.get("t") == "ev":
            if r.get("k") == "finished":
                if is_army_def(defs[r["d"]]):
                    soldiers.add(r["u"])
                born[r["u"]] = r["f"]
            if r.get("k") == "destroyed":
                died[r["u"]] = r["f"]
    rows = collections.defaultdict(lambda: {"n": 0, "join": [], "died": 0, "open": 0})
    for u, b in born.items():
        if u not in soldiers:
            continue
        m = (b // 1800) // 2 * 2
        row = rows[m]
        row["n"] += 1
        if u in first:
            row["join"].append((first[u] - b) / 30)
        elif u in died:
            row["died"] += 1
        else:
            row["open"] += 1
    print(os.path.basename(os.path.dirname(d.rstrip("/"))), f"(ends {end // 1800}:{end // 30 % 60:02d})")
    print("  made in    n   joined  median s  p90 s  over 120 s  died before joining  never joined by the end")
    for m in sorted(rows):
        row = rows[m]
        j = sorted(row["join"])
        med = f"{statistics.median(j):.0f}" if j else "-"
        p90 = f"{j[int(0.9 * (len(j) - 1))]:.0f}" if j else "-"
        print(f"  {m:2}-{m + 2:<2} min {row['n']:4} {len(j):7} {med:>9} {p90:>6} {sum(x > 120 for x in j):>11} {row['died']:>19} {row['open']:>24}")


for d in sys.argv[1:]:
    read(d)
